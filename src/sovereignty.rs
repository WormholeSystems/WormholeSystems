//! Sovereignty sync: keeps `system_sovereignty` (and the alliance/corp entities it names)
//! current so map nodes can show human-readable holders.
//!
//! The endpoints involved are public, so this needs no token and no per-character work. Each
//! tick fetches the whole sovereignty map in one call, resolves any holder entity we don't
//! have fresh, and upserts the per-system holder. Factions come from the SDE, never ESI.

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::entities::{self, EntityKind};
use crate::esi::EsiClient;
use crate::esi::sovereignty::SovereigntySystem;
use crate::server_status::ServerWatch;

/// How often to refresh sovereignty. It changes slowly (alliance-level territory), so an hour
/// is gentle on ESI while keeping the map reasonably fresh.
const INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Spawn the sync loop. Returns immediately; the loop runs for the process lifetime.
pub fn start(pool: PgPool, esi: EsiClient, server: ServerWatch) {
    crate::background::spawn_every("sovereignty sync", INTERVAL, move || {
        let (pool, esi, server) = (pool.clone(), esi.clone(), server.clone());
        async move {
            // Sovereignty does not move while nobody is playing, and the call would fail
            // anyway. The next tick picks it up once Tranquility is back.
            if server.should_poll() {
                sync_once(&pool, &esi).await;
            }
            Ok::<_, std::convert::Infallible>(())
        }
    });
}

async fn sync_once(pool: &PgPool, esi: &EsiClient) {
    let systems = match esi.sovereignty_systems().await {
        Ok(systems) => systems,
        Err(err) => {
            tracing::error!("sovereignty fetch failed: {err}");
            return;
        }
    };

    // Resolve holder entities first so the per-system upserts below don't trip the FKs.
    resolve_entities(pool, esi, &systems).await;

    match store(pool, &systems).await {
        // A system we don't have in the SDE, or an entity that failed to resolve. The
        // next tick retries it.
        Ok(skipped) if skipped > 0 => {
            tracing::warn!("sovereignty: {skipped} claim(s) skipped, unknown system or holder")
        }
        Ok(_) => {}
        Err(err) => tracing::error!("sovereignty store failed: {err}"),
    }
}

/// Fetch + upsert every alliance and corporation named by a claim that we don't already have
/// fresh. Failures are skipped (the next tick retries); factions are SDE-seeded, never fetched.
async fn resolve_entities(pool: &PgPool, esi: &EsiClient, systems: &[SovereigntySystem]) {
    let mut alliance_ids = Vec::new();
    let mut corporation_ids = Vec::new();
    for system in systems {
        if let Some(claim) = &system.claim.alliance {
            alliance_ids.push(claim.alliance_id);
            corporation_ids.push(claim.corporation_id);
        }
    }
    entities::ensure(pool, esi, EntityKind::Alliance, &alliance_ids).await;
    entities::ensure(pool, esi, EntityKind::Corporation, &corporation_ids).await;
}

/// Write every system's holder: alliance, faction, or none (unclaimed removes the row). Two
/// statements for the whole map rather than one per system, and a row is only rewritten
/// when its holder actually changed, which on most runs is almost none of them. Claims
/// naming a system or holder we do not have are left out and counted.
pub async fn store(pool: &PgPool, systems: &[SovereigntySystem]) -> Result<i64, sqlx::Error> {
    let mut ids = Vec::new();
    let mut alliances = Vec::new();
    let mut corporations = Vec::new();
    let mut factions = Vec::new();
    let mut since = Vec::new();
    let mut capitals = Vec::new();
    let mut unclaimed = Vec::new();
    for system in systems {
        let claim = &system.claim;
        if let Some(alliance) = &claim.alliance {
            ids.push(system.solar_system_id);
            alliances.push(Some(alliance.alliance_id));
            corporations.push(Some(alliance.corporation_id));
            factions.push(None);
            since.push(alliance.claimed_since.as_deref().and_then(parse_timestamp));
            capitals.push(alliance.is_capital_system);
        } else if let Some(faction) = &claim.faction {
            ids.push(system.solar_system_id);
            alliances.push(None);
            corporations.push(None);
            factions.push(Some(faction.faction_id));
            since.push(None);
            capitals.push(None);
        } else {
            unclaimed.push(system.solar_system_id);
        }
    }

    let mut tx = pool.begin().await?;
    let skipped = sqlx::query_scalar!(
        r#"with claims as (
               select * from unnest($1::bigint[], $2::bigint[], $3::bigint[], $4::bigint[],
                                    $5::timestamptz[], $6::boolean[])
                   as c(solar_system_id, alliance_id, corporation_id, faction_id,
                        claimed_since, is_capital_system)
           ),
           known as (
               select c.* from claims c
               where exists (select 1 from solar_systems s where s.id = c.solar_system_id)
                 and (c.alliance_id is null
                      or exists (select 1 from alliances a where a.id = c.alliance_id))
                 and (c.corporation_id is null
                      or exists (select 1 from corporations co where co.id = c.corporation_id))
                 and (c.faction_id is null
                      or exists (select 1 from factions f where f.id = c.faction_id))
           ),
           written as (
               insert into system_sovereignty as sov
                   (solar_system_id, alliance_id, corporation_id, faction_id, claimed_since,
                    is_capital_system)
               select * from known
               on conflict (solar_system_id) do update set
                   alliance_id = excluded.alliance_id,
                   corporation_id = excluded.corporation_id,
                   faction_id = excluded.faction_id,
                   claimed_since = excluded.claimed_since,
                   is_capital_system = excluded.is_capital_system,
                   updated_at = now()
               where (sov.alliance_id, sov.corporation_id, sov.faction_id, sov.claimed_since,
                      sov.is_capital_system)
                     is distinct from
                     (excluded.alliance_id, excluded.corporation_id, excluded.faction_id,
                      excluded.claimed_since, excluded.is_capital_system)
           )
           select (select count(*) from claims) - (select count(*) from known)
               as "skipped!""#,
        &ids,
        &alliances as &[Option<i64>],
        &corporations as &[Option<i64>],
        &factions as &[Option<i64>],
        &since as &[Option<DateTime<Utc>>],
        &capitals as &[Option<bool>],
    )
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query!(
        "delete from system_sovereignty where solar_system_id = any($1)",
        &unclaimed,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(skipped)
}

/// ESI timestamps are RFC 3339; drop any that don't parse rather than failing the row.
fn parse_timestamp(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
