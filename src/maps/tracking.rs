//! Building the chain from a jump.
//!
//! When a tracked character moves through a wormhole, the client offers the signatures in
//! the system it left and posts the one that was used. That single command places the new
//! system, connects it and links the signature, so a mis-picked signature is one undo
//! rather than three separate ones to unpick.
//!
//! Separate from [`super::jumps::record_transit`], which records the same movement for mass
//! accounting whether or not anyone is looking at the map. The two meet at
//! [`super::jumps::claim_pending_tx`]: a transit recorded before the hole was mapped is
//! claimed by the connection this creates.

use serde::{Deserialize, Serialize};

use super::command::{CommandOutput, Effect, MapCommand, Sequence, Tx};
use super::connection::{AddConnection, SetConnectionStatus, apply_add_connection};
use super::error::{MapError, Result};
use super::restore::RemoveRestored;
use super::signatures::{LinkSignature, UnlinkSignature, UpdateSignature};
use super::solar_system::{AddSystem, SetAlias};
use super::{ConnectionType, MassStatus, SignatureGroup, TimeStatus, WormholeSize};
use crate::realtime::MapEvent;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TrackJump {
    pub map_id: i64,
    /// The placement the character jumped *from*; it anchors the new system.
    pub from_map_solar_system_id: i64,
    pub to_solar_system_id: i64,
    /// Where to put the new system. The client owns the layout, so it picks the spot.
    pub x: f64,
    pub y: f64,
    /// The signature that turned out to be this hole. `None` = jumped an unscanned one.
    #[serde(default)]
    #[ts(optional)]
    pub signature_pk: Option<i64>,
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
}

/// Record a jump: place the system if it is new, connect it, and link the signature.
/// Member+.
pub async fn track_jump(pool: &sqlx::PgPool, actor: super::Actor, cmd: TrackJump) -> Result<()> {
    super::command::run(pool, actor, cmd).await
}

