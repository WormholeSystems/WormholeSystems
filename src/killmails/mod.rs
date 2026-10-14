//! Killmail ingest and threat analysis.
//!
//! [`stream`] polls zKillboard's R2Z2 stream for a minimal row per killmail; the full ESI
//! payload is not kept. [`archive`] imports EVE Ref's daily archives for the same 90-day
//! window the analysis looks over, since the live stream starts blind. [`analysis`] runs
//! the daily threat rules over what both wrote, and [`card`] is the read model the map's
//! killmail card fetches.
//!
//! All the background loops only run when `ZKB_LISTEN=1`.

mod analysis;
mod archive;
mod card;
mod stream;

pub use analysis::{analyze, threat_level};
pub use archive::{BACKFILL_DAYS, backfill};
pub use card::{
    CARD_LIMIT, CARD_WINDOW_DAYS, KillParty, KillmailFilter, MapKillmail, list_for_map,
};
pub use stream::NameResolver;

use serde::Deserialize;
use sqlx::PgPool;

use crate::esi::EsiClient;

const RETENTION_DAYS: i32 = 730;

/// zKillboard's own R2Z2 stream; `ZKB_BASE_URL` points elsewhere for a mirror or a stub.
pub const ZKB_BASE_URL: &str = "https://r2z2.zkillboard.com";

/// The compact per-killmail org record persisted in `killmails.orgs`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, Deserialize)]
pub struct Org {
    pub id: i64,
    pub kind: String,
}

/// A killmail's ESI body, narrowed to the fields anything here reads.
///
/// Typed rather than `serde_json::Value`: the full body lists every item the victim carried,
/// and against a year of archives those allocations dominate the import. Every field is
/// optional so a malformed payload costs one killmail rather than a batch.
#[derive(Default, Deserialize)]
struct EsiKillmail {
    killmail_id: Option<i64>,
    killmail_hash: Option<String>,
    killmail_time: Option<String>,
    solar_system_id: Option<i64>,
    #[serde(default)]
    victim: Participant,
    #[serde(default)]
    attackers: Vec<Participant>,
}

/// One side of a killmail; the victim never carries `final_blow`, so both share a shape.
#[derive(Default, Deserialize)]
struct Participant {
    character_id: Option<i64>,
    corporation_id: Option<i64>,
    alliance_id: Option<i64>,
    ship_type_id: Option<i64>,
    #[serde(default)]
    final_blow: bool,
}

/// The parts of a killmail a reader cares about, pulled out of the two payloads.
struct Detail {
    victim_character_id: Option<i64>,
    victim_corporation_id: Option<i64>,
    victim_alliance_id: Option<i64>,
    victim_ship_type_id: Option<i64>,
    total_value: Option<f64>,
    attacker_count: Option<i32>,
    is_npc: bool,
    is_solo: bool,
    final_blow_character_id: Option<i64>,
    final_blow_corporation_id: Option<i64>,
    final_blow_alliance_id: Option<i64>,
    final_blow_ship_type_id: Option<i64>,
}

fn extract_detail(esi: &EsiKillmail, zkb: &serde_json::Value) -> Detail {
    let victim = &esi.victim;
    // The killing blow is the one attacker worth naming; the rest are a count.
    let final_blow = esi.attackers.iter().find(|a| a.final_blow);
    Detail {
        victim_character_id: victim.character_id,
        victim_corporation_id: victim.corporation_id,
        victim_alliance_id: victim.alliance_id,
        victim_ship_type_id: victim.ship_type_id,
        total_value: zkb["totalValue"].as_f64(),
        attacker_count: Some(
            zkb["attackerCount"]
                .as_i64()
                .unwrap_or(esi.attackers.len() as i64) as i32,
        ),
        is_npc: zkb["npc"].as_bool().unwrap_or(false),
        is_solo: zkb["solo"].as_bool().unwrap_or(false),
        final_blow_character_id: final_blow.and_then(|a| a.character_id),
        final_blow_corporation_id: final_blow.and_then(|a| a.corporation_id),
        final_blow_alliance_id: final_blow.and_then(|a| a.alliance_id),
        final_blow_ship_type_id: final_blow.and_then(|a| a.ship_type_id),
    }
}

/// The organisations participating in a killmail (victim + every attacker), alliance
/// preferred over corporation, each org at most once.
fn extract_orgs(esi: &EsiKillmail) -> Vec<Org> {
    let mut seen = std::collections::HashSet::new();
    let mut orgs = Vec::new();
    let mut push = |entity: &Participant| {
        let (id, alliance) = match entity.alliance_id {
            Some(id) => (id, true),
            None => match entity.corporation_id {
                Some(id) => (id, false),
                None => return,
            },
        };
        if seen.insert((id, alliance)) {
            orgs.push(Org {
                id,
                kind: if alliance { "alliance" } else { "corporation" }.to_string(),
            });
        }
    };
    push(&esi.victim);
    for attacker in &esi.attackers {
        push(attacker);
    }
    orgs
}

