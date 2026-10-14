//! Ghost placements: the far side of a wormhole nobody has been through yet.
//!
//! A ghost is an ordinary `map_solar_systems` row with no solar system (see
//! [mapping.md](../../docs/database/mapping.md#ghost-placements)), so it draws, moves,
//! aliases and connects like any other node. What it cannot do is anything that needs a
//! system: hold signatures or intel, be routed through, be a waypoint.
//!
//! Resolving one is [`resolve_ghost_system`]. When the system turns out to be on the map
//! already (the hole led back into the chain) the ghost is merged into that placement
//! rather than duplicating it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::command::{CommandOutput, Effect, MapCommand, Sequence, Tx, run};
use super::connection::{AddConnection, SetConnectionStatus};
use super::error::{MapError, Result};
use super::solar_system::{MapSolarSystem, SetAlias};
use super::store::placements::{self, NewPlacement, PlacementRow};
use super::{Actor, ConnectionType, MassStatus, TimeStatus, WormholeSize};
use crate::realtime::MapEvent;

/// Node width in world px, and the clear space kept between nodes, both mirroring
/// `frontend/src/lib/map/helpers.ts` so a server-placed node lands on the same lattice.
const NODE_WIDTH: f64 = 180.0;
const NODE_GAP_CELLS: f64 = 1.0;

/// The first free slot beside `base`, then down that column: the client's `freePosition`,
/// for placements made inside a command. Siblings stack under the first one rather than
/// marching right across the map.
pub(super) fn free_position(placed: &[(f64, f64)], base: (f64, f64)) -> (f64, f64) {
    let grid = super::grid();
    let node_h = 2.0 * grid.cell_size;
    let gap = NODE_GAP_CELLS * grid.cell_size;
    let step_x = NODE_WIDTH + gap;
    let step_y = node_h + gap;
    let max_x = grid.world_width - NODE_WIDTH;
    let max_y = grid.world_height - node_h;
    let snap = |v: f64| (v / grid.cell_size).round() * grid.cell_size;
    let crowded = |x: f64, y: f64| {
        placed
            .iter()
            .any(|(px, py)| (x - px).abs() < step_x && (y - py).abs() < step_y)
    };

    let bx = snap(base.0.clamp(0.0, max_x));
    let by = snap(base.1.clamp(0.0, max_y));
    if !crowded(bx, by) {
        return (bx, by);
    }
    let mut column = 1.0;
    loop {
        let x = snap(bx + column * step_x);
        if x > max_x {
            return (bx, by);
        }
        let mut row = 0.0;
        loop {
            let y = snap(by + row * step_y);
            if y > max_y {
                break;
            }
            if !crowded(x, y) {
                return (x, y);
            }
            row += 1.0;
        }
        column += 1.0;
    }
}