pub(super) async fn apply_track_jump(tx: &mut Tx<'_>, cmd: TrackJump) -> Result<Effect> {
    // The origin is where the character was, so it is always a real system; a ghost there
    // means the placement changed under the jump.
    let from_system = sqlx::query_scalar!(
        "select solar_system_id from map_solar_systems where id = $1 and map_id = $2",
        cmd.from_map_solar_system_id,
        cmd.map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .flatten()
    .ok_or(MapError::NotFound)?;

    if from_system == cmd.to_solar_system_id {
        return Err(MapError::Validation("a jump must change system".into()));
    }

    // Gate travel never builds a wormhole. The client checks too, off a cached stargate
    // graph, but this is the copy that decides.
    let is_gate = sqlx::query_scalar!(
        "select exists(
             select 1 from stargates
             where solar_system_id = $1 and destination_system_id = $2
         )",
        from_system,
        cmd.to_solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if is_gate {
        return Err(MapError::Conflict(
            "those systems are connected by a stargate".into(),
        ));
    }

    // Noted before anything touches it, so undo can put the signature back as it was.
    let before = match cmd.signature_pk {
        Some(pk) => Some(signature_state(tx, cmd.map_id, pk).await?),
        None => None,
    };

    let existing = sqlx::query_scalar!(
        r#"select c.id from map_connections c
           join map_solar_systems f on f.id = c.from_system
           join map_solar_systems t on t.id = c.to_system
           where c.map_id = $1
             and ((f.solar_system_id = $2 and t.solar_system_id = $3)
                  or (f.solar_system_id = $3 and t.solar_system_id = $2))
           limit 1"#,
        cmd.map_id,
        from_system,
        cmd.to_solar_system_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    // Already mapped: the only news is which signature it turned out to be.
    if let Some(connection_id) = existing {
        let Some(signature_pk) = cmd.signature_pk else {
            return Err(MapError::Conflict(
                "that connection is already on the map".into(),
            ));
        };
        link(tx, cmd.map_id, signature_pk, connection_id).await?;
        return Ok(Effect::new(
            "tracking.linked",
            "matched a jump to a signature",
            CommandOutput::None,
        )
        .undo_with(MapCommand::Sequence(Sequence {
            map_id: cmd.map_id,
            steps: undo_signature(cmd.map_id, before.as_ref()),
        }))
        .emit(MapEvent::SignatureChanged {
            map_id: cmd.map_id,
            solar_system_id: from_system,
        })
        .emit(MapEvent::ConnectionChanged {
            map_id: cmd.map_id,
            connection_id,
        }));
    }

    let target_name = sqlx::query_scalar!(
        "select name from solar_systems where id = $1",
        cmd.to_solar_system_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| {
        MapError::Validation(format!("unknown solar system {}", cmd.to_solar_system_id))
    })?;

    let placed = sqlx::query_scalar!(
        "select id from map_solar_systems where map_id = $1 and solar_system_id = $2",
        cmd.map_id,
        cmd.to_solar_system_id,
    )
    .fetch_optional(&mut **tx)
    .await?;

    // The client checks too, but this is the copy that decides: a pilot passing through
    // a trade hub the map keeps off does not put the hub on the chain. Already placed, it
    // is on the map by somebody's choice and stays reachable.
    if placed.is_none()
        && super::ignored::is_ignored_tx(tx, cmd.map_id, cmd.to_solar_system_id).await?
    {
        return Err(MapError::Validation(format!(
            "{target_name} is on this map's ignore list"
        )));
    }

    // A system reached by another route is linked to, not duplicated.
    let (to_placement, added) = match placed {
        Some(id) => {
            if let Some(alias) = cmd.alias.clone() {
                super::solar_system::apply_set_alias(
                    tx,
                    SetAlias {
                        map_id: cmd.map_id,
                        map_solar_system_id: id,
                        alias: Some(alias),
                    },
                )
                .await?;
            }
            (id, None)
        }
        None => {
            let effect = super::solar_system::apply_add_system(
                tx,
                AddSystem {
                    map_id: cmd.map_id,
                    solar_system_id: cmd.to_solar_system_id,
                    x: cmd.x,
                    y: cmd.y,
                    alias: cmd.alias.clone(),
                },
            )
            .await?;
            let system = effect.output.take::<super::MapSolarSystem>()?;
            (system.id, Some(system.id))
        }
    };

    let effect = apply_add_connection(
        tx,
        AddConnection {
            map_id: cmd.map_id,
            from_system: cmd.from_map_solar_system_id,
            to_system: to_placement,
            kind: ConnectionType::Wormhole,
            size: cmd.size,
        },
    )
    .await?;
    let connection = effect.output.take::<super::MapConnection>()?;
    let connection_id = connection.id;

    if cmd.mass_status.is_some() || cmd.time_status.is_some() {
        super::connection::apply_set_connection_status(
            tx,
            SetConnectionStatus {
                map_id: cmd.map_id,
                connection_id,
                kind: None,
                mass_status: cmd.mass_status.map(Some),
                time_status: cmd.time_status.map(Some),
                size: None,
                preserve_mass: None,
            },
        )
        .await?;
    }

    if let Some(signature_pk) = cmd.signature_pk {
        link(tx, cmd.map_id, signature_pk, connection_id).await?;
    }

    // Undo walks it back: the signature first, so clearing the link does not fight the
    // connection's own delete, then the edge and the system it brought with it.
    let mut steps = undo_signature(cmd.map_id, before.as_ref());
    steps.push(MapCommand::RemoveRestored(RemoveRestored {
        map_id: cmd.map_id,
        system_ids: added.into_iter().collect(),
        connection_ids: vec![connection_id],
        signature_ids: Vec::new(),
    }));
    let inverse = MapCommand::Sequence(Sequence {
        map_id: cmd.map_id,
        steps,
    });

    let label = match added {
        Some(_) => format!("jumped into {target_name}"),
        None => format!("connected {target_name} by jumping it"),
    };
    let mut events = Vec::new();
    if let Some(id) = added {
        events.push(MapEvent::SystemAdded {
            map_id: cmd.map_id,
            map_solar_system_id: id,
        });
    } else {
        events.push(MapEvent::SystemDetailsChanged {
            map_id: cmd.map_id,
            map_solar_system_id: to_placement,
        });
    }
    events.push(MapEvent::ConnectionChanged {
        map_id: cmd.map_id,
        connection_id,
    });
    if cmd.signature_pk.is_some() {
        events.push(MapEvent::SignatureChanged {
            map_id: cmd.map_id,
            solar_system_id: from_system,
        });
    }
    Ok(Effect::new(
        "tracking.jumped",
        label,
        CommandOutput::Connection(Box::new(connection)),
    )
    .undo_with(inverse)
    .emit_all(events))
}

/// What the signature looked like before the jump touched it.
#[derive(Debug, Clone)]
pub(super) struct SignatureState {
    pk: i64,
    group: SignatureGroup,
    signature_type_id: Option<i64>,
    connection_id: Option<i64>,
}

pub(super) async fn signature_state(
    tx: &mut Tx<'_>,
    map_id: i64,
    pk: i64,
) -> Result<SignatureState> {
    let row = sqlx::query!(
        r#"select "group", signature_type_id, connection_id
           from signatures where id = $1 and map_id = $2"#,
        pk,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)?;
    Ok(SignatureState {
        pk,
        group: row.group,
        signature_type_id: row.signature_type_id,
        connection_id: row.connection_id,
    })
}

/// Put the signature back to how it was scanned. Deliberately not `RestoreSignatures`: that
/// undoes a deletion, so its own inverse deletes the rows again and redoing a jump would
/// take the signature with it.
pub(super) fn undo_signature(map_id: i64, before: Option<&SignatureState>) -> Vec<MapCommand> {
    let Some(before) = before else {
        return Vec::new();
    };
    // Group first: dropping out of `wormhole` clears the link on its way past, so the
    // link/unlink has to be the last word on it.
    let mut steps = vec![MapCommand::UpdateSignature(UpdateSignature {
        map_id,
        signature_pk: before.pk,
        signature_id: None,
        group: Some(before.group),
        signature_type_id: Some(before.signature_type_id),
        name: None,
        size: None,
        mass_status: None,
        time_status: None,
    })];
    steps.push(match before.connection_id {
        Some(connection_id) => MapCommand::LinkSignature(LinkSignature {
            map_id,
            signature_pk: before.pk,
            connection_id,
        }),
        None => MapCommand::UnlinkSignature(UnlinkSignature {
            map_id,
            signature_pk: before.pk,
        }),
    });
    steps
}

/// Link a signature to the connection, promoting it to a wormhole first. A jumped signature
/// is usually still `unknown`: the id was scanned but never classified.
pub(super) async fn link(
    tx: &mut Tx<'_>,
    map_id: i64,
    signature_pk: i64,
    connection_id: i64,
) -> Result<()> {
    sqlx::query!(
        r#"update signatures set "group" = $1, updated_at = now()
           where id = $2 and map_id = $3 and "group" = 'unknown'"#,
        SignatureGroup::Wormhole,
        signature_pk,
        map_id,
    )
    .execute(&mut **tx)
    .await?;

    super::signatures::apply_link_signature(
        tx,
        LinkSignature {
            map_id,
            signature_pk,
            connection_id,
        },
    )
    .await?;
    Ok(())
}
