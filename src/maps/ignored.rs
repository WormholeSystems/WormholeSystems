//! The map's ignore list: systems everyone steers around (legacy
//! `map_ignored_solarsystems`). The client router leaves them out, the server searches
//! never pass through them, and jump tracking never places them. Shared and Manager+,
//! unlike the viewer's own route-around list, which lives in the browser.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::require_role;
use super::command::{CommandOutput, Effect, MapCommand, Sequence, Tx, run};
use super::error::{MapError, Result};
use super::{Actor, MapEvent, Role};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IgnoredSystem {
    pub id: i64,
    pub map_id: i64,
    pub solar_system_id: i64,
}

/// Every ignored system on a map. Viewer+.
pub async fn list_ignored_systems(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
) -> Result<Vec<IgnoredSystem>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    read_ignored_systems(pool, map_id).await
}

/// The list itself, for callers that have already settled who is asking.
pub async fn read_ignored_systems(pool: &PgPool, map_id: i64) -> Result<Vec<IgnoredSystem>> {
    let rows = sqlx::query_as!(
        IgnoredSystem,
        "select id, map_id, solar_system_id from map_ignored_solar_systems
         where map_id = $1 order by id",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Just the ids, for the searches that route around them.
pub async fn ignored_ids(pool: &PgPool, map_id: i64) -> sqlx::Result<Vec<i64>> {
    sqlx::query_scalar!(
        "select solar_system_id from map_ignored_solar_systems where map_id = $1",
        map_id,
    )
    .fetch_all(pool)
    .await
}

pub(super) async fn is_ignored_tx(
    tx: &mut Tx<'_>,
    map_id: i64,
    solar_system_id: i64,
) -> Result<bool> {
    let ignored = sqlx::query_scalar!(
        "select exists(
             select 1 from map_ignored_solar_systems
             where map_id = $1 and solar_system_id = $2
         )",
        map_id,
        solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(ignored.unwrap_or(false))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddIgnoredSystem {
    pub map_id: i64,
    pub solar_system_id: i64,
}

/// Keep a system off the map's routes and out of jump tracking. Idempotent. Manager+.
pub async fn add_ignored_system(
    pool: &PgPool,
    actor: Actor,
    cmd: AddIgnoredSystem,
) -> Result<IgnoredSystem> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_add(tx: &mut Tx<'_>, cmd: AddIgnoredSystem) -> Result<Effect> {
    let known = sqlx::query_scalar!(
        "select exists(select 1 from solar_systems where id = $1)",
        cmd.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if !known {
        return Err(MapError::Validation(format!(
            "unknown solar system {}",
            cmd.solar_system_id
        )));
    }
    let row = sqlx::query_as!(
        IgnoredSystem,
        "insert into map_ignored_solar_systems (map_id, solar_system_id)
         values ($1, $2)
         on conflict (map_id, solar_system_id) do update set solar_system_id = excluded.solar_system_id
         returning id, map_id, solar_system_id",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    let inverse = MapCommand::RemoveIgnoredSystem(RemoveIgnoredSystem {
        map_id: cmd.map_id,
        solar_system_id: cmd.solar_system_id,
    });
    Ok(Effect::new(
        "ignored.added",
        "ignored a system",
        CommandOutput::Ignored(Box::new(row)),
    )
    .undo_with(inverse)
    .emit(MapEvent::IgnoredSystemsChanged { map_id: cmd.map_id }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveIgnoredSystem {
    pub map_id: i64,
    pub solar_system_id: i64,
}

/// Let a system back onto routes and the chain. Manager+.
pub async fn remove_ignored_system(
    pool: &PgPool,
    actor: Actor,
    cmd: RemoveIgnoredSystem,
) -> Result<()> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_remove(tx: &mut Tx<'_>, cmd: RemoveIgnoredSystem) -> Result<Effect> {
    let removed = sqlx::query!(
        "delete from map_ignored_solar_systems where map_id = $1 and solar_system_id = $2",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if removed == 0 {
        return Err(MapError::NotFound);
    }
    let inverse = MapCommand::AddIgnoredSystem(AddIgnoredSystem {
        map_id: cmd.map_id,
        solar_system_id: cmd.solar_system_id,
    });
    Ok(Effect::new(
        "ignored.removed",
        "stopped ignoring a system",
        CommandOutput::None,
    )
    .undo_with(inverse)
    .emit(MapEvent::IgnoredSystemsChanged { map_id: cmd.map_id }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ClearIgnoredSystems {
    pub map_id: i64,
}

/// Empty the list. Manager+. Returns how many rows went.
pub async fn clear_ignored_systems(
    pool: &PgPool,
    actor: Actor,
    cmd: ClearIgnoredSystems,
) -> Result<u64> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_clear(tx: &mut Tx<'_>, cmd: ClearIgnoredSystems) -> Result<Effect> {
    let removed: Vec<i64> = sqlx::query_scalar!(
        "delete from map_ignored_solar_systems where map_id = $1 returning solar_system_id",
        cmd.map_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    let count = removed.len() as u64;
    let inverse = MapCommand::Sequence(Sequence {
        map_id: cmd.map_id,
        steps: removed
            .into_iter()
            .map(|solar_system_id| {
                MapCommand::AddIgnoredSystem(AddIgnoredSystem {
                    map_id: cmd.map_id,
                    solar_system_id,
                })
            })
            .collect(),
    });
    Ok(Effect::new(
        "ignored.cleared",
        "cleared the ignore list",
        CommandOutput::Count(count),
    )
    .undo_with(inverse)
    .emit(MapEvent::IgnoredSystemsChanged { map_id: cmd.map_id }))
}
