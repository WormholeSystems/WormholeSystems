//! Putting EVE Scout's public holes on the map.
//!
//! EVE Scout scans the wormholes out of Thera and Turnur and publishes them. One command
//! places the hub, every system on the far side of its holes, the connections between
//! them, and the signatures on both ends, so a scout's list becomes part of the chain in
//! a single undoable step. What is on the map already is reused, never duplicated: the
//! command is safe to repeat as the list changes. Built to
//! [`docs/features/maps.md`](../../docs/features/maps.md#add_eve_scout_connectionsactor-map_id-hub_solar_system_id-at---count).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::command::{CommandOutput, Effect, MapCommand, Sequence, Tx, run};
use super::connection::{AddConnection, SetConnectionStatus};
use super::error::{MapError, Result};
use super::restore::RemoveRestored;
use super::signatures::AddSignature;
use super::solar_system::AddSystem;
use super::tracking::{SignatureState, signature_state, undo_signature};
use super::{
    Actor, ConnectionType, MapEvent, MassStatus, SignatureGroup, TimeStatus, WormholeSize,
};
use crate::util::security::ccp_round_security;

/// Thera and Turnur, the two systems EVE Scout keeps public connections for.
pub const HUBS: [(i64, &str); 2] = [(31000005, "Thera"), (30002086, "Turnur")];

/// Where a hub lands on a map that does not have it yet and nobody said where to put it.
const DEFAULT_HUB_SPOT: (f64, f64) = (200.0, 100.0);

pub fn hub_name(solar_system_id: i64) -> Option<&'static str> {
    HUBS.iter()
        .find(|(id, _)| *id == solar_system_id)
        .map(|(_, name)| *name)
}

/// One public hole as EVE Scout reports it, oriented hub-first and already in the map's
/// own vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EveScoutHole {
    pub solar_system_id: i64,
    pub hub_signature: String,
    pub signature: String,
    /// The wormhole code, e.g. `J377`, when a scout has identified it.
    pub wormhole_type: Option<String>,
    pub mass_status: MassStatus,
    pub time_status: TimeStatus,
    pub size: Option<WormholeSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEveScoutConnections {
    pub map_id: i64,
    pub hub_solar_system_id: i64,
    /// Where to put the hub if it is not on the map yet.
    pub at: Option<(f64, f64)>,
    /// The hub's current holes, as the API fetched them.
    pub holes: Vec<EveScoutHole>,
}

/// Put a hub's public holes on the map. Member+. Returns how many connections were added.
pub async fn add_eve_scout_connections(
    pool: &PgPool,
    actor: Actor,
    cmd: AddEveScoutConnections,
) -> Result<u64> {
    run(pool, actor, cmd).await
}

/// What one run put on the map, for the label, the undo and the events.
#[derive(Default)]
struct Outcome {
    system_ids: Vec<i64>,
    connection_ids: Vec<i64>,
    signature_ids: Vec<i64>,
    touched: Vec<SignatureState>,
    changed_systems: Vec<i64>,
}

