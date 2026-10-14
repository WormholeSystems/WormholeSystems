//! Connections between placed systems: adding, removing, and marking the wormhole
//! life-cycle state of edges. All are Member+ (see [access.md](../../docs/database/access.md)).
//!
//! A connection carries its own `mass_status` / `time_status` / `size` so a hole can be
//! marked massed/EOL before any signature is linked. Once one is linked, the `map_*_sync`
//! DB triggers (migration 0009) keep connection and signatures in lock-step: worst-wins on
//! link, verbatim on edit. See the [sync spec](../../docs/database/mapping.md).
//!
//! The clock does part of the marking itself: [`start_lifecycle`] ages wormhole edges to
//! EOL and critical from how long they have been on the map, and removes the ones too old
//! to still exist. Spec in [processes.md](../../docs/processes.md#connection-life-cycle).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ConnectionType, MapEvent, MassStatus, TimeStatus, WormholeSize};

use sqlx::PgPool;

use super::Actor;
use super::command::{CommandOutput, Effect, MapCommand, Tx, run};
use super::error::{MapError, Result};
use super::store::connections::{self, NewConnection};
use super::store::placements;

mod lifecycle;

pub use lifecycle::*;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapConnection {
    pub id: i64,
    pub map_id: i64,
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionType,
    /// Wormhole life-cycle state. `None` = unknown; ignored for stargate edges. Synced with
    /// linked signatures by the DB triggers, so reads here already reflect the merged state.
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub size: Option<WormholeSize>,
    /// Exclude this hole from mass bookkeeping (legacy flag; stored, not yet consumed).
    pub preserve_mass: bool,
    /// When `time_status` last changed (DB trigger), for "EOL since" displays.
    pub time_status_updated_at: Option<DateTime<Utc>>,
    /// How long a hole between these two systems lives, from the class pair (see the
    /// [life-cycle spec](../../docs/processes.md#connection-life-cycle)). `None` for
    /// stargates. With `created_at`, the client's countdown.
    pub lifetime_hours: Option<i64>,
    /// Full jump-log aggregates (the log itself is fetched separately, capped at 10).
    pub jumps_count: i64,
    pub jumps_mass_sum: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A map's connections with their jump aggregates, oldest first; `connection_id` narrows
/// the read to one edge. The one place the stats subselects are written: the map read and
/// every mutation's returned row all come through here.
pub(super) async fn connections_with_stats(
    exec: impl sqlx::PgExecutor<'_>,
    map_id: i64,
    connection_id: Option<i64>,
) -> sqlx::Result<Vec<MapConnection>> {
    let rows = sqlx::query!(
        r#"select c.id, c.map_id, c.from_system, c.to_system, c.type as kind,
                  c.mass_status,
                  c.time_status,
                  c.size,
                  (select count(*) from map_connection_jumps j
                   where j.connection_id = c.id) as "jumps_count!",
                  coalesce((select sum(j.mass) from map_connection_jumps j
                            where j.connection_id = c.id), 0)::bigint as "jumps_mass_sum!",
                  c.preserve_mass, c.time_status_updated_at, c.created_at, c.updated_at,
                  fs.wormhole_class_id as "from_class?", ts.wormhole_class_id as "to_class?"
           from map_connections c
           join map_solar_systems fm on fm.id = c.from_system
           left join solar_systems fs on fs.id = fm.solar_system_id
           join map_solar_systems tm on tm.id = c.to_system
           left join solar_systems ts on ts.id = tm.solar_system_id
           where c.map_id = $1 and ($2::bigint is null or c.id = $2)
           order by c.id"#,
        map_id,
        connection_id,
    )
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| MapConnection {
            id: r.id,
            map_id: r.map_id,
            from_system: r.from_system,
            to_system: r.to_system,
            kind: r.kind,
            mass_status: r.mass_status,
            time_status: r.time_status,
            size: r.size,
            preserve_mass: r.preserve_mass,
            time_status_updated_at: r.time_status_updated_at,
            lifetime_hours: (r.kind == ConnectionType::Wormhole)
                .then(|| lifetime_hours(r.from_class, r.to_class)),
            jumps_count: r.jumps_count,
            jumps_mass_sum: r.jumps_mass_sum,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

async fn fetch_connection_tx(
    tx: &mut Tx<'_>,
    map_id: i64,
    connection_id: i64,
) -> Result<MapConnection> {
    connections_with_stats(&mut **tx, map_id, Some(connection_id))
        .await?
        .pop()
        .ok_or(MapError::NotFound)
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddConnection {
    pub map_id: i64,
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionType,
    /// Initial max ship size, when the client can infer it (e.g. a C13 endpoint is
    /// frigate-sized). `None` = unknown.
    #[serde(default)]
    #[ts(optional)]
    pub size: Option<WormholeSize>,
}

impl AddConnection {
    pub fn validate(&self) -> Result<()> {
        if self.from_system == self.to_system {
            return Err(MapError::Validation(
                "cannot connect a system to itself".into(),
            ));
        }
        Ok(())
    }
}

/// Connect two placed systems. Endpoints are `map_solar_systems` ids, must be distinct and
/// both on this map. The same pair may be connected more than once (e.g. two separate
/// wormholes between the same systems), so duplicate edges are allowed. A new connection
/// starts with unknown life-cycle state.
pub async fn add_connection(
    pool: &PgPool,
    actor: Actor,
    cmd: AddConnection,
) -> Result<MapConnection> {
    // Drawing an edge out of an unmapped hole claims the system on its far side leads
    // somewhere, which is the one thing nobody knows yet. Checked here rather than in the
    // apply, which raising a ghost goes through to make the hole's own edge.
    let ghosts = sqlx::query_scalar!(
        "select count(*) from map_solar_systems
         where map_id = $1 and (id = $2 or id = $3) and solar_system_id is null",
        cmd.map_id,
        cmd.from_system,
        cmd.to_system,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    if ghosts > 0 {
        return Err(MapError::Validation(
            "nobody has been through that hole yet, so nothing can be connected to it".into(),
        ));
    }

    run(pool, actor, cmd).await
}

pub(super) async fn apply_add_connection(tx: &mut Tx<'_>, cmd: AddConnection) -> Result<Effect> {
    cmd.validate()?;

    let on_map = sqlx::query_scalar!(
        "select count(*) from map_solar_systems where map_id = $1 and (id = $2 or id = $3)",
        cmd.map_id,
        cmd.from_system,
        cmd.to_system,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(0);
    if on_map != 2 {
        return Err(MapError::Validation(
            "both endpoints must be systems on this map".into(),
        ));
    }

    let id = connections::insert(
        tx,
        cmd.map_id,
        &NewConnection::drawn(cmd.from_system, cmd.to_system, cmd.kind, cmd.size),
    )
    .await?;
    let connection = fetch_connection_tx(tx, cmd.map_id, id).await?;

    // A hole often gets jumped moments before it's mapped: adopt those observations.
    if connection.kind == ConnectionType::Wormhole {
        let endpoints = sqlx::query!(
            "select f.solar_system_id as from_sys, t.solar_system_id as to_sys
             from map_solar_systems f, map_solar_systems t
             where f.id = $1 and t.id = $2",
            connection.from_system,
            connection.to_system,
        )
        .fetch_one(&mut **tx)
        .await?;
        // Transits are recorded between two systems, so an edge into a ghost has none to
        // claim; it picks them up when the ghost is resolved.
        if let (Some(from_sys), Some(to_sys)) = (endpoints.from_sys, endpoints.to_sys) {
            super::jumps::claim_pending_tx(tx, connection.map_id, connection.id, from_sys, to_sys)
                .await?;
        }
    }
    let inverse = MapCommand::RemoveConnection(RemoveConnection {
        map_id: connection.map_id,
        connection_id: connection.id,
    });
    let event = MapEvent::ConnectionChanged {
        map_id: connection.map_id,
        connection_id: connection.id,
    };
    Ok(Effect::new(
        "connections.added",
        "mapped a connection",
        CommandOutput::Connection(Box::new(connection)),
    )
    .undo_with(inverse)
    .emit(event))
}

/// A partial update of a connection's wormhole state. `None` leaves a field unchanged;
/// `Some(None)` clears it to unknown; `Some(Some(v))` sets it. Setting any field triggers
/// the DB sync, so linked signatures follow.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetConnectionStatus {
    pub map_id: i64,
    pub connection_id: i64,
    /// `None` leaves the edge kind unchanged; `Some(k)` switches wormhole/stargate.
    #[serde(default)]
    #[ts(optional)]
    pub kind: Option<ConnectionType>,
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub mass_status: Option<Option<MassStatus>>,
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub time_status: Option<Option<TimeStatus>>,
    #[serde(default, deserialize_with = "super::double_option")]
    #[ts(optional)]
    pub size: Option<Option<WormholeSize>>,
    /// `None` leaves the preserve-mass flag unchanged.
    #[serde(default)]
    #[ts(optional)]
    pub preserve_mass: Option<bool>,
}

/// Mark a connection's mass / EOL / size. Member+. Works whether or not a signature is
/// linked; when one is, the trigger propagates the change to it (and its sibling).
pub async fn set_connection_status(
    pool: &PgPool,
    actor: Actor,
    cmd: SetConnectionStatus,
) -> Result<MapConnection> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_set_connection_status(
    tx: &mut Tx<'_>,
    cmd: SetConnectionStatus,
) -> Result<Effect> {
    let current = fetch_connection_tx(tx, cmd.map_id, cmd.connection_id).await?;

    let kind = cmd.kind.unwrap_or(current.kind);
    let mass = cmd.mass_status.unwrap_or(current.mass_status);
    let time = cmd.time_status.unwrap_or(current.time_status);
    let size = cmd.size.unwrap_or(current.size);
    let preserve = cmd.preserve_mass.unwrap_or(current.preserve_mass);

    sqlx::query!(
        "update map_connections
         set type = $1, mass_status = $2, time_status = $3, size = $4, preserve_mass = $5
         where id = $6 and map_id = $7",
        kind,
        mass,
        time,
        size,
        preserve,
        cmd.connection_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?;
    let connection = fetch_connection_tx(tx, cmd.map_id, cmd.connection_id).await?;
    // Undo restores every field, since the sync triggers may have moved more than one.
    let inverse = MapCommand::SetConnectionStatus(SetConnectionStatus {
        map_id: cmd.map_id,
        connection_id: cmd.connection_id,
        kind: Some(current.kind),
        mass_status: Some(current.mass_status),
        time_status: Some(current.time_status),
        size: Some(current.size),
        preserve_mass: Some(current.preserve_mass),
    });
    let event = MapEvent::ConnectionChanged {
        map_id: cmd.map_id,
        connection_id: cmd.connection_id,
    };
    Ok(Effect::new(
        "connections.updated",
        "updated a connection",
        CommandOutput::Connection(Box::new(connection)),
    )
    .undo_with(inverse)
    .emit(event))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveConnection {
    pub map_id: i64,
    pub connection_id: i64,
}

/// Remove a connection. Linked signatures survive with their `connection_id` cleared.
pub async fn remove_connection(pool: &PgPool, actor: Actor, cmd: RemoveConnection) -> Result<()> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_remove_connection(
    tx: &mut Tx<'_>,
    cmd: RemoveConnection,
) -> Result<Effect> {
    let mut restore = super::restore::capture_connection(tx, cmd.map_id, cmd.connection_id)
        .await?
        .ok_or(MapError::NotFound)?;

    // An unmapped hole is this connection's far side and nothing else, so taking the
    // connection away takes it too. Captured first, so one undo puts both back.
    let ghosts = super::ghost::stranded_ghosts(tx, cmd.map_id, &[], &[cmd.connection_id]).await?;
    if !ghosts.is_empty() {
        restore.systems = super::restore::capture_systems(tx, cmd.map_id, &ghosts)
            .await?
            .systems;
        placements::delete(tx, cmd.map_id, &ghosts).await?;
    }

    connections::delete(tx, cmd.map_id, &[cmd.connection_id]).await?;

    let label = match ghosts.len() {
        0 => "removed a connection".to_string(),
        1 => "removed a connection and the hole it led to".to_string(),
        n => format!("removed a connection and the {n} holes it led to"),
    };
    let mut events = vec![MapEvent::ConnectionChanged {
        map_id: cmd.map_id,
        connection_id: cmd.connection_id,
    }];
    events.extend(ghosts.iter().map(|id| MapEvent::SystemRemoved {
        map_id: cmd.map_id,
        map_solar_system_id: *id,
    }));
    Ok(
        Effect::new("connections.removed", label, CommandOutput::None)
            .undo_with(MapCommand::RestoreSystems(restore))
            .emit_all(events),
    )
}
