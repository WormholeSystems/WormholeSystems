//! The per-map watchlist: systems whose jump distance the navigation panel tracks
//! (legacy `map_route_solarsystems`). Rows are map-scoped and shared; routing against
//! them happens client-side. Mutations are Member+; reads Viewer+.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::require_role;
use super::command::{CommandOutput, Effect, MapCommand, Tx, run};
use super::error::{MapError, Result};
use super::{Actor, MapEvent, Role};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct WatchlistEntry {
    pub id: i64,
    pub map_id: i64,
    pub solar_system_id: i64,
    pub is_pinned: bool,
}

/// Every watchlist entry on a map. Viewer+.
pub async fn list_watchlist(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
) -> Result<Vec<WatchlistEntry>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    read_watchlist(pool, map_id).await
}

/// The watchlist itself, for callers that have already settled who is asking (a watcher
/// on a shared map has no grant to check).
pub async fn read_watchlist(pool: &PgPool, map_id: i64) -> Result<Vec<WatchlistEntry>> {
    let entries = sqlx::query_as!(
        WatchlistEntry,
        "select id, map_id, solar_system_id, is_pinned
         from map_watchlist where map_id = $1 order by id",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(entries)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddWatchlistEntry {
    pub map_id: i64,
    pub solar_system_id: i64,
}

/// Watch a system on this map. Idempotent (unique per map+system). Member+.
pub async fn add_watchlist_entry(
    pool: &PgPool,
    actor: Actor,
    cmd: AddWatchlistEntry,
) -> Result<WatchlistEntry> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_add_entry(tx: &mut Tx<'_>, cmd: AddWatchlistEntry) -> Result<Effect> {
    let entry = sqlx::query_as!(
        WatchlistEntry,
        "insert into map_watchlist (map_id, solar_system_id)
         values ($1, $2)
         on conflict (map_id, solar_system_id) do update set solar_system_id = excluded.solar_system_id
         returning id, map_id, solar_system_id, is_pinned",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    let inverse = MapCommand::RemoveWatchlistEntry(RemoveWatchlistEntry {
        map_id: cmd.map_id,
        entry_id: entry.id,
    });
    Ok(Effect::new(
        "watchlist.added",
        "watched a system",
        CommandOutput::Watchlist(Box::new(entry)),
    )
    .undo_with(inverse)
    .emit(MapEvent::WatchlistChanged { map_id: cmd.map_id }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetWatchlistPinned {
    pub map_id: i64,
    pub entry_id: i64,
    pub value: bool,
}

/// Pin/unpin a watchlist entry (pinned entries surface as route quick-picks). Member+.
pub async fn set_watchlist_pinned(
    pool: &PgPool,
    actor: Actor,
    cmd: SetWatchlistPinned,
) -> Result<WatchlistEntry> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_set_pinned(tx: &mut Tx<'_>, cmd: SetWatchlistPinned) -> Result<Effect> {
    let entry = sqlx::query_as!(
        WatchlistEntry,
        "update map_watchlist set is_pinned = $1 where id = $2 and map_id = $3
         returning id, map_id, solar_system_id, is_pinned",
        cmd.value,
        cmd.entry_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    let inverse = MapCommand::SetWatchlistPinned(SetWatchlistPinned {
        map_id: cmd.map_id,
        entry_id: cmd.entry_id,
        value: !cmd.value,
    });
    let label = if cmd.value {
        "pinned a watchlist entry"
    } else {
        "unpinned a watchlist entry"
    };
    Ok(Effect::new(
        label_kind(cmd.value),
        label,
        CommandOutput::Watchlist(Box::new(entry)),
    )
    .undo_with(inverse)
    .emit(MapEvent::WatchlistChanged { map_id: cmd.map_id }))
}

fn label_kind(pinned: bool) -> &'static str {
    if pinned {
        "watchlist.pinned"
    } else {
        "watchlist.unpinned"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveWatchlistEntry {
    pub map_id: i64,
    pub entry_id: i64,
}

/// Stop watching. Member+.
pub async fn remove_watchlist_entry(
    pool: &PgPool,
    actor: Actor,
    cmd: RemoveWatchlistEntry,
) -> Result<()> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_remove_entry(
    tx: &mut Tx<'_>,
    cmd: RemoveWatchlistEntry,
) -> Result<Effect> {
    let row = sqlx::query!(
        "delete from map_watchlist where id = $1 and map_id = $2
         returning solar_system_id, is_pinned, created_at",
        cmd.entry_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    let inverse = MapCommand::RestoreWatchlistEntry(RestoreWatchlistEntry {
        map_id: cmd.map_id,
        entry_id: cmd.entry_id,
        solar_system_id: row.solar_system_id,
        is_pinned: row.is_pinned,
        created_at: row.created_at,
    });
    Ok(Effect::new(
        "watchlist.removed",
        "unwatched a system",
        CommandOutput::None,
    )
    .undo_with(inverse)
    .emit(MapEvent::WatchlistChanged { map_id: cmd.map_id }))
}

/// Put a removed entry back as it was, under its own id so the steps that name it still
/// find it. Internal, the inverse of [`RemoveWatchlistEntry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreWatchlistEntry {
    pub map_id: i64,
    pub entry_id: i64,
    pub solar_system_id: i64,
    pub is_pinned: bool,
    pub created_at: DateTime<Utc>,
}

pub(super) async fn apply_restore_entry(
    tx: &mut Tx<'_>,
    cmd: RestoreWatchlistEntry,
) -> Result<Effect> {
    sqlx::query!(
        "insert into map_watchlist (id, map_id, solar_system_id, is_pinned, created_at)
         overriding system value
         values ($1, $2, $3, $4, $5)
         on conflict do nothing",
        cmd.entry_id,
        cmd.map_id,
        cmd.solar_system_id,
        cmd.is_pinned,
        cmd.created_at,
    )
    .execute(&mut **tx)
    .await?;
    let inverse = MapCommand::RemoveWatchlistEntry(RemoveWatchlistEntry {
        map_id: cmd.map_id,
        entry_id: cmd.entry_id,
    });
    Ok(Effect::new(
        "watchlist.added",
        "watched a system again",
        CommandOutput::None,
    )
    .undo_with(inverse)
    .emit(MapEvent::WatchlistChanged { map_id: cmd.map_id }))
}