pub(super) async fn apply_add_eve_scout_connections(
    tx: &mut Tx<'_>,
    cmd: AddEveScoutConnections,
) -> Result<Effect> {
    let hub = hub_name(cmd.hub_solar_system_id).ok_or_else(|| {
        MapError::Validation("EVE Scout only publishes holes out of Thera and Turnur".into())
    })?;

    let holes = sorted_far_sides(tx, &cmd).await?;
    let mut outcome = Outcome::default();
    let mut placed = placements(tx, cmd.map_id).await?;

    let hub_placement = match placed
        .iter()
        .find(|p| p.solar_system_id == cmd.hub_solar_system_id)
    {
        Some(p) => p.id,
        None => {
            let spot = free_spot(&placed, cmd.at.unwrap_or(DEFAULT_HUB_SPOT));
            let id = place(tx, cmd.map_id, cmd.hub_solar_system_id, spot).await?;
            placed.push(Placement {
                id,
                solar_system_id: cmd.hub_solar_system_id,
                at: spot,
            });
            outcome.system_ids.push(id);
            id
        }
    };
    let hub_at = placed
        .iter()
        .find(|p| p.id == hub_placement)
        .map(|p| p.at)
        .unwrap_or(DEFAULT_HUB_SPOT);

    for hole in holes {
        let far_placement = match placed
            .iter()
            .find(|p| p.solar_system_id == hole.solar_system_id)
        {
            Some(p) => p.id,
            None => {
                let spot = free_spot(&placed, hub_at);
                let id = place(tx, cmd.map_id, hole.solar_system_id, spot).await?;
                placed.push(Placement {
                    id,
                    solar_system_id: hole.solar_system_id,
                    at: spot,
                });
                outcome.system_ids.push(id);
                id
            }
        };

        let connection_id =
            match existing_connection(tx, cmd.map_id, hub_placement, far_placement).await? {
                Some(id) => id,
                None => {
                    let id = connect(tx, cmd.map_id, hub_placement, far_placement, &hole).await?;
                    outcome.connection_ids.push(id);
                    id
                }
            };

        let type_id = match &hole.wormhole_type {
            Some(code) => wormhole_signature_type(tx, code).await?,
            None => None,
        };
        for (solar_system_id, signature_id) in [
            (cmd.hub_solar_system_id, hole.hub_signature.as_str()),
            (hole.solar_system_id, hole.signature.as_str()),
        ] {
            ensure_signature(
                tx,
                cmd.map_id,
                solar_system_id,
                signature_id,
                type_id,
                &hole,
                connection_id,
                &mut outcome,
            )
            .await?;
        }
    }

    let added = outcome.connection_ids.len() as u64;
    let label = match added {
        0 => format!("checked EVE Scout's holes out of {hub}"),
        1 => format!("added an EVE Scout hole out of {hub}"),
        n => format!("added {n} EVE Scout holes out of {hub}"),
    };
    let changed = outcome.system_ids.len()
        + outcome.connection_ids.len()
        + outcome.signature_ids.len()
        + outcome.touched.len();
    let mut effect = Effect::new("evescout.added", label, CommandOutput::Count(added))
        .entries((changed as i64).max(1));
    if changed == 0 {
        return Ok(effect);
    }

    // Signatures first, back to how they were scanned; then everything the run added.
    let mut steps: Vec<MapCommand> = outcome
        .touched
        .iter()
        .flat_map(|before| undo_signature(cmd.map_id, Some(before)))
        .collect();
    steps.push(MapCommand::RemoveRestored(RemoveRestored {
        map_id: cmd.map_id,
        system_ids: outcome.system_ids.clone(),
        connection_ids: outcome.connection_ids.clone(),
        signature_ids: outcome.signature_ids.clone(),
    }));
    effect = effect.undo_with(MapCommand::Sequence(Sequence {
        map_id: cmd.map_id,
        steps,
    }));

    let mut events: Vec<MapEvent> = outcome
        .system_ids
        .iter()
        .map(|id| MapEvent::SystemAdded {
            map_id: cmd.map_id,
            map_solar_system_id: *id,
        })
        .collect();
    events.extend(
        outcome
            .connection_ids
            .iter()
            .map(|id| MapEvent::ConnectionChanged {
                map_id: cmd.map_id,
                connection_id: *id,
            }),
    );
    outcome.changed_systems.sort_unstable();
    outcome.changed_systems.dedup();
    events.extend(
        outcome
            .changed_systems
            .iter()
            .map(|id| MapEvent::SignatureChanged {
                map_id: cmd.map_id,
                solar_system_id: *id,
            }),
    );
    Ok(effect.emit_all(events))
}

struct Placement {
    id: i64,
    solar_system_id: i64,
    at: (f64, f64),
}

