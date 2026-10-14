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
use super::Role;
use super::access::require_role;
use super::command::{CommandOutput, Effect, EventActor, MapCommand, Tx, execute, execute_as};
use super::error::{MapError, Result};
use super::store::connections::{self, NewConnection};
use super::store::placements;

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

    execute(pool, actor, MapCommand::AddConnection(cmd))
        .await?
        .connection()
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
    execute(pool, actor, MapCommand::SetConnectionStatus(cmd))
        .await?
        .connection()
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
    execute(pool, actor, MapCommand::RemoveConnection(cmd)).await?;
    Ok(())
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

/// How long a connection must have been marked critical before the map offers to sweep it.
/// Legacy's rule: a critical hole is minutes from collapsing, so an hour without an update
/// means nobody is flying it any more.
pub const STALE_AFTER_MINUTES: i64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CleanStaleConnections {
    pub map_id: i64,
}

/// One stale edge, for the status bar's cleanup popover.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StaleConnection {
    pub connection_id: i64,
    pub from_name: String,
    pub to_name: String,
    /// When it was marked critical or expired, whichever it is now.
    pub critical_since: DateTime<Utc>,
}

/// Every connection that has been critical or expired for longer than
/// [`STALE_AFTER_MINUTES`]. Viewer+.
pub async fn list_stale_connections(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
) -> Result<Vec<StaleConnection>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    let rows = sqlx::query_as!(
        StaleConnection,
        r#"select c.id as "connection_id!",
                  fs.name as "from_name!", ts.name as "to_name!",
                  c.time_status_updated_at as "critical_since!"
           from map_connections c
           join map_solar_systems fm on fm.id = c.from_system
           join solar_systems fs on fs.id = fm.solar_system_id
           join map_solar_systems tm on tm.id = c.to_system
           join solar_systems ts on ts.id = tm.solar_system_id
           where c.map_id = $1
             and c.time_status in ('critical', 'expired')
             and c.time_status_updated_at < now() - make_interval(mins => $2)
           order by c.time_status_updated_at"#,
        map_id,
        STALE_AFTER_MINUTES as i32,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Sweep every stale connection, plus any placement they leave behind. Member+. Recorded
/// as one entry, so a sweep that went too far is a single undo.
pub async fn clean_stale_connections(
    pool: &PgPool,
    actor: Actor,
    cmd: CleanStaleConnections,
) -> Result<u64> {
    execute(pool, actor, MapCommand::CleanStaleConnections(cmd))
        .await?
        .count()
}

pub(super) async fn apply_clean_stale(
    tx: &mut Tx<'_>,
    cmd: CleanStaleConnections,
) -> Result<Effect> {
    let stale = sqlx::query!(
        r#"select id as "id!", from_system as "from_system!", to_system as "to_system!"
           from map_connections
           where map_id = $1
             and time_status in ('critical', 'expired')
             and time_status_updated_at < now() - make_interval(mins => $2)"#,
        cmd.map_id,
        STALE_AFTER_MINUTES as i32,
    )
    .fetch_all(&mut **tx)
    .await?;
    if stale.is_empty() {
        return Err(MapError::Conflict("nothing to clean".into()));
    }
    let edges = stale
        .into_iter()
        .map(|r| SweptEdge {
            id: r.id,
            from_system: r.from_system,
            to_system: r.to_system,
        })
        .collect();
    sweep(
        tx,
        cmd.map_id,
        edges,
        "connections.cleaned",
        |n, m| match (n, m) {
            (1, 0) => "cleaned a stale connection".to_string(),
            (n, 0) => format!("cleaned {n} stale connections"),
            (1, m) => format!("cleaned a stale connection and {m} system(s)"),
            (n, m) => format!("cleaned {n} stale connections and {m} system(s)"),
        },
    )
    .await
}

struct SweptEdge {
    id: i64,
    from_system: i64,
    to_system: i64,
}

/// Delete a set of edges plus any placement they leave behind, as one undoable step. The
/// stale clean and the expiry differ only in which edges they pick and what they call it.
async fn sweep(
    tx: &mut Tx<'_>,
    map_id: i64,
    edges: Vec<SweptEdge>,
    kind: &'static str,
    label: impl Fn(u64, usize) -> String,
) -> Result<Effect> {
    // Snapshot the edges before deleting them, then the placements they orphan. Order
    // matters: a placement only looks orphaned once its edges are gone.
    let mut snapshot = super::restore::RestoreSystems {
        map_id,
        systems: Vec::new(),
        connections: Vec::new(),
        signatures: Vec::new(),
    };
    for edge in &edges {
        if let Some(one) = super::restore::capture_connection(tx, map_id, edge.id).await? {
            snapshot.connections.extend(one.connections);
        }
    }
    let ids: Vec<i64> = edges.iter().map(|e| e.id).collect();
    connections::delete(tx, map_id, &ids).await?;

    let mut endpoints: Vec<i64> = edges
        .iter()
        .flat_map(|e| [e.from_system, e.to_system])
        .collect();
    endpoints.sort_unstable();
    endpoints.dedup();

    // Same orphan rule as the signature cascade: unpinned, unmarked, and now edgeless.
    let orphans = placements::orphans(tx, map_id, &endpoints).await?;

    if !orphans.is_empty() {
        let captured = super::restore::capture_systems(tx, map_id, &orphans).await?;
        snapshot.systems = captured.systems;
        snapshot.signatures = captured.signatures;
        placements::delete(tx, map_id, &orphans).await?;
    }

    let removed = ids.len() as u64;
    let mut events: Vec<MapEvent> = ids
        .iter()
        .map(|id| MapEvent::ConnectionChanged {
            map_id,
            connection_id: *id,
        })
        .collect();
    events.extend(orphans.iter().map(|id| MapEvent::SystemRemoved {
        map_id,
        map_solar_system_id: *id,
    }));
    Ok(Effect::new(
        kind,
        label(removed, orphans.len()),
        CommandOutput::Count(removed),
    )
    .entries(removed as i64 + orphans.len() as i64)
    .undo_with(MapCommand::RestoreSystems(snapshot))
    .emit_all(events))
}

/// Legacy's lifetime table, by the class pair a hole joins. Most wormholes live 24 hours;
/// the ones between C6 and known space 48; the drifter holes into known space 16.
const LIFETIME_HOURS: i64 = 24;
const C6_KSPACE_LIFETIME_HOURS: i64 = 48;
const DRIFTER_KSPACE_LIFETIME_HOURS: i64 = 16;
/// EOL means "under 4 hours left" and critical "under 1 hour", so the marks land that far
/// before the lifetime runs out.
const EOL_LEAD_HOURS: i64 = 4;
const CRITICAL_LEAD_HOURS: i64 = 1;
/// A hole marked EOL by hand is under 4 hours from dying whatever its age says, so it goes
/// critical 3 hours after the mark.
const EOL_TO_CRITICAL_HOURS: i64 = 3;
/// No wormhole outlives this, so an edge older than it is a leftover, not a hole.
pub const EXPIRE_AFTER_DAYS: i32 = 3;
/// How often the life-cycle loop runs. The thresholds are hours, so minutes of lag is fine.
const LIFECYCLE_INTERVAL_SECONDS: u64 = 10 * 60;

fn is_known_space(class: Option<i32>) -> bool {
    matches!(class, Some(7..=9))
}

fn is_c6(class: Option<i32>) -> bool {
    class == Some(6)
}

fn is_drifter(class: Option<i32>) -> bool {
    matches!(class, Some(14..=18))
}

fn lifetime_hours(from_class: Option<i32>, to_class: Option<i32>) -> i64 {
    let joins_known_space = |special: fn(Option<i32>) -> bool| {
        (special(from_class) && is_known_space(to_class))
            || (special(to_class) && is_known_space(from_class))
    };
    if joins_known_space(is_drifter) {
        DRIFTER_KSPACE_LIFETIME_HOURS
    } else if joins_known_space(is_c6) {
        C6_KSPACE_LIFETIME_HOURS
    } else {
        LIFETIME_HOURS
    }
}

/// The lifetime mark a hole's age has earned, or `None` when its current mark is already as
/// bad or worse. Only ever escalates: a pilot's own mark is better information than the
/// clock, and an unknown hole is left unknown rather than declared stable. The clock stops
/// at critical: expired is only ever read off the hole in game, never inferred from age. A hole sitting
/// at EOL is judged from when it was marked, not from its age, since the mark may have
/// come from a scan of a hole older than the map knows.
pub fn aged_time_status(
    current: Option<TimeStatus>,
    hours_alive: i64,
    hours_since_marked: Option<i64>,
    from_class: Option<i32>,
    to_class: Option<i32>,
) -> Option<TimeStatus> {
    let earned = match (current, hours_since_marked) {
        (Some(TimeStatus::Eol), Some(since)) if since >= EOL_TO_CRITICAL_HOURS => {
            TimeStatus::Critical
        }
        (Some(TimeStatus::Eol), Some(_)) => TimeStatus::Eol,
        _ => {
            let lifetime = lifetime_hours(from_class, to_class);
            if hours_alive >= lifetime - CRITICAL_LEAD_HOURS {
                TimeStatus::Critical
            } else if hours_alive >= lifetime - EOL_LEAD_HOURS {
                TimeStatus::Eol
            } else {
                TimeStatus::Stable
            }
        }
    };
    (earned > current.unwrap_or(TimeStatus::Stable)).then_some(earned)
}

/// Move a wormhole's lifetime mark forward because of its age. Background only: the loop
/// decides the mark, and the apply refuses anything that is not an escalation, so a pilot
/// who marked the hole in between wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgeConnection {
    pub map_id: i64,
    pub connection_id: i64,
    pub time_status: TimeStatus,
}

