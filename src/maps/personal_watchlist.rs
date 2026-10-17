//! A user's own watchlist on a map: systems they want a standing jump count to that the
//! rest of the map does not need to see. Same shape as the shared [`super::watchlist`],
//! but it is nobody else's business, so it is not a map command: no history, no undo, and
//! only the user's own tabs hear about a change. Anyone who can see the map may keep one.
//!
//! After Hiryus's personal watched systems in legacy
//! (wormholesystems/wormholesystems-legacy#101).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::require_role;
use super::error::{MapError, Result};
use super::watchlist::WatchlistEntry;
use super::{Actor, Role};

/// The caller's own entries on a map. Viewer+.
pub async fn list(pool: &PgPool, actor: Actor, map_id: i64) -> Result<Vec<WatchlistEntry>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    let entries = sqlx::query_as!(
        WatchlistEntry,
        "select id, map_id, solar_system_id, is_pinned
         from map_personal_watchlist where map_id = $1 and user_id = $2 order by id",
        map_id,
        actor.user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(entries)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddPersonalWatchlistEntry {
    pub solar_system_id: i64,
}

/// Watch a system for yourself. Idempotent. Viewer+.
pub async fn add(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    body: AddPersonalWatchlistEntry,
) -> Result<WatchlistEntry> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    let entry = sqlx::query_as!(
        WatchlistEntry,
        "insert into map_personal_watchlist (map_id, user_id, solar_system_id)
         values ($1, $2, $3)
         on conflict (map_id, user_id, solar_system_id)
             do update set solar_system_id = excluded.solar_system_id
         returning id, map_id, solar_system_id, is_pinned",
        map_id,
        actor.user_id,
        body.solar_system_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(entry)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetPersonalWatchlistPinned {
    pub entry_id: i64,
    pub value: bool,
}

/// Pin or unpin one of your own entries, which makes it a route quick-pick for you.
pub async fn set_pinned(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    body: SetPersonalWatchlistPinned,
) -> Result<WatchlistEntry> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    sqlx::query_as!(
        WatchlistEntry,
        "update map_personal_watchlist set is_pinned = $1
         where id = $2 and map_id = $3 and user_id = $4
         returning id, map_id, solar_system_id, is_pinned",
        body.value,
        body.entry_id,
        map_id,
        actor.user_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemovePersonalWatchlistEntry {
    pub entry_id: i64,
}

/// Stop watching one of your own entries. Someone else's is not found, not forbidden.
pub async fn remove(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    body: RemovePersonalWatchlistEntry,
) -> Result<()> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    let removed = sqlx::query!(
        "delete from map_personal_watchlist where id = $1 and map_id = $2 and user_id = $3",
        body.entry_id,
        map_id,
        actor.user_id,
    )
    .execute(pool)
    .await?
    .rows_affected();
    if removed == 0 {
        return Err(MapError::NotFound);
    }
    Ok(())
}
