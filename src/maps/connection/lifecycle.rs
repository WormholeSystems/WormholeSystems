//! The clock's side of a connection's life: ageing wormhole edges to EOL and critical from
//! how long they have been on the map, removing the ones too old to still exist, and
//! sweeping the ones left critical for an hour when somebody asks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::maps::access::require_role;
use crate::maps::command::{CommandOutput, Effect, EventActor, MapCommand, Tx, run, run_as};
use crate::maps::error::{MapError, Result};
use crate::maps::store::connections;
use crate::maps::store::placements;
use crate::maps::{Actor, ConnectionType, Role, TimeStatus};
use crate::realtime::MapEvent;

use super::fetch_connection_tx;

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
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_clean_stale(
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
    let mut snapshot = crate::maps::restore::RestoreSystems {
        map_id,
        systems: Vec::new(),
        connections: Vec::new(),
        signatures: Vec::new(),
        jumps: Vec::new(),
    };
    for edge in &edges {
        if let Some(one) = crate::maps::restore::capture_connection(tx, map_id, edge.id).await? {
            snapshot.absorb(one);
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
        let captured = crate::maps::restore::capture_systems(tx, map_id, &orphans).await?;
        snapshot.absorb(captured);
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

pub(super) fn lifetime_hours(from_class: Option<i32>, to_class: Option<i32>) -> i64 {
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

pub(in crate::maps) async fn apply_age_connection(
    tx: &mut Tx<'_>,
    cmd: AgeConnection,
) -> Result<Effect> {
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
        let cmd = AgeConnection {
            map_id: row.map_id,
            connection_id: row.id,
            time_status,
        };
        match run_as(pool, EventActor::System, cmd).await {
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

pub(in crate::maps) async fn apply_expire_connections(
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
        match run_as(pool, EventActor::System, ExpireConnections { map_id }).await {
            Ok(count) => removed += count,
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
    crate::background::spawn_every(
        "connection ageing",
        std::time::Duration::from_secs(LIFECYCLE_INTERVAL_SECONDS),
        move || {
            let pool = pool.clone();
            async move {
                // A failed expiry should not hold up the ageing, so it is logged here and
                // the pass carries on.
                match expire_connections(&pool).await {
                    Ok(0) => {}
                    Ok(n) => tracing::info!("connections: {n} dead connection(s) removed"),
                    Err(err) => tracing::error!("connection expiry failed: {err}"),
                }
                let aged = age_connections(&pool).await?;
                if aged > 0 {
                    tracing::info!("connections: {aged} connection(s) aged");
                }
                Ok::<_, crate::maps::MapError>(())
            }
        },
    );
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
