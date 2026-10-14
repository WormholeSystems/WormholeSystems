//! Placed solar systems: placing, moving, removing, and aliasing systems on a map. All
//! are Member+ (see [access.md](../../docs/database/access.md)).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use super::access::require_role;
use super::command::{CommandOutput, Effect, MapCommand, Tx, execute};
use super::error::{MapError, Result};
use super::restore::{capture_systems, remove_captured_signatures};
use super::store::placements::{self, NewPlacement};
use super::{Actor, MapEvent, Role};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapSolarSystem {
    pub id: i64,
    pub map_id: i64,
    /// `None` for a ghost: the far side of a wormhole nobody has been through yet.
    pub solar_system_id: Option<i64>,
    pub position_x: f64,
    pub position_y: f64,
    pub alias: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddSystem {
    pub map_id: i64,
    pub solar_system_id: i64,
    pub x: f64,
    pub y: f64,
    pub alias: Option<String>,
}

/// Place a solar system on a map. The system must exist in the SDE and not already be on
/// the map. Adding a system does not touch its persisted details.
pub async fn add_system(pool: &PgPool, actor: Actor, cmd: AddSystem) -> Result<MapSolarSystem> {
    execute(pool, actor, MapCommand::AddSystem(cmd))
        .await?
        .system()
}

pub(super) async fn apply_add_system(tx: &mut Tx<'_>, cmd: AddSystem) -> Result<Effect> {
    let name = sqlx::query_scalar!(
        "select name from solar_systems where id = $1",
        cmd.solar_system_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| MapError::Validation(format!("unknown solar system {}", cmd.solar_system_id)))?;

    let already = sqlx::query_scalar!(
        "select exists(select 1 from map_solar_systems where map_id = $1 and solar_system_id = $2)",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if already {
        return Err(MapError::Conflict("system already on the map".into()));
    }

    let placed = placements::insert(
        tx,
        cmd.map_id,
        &NewPlacement::system(cmd.solar_system_id, cmd.x, cmd.y, cmd.alias.as_deref()),
    )
    .await?;

    let inverse = MapCommand::RemoveSystems(RemoveSystems {
        map_id: cmd.map_id,
        map_solar_system_ids: vec![placed.id],
    });
    let event = MapEvent::SystemAdded {
        map_id: cmd.map_id,
        map_solar_system_id: placed.id,
    };
    Ok(Effect::new(
        "systems.added",
        format!("added {name}"),
        CommandOutput::System(Box::new(placed)),
    )
    .undo_with(inverse)
    .emit(event))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveSystem {
    pub map_id: i64,
    pub map_solar_system_id: i64,
}

/// Remove a system from a map. Cascades the system's signatures and any connections it
/// is an endpoint of; its persisted details survive.
pub async fn remove_system(pool: &PgPool, actor: Actor, cmd: RemoveSystem) -> Result<()> {
    execute(pool, actor, MapCommand::RemoveSystem(cmd)).await?;
    Ok(())
}

/// An effect that changed nothing. No inverse, so it stays out of the undo history: pressing
/// delete on a pinned system should not leave a step that undoes to nothing.
fn kept(kind: &'static str, label: &'static str) -> Effect {
    Effect::new(kind, label, CommandOutput::Count(0)).entries(0)
}

pub(super) async fn apply_remove_system(tx: &mut Tx<'_>, cmd: RemoveSystem) -> Result<Effect> {
    let protected = sqlx::query_scalar!(
        r#"select (is_home or is_pinned) as "protected!" from map_solar_systems
           where id = $1 and map_id = $2"#,
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    if protected {
        return Ok(kept("systems.removed", "kept a protected system"));
    }

    // The unmapped holes hanging off it go with it, and into the same snapshot, so one undo
    // brings both back.
    let mut ids = vec![cmd.map_solar_system_id];
    ids.extend(super::ghost::stranded_ghosts(tx, cmd.map_id, &ids, &[]).await?);

    let snapshot = capture_systems(tx, cmd.map_id, &ids).await?;
    let deleted = placements::delete_unprotected(tx, cmd.map_id, &ids).await?;
    if deleted == 0 {
        return Err(MapError::NotFound);
    }
    remove_captured_signatures(tx, cmd.map_id, &snapshot).await?;
    let label = format!("removed {}", snapshot.label());
    let events: Vec<MapEvent> = ids
        .iter()
        .map(|id| MapEvent::SystemRemoved {
            map_id: cmd.map_id,
            map_solar_system_id: *id,
        })
        .collect();
    Ok(Effect::new("systems.removed", label, CommandOutput::None)
        .undo_with(MapCommand::RestoreSystems(snapshot))
        .emit_all(events))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveSystems {
    pub map_id: i64,
    pub map_solar_system_ids: Vec<i64>,
}

/// Remove several placed systems at once (multi-select delete). Same cascade as
/// [`remove_system`]. Returns the number actually removed; an empty id list is a no-op.
pub async fn remove_systems(pool: &PgPool, actor: Actor, cmd: RemoveSystems) -> Result<u64> {
    execute(pool, actor, MapCommand::RemoveSystems(cmd))
        .await?
        .count()
}

pub(super) async fn apply_remove_systems(tx: &mut Tx<'_>, cmd: RemoveSystems) -> Result<Effect> {
    // Home and pinned systems are deliberate markers that every sweep passes over, so a
    // marquee across the chain must not take them either.
    let rows = sqlx::query!(
        r#"select id, (is_home or is_pinned) as "protected!"
           from map_solar_systems where map_id = $1 and id = any($2)"#,
        cmd.map_id,
        &cmd.map_solar_system_ids,
    )
    .fetch_all(&mut **tx)
    .await?;
    let removable: Vec<i64> = rows.iter().filter(|r| !r.protected).map(|r| r.id).collect();
    let held = rows.len() - removable.len();

    if removable.is_empty() {
        // A protected system is passed over rather than failing the whole delete.
        return Ok(kept("systems.removed", "kept the protected systems"));
    }

    // Same as the single removal: the unmapped holes these leave behind come too.
    let mut removable = removable;
    removable.extend(super::ghost::stranded_ghosts(tx, cmd.map_id, &removable, &[]).await?);

    let snapshot = capture_systems(tx, cmd.map_id, &removable).await?;
    let deleted = placements::delete_unprotected(tx, cmd.map_id, &removable).await?;
    remove_captured_signatures(tx, cmd.map_id, &snapshot).await?;

    let label = match held {
        0 => format!("removed {}", snapshot.label()),
        n => format!("removed {} (kept {n} protected)", snapshot.label()),
    };
    Ok(
        Effect::new("systems.removed", label, CommandOutput::Count(deleted))
            .entries(deleted as i64)
            .undo_with(MapCommand::RestoreSystems(snapshot))
            .emit(MapEvent::MapUpdated { map_id: cmd.map_id }),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ClearMap {
    pub map_id: i64,
}

/// Remove every placed system on a map except the home system and any pinned systems.
/// Connections to removed systems cascade. Returns the number removed.
pub async fn clear_map(pool: &PgPool, actor: Actor, cmd: ClearMap) -> Result<u64> {
    execute(pool, actor, MapCommand::ClearMap(cmd))
        .await?
        .count()
}

pub(super) async fn apply_clear_map(tx: &mut Tx<'_>, cmd: ClearMap) -> Result<Effect> {
    let doomed: Vec<i64> = sqlx::query_scalar!(
        "select id from map_solar_systems
         where map_id = $1 and not is_home and not is_pinned",
        cmd.map_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    let snapshot = capture_systems(tx, cmd.map_id, &doomed).await?;
    let deleted = placements::delete_all_unprotected(tx, cmd.map_id).await?;
    Ok(Effect::new(
        "map.cleared",
        format!("cleared the map ({deleted} systems)"),
        CommandOutput::Count(deleted),
    )
    .entries(deleted as i64)
    .undo_with(MapCommand::RestoreSystems(snapshot))
    .emit(MapEvent::MapUpdated { map_id: cmd.map_id }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MoveSystem {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub x: f64,
    pub y: f64,
}

/// Move a placed system to a new position.
pub async fn move_system(pool: &PgPool, actor: Actor, cmd: MoveSystem) -> Result<()> {
    execute(pool, actor, MapCommand::MoveSystem(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_move_system(tx: &mut Tx<'_>, cmd: MoveSystem) -> Result<Effect> {
    let before = sqlx::query!(
        "select position_x, position_y, is_pinned from map_solar_systems
         where id = $1 and map_id = $2",
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    // The server holds the drag lock too, rather than trusting the client. Home is not
    // pinned by being home: it can be dragged around like anything else.
    if before.is_pinned {
        return Ok(kept("systems.moved", "a pinned system stayed put"));
    }
    placements::move_unpinned(tx, cmd.map_id, cmd.map_solar_system_id, cmd.x, cmd.y).await?;
    let inverse = MapCommand::MoveSystem(MoveSystem {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        x: before.position_x,
        y: before.position_y,
    });
    Ok(
        Effect::new("systems.moved", "moved a system", CommandOutput::None)
            .undo_with(inverse)
            .emit(MapEvent::SystemMoved {
                map_id: cmd.map_id,
                map_solar_system_id: cmd.map_solar_system_id,
            }),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SystemMove {
    pub map_solar_system_id: i64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MoveSystems {
    pub map_id: i64,
    pub moves: Vec<SystemMove>,
}

/// Move several placed systems at once (multi-drag), in one transaction.
pub async fn move_systems(pool: &PgPool, actor: Actor, cmd: MoveSystems) -> Result<()> {
    execute(pool, actor, MapCommand::MoveSystems(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_move_systems(tx: &mut Tx<'_>, cmd: MoveSystems) -> Result<Effect> {
    let ids: Vec<i64> = cmd.moves.iter().map(|m| m.map_solar_system_id).collect();
    // Only the ones that will actually move, so undo puts back exactly what shifted.
    let before: Vec<SystemMove> = sqlx::query!(
        "select id, position_x, position_y from map_solar_systems
         where map_id = $1 and id = any($2) and not is_pinned",
        cmd.map_id,
        &ids,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| SystemMove {
        map_solar_system_id: r.id,
        x: r.position_x,
        y: r.position_y,
    })
    .collect();

    if before.is_empty() {
        return Ok(kept("systems.moved", "the pinned systems stayed put"));
    }

    let movable: std::collections::HashSet<i64> =
        before.iter().map(|m| m.map_solar_system_id).collect();
    for m in cmd
        .moves
        .iter()
        .filter(|m| movable.contains(&m.map_solar_system_id))
    {
        placements::move_unpinned(tx, cmd.map_id, m.map_solar_system_id, m.x, m.y).await?;
    }
    let count = movable.len();
    let label = if count == 1 {
        "moved a system".to_string()
    } else {
        format!("moved {count} systems")
    };
    let inverse = MapCommand::MoveSystems(MoveSystems {
        map_id: cmd.map_id,
        moves: before,
    });
    Ok(Effect::new("systems.moved", label, CommandOutput::None)
        .entries(count as i64)
        .undo_with(inverse)
        .emit(MapEvent::MapUpdated { map_id: cmd.map_id }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetAlias {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub alias: Option<String>,
}

/// Set or clear a placement's ephemeral alias.
pub async fn set_alias(pool: &PgPool, actor: Actor, cmd: SetAlias) -> Result<()> {
    execute(pool, actor, MapCommand::SetAlias(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_set_alias(tx: &mut Tx<'_>, cmd: SetAlias) -> Result<Effect> {
    let before = sqlx::query_scalar!(
        "select alias from map_solar_systems where id = $1 and map_id = $2",
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    sqlx::query!(
        "update map_solar_systems set alias = $1 where id = $2 and map_id = $3",
        cmd.alias.as_deref(),
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?;
    let label = match &cmd.alias {
        Some(alias) => format!("renamed a system to {alias}"),
        None => "cleared a system alias".to_string(),
    };
    let inverse = MapCommand::SetAlias(SetAlias {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        alias: before,
    });
    Ok(Effect::new("systems.aliased", label, CommandOutput::None)
        .undo_with(inverse)
        .emit(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.map_solar_system_id,
        }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetStatus {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub status: super::SystemStatus,
}

/// Set a placed system's intel status. Upserts the persisted details row, keyed by the
/// placement's `(map_id, solar_system_id)`.
pub async fn set_status(pool: &PgPool, actor: Actor, cmd: SetStatus) -> Result<()> {
    execute(pool, actor, MapCommand::SetStatus(cmd)).await?;
    Ok(())
}

/// The system behind a placement, or a plain refusal when it is still a ghost. Intel is
/// keyed by system rather than by placement, so there is nowhere to put it yet.
async fn system_behind(tx: &mut Tx<'_>, map_id: i64, map_solar_system_id: i64) -> Result<i64> {
    sqlx::query_scalar!(
        "select solar_system_id from map_solar_systems where id = $1 and map_id = $2",
        map_solar_system_id,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?
    .ok_or_else(|| {
        MapError::Validation("assign a system to that hole before recording intel on it".into())
    })
}

pub(super) async fn apply_set_status(tx: &mut Tx<'_>, cmd: SetStatus) -> Result<Effect> {
    system_behind(tx, cmd.map_id, cmd.map_solar_system_id).await?;
    let before = sqlx::query_scalar!(
        r#"select coalesce(d.status, 'unknown') as "status!: super::SystemStatus"
           from map_solar_systems mss
           left join map_solar_system_details d
               on d.map_id = mss.map_id and d.solar_system_id = mss.solar_system_id
           where mss.id = $1 and mss.map_id = $2"#,
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;

    sqlx::query!(
        "insert into map_solar_system_details (map_id, solar_system_id, status)
         select map_id, solar_system_id, $3 from map_solar_systems where id = $2 and map_id = $1
         on conflict (map_id, solar_system_id)
             do update set status = excluded.status, updated_at = now()",
        cmd.map_id,
        cmd.map_solar_system_id,
        cmd.status,
    )
    .execute(&mut **tx)
    .await?;
    let inverse = MapCommand::SetStatus(SetStatus {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        status: before,
    });
    Ok(Effect::new(
        "systems.status",
        format!("set a system to {}", cmd.status.as_str()),
        CommandOutput::None,
    )
    .undo_with(inverse)
    .emit(MapEvent::SystemDetailsChanged {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
    }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetOccupier {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub occupier: Option<String>,
}

/// Set or clear who occupies a placed system (free text, like the alias). Upserts the
/// persisted details row.
pub async fn set_occupier(pool: &PgPool, actor: Actor, cmd: SetOccupier) -> Result<()> {
    execute(pool, actor, MapCommand::SetOccupier(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_set_occupier(tx: &mut Tx<'_>, cmd: SetOccupier) -> Result<Effect> {
    system_behind(tx, cmd.map_id, cmd.map_solar_system_id).await?;
    let before = detail_text(
        tx,
        cmd.map_id,
        cmd.map_solar_system_id,
        DetailColumn::Occupier,
    )
    .await?;
    sqlx::query!(
        "insert into map_solar_system_details (map_id, solar_system_id, occupying_group)
         select map_id, solar_system_id, $3 from map_solar_systems where id = $2 and map_id = $1
         on conflict (map_id, solar_system_id)
             do update set occupying_group = excluded.occupying_group, updated_at = now()",
        cmd.map_id,
        cmd.map_solar_system_id,
        cmd.occupier.as_deref(),
    )
    .execute(&mut **tx)
    .await?;
    let label = match &cmd.occupier {
        Some(occupier) => format!("set the occupier to {occupier}"),
        None => "cleared an occupier".to_string(),
    };
    let inverse = MapCommand::SetOccupier(SetOccupier {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        occupier: before,
    });
    Ok(Effect::new("systems.occupier", label, CommandOutput::None)
        .undo_with(inverse)
        .emit(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.map_solar_system_id,
        }))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetNotes {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub notes: Option<String>,
}

/// Set or clear a placed system's notes (markdown free text). Member+ only, like every
/// other intel write; upserts the persisted details row.
pub async fn set_notes(pool: &PgPool, actor: Actor, cmd: SetNotes) -> Result<()> {
    execute(pool, actor, MapCommand::SetNotes(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_set_notes(tx: &mut Tx<'_>, cmd: SetNotes) -> Result<Effect> {
    system_behind(tx, cmd.map_id, cmd.map_solar_system_id).await?;
    let before = detail_text(tx, cmd.map_id, cmd.map_solar_system_id, DetailColumn::Notes).await?;
    sqlx::query!(
        "insert into map_solar_system_details (map_id, solar_system_id, notes)
         select map_id, solar_system_id, $3 from map_solar_systems where id = $2 and map_id = $1
         on conflict (map_id, solar_system_id)
             do update set notes = excluded.notes, updated_at = now()",
        cmd.map_id,
        cmd.map_solar_system_id,
        cmd.notes.as_deref(),
    )
    .execute(&mut **tx)
    .await?;
    let inverse = MapCommand::SetNotes(SetNotes {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        notes: before,
    });
    Ok(
        Effect::new("systems.notes", "edited notes", CommandOutput::None)
            .undo_with(inverse)
            .emit(MapEvent::SystemDetailsChanged {
                map_id: cmd.map_id,
                map_solar_system_id: cmd.map_solar_system_id,
            }),
    )
}

/// A placed system's member-gated intel details (currently just the notes). Viewers never
/// receive this: the read itself requires Member.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SystemDetails {
    pub notes: Option<String>,
}

/// The member-gated details for a placement. `Forbidden` for viewers.
pub async fn system_details(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    map_solar_system_id: i64,
) -> Result<SystemDetails> {
    require_role(pool, map_id, actor.user_id, Role::Member).await?;
    let row = sqlx::query!(
        "select d.notes
         from map_solar_systems mss
         left join map_solar_system_details d
             on d.map_id = mss.map_id and d.solar_system_id = mss.solar_system_id
         where mss.id = $2 and mss.map_id = $1",
        map_id,
        map_solar_system_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)?;
    Ok(SystemDetails { notes: row.notes })
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetHome {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub value: bool,
}

/// Mark a placement as the map's home system (or clear it). A map has at most one home (a
/// partial unique index enforces it), so setting a new home first clears the previous one.
pub async fn set_home(pool: &PgPool, actor: Actor, cmd: SetHome) -> Result<()> {
    execute(pool, actor, MapCommand::SetHome(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_set_home(tx: &mut Tx<'_>, cmd: SetHome) -> Result<Effect> {
    let previous: Option<i64> = sqlx::query_scalar!(
        "select id from map_solar_systems where map_id = $1 and is_home",
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?;
    if cmd.value {
        sqlx::query!(
            "update map_solar_systems set is_home = false where map_id = $1 and is_home",
            cmd.map_id,
        )
        .execute(&mut **tx)
        .await?;
    }
    let updated = sqlx::query!(
        "update map_solar_systems set is_home = $1 where id = $2 and map_id = $3",
        cmd.value,
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(MapError::NotFound);
    }
    // Undo puts the previous home back rather than just clearing this one.
    let inverse = MapCommand::SetHome(match previous {
        Some(id) => SetHome {
            map_id: cmd.map_id,
            map_solar_system_id: id,
            value: true,
        },
        None => SetHome {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.map_solar_system_id,
            value: false,
        },
    });
    let label = if cmd.value {
        "set the home system"
    } else {
        "cleared the home system"
    };
    let mut effect = Effect::new("systems.home", label, CommandOutput::None)
        .undo_with(inverse)
        .emit(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.map_solar_system_id,
        });
    if let Some(previous) = previous
        && previous != cmd.map_solar_system_id
    {
        effect = effect.emit(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: previous,
        });
    }
    Ok(effect)
}

/// How many rally points a map holds at once. Setting one more replaces the oldest.
pub const MAX_RALLY_POINTS: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetRally {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub value: bool,
}

/// Mark a placement as one of the map's rally points, or clear just that one. A map holds
/// up to [`MAX_RALLY_POINTS`]: setting one more clears the oldest, and setting one that
/// already is a rally point makes it the newest.
pub async fn set_rally(pool: &PgPool, actor: Actor, cmd: SetRally) -> Result<()> {
    execute(pool, actor, MapCommand::SetRally(cmd)).await?;
    Ok(())
}

/// When a placement became a rally point, `None` when it is not one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RallyMark {
    pub map_solar_system_id: i64,
    pub rallied_at: Option<DateTime<Utc>>,
}

/// Put rally points back exactly as they were. Internal, the inverse of [`SetRally`]: a set
/// can push the oldest rally point out as well as touch its own, and each has to return to
/// its old place in the order so the next eviction picks the one it would have.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRallies {
    pub map_id: i64,
    pub marks: Vec<RallyMark>,
}

pub(super) async fn apply_set_rally(tx: &mut Tx<'_>, cmd: SetRally) -> Result<Effect> {
    // Two members setting rally points at once would each see room for theirs and leave
    // the map over the cap, so the map is held while counting.
    sqlx::query!(
        "select id from maps where id = $1 for no key update",
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?;
    let rallied_at = sqlx::query_scalar!(
        "select rallied_at from map_solar_systems where id = $1 and map_id = $2",
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    let mut before = vec![RallyMark {
        map_solar_system_id: cmd.map_solar_system_id,
        rallied_at,
    }];

    if cmd.value {
        let others = sqlx::query!(
            r#"select id, rallied_at as "rallied_at!" from map_solar_systems
               where map_id = $1 and rallied_at is not null and id <> $2
               order by rallied_at, id"#,
            cmd.map_id,
            cmd.map_solar_system_id,
        )
        .fetch_all(&mut **tx)
        .await?;
        // Room for this one, and the oldest of the rest make way.
        let excess = (others.len() + 1).saturating_sub(MAX_RALLY_POINTS);
        let evicted: Vec<RallyMark> = others
            .into_iter()
            .take(excess)
            .map(|o| RallyMark {
                map_solar_system_id: o.id,
                rallied_at: Some(o.rallied_at),
            })
            .collect();
        if !evicted.is_empty() {
            let ids: Vec<i64> = evicted.iter().map(|m| m.map_solar_system_id).collect();
            sqlx::query!(
                "update map_solar_systems set rallied_at = null
                 where map_id = $1 and id = any($2)",
                cmd.map_id,
                &ids,
            )
            .execute(&mut **tx)
            .await?;
        }
        before.extend(evicted);
    }

    sqlx::query!(
        "update map_solar_systems set rallied_at = case when $1 then now() end
         where id = $2 and map_id = $3",
        cmd.value,
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?;

    let label = if cmd.value {
        "set a rally point"
    } else {
        "cleared a rally point"
    };
    let events = rally_events(cmd.map_id, &before);
    Ok(Effect::new("systems.rally", label, CommandOutput::None)
        .undo_with(MapCommand::RestoreRallies(RestoreRallies {
            map_id: cmd.map_id,
            marks: before,
        }))
        .emit_all(events))
}

pub(super) async fn apply_restore_rallies(tx: &mut Tx<'_>, cmd: RestoreRallies) -> Result<Effect> {
    let mut before = Vec::with_capacity(cmd.marks.len());
    for mark in cmd.marks {
        let current = sqlx::query_scalar!(
            "select rallied_at from map_solar_systems where id = $1 and map_id = $2",
            mark.map_solar_system_id,
            cmd.map_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        // A placement that has left the map since has nothing to put back.
        let Some(current) = current else { continue };
        sqlx::query!(
            "update map_solar_systems set rallied_at = $1 where id = $2 and map_id = $3",
            mark.rallied_at,
            mark.map_solar_system_id,
            cmd.map_id,
        )
        .execute(&mut **tx)
        .await?;
        before.push(RallyMark {
            map_solar_system_id: mark.map_solar_system_id,
            rallied_at: current,
        });
    }
    let events = rally_events(cmd.map_id, &before);
    Ok(Effect::new(
        "systems.rally",
        "changed the rally points",
        CommandOutput::None,
    )
    .undo_with(MapCommand::RestoreRallies(RestoreRallies {
        map_id: cmd.map_id,
        marks: before,
    }))
    .emit_all(events))
}

fn rally_events(map_id: i64, marks: &[RallyMark]) -> Vec<MapEvent> {
    marks
        .iter()
        .map(|m| MapEvent::SystemDetailsChanged {
            map_id,
            map_solar_system_id: m.map_solar_system_id,
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetPinned {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub value: bool,
}

/// Pin or unpin a placement. Pinned systems are drag-locked client-side and survive
/// "clear map". Any number of systems may be pinned.
pub async fn set_pinned(pool: &PgPool, actor: Actor, cmd: SetPinned) -> Result<()> {
    execute(pool, actor, MapCommand::SetPinned(cmd)).await?;
    Ok(())
}

pub(super) async fn apply_set_pinned(tx: &mut Tx<'_>, cmd: SetPinned) -> Result<Effect> {
    // Pinning holds the node still, roots the tree layout, and shields it from every sweep,
    // none of which mean anything for a hole nobody has been through.
    if cmd.value {
        let system = sqlx::query_scalar!(
            "select solar_system_id from map_solar_systems where id = $1 and map_id = $2",
            cmd.map_solar_system_id,
            cmd.map_id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(MapError::NotFound)?;
        if system.is_none() {
            return Err(MapError::Validation(
                "assign a system to that hole before pinning it".into(),
            ));
        }
    }
    let updated = sqlx::query!(
        "update map_solar_systems set is_pinned = $1 where id = $2 and map_id = $3",
        cmd.value,
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if updated == 0 {
        return Err(MapError::NotFound);
    }
    let inverse = MapCommand::SetPinned(SetPinned {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        value: !cmd.value,
    });
    let label = if cmd.value {
        "pinned a system"
    } else {
        "unpinned a system"
    };
    Ok(Effect::new("systems.pinned", label, CommandOutput::None)
        .undo_with(inverse)
        .emit(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.map_solar_system_id,
        }))
}

/// The current occupier / notes value, erroring when the placement is missing. Two
/// literal queries rather than a built string, so the SQL stays checked at compile time.
async fn detail_text(
    tx: &mut Tx<'_>,
    map_id: i64,
    map_solar_system_id: i64,
    column: DetailColumn,
) -> Result<Option<String>> {
    let value = match column {
        DetailColumn::Occupier => {
            sqlx::query_scalar!(
                "select d.occupying_group
             from map_solar_systems mss
             left join map_solar_system_details d
                 on d.map_id = mss.map_id and d.solar_system_id = mss.solar_system_id
             where mss.id = $1 and mss.map_id = $2",
                map_solar_system_id,
                map_id,
            )
            .fetch_optional(&mut **tx)
            .await?
        }
        DetailColumn::Notes => {
            sqlx::query_scalar!(
                "select d.notes
             from map_solar_systems mss
             left join map_solar_system_details d
                 on d.map_id = mss.map_id and d.solar_system_id = mss.solar_system_id
             where mss.id = $1 and mss.map_id = $2",
                map_solar_system_id,
                map_id,
            )
            .fetch_optional(&mut **tx)
            .await?
        }
    };
    value.ok_or(MapError::NotFound)
}

enum DetailColumn {
    Occupier,
    Notes,
}