/// One killmail reduced to exactly what the insert binds, however it arrived.
struct KillmailRow {
    id: i64,
    hash: String,
    solar_system_id: i64,
    time: String,
    orgs: serde_json::Value,
    detail: Detail,
}

/// Insert killmail rows, skipping anything older than the retention window and anything
/// already stored. Returns how many were new. The one insert statement for both the live
/// stream and the archive import, batched with `unnest` so a whole archived day fits.
///
/// `live` rows get an `ingest_seq`, which is how the name resolver finds what is new. The
/// stream is the only live writer and inserts one kill at a time, so the numbers commit in
/// order and the resolver's high-water mark cannot step over one still in flight.
async fn insert_rows(pool: &PgPool, rows: &[KillmailRow], live: bool) -> sqlx::Result<u64> {
    let mut inserted = 0u64;
    // Fixed arrays rather than one placeholder per value, so the batch is bounded by
    // memory rather than by Postgres's parameter limit.
    for chunk in rows.chunks(2_000) {
        let ids: Vec<i64> = chunk.iter().map(|k| k.id).collect();
        let hashes: Vec<String> = chunk.iter().map(|k| k.hash.clone()).collect();
        let systems: Vec<i64> = chunk.iter().map(|k| k.solar_system_id).collect();
        let times: Vec<String> = chunk.iter().map(|k| k.time.clone()).collect();
        let orgs: Vec<serde_json::Value> = chunk.iter().map(|k| k.orgs.clone()).collect();
        let victim_chars: Vec<Option<i64>> =
            chunk.iter().map(|k| k.detail.victim_character_id).collect();
        let victim_corps: Vec<Option<i64>> = chunk
            .iter()
            .map(|k| k.detail.victim_corporation_id)
            .collect();
        let victim_allis: Vec<Option<i64>> =
            chunk.iter().map(|k| k.detail.victim_alliance_id).collect();
        let victim_ships: Vec<Option<i64>> =
            chunk.iter().map(|k| k.detail.victim_ship_type_id).collect();
        let values: Vec<Option<f64>> = chunk.iter().map(|k| k.detail.total_value).collect();
        let attackers: Vec<Option<i32>> = chunk.iter().map(|k| k.detail.attacker_count).collect();
        let npcs: Vec<bool> = chunk.iter().map(|k| k.detail.is_npc).collect();
        let solos: Vec<bool> = chunk.iter().map(|k| k.detail.is_solo).collect();
        let fb_chars: Vec<Option<i64>> = chunk
            .iter()
            .map(|k| k.detail.final_blow_character_id)
            .collect();
        let fb_corps: Vec<Option<i64>> = chunk
            .iter()
            .map(|k| k.detail.final_blow_corporation_id)
            .collect();
        let fb_allis: Vec<Option<i64>> = chunk
            .iter()
            .map(|k| k.detail.final_blow_alliance_id)
            .collect();
        let fb_ships: Vec<Option<i64>> = chunk
            .iter()
            .map(|k| k.detail.final_blow_ship_type_id)
            .collect();
        let n = sqlx::query(
            "insert into killmails (
                 id, hash, solar_system_id, time, orgs,
                 victim_character_id, victim_corporation_id, victim_alliance_id,
                 victim_ship_type_id, total_value, attacker_count, is_npc, is_solo,
                 final_blow_character_id, final_blow_corporation_id,
                 final_blow_alliance_id, final_blow_ship_type_id, ingest_seq
             )
             select t.*, case when $19 then nextval('killmails_ingest_seq') end
             from unnest($1::bigint[], $2::text[], $3::bigint[],
                                  $4::text[]::timestamptz[], $5::jsonb[],
                                  $6::bigint[], $7::bigint[], $8::bigint[], $9::bigint[],
                                  $10::double precision[], $11::int[], $12::boolean[],
                                  $13::boolean[], $14::bigint[], $15::bigint[],
                                  $16::bigint[], $17::bigint[])
                 as t(id, hash, solar_system_id, time, orgs,
                      victim_character_id, victim_corporation_id, victim_alliance_id,
                      victim_ship_type_id, total_value, attacker_count, is_npc, is_solo,
                      final_blow_character_id, final_blow_corporation_id,
                      final_blow_alliance_id, final_blow_ship_type_id)
             where t.time >= now() - make_interval(days => $18)
             on conflict (id) do nothing",
        )
        .bind(&ids)
        .bind(&hashes)
        .bind(&systems)
        .bind(&times)
        .bind(&orgs)
        .bind(&victim_chars)
        .bind(&victim_corps)
        .bind(&victim_allis)
        .bind(&victim_ships)
        .bind(&values)
        .bind(&attackers)
        .bind(&npcs)
        .bind(&solos)
        .bind(&fb_chars)
        .bind(&fb_corps)
        .bind(&fb_allis)
        .bind(&fb_ships)
        .bind(RETENTION_DAYS)
        .bind(live)
        .execute(pool)
        .await?
        .rows_affected();
        inserted += n;
    }
    Ok(inserted)
}