/// Bring the map's ghosts back in line with its scans. A ghost is owed to every wormhole
/// signature that has nowhere to lead yet, and owed to nothing else: not to a signature
/// that has been deleted, retyped or unlinked, not to one whose far side has since been
/// removed, and not at all on a map that has asked for no ghosts.
///
/// Running this after every command is what makes that hold however the map was changed,
/// rather than only on the handful of writes that remember to. Returns the undo steps for
/// what it did, in the order they should run, before the command's own inverse, plus the
/// events announcing it: reconciliation happens inside somebody else's command, so no
/// handler knows to announce these changes for it.
pub(super) async fn reconcile(
    tx: &mut Tx<'_>,
    map_id: i64,
) -> Result<(Vec<MapCommand>, Vec<MapEvent>)> {
    let wanted = sqlx::query_scalar!(
        "select ghost_unlinked_wormholes from maps where id = $1",
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .unwrap_or(false);

    let mut undo = Vec::new();
    let mut events = Vec::new();
    let unclaimed = unclaimed_ghosts(tx, map_id, wanted).await?;
    if !unclaimed.is_empty() {
        undo.extend(drop_ghosts(tx, map_id, &unclaimed).await?);
        events.extend(unclaimed.iter().map(|id| MapEvent::SystemRemoved {
            map_id,
            map_solar_system_id: *id,
        }));
    }
    if wanted {
        let (raised, raised_events) = raise_unmapped_holes(tx, map_id).await?;
        events.extend(raised_events);
        if !raised.is_empty() {
            undo.push(MapCommand::RemoveRestored(super::restore::RemoveRestored {
                map_id,
                system_ids: raised,
                connection_ids: Vec::new(),
                signature_ids: Vec::new(),
            }));
        }
    }
    undo.reverse();
    Ok((undo, events))
}

/// The ghosts the map no longer owes: those whose scan has stopped being an unmapped
/// wormhole, whether it was retyped, linked to a real system or unlinked from this node,
/// and, on a map that wants none, all of them. A scan that has been deleted outright, or a
/// system that has been taken off the map, are the database's business rather than ours:
/// both cascade. Home, pinned and rally nodes are deliberate markers and survive here as
/// they do everywhere else.
async fn unclaimed_ghosts(tx: &mut Tx<'_>, map_id: i64, wanted: bool) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar!(
        r#"select g.id as "id!" from map_solar_systems g
           join signatures s on s.id = g.raised_by_signature_id
           where g.map_id = $1
             and g.solar_system_id is null
             and not g.is_home and not g.is_pinned and not g.is_rally
             and (not $2
                  or s."group" <> 'wormhole'
                  or not exists (
                      select 1 from map_connections c
                      where c.id = s.connection_id
                        and (c.from_system = g.id or c.to_system = g.id)
                  ))"#,
        map_id,
        wanted,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Take ghosts off the map. The snapshot is taken first so the undo puts back the node and
/// its edge; the scan on the far side is not ours to delete, so its link is put back by
/// hand rather than by the snapshot, which would decline to overwrite a row still there.
async fn drop_ghosts(tx: &mut Tx<'_>, map_id: i64, ids: &[i64]) -> Result<Vec<MapCommand>> {
    let snapshot = super::restore::capture_systems(tx, map_id, ids).await?;
    let relink: Vec<MapCommand> = snapshot
        .signatures
        .iter()
        .filter_map(|s| {
            Some(MapCommand::LinkSignature(
                super::signatures::LinkSignature {
                    map_id,
                    signature_pk: s.id,
                    connection_id: s.connection_id?,
                },
            ))
        })
        .collect();

    placements::delete(tx, map_id, ids).await?;

    let mut undo = vec![MapCommand::RestoreSystems(snapshot)];
    undo.extend(relink);
    Ok(undo)
}

/// Raise a ghost for every wormhole scanned on this map that has nowhere to lead yet.
async fn raise_unmapped_holes(tx: &mut Tx<'_>, map_id: i64) -> Result<(Vec<i64>, Vec<MapEvent>)> {
    let holes = sqlx::query!(
        r#"select s.id, s.size,
                  p.id as "from_id!", p.position_x as "from_x!", p.position_y as "from_y!"
           from signatures s
           join map_solar_systems p
             on p.map_id = s.map_id and p.solar_system_id = s.solar_system_id
           where s.map_id = $1 and s."group" = 'wormhole' and s.connection_id is null
           order by p.id, s.signature_id"#,
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    if holes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut placed: Vec<(f64, f64)> = sqlx::query!(
        "select position_x, position_y from map_solar_systems where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| (r.position_x, r.position_y))
    .collect();

    let mut raised = Vec::new();
    let mut events = Vec::new();
    for hole in holes {
        let (x, y) = free_position(&placed, (hole.from_x, hole.from_y));
        placed.push((x, y));
        let effect = apply_add_ghost_system(
            tx,
            AddGhostSystem {
                map_id,
                from_system: hole.from_id,
                signature_pk: Some(hole.id),
                x,
                y,
                alias: None,
                size: hole.size,
            },
        )
        .await?;
        events.extend(effect.events);
        raised.push(effect.output.take::<super::MapSolarSystem>()?.id);
    }
    Ok((raised, events))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddGhostSystem {
    pub map_id: i64,
    /// The placement the signature was scanned in. The ghost hangs off it.
    pub from_system: i64,
    /// The wormhole signature this hole is, so the two stay in lock-step from the start.
    #[serde(default)]
    pub signature_pk: Option<i64>,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub alias: Option<String>,
    #[serde(default)]
    pub size: Option<WormholeSize>,
}

pub(super) async fn apply_add_ghost_system(tx: &mut Tx<'_>, cmd: AddGhostSystem) -> Result<Effect> {
    let source = sqlx::query!(
        "select solar_system_id from map_solar_systems where id = $1 and map_id = $2",
        cmd.from_system,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;

    // The signature has to be one scanned in the source system, and still unlinked: a
    // second ghost for a hole already on the map is exactly the duplicate this avoids.
    let before = match cmd.signature_pk {
        Some(pk) => {
            let sig = sqlx::query!(
                "select solar_system_id, connection_id from signatures where id = $1 and map_id = $2",
                pk,
                cmd.map_id,
            )
            .fetch_optional(&mut **tx)
            .await?
            .ok_or(MapError::NotFound)?;
            if Some(sig.solar_system_id) != source.solar_system_id {
                return Err(MapError::Validation(
                    "that signature was scanned in another system".into(),
                ));
            }
            if sig.connection_id.is_some() {
                return Err(MapError::Conflict(
                    "that signature is already on the map".into(),
                ));
            }
            Some(super::tracking::signature_state(tx, cmd.map_id, pk).await?)
        }
        None => None,
    };

    // A ghost has to name the scan that raised it and the system it hangs off, which is
    // what makes it go when either of them does.
    let Some(signature_pk) = cmd.signature_pk else {
        return Err(MapError::Validation(
            "an unmapped hole has to name the signature it was scanned as".into(),
        ));
    };
    let ghost = placements::insert(
        tx,
        cmd.map_id,
        &NewPlacement::ghost(
            signature_pk,
            cmd.from_system,
            cmd.x,
            cmd.y,
            cmd.alias.as_deref(),
        ),
    )
    .await?;

    let effect = super::connection::connect(
        tx,
        AddConnection {
            map_id: cmd.map_id,
            from_system: cmd.from_system,
            to_system: ghost.id,
            kind: ConnectionType::Wormhole,
            size: cmd.size,
        },
    )
    .await?;
    let connection = effect.output.take::<super::MapConnection>()?;

    super::tracking::link(tx, cmd.map_id, signature_pk, connection.id).await?;

    let mut steps = super::tracking::undo_signature(cmd.map_id, before.as_ref());
    steps.push(MapCommand::RemoveRestored(super::restore::RemoveRestored {
        map_id: cmd.map_id,
        system_ids: vec![ghost.id],
        connection_ids: vec![connection.id],
        signature_ids: Vec::new(),
    }));

    let mut events = vec![MapEvent::SystemAdded {
        map_id: cmd.map_id,
        map_solar_system_id: ghost.id,
    }];
    events.extend(effect.events);
    if let Some(solar_system_id) = source.solar_system_id {
        events.push(MapEvent::SignatureChanged {
            map_id: cmd.map_id,
            solar_system_id,
        });
    }
    Ok(Effect::new(
        "systems.added",
        "put an unscanned hole on the map",
        CommandOutput::System(Box::new(ghost)),
    )
    .undo_with(MapCommand::Sequence(Sequence {
        map_id: cmd.map_id,
        steps,
    }))
    .emit_all(events))
}

/// Say which system a ghost turned out to be. `solar_system_id: None` takes it back to a
/// ghost, and exists as the inverse of the first form rather than as an action of its own.
///
/// The rest is what flying the hole taught us about it, carried here rather than sent as
/// follow-up writes so that one jump stays one undo. All of it is ignored when taking a
/// node back to a ghost.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ResolveGhostSystem {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    #[serde(default)]
    #[ts(optional)]
    pub solar_system_id: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub alias: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub size: Option<WormholeSize>,
    #[serde(default)]
    #[ts(optional)]
    pub mass_status: Option<MassStatus>,
    #[serde(default)]
    #[ts(optional)]
    pub time_status: Option<TimeStatus>,
    /// Only ever set by the inverse of a resolve; see [`ResolveGhostSystem::unresolve`].
    #[serde(default)]
    #[ts(optional)]
    pub raised_by_signature_id: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub hangs_off_id: Option<i64>,
}

impl ResolveGhostSystem {
    /// The inverse form: take this node back to being the hole it was drawn as. The scan
    /// and the system it hung off travel with it, because a ghost is not allowed to exist
    /// without naming them, and by then they are no longer on the row to be read off.
    fn unresolve(map_id: i64, map_solar_system_id: i64, was: Raised) -> Self {
        ResolveGhostSystem {
            map_id,
            map_solar_system_id,
            raised_by_signature_id: Some(was.signature_pk),
            hangs_off_id: Some(was.from_system),
            ..Default::default()
        }
    }
}

/// What a node was raised as, kept so a resolve can be undone.
#[derive(Debug, Clone, Copy)]
struct Raised {
    signature_pk: i64,
    from_system: i64,
}

/// Assign a system to a ghost, merging into the existing placement if the map already has
/// that system. Member+.
pub async fn resolve_ghost_system(
    pool: &PgPool,
    actor: Actor,
    cmd: ResolveGhostSystem,
) -> Result<MapSolarSystem> {
    run(pool, actor, cmd).await
}

pub(super) async fn apply_resolve_ghost_system(
    tx: &mut Tx<'_>,
    cmd: ResolveGhostSystem,
) -> Result<Effect> {
    let placement = sqlx::query!(
        "select solar_system_id, position_x, position_y, alias,
                raised_by_signature_id, hangs_off_id
         from map_solar_systems where id = $1 and map_id = $2",
        cmd.map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;

    let Some(solar_system_id) = cmd.solar_system_id else {
        return unresolve(tx, &cmd, placement.solar_system_id).await;
    };

    if placement.solar_system_id.is_some() {
        return Err(MapError::Validation("that node is already a system".into()));
    }

    let name = sqlx::query_scalar!(
        "select name from solar_systems where id = $1",
        solar_system_id
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| MapError::Validation(format!("unknown solar system {solar_system_id}")))?;

    let existing = sqlx::query_scalar!(
        "select id from map_solar_systems where map_id = $1 and solar_system_id = $2",
        cmd.map_id,
        solar_system_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    // Whatever survives the merge is what carries the alias; the ghost row may not.
    let surviving = existing.unwrap_or(cmd.map_solar_system_id);
    let mut undo = Vec::new();
    let mut events = Vec::new();
    if let Some(alias) = cmd.alias.clone() {
        let effect = super::solar_system::apply_set_alias(
            tx,
            SetAlias {
                map_id: cmd.map_id,
                map_solar_system_id: surviving,
                alias: Some(alias),
            },
        )
        .await?;
        undo.extend(effect.inverse);
        events.extend(effect.events);
    }
    if cmd.size.is_some() || cmd.mass_status.is_some() || cmd.time_status.is_some() {
        // Connections keep their id through a merge, so this holds either way.
        let connection_id = sqlx::query_scalar!(
            "select id from map_connections
             where map_id = $1 and (from_system = $2 or to_system = $2)
             order by id limit 1",
            cmd.map_id,
            cmd.map_solar_system_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(connection_id) = connection_id {
            let effect = super::connection::apply_set_connection_status(
                tx,
                SetConnectionStatus {
                    map_id: cmd.map_id,
                    connection_id,
                    kind: None,
                    mass_status: cmd.mass_status.map(Some),
                    time_status: cmd.time_status.map(Some),
                    size: cmd.size.map(Some),
                    preserve_mass: None,
                },
            )
            .await?;
            undo.extend(effect.inverse);
            events.extend(effect.events);
        }
    }

    let effect = match existing {
        Some(target) => merge(tx, cmd.map_id, cmd.map_solar_system_id, target, &name).await,
        None => {
            let was = Raised {
                signature_pk: placement.raised_by_signature_id.ok_or(MapError::NotFound)?,
                from_system: placement.hangs_off_id.ok_or(MapError::NotFound)?,
            };
            let placed =
                placements::assign_system(tx, cmd.map_id, cmd.map_solar_system_id, solar_system_id)
                    .await?;
            claim_pending_for(tx, cmd.map_id, cmd.map_solar_system_id, solar_system_id).await?;
            Ok(Effect::new(
                "systems.details",
                format!("scanned that hole as {name}"),
                CommandOutput::System(Box::new(placed)),
            )
            .undo_with(MapCommand::ResolveGhostSystem(
                ResolveGhostSystem::unresolve(cmd.map_id, cmd.map_solar_system_id, was),
            ))
            .emit(MapEvent::SystemDetailsChanged {
                map_id: cmd.map_id,
                map_solar_system_id: cmd.map_solar_system_id,
            }))
        }
    }?;

    Ok(with_undo_steps(cmd.map_id, undo, effect).emit_all(events))
}

/// Put the alias and status back before the node itself goes, so one jump is one undo.
/// A change that was not undoable in the first place stays that way.
pub(super) fn with_undo_steps(
    map_id: i64,
    mut steps: Vec<MapCommand>,
    mut effect: Effect,
) -> Effect {
    if steps.is_empty() {
        return effect;
    }
    let Some(inverse) = effect.inverse.take() else {
        return effect;
    };
    steps.push(inverse);
    effect.undo_with(MapCommand::Sequence(Sequence { map_id, steps }))
}

/// Back to a ghost. Only the inverse of a resolve gets here, because only it knows which
/// scan the node was drawn for; a hole with no signature behind it is not a thing the map
/// can hold. Refuses once the system holds a scan of its own, because those rows hang off
/// the system id and would be cascaded away by clearing it.
async fn unresolve(
    tx: &mut Tx<'_>,
    cmd: &ResolveGhostSystem,
    current: Option<i64>,
) -> Result<Effect> {
    let Some(current) = current else {
        return Err(MapError::Validation("that node is already a ghost".into()));
    };
    let (Some(signature_pk), Some(from_system)) = (cmd.raised_by_signature_id, cmd.hangs_off_id)
    else {
        return Err(MapError::Validation(
            "that node was not drawn for a scan, so it cannot go back to being one".into(),
        ));
    };
    let signatures = sqlx::query_scalar!(
        "select count(*) from signatures where map_id = $1 and solar_system_id = $2",
        cmd.map_id,
        current,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(0);
    if signatures > 0 {
        return Err(MapError::Conflict(
            "that system has signatures now; remove them first".into(),
        ));
    }

    let placed = placements::back_to_ghost(
        tx,
        cmd.map_id,
        cmd.map_solar_system_id,
        signature_pk,
        from_system,
    )
    .await?;

    Ok(Effect::new(
        "systems.details",
        "took that system back off the map",
        CommandOutput::System(Box::new(placed)),
    )
    .undo_with(MapCommand::ResolveGhostSystem(ResolveGhostSystem {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
        solar_system_id: Some(current),
        ..Default::default()
    }))
    .emit(MapEvent::SystemDetailsChanged {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.map_solar_system_id,
    }))
}

/// The ghost turned out to be a system already on the map: hand its edges over and drop
/// it, rather than placing the same system twice.
async fn merge(
    tx: &mut Tx<'_>,
    map_id: i64,
    ghost_id: i64,
    target: i64,
    name: &str,
) -> Result<Effect> {
    let loops = sqlx::query_scalar!(
        "select count(*) from map_connections
         where map_id = $1
           and ((from_system = $2 and to_system = $3) or (from_system = $3 and to_system = $2))",
        map_id,
        ghost_id,
        target,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(0);
    if loops > 0 {
        return Err(MapError::Conflict(format!(
            "that hole already connects to {name}, so it cannot lead there"
        )));
    }

    let from_ids: Vec<i64> = sqlx::query_scalar!(
        "update map_connections set from_system = $1, updated_at = now()
         where map_id = $2 and from_system = $3 returning id",
        target,
        map_id,
        ghost_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    let to_ids: Vec<i64> = sqlx::query_scalar!(
        "update map_connections set to_system = $1, updated_at = now()
         where map_id = $2 and to_system = $3 returning id",
        target,
        map_id,
        ghost_id,
    )
    .fetch_all(&mut **tx)
    .await?;

    let ghost = sqlx::query!(
        r#"delete from map_solar_systems where id = $1 and map_id = $2
           returning position_x, position_y, alias, created_at,
               raised_by_signature_id as "raised_by_signature_id!",
               hangs_off_id as "hangs_off_id!""#,
        ghost_id,
        map_id,
    )
    .fetch_one(&mut **tx)
    .await?;

    let placed = sqlx::query_as!(
        MapSolarSystem,
        "select id, map_id, solar_system_id, position_x, position_y, alias, created_at
         from map_solar_systems where id = $1",
        target,
    )
    .fetch_one(&mut **tx)
    .await?;
    if let Some(system) = placed.solar_system_id {
        claim_pending_for(tx, map_id, target, system).await?;
    }

    let mut events = vec![
        MapEvent::SystemRemoved {
            map_id,
            map_solar_system_id: ghost_id,
        },
        MapEvent::SystemDetailsChanged {
            map_id,
            map_solar_system_id: target,
        },
    ];
    events.extend(
        from_ids
            .iter()
            .chain(to_ids.iter())
            .map(|id| MapEvent::ConnectionChanged {
                map_id,
                connection_id: *id,
            }),
    );
    Ok(Effect::new(
        "systems.details",
        format!("scanned that hole as {name}, already on the map"),
        CommandOutput::System(Box::new(placed)),
    )
    .emit_all(events)
    .undo_with(MapCommand::RestoreGhostSystem(RestoreGhostSystem {
        map_id,
        id: ghost_id,
        position_x: ghost.position_x,
        position_y: ghost.position_y,
        alias: ghost.alias,
        raised_by_signature_id: ghost.raised_by_signature_id,
        hangs_off_id: ghost.hangs_off_id,
        from_connection_ids: from_ids,
        to_connection_ids: to_ids,
        created_at: Some(ghost.created_at),
    })))
}

/// Undo of a merge: the ghost row comes back with its own id, and the edges that were
/// handed over point at it again.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreGhostSystem {
    pub map_id: i64,
    pub id: i64,
    pub position_x: f64,
    pub position_y: f64,
    pub alias: Option<String>,
    pub raised_by_signature_id: i64,
    pub hangs_off_id: i64,
    pub from_connection_ids: Vec<i64>,
    pub to_connection_ids: Vec<i64>,
    /// When the ghost was raised, so it comes back as old as it was. Absent from older
    /// history, which brings it back as new.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

pub(super) async fn apply_restore_ghost_system(
    tx: &mut Tx<'_>,
    cmd: RestoreGhostSystem,
) -> Result<Effect> {
    // The system the ghost was merged into, read off the edges it was handed before they go
    // back: redoing the merge is resolving the ghost as that system again.
    let merged_into = sqlx::query_scalar!(
        "select p.solar_system_id
         from map_connections c
         join map_solar_systems p
           on p.id = case when c.id = any($2) then c.from_system else c.to_system end
         where c.map_id = $1 and (c.id = any($2) or c.id = any($3))
         limit 1",
        cmd.map_id,
        &cmd.from_connection_ids,
        &cmd.to_connection_ids,
    )
    .fetch_optional(&mut **tx)
    .await?
    .flatten();

    // A ghost carries none of the marks a system earns, so the rest of the row is empty.
    placements::reinsert(
        tx,
        cmd.map_id,
        &PlacementRow {
            id: cmd.id,
            solar_system_id: None,
            position_x: cmd.position_x,
            position_y: cmd.position_y,
            alias: cmd.alias.clone(),
            is_home: false,
            is_rally: false,
            rallied_at: None,
            is_pinned: false,
            raised_by_signature_id: Some(cmd.raised_by_signature_id),
            hangs_off_id: Some(cmd.hangs_off_id),
            signatures_pasted_at: None,
            created_at: cmd.created_at,
        },
    )
    .await?;
    sqlx::query!(
        "update map_connections set from_system = $1 where map_id = $2 and id = any($3)",
        cmd.id,
        cmd.map_id,
        &cmd.from_connection_ids,
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "update map_connections set to_system = $1 where map_id = $2 and id = any($3)",
        cmd.id,
        cmd.map_id,
        &cmd.to_connection_ids,
    )
    .execute(&mut **tx)
    .await?;

    let ghost = sqlx::query_as!(
        MapSolarSystem,
        "select id, map_id, solar_system_id, position_x, position_y, alias, created_at
         from map_solar_systems where id = $1",
        cmd.id,
    )
    .fetch_one(&mut **tx)
    .await?;

    let inverse = MapCommand::ResolveGhostSystem(match merged_into {
        Some(solar_system_id) => ResolveGhostSystem {
            map_id: cmd.map_id,
            map_solar_system_id: cmd.id,
            solar_system_id: Some(solar_system_id),
            ..Default::default()
        },
        // No edge to say where it went, which a ghost always has: the old answer stands.
        None => ResolveGhostSystem::unresolve(
            cmd.map_id,
            cmd.id,
            Raised {
                signature_pk: cmd.raised_by_signature_id,
                from_system: cmd.hangs_off_id,
            },
        ),
    });
    let mut events = vec![MapEvent::SystemAdded {
        map_id: cmd.map_id,
        map_solar_system_id: cmd.id,
    }];
    events.extend(
        cmd.from_connection_ids
            .iter()
            .chain(cmd.to_connection_ids.iter())
            .map(|id| MapEvent::ConnectionChanged {
                map_id: cmd.map_id,
                connection_id: *id,
            }),
    );
    Ok(Effect::new(
        "systems.added",
        "put the unscanned hole back",
        CommandOutput::System(Box::new(ghost)),
    )
    .undo_with(inverse)
    .emit_all(events))
}

/// The nodes these scans raised. Deleting a scan takes them by the foreign key, so this is
/// asked beforehand, while they are still there to be named.
pub(super) async fn raised_by(
    tx: &mut Tx<'_>,
    map_id: i64,
    signature_pks: &[i64],
) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar!(
        "select id from map_solar_systems
         where map_id = $1 and raised_by_signature_id = any($2)",
        map_id,
        signature_pks,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// The ghosts that removing these placements or connections would strand. A ghost with no
/// connection left is not a place at all and goes with whatever it hung off; real systems
/// are left alone even when they end up edgeless. Asked before the deletion, so the caller
/// can fold the answer into its snapshot and undo restores both in one step.
pub(super) async fn stranded_ghosts(
    tx: &mut Tx<'_>,
    map_id: i64,
    removed_systems: &[i64],
    removed_connections: &[i64],
) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar!(
        r#"select g.id as "id!" from map_solar_systems g
           where g.map_id = $1
             and g.solar_system_id is null
             and not g.is_home and not g.is_pinned and not g.is_rally
             and exists (
                 select 1 from map_connections c
                 where c.map_id = $1 and (c.from_system = g.id or c.to_system = g.id)
             )
             and not exists (
                 select 1 from map_connections c
                 where c.map_id = $1
                   and (c.from_system = g.id or c.to_system = g.id)
                   and c.id <> all($3)
                   and c.from_system <> all($2)
                   and c.to_system <> all($2)
             )"#,
        map_id,
        removed_systems,
        removed_connections,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// A hole is often flown before it is named. Once the far side is known, transits that
/// were recorded without a connection can be claimed by the edges of this placement.
async fn claim_pending_for(
    tx: &mut Tx<'_>,
    map_id: i64,
    placement_id: i64,
    solar_system_id: i64,
) -> Result<()> {
    let edges = sqlx::query!(
        r#"select c.id,
                  case when c.from_system = $2 then t.solar_system_id
                       else f.solar_system_id end as "other?"
           from map_connections c
           join map_solar_systems f on f.id = c.from_system
           join map_solar_systems t on t.id = c.to_system
           where c.map_id = $1 and (c.from_system = $2 or c.to_system = $2)
             and c.type = 'wormhole'"#,
        map_id,
        placement_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    for edge in edges {
        let Some(other) = edge.other else { continue };
        super::jumps::claim_pending_tx(tx, map_id, edge.id, solar_system_id, other).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::free_position;

    // The default grid: 20px cells, nodes 180x40, one cell of clear space. A step is
    // 200 across and 60 down.

    #[test]
    fn an_empty_map_gives_the_base_back_snapped_to_the_grid() {
        assert_eq!(free_position(&[], (400.0, 300.0)), (400.0, 300.0));
        assert_eq!(free_position(&[], (413.0, 307.0)), (420.0, 300.0));
    }

    #[test]
    fn a_base_outside_the_world_is_clamped_onto_it() {
        let (x, y) = free_position(&[], (99999.0, 99999.0));
        assert_eq!((x, y), (3820.0, 1960.0));
        assert_eq!(free_position(&[], (-50.0, -50.0)), (0.0, 0.0));
    }

    #[test]
    fn a_crowded_base_moves_one_column_right() {
        let placed = [(400.0, 300.0)];
        assert_eq!(free_position(&placed, (400.0, 300.0)), (600.0, 300.0));
    }

    #[test]
    fn siblings_stack_down_the_column_before_starting_the_next() {
        let placed = [(400.0, 300.0), (600.0, 300.0)];
        assert_eq!(free_position(&placed, (400.0, 300.0)), (600.0, 360.0));
    }

    #[test]
    fn a_full_column_spills_into_the_next_one() {
        // Everything from the top of column one to the bottom of the world is taken.
        let mut placed = vec![(400.0, 300.0)];
        let mut y = 300.0;
        while y <= 1960.0 {
            placed.push((600.0, y));
            y += 60.0;
        }
        assert_eq!(free_position(&placed, (400.0, 300.0)), (800.0, 300.0));
    }

    #[test]
    fn no_room_at_all_falls_back_to_the_base() {
        // A wall of nodes from the base to the right edge, every row, leaves nowhere to
        // go: the ghost lands on the base and overlaps rather than leaving the world.
        let mut placed = vec![];
        let mut x = 3700.0;
        while x <= 3820.0 {
            let mut y = 0.0;
            while y <= 1960.0 {
                placed.push((x, y));
                y += 60.0;
            }
            x += 200.0;
        }
        assert_eq!(free_position(&placed, (3700.0, 1000.0)), (3700.0, 1000.0));
    }
}
