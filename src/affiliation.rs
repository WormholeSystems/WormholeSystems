//! Affiliation refresh: keeps each signed-in character's corporation and alliance current.
//!
//! Those two columns are what a corporation or alliance grant on a map matches against
//! (see [`crate::maps::access::effective_role`]), so a character that leaves a corp has to
//! stop matching its grants without waiting to log in again. Login refreshes the one
//! character signing in; this loop covers everyone else. See
//! [processes.md](../docs/processes.md#affiliation-refresh).
//!
//! Every character that belongs to an account is refreshed, not only the ones with a live
//! session. Access is also exercised by personal access tokens, the Discord bot and alerts
//! checking their creator still belongs, none of which need a session, and the bulk
//! endpoint makes the whole set cheap: one public call per thousand characters.

use std::time::Duration;

use sqlx::PgPool;

use crate::entities::{self, EntityKind};
use crate::esi::{Affiliation, EsiClient};
use crate::server_status::ServerWatch;

/// ESI caches an affiliation for an hour, so asking more often would only hand back the
/// same answer. The lag between leaving a corp and losing its maps is at most about two of
/// these: ESI's cache, then our next pass.
const INTERVAL: Duration = Duration::from_secs(60 * 60);

/// The most ids `POST /characters/affiliation` takes in one call.
const BATCH: usize = 1000;

/// Spawn the refresh loop. Returns immediately; the loop runs for the process lifetime.
pub fn start(pool: PgPool, esi: EsiClient, server: ServerWatch) {
    crate::background::spawn_every("affiliation refresh", INTERVAL, move || {
        let (pool, esi, server) = (pool.clone(), esi.clone(), server.clone());
        async move {
            // Nobody changes corp while Tranquility is down, and the call would fail. The
            // next tick picks it up.
            if !server.should_poll() {
                return Ok(());
            }
            let changed = refresh(&pool, &esi).await?;
            if changed > 0 {
                tracing::info!(
                    "affiliation: {changed} character(s) changed corporation or alliance"
                );
            }
            Ok::<_, sqlx::Error>(())
        }
    });
}

/// One pass over every character that belongs to an account. Returns how many changed
/// corporation or alliance.
///
/// A batch ESI refuses is skipped rather than failing the pass: those characters keep what
/// they had until the next one, which is no worse than before it ran.
pub async fn refresh(pool: &PgPool, esi: &EsiClient) -> sqlx::Result<u64> {
    let ids =
        sqlx::query_scalar!("select id from characters where user_id is not null order by id")
            .fetch_all(pool)
            .await?;
    let mut changed = 0;
    for chunk in ids.chunks(BATCH) {
        let affiliations = match esi.affiliation(chunk).await {
            Ok(affiliations) => affiliations,
            Err(err) => {
                tracing::warn!("affiliation batch of {} skipped: {err}", chunk.len());
                continue;
            }
        };
        // The columns are foreign keys to the named entities, so any corporation or
        // alliance we have not seen is fetched first. Alliances before corporations, so a
        // new corporation's own alliance link resolves too.
        let alliances: Vec<i64> = affiliations.iter().filter_map(|a| a.alliance_id).collect();
        let corporations: Vec<i64> = affiliations.iter().map(|a| a.corporation_id).collect();
        entities::ensure(pool, esi, EntityKind::Alliance, &alliances).await;
        entities::ensure(pool, esi, EntityKind::Corporation, &corporations).await;
        changed += store(pool, &affiliations).await?;
    }
    Ok(changed)
}

/// Write one batch. Every row is stamped as checked; the count is of the ones whose corp or
/// alliance actually moved.
///
/// An organisation that could not be named is stored as null rather than failing the batch
/// or keeping the old one. That errs towards less access, never more, and the next pass
/// fills it in.
async fn store(pool: &PgPool, affiliations: &[Affiliation]) -> sqlx::Result<u64> {
    let ids: Vec<i64> = affiliations.iter().map(|a| a.character_id).collect();
    let corporations: Vec<i64> = affiliations.iter().map(|a| a.corporation_id).collect();
    let alliances: Vec<Option<i64>> = affiliations.iter().map(|a| a.alliance_id).collect();
    let changed = sqlx::query_scalar!(
        r#"with fresh as (
               select * from unnest($1::bigint[], $2::bigint[], $3::bigint[])
                   as f(id, corporation_id, alliance_id)
           ),
           old as (
               select c.id, c.corporation_id, c.alliance_id
               from characters c join fresh using (id)
           ),
           updated as (
               update characters c set
                   corporation_id = (select id from corporations where id = fresh.corporation_id),
                   alliance_id = (select id from alliances where id = fresh.alliance_id),
                   updated_at = now()
               from fresh
               where c.id = fresh.id
               returning c.id, c.corporation_id, c.alliance_id
           )
           select count(*) as "changed!"
           from updated join old using (id)
           where updated.corporation_id is distinct from old.corporation_id
              or updated.alliance_id is distinct from old.alliance_id"#,
        &ids,
        &corporations,
        &alliances as &[Option<i64>],
    )
    .fetch_one(pool)
    .await?;
    Ok(changed as u64)
}