/// zKillboard and EVE Ref reject anonymous clients (403), so identify ourselves.
fn http_client() -> reqwest::Client {
    crate::user_agent::client()
}

/// Spawn the ingest, backfill and analysis loops, when the config turns them on.
pub fn start(
    pool: PgPool,
    esi: EsiClient,
    alerts: Option<std::sync::Arc<crate::alerts::Runtime>>,
    config: &crate::config::KillmailConfig,
) {
    if !config.listen {
        return;
    }
    let days = config.backfill_days;
    tokio::spawn(stream::listen(
        pool.clone(),
        alerts,
        config.base_url.clone(),
    ));
    stream::start_resolver(pool.clone(), esi.clone());
    if days > 0 {
        tokio::spawn(archive::backfill_loop(pool.clone(), esi.clone(), days));
    }
    analysis::start(pool, esi);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests state the payload as it arrives on the wire, then read it the way the code
    /// does, so a field the struct forgets shows up here.
    fn body(value: serde_json::Value) -> EsiKillmail {
        serde_json::from_value(value).expect("killmail body")
    }

    #[test]
    fn orgs_prefer_alliance_and_dedupe() {
        let esi = body(serde_json::json!({
            "victim": {"corporation_id": 100, "alliance_id": 200},
            "attackers": [
                {"corporation_id": 100, "alliance_id": 200},
                {"corporation_id": 300},
                {"corporation_id": 300},
                {"ship_type_id": 1}
            ]
        }));
        let orgs = extract_orgs(&esi);
        assert_eq!(
            orgs,
            vec![
                Org {
                    id: 200,
                    kind: "alliance".into()
                },
                Org {
                    id: 300,
                    kind: "corporation".into()
                },
            ]
        );
    }
}

#[cfg(test)]
mod detail_tests {
    use super::*;

    fn body(value: serde_json::Value) -> EsiKillmail {
        serde_json::from_value(value).expect("killmail body")
    }

    fn payload() -> (EsiKillmail, serde_json::Value) {
        let esi = body(serde_json::json!({
            "victim": {
                "character_id": 11, "corporation_id": 22, "alliance_id": 33,
                "ship_type_id": 670, "damage_taken": 1234
            },
            "attackers": [
                {"character_id": 44, "corporation_id": 55, "final_blow": false,
                 "ship_type_id": 111},
                {"character_id": 66, "corporation_id": 77, "alliance_id": 88,
                 "final_blow": true, "ship_type_id": 222}
            ]
        }));
        let zkb = serde_json::json!({
            "totalValue": 1_234_567.89, "attackerCount": 2, "npc": false, "solo": false
        });
        (esi, zkb)
    }

    #[test]
    fn a_killmail_yields_what_a_reader_wants() {
        let (esi, zkb) = payload();
        let d = extract_detail(&esi, &zkb);
        assert_eq!(d.victim_character_id, Some(11));
        assert_eq!(d.victim_ship_type_id, Some(670));
        assert_eq!(d.total_value, Some(1_234_567.89));
        assert_eq!(d.attacker_count, Some(2));
        // The killing blow, not merely the first or last attacker.
        assert_eq!(d.final_blow_character_id, Some(66));
        assert_eq!(d.final_blow_alliance_id, Some(88));
        assert_eq!(d.final_blow_ship_type_id, Some(222));
    }

    #[test]
    fn a_frame_without_zkb_still_gives_an_attacker_count() {
        let (esi, _) = payload();
        let d = extract_detail(&esi, &serde_json::Value::Null);
        // Falls back to counting the array, so the row is never mysteriously blank.
        assert_eq!(d.attacker_count, Some(2));
        assert_eq!(d.total_value, None);
        assert!(!d.is_npc && !d.is_solo);
    }

    #[test]
    fn a_kill_with_nobody_flagged_names_no_attacker() {
        let esi = body(serde_json::json!({ "victim": {}, "attackers": [{"character_id": 1}] }));
        let d = extract_detail(&esi, &serde_json::Value::Null);
        assert_eq!(d.final_blow_character_id, None);
        assert_eq!(d.attacker_count, Some(1));
    }
}
