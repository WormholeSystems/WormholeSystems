//! Backfill from EVE Ref's daily killmail archives, so a fresh instance is not blind for
//! the 90 days the threat analysis looks over.

use sqlx::PgPool;

use super::{EsiKillmail, KillmailRow, extract_detail, extract_orgs, http_client};
use crate::entities::EntityKind;
use crate::esi::EsiClient;

/// How far back a boot fills, matching the window threat analysis looks over. The live
/// listener only ever sees kills from now on, so without this a fresh instance is useless
/// for months.
pub const BACKFILL_DAYS: u32 = 90;

type BoxError = Box<dyn std::error::Error>;

/// Turn a downloaded `.tar.bz2` into rows, without touching the disk. A day is around 24,000
/// separate JSON files, and writing them out to extract them cost more than the decompression
/// and the insert put together.
///
/// Blocking and CPU-bound, so callers run it off the async runtime.
fn read_archive(bytes: &[u8]) -> std::io::Result<Vec<KillmailRow>> {
    use std::io::Read;

    // Multi-stream: the archives are concatenated bzip2 streams, and a plain decoder would
    // stop at the end of the first one and silently return part of the day.
    let mut archive = tar::Archive::new(bzip2::read::MultiBzDecoder::new(bytes));
    let mut kills = Vec::new();
    let mut text = String::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()?.extension().is_none_or(|e| e != "json") {
            continue;
        }
        text.clear();
        if entry.read_to_string(&mut text).is_err() {
            continue;
        }
        let Ok(km) = serde_json::from_str::<EsiKillmail>(&text) else {
            continue;
        };
        let (Some(id), Some(solar_system_id), Some(time)) =
            (km.killmail_id, km.solar_system_id, km.killmail_time.clone())
        else {
            continue;
        };
        kills.push(KillmailRow {
            id,
            hash: km.killmail_hash.clone().unwrap_or_default(),
            solar_system_id,
            time,
            orgs: serde_json::to_value(extract_orgs(&km)).map_err(std::io::Error::other)?,
            // The archives carry the ESI body but not zKillboard's block, so everything but
            // the ISK value and the solo/NPC flags comes through. The card ignores rows with
            // no victim ship, so storing only the bare minimum would import invisible history.
            detail: extract_detail(&km, &serde_json::Value::Null),
        });
    }
    Ok(kills)
}

/// The days in the last `days` that are not imported yet, most recent first.
async fn missing_days(pool: &PgPool, days: u32) -> Result<Vec<chrono::NaiveDate>, BoxError> {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let wanted: Vec<chrono::NaiveDate> = (1..=i64::from(days))
        .map(|offset| {
            chrono::DateTime::from_timestamp(now_secs - offset * 86_400, 0)
                .expect("valid timestamp")
                .date_naive()
        })
        .collect();
    let done: std::collections::HashSet<chrono::NaiveDate> = sqlx::query_scalar!(
        "select day from killmail_imports where day = any($1)",
        &wanted
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect();
    Ok(wanted.into_iter().filter(|d| !done.contains(d)).collect())
}

/// Import one archived day. `Ok(None)` when EVE Ref has nothing for it yet.
async fn import_day(
    pool: &PgPool,
    esi: &EsiClient,
    http: &reqwest::Client,
    day: chrono::NaiveDate,
) -> Result<Option<String>, BoxError> {
    let name = format!("killmails-{}.tar.bz2", day.format("%Y-%m-%d"));
    let url = format!(
        "https://data.everef.net/killmails/{}/{name}",
        day.format("%Y")
    );

    let res = http.get(&url).send().await?;
    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let bytes = res.error_for_status()?.bytes().await?;
    let kills = tokio::task::spawn_blocking(move || read_archive(&bytes)).await??;

    // Every entity the day mentions, deduped before anything is fetched: tens of thousands
    // of killmails name only a few thousand distinct pilots.
    let mut day_characters: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut day_corporations: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut day_alliances: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for k in &kills {
        day_characters.extend(k.detail.victim_character_id);
        day_characters.extend(k.detail.final_blow_character_id);
        day_corporations.extend(k.detail.victim_corporation_id);
        day_corporations.extend(k.detail.final_blow_corporation_id);
        day_alliances.extend(k.detail.victim_alliance_id);
        day_alliances.extend(k.detail.final_blow_alliance_id);
    }

    let inserted = super::insert_rows(pool, &kills, false).await?;

    // Organisations one at a time, because their tickers are what a row shows and only the
    // per-entity endpoint returns them. Characters go through the bulk endpoint.
    let alliances: Vec<i64> = day_alliances.into_iter().collect();
    let corporations: Vec<i64> = day_corporations.into_iter().collect();
    let characters: Vec<i64> = day_characters.into_iter().collect();
    crate::entities::ensure(pool, esi, EntityKind::Alliance, &alliances).await;
    crate::entities::ensure(pool, esi, EntityKind::Corporation, &corporations).await;
    let named = crate::entities::ensure_character_names(pool, esi, &characters)
        .await
        .named;

    sqlx::query!(
        "insert into killmail_imports (day, killmails) values ($1, $2)
         on conflict (day) do update set killmails = excluded.killmails, imported_at = now()",
        day,
        kills.len() as i32,
    )
    .execute(pool)
    .await?;

    Ok(Some(format!(
        "{} killmails ({inserted} new), named {named} of {} pilots and resolved {} orgs",
        kills.len(),
        characters.len(),
        alliances.len() + corporations.len()
    )))
}

/// Backfill killmails from EVE Ref's daily archives (`wormholesystems killmails-backfill <days>`),
/// most recent day first. Days already in the ledger are skipped, so re-running only fetches
/// what is missing. Ends with a threat analysis run so the data shows up immediately.
pub async fn backfill(pool: &PgPool, esi: &EsiClient, days: u32) -> Result<(), BoxError> {
    let http = http_client();
    let missing = missing_days(pool, days).await?;
    println!("{} of the last {days} days to import", missing.len());
    for day in missing {
        print!("{day}: downloading… ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        match import_day(pool, esi, &http, day).await? {
            Some(summary) => println!("{summary}"),
            None => println!("no archive"),
        }
    }

    println!("running threat analysis…");
    super::analyze(pool, esi).await?;
    println!("backfill complete.");
    Ok(())
}

/// Background: fill in the archived days this instance is missing. Once per boot rather than
/// on a schedule, because the live listener covers everything from startup onwards.
pub(super) async fn backfill_loop(pool: PgPool, esi: EsiClient, days: u32) {
    let http = http_client();
    let missing = match missing_days(&pool, days).await {
        Ok(days) => days,
        Err(err) => return eprintln!("killmail backfill: {err}"),
    };
    if missing.is_empty() {
        return;
    }
    println!("killmail backfill: {} days missing", missing.len());
    let mut imported = 0usize;
    for day in missing {
        match import_day(&pool, &esi, &http, day).await {
            Ok(Some(_)) => imported += 1,
            Ok(None) => {}
            Err(err) => eprintln!("killmail backfill {day}: {err}"),
        }
    }
    println!("killmail backfill: {imported} days imported");
    if imported > 0
        && let Err(err) = super::analyze(&pool, &esi).await
    {
        eprintln!("threat analysis after backfill failed: {err}");
    }
}