pub(super) async fn apply_age_connection(tx: &mut Tx<'_>, cmd: AgeConnection) -> Result<Effect> {
    let current = fetch_connection_tx(tx, cmd.map_id, cmd.connection_id).await?;
    if current.kind != ConnectionType::Wormhole {
        return Err(MapError::Validation("only wormholes age".into()));
    }
    if current.time_status >= Some(cmd.time_status) {
        return Err(MapError::Conflict("already marked".into()));
    }
    sqlx::query!(
        "update map_connections set time_status = $1 where id = $2 and map_id = $3",
        cmd.time_status,
        cmd.connection_id,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?;
    let connection = fetch_connection_tx(tx, cmd.map_id, cmd.connection_id).await?;
    let label = match cmd.time_status {
        TimeStatus::Critical => "marked a connection critical by age",
        _ => "marked a connection end of life by age",
    };
    Ok(Effect::new(
        "connections.aged",
        label,
        CommandOutput::Connection(Box::new(connection)),
    )
    .emit(MapEvent::ConnectionChanged {
        map_id: cmd.map_id,
        connection_id: cmd.connection_id,
    }))
}

/// Every wormhole edge whose age has earned it a worse mark than it carries, marked. One
/// audit entry per hole, so the history says when the map gave up on it. Returns how many
/// were marked.
pub async fn age_connections(pool: &PgPool) -> Result<u64> {
    let candidates = sqlx::query!(
        r#"select c.id, c.map_id, c.time_status as "time_status: TimeStatus",
                  c.time_status_updated_at, c.created_at,
                  fs.wormhole_class_id as "from_class?", ts.wormhole_class_id as "to_class?"
           from map_connections c
           join map_solar_systems fm on fm.id = c.from_system
           left join solar_systems fs on fs.id = fm.solar_system_id
           join map_solar_systems tm on tm.id = c.to_system
           left join solar_systems ts on ts.id = tm.solar_system_id
           where c.type = 'wormhole'
             and c.time_status is distinct from 'critical'
             and c.time_status is distinct from 'expired'"#,
    )
    .fetch_all(pool)
    .await?;

    let now = Utc::now();
    let mut aged = 0;
    for row in candidates {
        let hours_alive = (now - row.created_at).num_hours();
        let hours_since_marked = row.time_status_updated_at.map(|at| (now - at).num_hours());
        let Some(time_status) = aged_time_status(
            row.time_status,
            hours_alive,
            hours_since_marked,
            row.from_class,
            row.to_class,
        ) else {
            continue;
        };
        let cmd = MapCommand::AgeConnection(AgeConnection {
            map_id: row.map_id,
            connection_id: row.id,
            time_status,
        });
        match execute_as(pool, EventActor::System, cmd).await {
            Ok(_) => aged += 1,
            // Someone marked or removed the hole between the read and the write.
            Err(MapError::Conflict(_) | MapError::NotFound) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(aged)
}

/// Remove a map's wormhole edges older than [`EXPIRE_AFTER_DAYS`], plus the placements
/// they strand. Background only. The signatures they were linked to are left unlinked for
/// the signature expiry to collect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpireConnections {
    pub map_id: i64,
}

pub(super) async fn apply_expire_connections(
    tx: &mut Tx<'_>,
    cmd: ExpireConnections,
) -> Result<Effect> {
    let dead = sqlx::query!(
        r#"select id as "id!", from_system as "from_system!", to_system as "to_system!"
           from map_connections
           where map_id = $1
             and type = 'wormhole'
             and created_at < now() - make_interval(days => $2)"#,
        cmd.map_id,
        EXPIRE_AFTER_DAYS,
    )
    .fetch_all(&mut **tx)
    .await?;
    if dead.is_empty() {
        return Err(MapError::Conflict("nothing to expire".into()));
    }
    let edges = dead
        .into_iter()
        .map(|r| SweptEdge {
            id: r.id,
            from_system: r.from_system,
            to_system: r.to_system,
        })
        .collect();
    sweep(
        tx,
        cmd.map_id,
        edges,
        "connections.expired",
        |n, m| match (n, m) {
            (1, 0) => "removed a dead connection".to_string(),
            (n, 0) => format!("removed {n} dead connections"),
            (1, m) => format!("removed a dead connection and {m} system(s)"),
            (n, m) => format!("removed {n} dead connections and {m} system(s)"),
        },
    )
    .await
}

/// Expire the dead wormholes on every map, one audit entry per map. Returns how many edges
/// were removed.
pub async fn expire_connections(pool: &PgPool) -> Result<u64> {
    let maps = sqlx::query_scalar!(
        r#"select distinct map_id as "map_id!" from map_connections
           where type = 'wormhole' and created_at < now() - make_interval(days => $1)"#,
        EXPIRE_AFTER_DAYS,
    )
    .fetch_all(pool)
    .await?;

    let mut removed = 0;
    for map_id in maps {
        let cmd = MapCommand::ExpireConnections(ExpireConnections { map_id });
        match execute_as(pool, EventActor::System, cmd).await {
            Ok(output) => removed += output.count()?,
            Err(MapError::Conflict(_) | MapError::NotFound) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(removed)
}

/// Run the wormhole life-cycle on a timer: expire what cannot still exist, then age the
/// rest. Each change goes through the command layer, so open maps hear about it the same
/// way they hear about a pilot's edit.
pub fn start_lifecycle(pool: PgPool) {
    tokio::spawn(async move {
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(LIFECYCLE_INTERVAL_SECONDS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            match expire_connections(&pool).await {
                Ok(0) => {}
                Ok(n) => println!("connections: {n} dead connection(s) removed"),
                Err(err) => eprintln!("connection expiry failed: {err}"),
            }
            match age_connections(&pool).await {
                Ok(0) => {}
                Ok(n) => println!("connections: {n} connection(s) aged"),
                Err(err) => eprintln!("connection ageing failed: {err}"),
            }
        }
    });
}

#[cfg(test)]
mod age_tests {
    use super::*;

    const KSPACE: Option<i32> = Some(7);
    const C3: Option<i32> = Some(3);
    const C6: Option<i32> = Some(6);
    const DRIFTER: Option<i32> = Some(15);

    #[test]
    fn a_plain_hole_goes_eol_at_20h_and_critical_at_23h() {
        assert_eq!(aged_time_status(None, 19, None, C3, KSPACE), None);
        assert_eq!(
            aged_time_status(None, 20, None, C3, KSPACE),
            Some(TimeStatus::Eol)
        );
        assert_eq!(
            aged_time_status(None, 23, None, C3, C3),
            Some(TimeStatus::Critical)
        );
    }

    #[test]
    fn c6_and_drifter_holes_into_known_space_have_their_own_clocks() {
        assert_eq!(aged_time_status(None, 43, None, C6, KSPACE), None);
        assert_eq!(
            aged_time_status(None, 44, None, KSPACE, C6),
            Some(TimeStatus::Eol)
        );
        assert_eq!(
            aged_time_status(None, 47, None, C6, KSPACE),
            Some(TimeStatus::Critical)
        );
        assert_eq!(
            aged_time_status(None, 12, None, DRIFTER, KSPACE),
            Some(TimeStatus::Eol)
        );
        assert_eq!(
            aged_time_status(None, 15, None, KSPACE, DRIFTER),
            Some(TimeStatus::Critical)
        );
        // The long clocks only apply against known space: C6 to C6 is an ordinary hole.
        assert_eq!(
            aged_time_status(None, 20, None, C6, C6),
            Some(TimeStatus::Eol)
        );
    }

    #[test]
    fn an_unknown_hole_is_never_declared_stable() {
        assert_eq!(aged_time_status(None, 0, None, C3, KSPACE), None);
        assert_eq!(
            aged_time_status(Some(TimeStatus::Stable), 5, Some(5), C3, KSPACE),
            None
        );
    }

    #[test]
    fn an_eol_mark_goes_critical_three_hours_later_whatever_the_age() {
        assert_eq!(
            aged_time_status(Some(TimeStatus::Eol), 2, Some(2), C3, KSPACE),
            None
        );
        assert_eq!(
            aged_time_status(Some(TimeStatus::Eol), 2, Some(3), C3, KSPACE),
            Some(TimeStatus::Critical)
        );
        // An old hole marked EOL a moment ago is trusted over its age.
        assert_eq!(
            aged_time_status(Some(TimeStatus::Eol), 30, Some(0), C3, KSPACE),
            None
        );
    }

    #[test]
    fn the_clock_never_declares_a_hole_expired() {
        assert_eq!(
            aged_time_status(Some(TimeStatus::Critical), 200, Some(100), C3, KSPACE),
            None
        );
        assert_eq!(
            aged_time_status(None, 200, None, C6, KSPACE),
            Some(TimeStatus::Critical)
        );
    }

    #[test]
    fn the_clock_never_downgrades_a_mark() {
        assert_eq!(
            aged_time_status(Some(TimeStatus::Expired), 30, Some(5), C3, KSPACE),
            None
        );
        assert_eq!(
            aged_time_status(Some(TimeStatus::Expired), 0, Some(0), C3, KSPACE),
            None
        );
        assert_eq!(
            aged_time_status(Some(TimeStatus::Critical), 1, Some(0), C3, KSPACE),
            None
        );
        assert_eq!(
            aged_time_status(Some(TimeStatus::Eol), 23, Some(1), C3, KSPACE),
            None
        );
    }
}