/// Every real placement on the map, ghosts included for their positions only.
async fn placements(tx: &mut Tx<'_>, map_id: i64) -> Result<Vec<Placement>> {
    let rows = sqlx::query!(
        "select id, solar_system_id, position_x, position_y from map_solar_systems where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Placement {
            id: r.id,
            // A ghost matches no system, so it can never be reused as one.
            solar_system_id: r.solar_system_id.unwrap_or(0),
            at: (r.position_x, r.position_y),
        })
        .collect())
}

fn free_spot(placed: &[Placement], base: (f64, f64)) -> (f64, f64) {
    let taken: Vec<(f64, f64)> = placed.iter().map(|p| p.at).collect();
    super::ghost::free_position(&taken, base)
}

/// The holes whose far side the SDE knows, in the order a scout looks for an exit: known
/// space by security, safest first, then wormhole space by class.
async fn sorted_far_sides(
    tx: &mut Tx<'_>,
    cmd: &AddEveScoutConnections,
) -> Result<Vec<EveScoutHole>> {
    let ids: Vec<i64> = cmd.holes.iter().map(|h| h.solar_system_id).collect();
    let rows = sqlx::query!(
        r#"select s.id as "id!", s.security_status as "security_status!",
                  w.wormhole_class_id as "wormhole_class_id?"
           from solar_systems s
           left join wormhole_systems w on w.solar_system_id = s.id
           where s.id = any($1)"#,
        &ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut keyed: Vec<(i64, i32, EveScoutHole)> = cmd
        .holes
        .iter()
        .filter(|h| h.solar_system_id != cmd.hub_solar_system_id)
        .filter_map(|hole| {
            let row = rows.iter().find(|r| r.id == hole.solar_system_id)?;
            let key = match row.wormhole_class_id {
                Some(class) => (1, class),
                // Negated so that the safest sorts first with one ascending sort.
                None => (
                    0,
                    -(ccp_round_security(row.security_status) * 10.0).round() as i32,
                ),
            };
            Some((key.0, key.1, hole.clone()))
        })
        .collect();
    keyed.sort_by_key(|(space, rank, _)| (*space, *rank));
    Ok(keyed.into_iter().map(|(_, _, hole)| hole).collect())
}

async fn place(tx: &mut Tx<'_>, map_id: i64, solar_system_id: i64, at: (f64, f64)) -> Result<i64> {
    let effect = super::solar_system::apply_add_system(
        tx,
        AddSystem {
            map_id,
            solar_system_id,
            x: at.0,
            y: at.1,
            alias: None,
        },
    )
    .await?;
    Ok(effect.output.take::<super::MapSolarSystem>()?.id)
}

async fn existing_connection(tx: &mut Tx<'_>, map_id: i64, a: i64, b: i64) -> Result<Option<i64>> {
    Ok(sqlx::query_scalar!(
        "select id from map_connections
         where map_id = $1
           and ((from_system = $2 and to_system = $3) or (from_system = $3 and to_system = $2))
         limit 1",
        map_id,
        a,
        b,
    )
    .fetch_optional(&mut **tx)
    .await?)
}

async fn connect(
    tx: &mut Tx<'_>,
    map_id: i64,
    hub: i64,
    far: i64,
    hole: &EveScoutHole,
) -> Result<i64> {
    let effect = super::connection::apply_add_connection(
        tx,
        AddConnection {
            map_id,
            from_system: hub,
            to_system: far,
            kind: ConnectionType::Wormhole,
            size: hole.size,
        },
    )
    .await?;
    let connection_id = effect.output.take::<super::MapConnection>()?.id;
    super::connection::apply_set_connection_status(
        tx,
        SetConnectionStatus {
            map_id,
            connection_id,
            mass_status: Some(Some(hole.mass_status)),
            time_status: Some(Some(hole.time_status)),
            ..Default::default()
        },
    )
    .await?;
    Ok(connection_id)
}

/// The catalogue row for a wormhole code, if the catalogue has one.
async fn wormhole_signature_type(tx: &mut Tx<'_>, code: &str) -> Result<Option<i64>> {
    Ok(sqlx::query_scalar!(
        "select id from signature_types
         where signature = $1 and signature_category_id = $2
         limit 1",
        code,
        super::signatures::category_id_for(SignatureGroup::Wormhole),
    )
    .fetch_optional(&mut **tx)
    .await?)
}

/// See that `signature_id` in `solar_system_id` is a wormhole linked to `connection_id`,
/// adding or promoting it as needed. Skips ids EVE Scout has not scanned yet.
#[allow(clippy::too_many_arguments)]
async fn ensure_signature(
    tx: &mut Tx<'_>,
    map_id: i64,
    solar_system_id: i64,
    signature_id: &str,
    type_id: Option<i64>,
    hole: &EveScoutHole,
    connection_id: i64,
    outcome: &mut Outcome,
) -> Result<()> {
    if signature_id.len() != 7 {
        return Ok(());
    }
    let existing = sqlx::query!(
        r#"select id, "group" as "group: SignatureGroup", connection_id
           from signatures
           where map_id = $1 and solar_system_id = $2 and signature_id = $3"#,
        map_id,
        solar_system_id,
        signature_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    let Some(existing) = existing else {
        let effect = super::signatures::apply_add_signature(
            tx,
            AddSignature {
                map_id,
                solar_system_id,
                signature_id: signature_id.to_string(),
                group: SignatureGroup::Wormhole,
                signature_type_id: type_id,
                name: None,
                size: hole.size,
                mass_status: Some(hole.mass_status),
                time_status: Some(hole.time_status),
            },
        )
        .await?;
        let pk = effect.output.take::<super::Signature>()?.id;
        super::tracking::link(tx, map_id, pk, connection_id).await?;
        outcome.signature_ids.push(pk);
        outcome.changed_systems.push(solar_system_id);
        return Ok(());
    };

    // Somebody's own scan wins: a linked hole, or one classed as something else, stands.
    let unclaimed = existing.connection_id.is_none()
        && matches!(
            existing.group,
            SignatureGroup::Unknown | SignatureGroup::Wormhole
        );
    if !unclaimed {
        return Ok(());
    }
    outcome
        .touched
        .push(signature_state(tx, map_id, existing.id).await?);
    sqlx::query!(
        r#"update signatures
           set "group" = $1, signature_type_id = coalesce(signature_type_id, $2), updated_at = now()
           where id = $3 and map_id = $4"#,
        SignatureGroup::Wormhole,
        type_id,
        existing.id,
        map_id,
    )
    .execute(&mut **tx)
    .await?;
    super::tracking::link(tx, map_id, existing.id, connection_id).await?;
    outcome.changed_systems.push(solar_system_id);
    Ok(())
}

/// EVE Scout's ship-size words, in the map's sizes. Anything unrecognised is unknown.
pub fn size_from_eve_scout(word: &str) -> Option<WormholeSize> {
    let word = word.to_ascii_lowercase();
    if word.contains("frig") || word.contains("small") {
        Some(WormholeSize::Small)
    } else if word.contains("medium") {
        Some(WormholeSize::Medium)
    } else if word.contains("cap") || word.contains("xl") || word.contains("xlarge") {
        Some(WormholeSize::Xl)
    } else if word.contains("large") {
        Some(WormholeSize::Large)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eve_scout_ship_sizes_map_onto_the_maps_sizes() {
        assert_eq!(size_from_eve_scout("frigate"), Some(WormholeSize::Small));
        assert_eq!(size_from_eve_scout("medium"), Some(WormholeSize::Medium));
        assert_eq!(size_from_eve_scout("large"), Some(WormholeSize::Large));
        assert_eq!(size_from_eve_scout("xlarge"), Some(WormholeSize::Xl));
        assert_eq!(size_from_eve_scout("capital"), Some(WormholeSize::Xl));
        assert_eq!(size_from_eve_scout(""), None);
    }

    #[test]
    fn only_the_two_hubs_are_hubs() {
        assert_eq!(hub_name(31000005), Some("Thera"));
        assert_eq!(hub_name(30002086), Some("Turnur"));
        assert_eq!(hub_name(30000142), None);
    }
}
