//! The live feed: zKillboard's R2Z2 stream, one killmail at a time.

use std::time::Duration;

use serde::Deserialize;
use sqlx::PgPool;
use tokio::sync::mpsc;

use super::{Detail, EsiKillmail, KillmailRow, extract_detail, extract_orgs, http_client};
use crate::entities::EntityKind;
use crate::esi::EsiClient;

const DEFAULT_BASE: &str = "https://r2z2.zkillboard.com";

#[derive(Deserialize)]
struct R2Z2Sequence {
    sequence: i64,
}

#[derive(Deserialize)]
struct R2Z2Killmail {
    killmail_id: i64,
    hash: String,
    esi: EsiKillmail,
    /// zKillboard's own summary: value, attacker count, and the solo/NPC flags. Absent on
    /// a malformed frame, which is why every field below is read defensively.
    #[serde(default)]
    zkb: serde_json::Value,
}

/// How long to wait after the stream had nothing new. zKillboard asks for 6 seconds: a kill
/// arrives every 5.5 on average, so polling faster gains nothing.
const IDLE: Duration = Duration::from_secs(6);

/// The least time between two fetches while there is a backlog. R2Z2 bans an address for an
/// hour past 15 requests a second, so catching up stays well under that.
const PACE: Duration = Duration::from_millis(100);

/// Kills waiting for alert evaluation. Past this, evaluation is minutes behind (the
/// database is struggling), and an alert about a kill that old is not worth queueing.
const ALERT_QUEUE: usize = 1024;

/// Kills evaluated together, so a burst reads each map's chain once.
const ALERT_BATCH: usize = 100;

