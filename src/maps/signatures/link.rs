//! Saying which connection a wormhole signature is, and letting it go again.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::maps::command::{CommandOutput, Effect, MapCommand, Sequence, Tx, run};
use crate::maps::connection::SetConnectionStatus;
use crate::maps::error::{MapError, Result};
use crate::maps::store;
use crate::maps::{Actor, MapEvent};

use super::{Signature, UpdateSignature, fetch_signature_tx, is_wormhole};

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct LinkSignature {
    pub map_id: i64,
    pub signature_pk: i64,
    pub connection_id: i64,
}

/// Link a wormhole signature to a connection as one of its ends. Fires the DB sync, which
/// reconciles the connection and its signatures to the worst state per field (then keeps
/// them in lock-step). Returns the signature *after* the merge. The signature must be a
/// `wormhole`, and the connection must be on this map and have an endpoint in the
/// signature's system.
pub async fn link_signature(pool: &PgPool, actor: Actor, cmd: LinkSignature) -> Result<Signature> {
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_link_signature(
    tx: &mut Tx<'_>,
    cmd: LinkSignature,
) -> Result<Effect> {
    let sig = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;
    if !is_wormhole(sig.group) {
        return Err(MapError::Validation(
            "only a wormhole signature can link to a connection".into(),
        ));
    }

    // The connection must be on this map and touch the signature's system (one of its two
    // endpoint placements is in that solar system).
    let ok = sqlx::query_scalar!(
        "select exists(
             select 1
             from map_connections c
             join map_solar_systems mss on mss.id in (c.from_system, c.to_system)
             where c.id = $1 and c.map_id = $2 and mss.solar_system_id = $3
         )",
        cmd.connection_id,
        cmd.map_id,
        sig.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if !ok {
        return Err(MapError::Validation(
            "connection must be on this map and reach the signature's system".into(),
        ));
    }

    // One signature per side: the group's state is the hole's, and two signatures sharing
    // it would mean two scans of one hole. Saying "this one is the hole" therefore lets
    // the other go, rather than refusing; the loose one keeps its own scanned state and
    // can take the connection it really belongs to.
    // The connection's state before the merge raises it, for the undo.
    let connection = sqlx::query!(
        "select mass_status, time_status, size from map_connections
         where id = $1 and map_id = $2",
        cmd.connection_id,
        cmd.map_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    let displaced = sqlx::query_scalar!(
        "update signatures set connection_id = null, updated_at = now()
         where map_id = $1 and connection_id = $2 and solar_system_id = $3 and id <> $4
         returning id",
        cmd.map_id,
        cmd.connection_id,
        sig.solar_system_id,
        cmd.signature_pk,
    )
    .fetch_optional(&mut **tx)
    .await?;

    store::signatures::set_connection(tx, cmd.map_id, cmd.signature_pk, Some(cmd.connection_id))
        .await?;

    // Re-read: the merge trigger may have changed this row's state after the UPDATE.
    let linked = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;
    // Undo takes the link back first, so the old life-cycle state goes back onto this
    // signature alone and the connection's own state can be put back beside it. Then the
    // connection goes back to the signature this link took it from, if any.
    let unlink = match sig.connection_id {
        Some(old) if old == cmd.connection_id => None,
        Some(old) => Some(MapCommand::LinkSignature(LinkSignature {
            map_id: cmd.map_id,
            signature_pk: cmd.signature_pk,
            connection_id: old,
        })),
        None => Some(MapCommand::UnlinkSignature(UnlinkSignature {
            map_id: cmd.map_id,
            signature_pk: cmd.signature_pk,
        })),
    };
    let restore_state = MapCommand::UpdateSignature(UpdateSignature {
        map_id: cmd.map_id,
        signature_pk: cmd.signature_pk,
        signature_id: None,
        group: None,
        signature_type_id: None,
        name: None,
        size: Some(sig.size),
        mass_status: Some(sig.mass_status),
        time_status: Some(sig.time_status),
    });
    let restore_connection = MapCommand::SetConnectionStatus(SetConnectionStatus {
        map_id: cmd.map_id,
        connection_id: cmd.connection_id,
        mass_status: Some(connection.mass_status),
        time_status: Some(connection.time_status),
        size: Some(connection.size),
        ..SetConnectionStatus::default()
    });
    let mut steps: Vec<MapCommand> = unlink.into_iter().collect();
    steps.extend([restore_state, restore_connection]);
    steps.extend(displaced.map(|signature_pk| {
        MapCommand::LinkSignature(LinkSignature {
            map_id: cmd.map_id,
            signature_pk,
            connection_id: cmd.connection_id,
        })
    }));
    let inverse = MapCommand::Sequence(Sequence {
        map_id: cmd.map_id,
        steps,
    });
    let events = [
        MapEvent::SignatureChanged {
            map_id: cmd.map_id,
            solar_system_id: linked.solar_system_id,
        },
        MapEvent::ConnectionChanged {
            map_id: cmd.map_id,
            connection_id: cmd.connection_id,
        },
    ];
    Ok(Effect::new(
        "signatures.linked",
        "linked a signature",
        CommandOutput::Signature(Box::new(linked)),
    )
    .undo_with(inverse)
    .emit_all(events))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UnlinkSignature {
    pub map_id: i64,
    pub signature_pk: i64,
}

/// Detach a signature from its connection. The signature keeps its last state as a standalone
/// scanned wormhole; the connection and any sibling signature are untouched.
pub async fn unlink_signature(
    pool: &PgPool,
    actor: Actor,
    cmd: UnlinkSignature,
) -> Result<Signature> {
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_unlink_signature(
    tx: &mut Tx<'_>,
    cmd: UnlinkSignature,
) -> Result<Effect> {
    let before = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;
    store::signatures::set_connection(tx, cmd.map_id, cmd.signature_pk, None).await?;
    let sig = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;
    // Unlinking something that was never linked has nothing to restore.
    let inverse = before.connection_id.map(|connection_id| {
        MapCommand::LinkSignature(LinkSignature {
            map_id: cmd.map_id,
            signature_pk: cmd.signature_pk,
            connection_id,
        })
    });
    let event = MapEvent::SignatureChanged {
        map_id: cmd.map_id,
        solar_system_id: sig.solar_system_id,
    };
    let mut effect = Effect::new(
        "signatures.unlinked",
        "unlinked a signature",
        CommandOutput::Signature(Box::new(sig)),
    )
    .emit(event);
    if let Some(inverse) = inverse {
        effect = effect.undo_with(inverse);
    }
    Ok(effect)
}