pub(super) async fn listen(
    pool: PgPool,
    maps: crate::maps::MapHub,
    alerts: Option<std::sync::Arc<crate::alerts::Runtime>>,
) {
    let base = std::env::var("ZKB_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string());
    let http = http_client();
    let alerts = alerts.map(|runtime| {
        let (tx, rx) = mpsc::channel(ALERT_QUEUE);
        tokio::spawn(alert_loop(pool.clone(), runtime, rx));
        tx
    });
    // Read from the table once, then kept here: the table is only for the next boot.
    let mut cursor: Option<i64> = None;
    loop {
        let last = match cursor {
            Some(seq) => seq,
            None => match start_at(&pool, &http, &base).await {
                Ok(seq) => {
                    cursor = Some(seq);
                    seq
                }
                Err(err) => {
                    eprintln!("killmail ingest error: {err}");
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    continue;
                }
            },
        };
        match ingest(&pool, &http, &base, &maps, alerts.as_ref(), last + 1).await {
            Ok(true) => {
                cursor = Some(last + 1);
                tokio::time::sleep(PACE).await;
            }
            Ok(false) => tokio::time::sleep(IDLE).await,
            Err(err) => {
                eprintln!("killmail ingest error: {err}");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }
    }
}

/// Where the stream left off, or the live head of it on the very first run.
async fn start_at(
    pool: &PgPool,
    http: &reqwest::Client,
    base: &str,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let cursor: Option<i64> = sqlx::query_scalar!("select sequence_id from zkb_state")
        .fetch_optional(pool)
        .await?;
    if let Some(seq) = cursor {
        return Ok(seq);
    }
    let head: R2Z2Sequence = http
        .get(format!("{base}/ephemeral/sequence.json"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    sqlx::query!(
        "insert into zkb_state (id, sequence_id) values (true, $1)",
        head.sequence
    )
    .execute(pool)
    .await?;
    Ok(head.sequence)
}

/// Fetch and persist killmail `next` in the sequence. Returns whether there was one.
async fn ingest(
    pool: &PgPool,
    http: &reqwest::Client,
    base: &str,
    maps: &crate::maps::MapHub,
    alerts: Option<&mpsc::Sender<Pending>>,
    next: i64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let res = http
        .get(format!("{base}/ephemeral/{next}.json"))
        .send()
        .await?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(false); // caught up
    }
    let body = res.error_for_status()?.text().await?;
    let Ok(km) = serde_json::from_str::<R2Z2Killmail>(&body) else {
        // Empty/garbage frame: advance past it.
        advance(pool, next).await?;
        return Ok(true);
    };

    let solar_system_id = km.esi.solar_system_id.unwrap_or(0);
    let time = km.esi.killmail_time.clone().unwrap_or_default();
    if solar_system_id != 0 && !time.is_empty() {
        let row = KillmailRow {
            id: km.killmail_id,
            hash: km.hash,
            solar_system_id,
            time,
            orgs: serde_json::to_value(extract_orgs(&km.esi))?,
            detail: extract_detail(&km.esi, &km.zkb),
        };
        super::insert_rows(pool, std::slice::from_ref(&row)).await?;
        announce(pool, maps, solar_system_id).await;
        if let Some(alerts) = alerts {
            offer(alerts, row);
        }
    }
    advance(pool, next).await?;
    Ok(true)
}

/// A stored kill on its way to the alerts.
struct Pending {
    killmail_id: i64,
    solar_system_id: i64,
    detail: Detail,
}

/// Hand a kill to the alert worker without waiting for it. A full queue drops the kill's
/// alerts, not the kill: it is stored either way, and ingest must never wait on Discord or
/// on a slow evaluation.
fn offer(alerts: &mpsc::Sender<Pending>, row: KillmailRow) {
    let pending = Pending {
        killmail_id: row.id,
        solar_system_id: row.solar_system_id,
        detail: row.detail,
    };
    if let Err(mpsc::error::TrySendError::Full(pending)) = alerts.try_send(pending) {
        eprintln!(
            "killmail alerts: queue full, no alerts for killmail {}",
            pending.killmail_id
        );
    }
}

/// Evaluate queued kills against the alerts, in batches of whatever has piled up.
async fn alert_loop(
    pool: PgPool,
    alerts: std::sync::Arc<crate::alerts::Runtime>,
    mut queue: mpsc::Receiver<Pending>,
) {
    let mut pending = Vec::with_capacity(ALERT_BATCH);
    while queue.recv_many(&mut pending, ALERT_BATCH).await > 0 {
        let mut kills = Vec::with_capacity(pending.len());
        for p in pending.drain(..) {
            kills.push(kill_for(&pool, p.killmail_id, p.solar_system_id, &p.detail).await);
        }
        let out = alerts.killmails(&pool, &kills).await;
        alerts.enqueue(&pool, out).await;
    }
}

/// A kill as the alerts read it. Names come from what is already stored, since "Someone
/// lost a Loki" now beats a complete message a minute later.
async fn kill_for(
    pool: &PgPool,
    killmail_id: i64,
    solar_system_id: i64,
    detail: &Detail,
) -> crate::alerts::killmail::Kill {
    use crate::alerts::filters::Candidates;
    let named = sqlx::query!(
        r#"select vc.name as "victim_name?", vt.name as "victim_ship?",
                  fc.name as "attacker_name?", vt.group_id as "victim_ship_group?",
                  ft.group_id as "attacker_ship_group?"
           from (select 1) as one
           left join characters vc on vc.id = $1
           left join characters fc on fc.id = $2
           left join types vt on vt.id = $3
           left join types ft on ft.id = $4"#,
        detail.victim_character_id,
        detail.final_blow_character_id,
        detail.victim_ship_type_id,
        detail.final_blow_ship_type_id,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    crate::alerts::killmail::Kill {
        id: killmail_id,
        solar_system_id,
        candidates: Candidates {
            victim_character: detail.victim_character_id,
            victim_corporation: detail.victim_corporation_id,
            victim_alliance: detail.victim_alliance_id,
            victim_ship_type: detail.victim_ship_type_id,
            victim_ship_group: named.as_ref().and_then(|n| n.victim_ship_group),
            attacker_character: detail.final_blow_character_id,
            attacker_corporation: detail.final_blow_corporation_id,
            attacker_alliance: detail.final_blow_alliance_id,
            attacker_ship_type: detail.final_blow_ship_type_id,
            attacker_ship_group: named.as_ref().and_then(|n| n.attacker_ship_group),
        },
        victim_name: named.as_ref().and_then(|n| n.victim_name.clone()),
        victim_ship: named.as_ref().and_then(|n| n.victim_ship.clone()),
        victim_ship_type_id: detail.victim_ship_type_id,
        attacker_name: named.as_ref().and_then(|n| n.attacker_name.clone()),
        total_value: detail.total_value,
        attacker_count: detail.attacker_count,
        is_solo: detail.is_solo,
        is_npc: detail.is_npc,
    }
}

/// Tell every map holding this system that something died in it. The event carries no
/// payload: what a client shows depends on its own filter, so it refetches.
async fn announce(pool: &PgPool, maps: &crate::maps::MapHub, solar_system_id: i64) {
    let Ok(map_ids) = sqlx::query_scalar!(
        "select distinct map_id from map_solar_systems where solar_system_id = $1",
        solar_system_id,
    )
    .fetch_all(pool)
    .await
    else {
        return;
    };
    for map_id in map_ids {
        maps.publish(crate::maps::MapEvent::KillmailReceived { map_id });
    }
}

async fn advance(pool: &PgPool, seq: i64) -> Result<(), sqlx::Error> {
    sqlx::query!("update zkb_state set sequence_id = $1", seq)
        .execute(pool)
        .await?;
    Ok(())
}

/// Put names to the ids on recent killmails. A separate loop from the ingest: a killmail
/// must be recorded whether or not ESI is answering, and the names are only wanted by the
/// time someone opens the card.
pub(super) async fn resolve_loop(pool: PgPool, esi: EsiClient) {
    loop {
        resolve_recent(&pool, &esi).await;
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn resolve_recent(pool: &PgPool, esi: &EsiClient) {
    // Windowed to what the card can show; `distinct` because a busy system repeats orgs.
    let Ok(rows) = sqlx::query!(
        r#"select distinct victim_character_id, victim_corporation_id, victim_alliance_id,
                  final_blow_character_id, final_blow_corporation_id, final_blow_alliance_id
           from killmails
           where time >= now() - make_interval(days => $1)"#,
        super::CARD_WINDOW_DAYS,
    )
    .fetch_all(pool)
    .await
    else {
        return;
    };

    let mut characters = Vec::new();
    let mut corporations = Vec::new();
    let mut alliances = Vec::new();
    for row in rows {
        characters.extend(row.victim_character_id);
        characters.extend(row.final_blow_character_id);
        corporations.extend(row.victim_corporation_id);
        corporations.extend(row.final_blow_corporation_id);
        alliances.extend(row.victim_alliance_id);
        alliances.extend(row.final_blow_alliance_id);
    }
    // Organisations first: most rows named per fetch if a rate limit cuts the run short.
    crate::entities::ensure(pool, esi, EntityKind::Alliance, &alliances).await;
    crate::entities::ensure(pool, esi, EntityKind::Corporation, &corporations).await;
    crate::entities::ensure(pool, esi, EntityKind::Character, &characters).await;
}
