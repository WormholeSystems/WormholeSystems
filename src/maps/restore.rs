//! Snapshot and restore: the inverse of every removal. A snapshot captures placements and
//! everything that cascaded with them (connections, signatures), and restoring replays it
//! under the original ids so the pieces still line up.

use serde::{Deserialize, Serialize};

use super::command::{CommandOutput, Effect, MapCommand, Tx};
use super::error::Result;
use super::store::connections::{self, ConnectionRow};
use super::store::jumps::{self, JumpRow};
use super::store::placements::{self, PlacementRow};
use super::store::signatures::{self, SignatureRow};
use crate::realtime::MapEvent;

/// The inverse of a removal: placements plus everything that cascaded with them. Internal,
/// produced by the remove commands and never routed by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSystems {
    pub map_id: i64,
    pub systems: Vec<PlacementRow>,
    pub connections: Vec<ConnectionRow>,
    pub signatures: Vec<SignatureRow>,
    /// The jump log of those connections, which goes with them by the foreign key. Absent
    /// from snapshots taken before it was kept, which bring the connection back without it.
    #[serde(default)]
    pub jumps: Vec<JumpRow>,
}

impl RestoreSystems {
    pub(super) fn label(&self) -> String {
        match self.systems.len() {
            1 => "a system".to_string(),
            n => format!("{n} systems"),
        }
    }

    /// A connection-only snapshot restores the edge, not any placement.
    fn is_connection_only(&self) -> bool {
        self.systems.is_empty() && !self.connections.is_empty()
    }

    /// Take in another snapshot of the same map, each row once.
    pub(super) fn absorb(&mut self, other: RestoreSystems) {
        for s in other.systems {
            if !self.systems.iter().any(|known| known.id == s.id) {
                self.systems.push(s);
            }
        }
        for c in other.connections {
            if !self.connections.iter().any(|known| known.id == c.id) {
                self.connections.push(c);
            }
        }
        for s in other.signatures {
            if !self.signatures.iter().any(|known| known.id == s.id) {
                self.signatures.push(s);
            }
        }
        for j in other.jumps {
            if !self.jumps.iter().any(|known| known.id == j.id) {
                self.jumps.push(j);
            }
        }
    }
}

/// Take the signatures the snapshot claimed that the database will not cascade: the ones in
/// the systems staying on the map, linked to a connection that died with its endpoint.
/// Runs after the placements go, because deleting a scan takes the node it raised, and one
/// of those may be the placement this removal is about.
pub(super) async fn remove_captured_signatures(
    tx: &mut Tx<'_>,
    map_id: i64,
    snapshot: &RestoreSystems,
) -> Result<()> {
    let ids: Vec<i64> = snapshot.signatures.iter().map(|s| s.id).collect();
    if ids.is_empty() {
        return Ok(());
    }
    signatures::delete(tx, map_id, &ids).await?;
    Ok(())
}

/// Snapshot placements and their cascade so a removal can be undone.
pub(super) async fn capture_systems(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<RestoreSystems> {
    let systems = placements::snapshot(tx, map_id, ids).await?;
    // Ghosts hold no signatures, having no system to hold them against.
    let system_ids: Vec<i64> = systems.iter().filter_map(|s| s.solar_system_id).collect();
    let connections = connections::snapshot_touching(tx, map_id, ids).await?;
    // The placements on the far side of those edges as well, so the snapshot can stand on
    // its own. A restore puts its placements back before its edges, but an edge whose
    // other end it does not carry depends on that end still being there -- and it may not
    // be. Replaying a removal walks through states the original never had: taking a scan
    // leaves its hole's far side with no edges for a moment, `delete_if_orphan` clears it,
    // and the restore that follows is left holding an edge to nothing. Reinserting a
    // placement that never went is a no-op, so carrying them costs nothing.
    let far_ends: Vec<i64> = connections
        .iter()
        .flat_map(|c| [c.from_system, c.to_system])
        .filter(|id| !ids.contains(id))
        .collect();
    let systems = {
        let mut all = systems;
        all.extend(placements::snapshot(tx, map_id, &far_ends).await?);
        all
    };
    // Both sides of the hole: this system's own scan, and the signature in the system
    // across each dying connection.
    let connection_ids: Vec<i64> = connections.iter().map(|c| c.id).collect();
    let signatures = signatures::snapshot_around(tx, map_id, &system_ids, &connection_ids).await?;
    let jumps = jumps::snapshot_through(tx, map_id, &connection_ids).await?;

    Ok(RestoreSystems {
        map_id,
        systems,
        connections,
        signatures,
        jumps,
    })
}

pub(super) async fn apply_restore_systems(tx: &mut Tx<'_>, cmd: RestoreSystems) -> Result<Effect> {
    // Original ids are reused so the restored connections and signatures still line up.
    // A snapshot carries the placements on the far side of its edges as well, which are
    // usually still there; only the ones this restore actually put back are its to take
    // away again.
    let mut restored_systems = Vec::new();
    for s in &cmd.systems {
        if placements::reinsert(tx, cmd.map_id, s).await? {
            restored_systems.push(s.id);
        }
    }
    for c in &cmd.connections {
        connections::reinsert(tx, cmd.map_id, c).await?;
    }
    let mut inserted = Vec::new();
    for s in &cmd.signatures {
        if signatures::reinsert(tx, cmd.map_id, s).await? {
            inserted.push(s.id);
        }
    }
    for j in &cmd.jumps {
        jumps::reinsert(tx, cmd.map_id, j).await?;
    }

    // Undoing a restore drops exactly what was put back, connections and signatures
    // included: a stale-clean can restore edges whose endpoints were never removed, and a
    // removal takes the scan on the far side of a dying hole from a system that stays, so
    // cascading from the placements alone would leave those behind. A signature that was
    // still there and only got its link back stays when the link goes again.
    let inverse = MapCommand::RemoveRestored(RemoveRestored {
        map_id: cmd.map_id,
        system_ids: restored_systems,
        connection_ids: cmd.connections.iter().map(|c| c.id).collect(),
        signature_ids: inserted,
    });
    if cmd.is_connection_only() {
        let events: Vec<MapEvent> = cmd
            .connections
            .iter()
            .map(|c| MapEvent::ConnectionChanged {
                map_id: cmd.map_id,
                connection_id: c.id,
            })
            .collect();
        return Ok(Effect::new(
            "connections.restored",
            "restored a connection",
            CommandOutput::None,
        )
        .undo_with(inverse)
        .emit_all(events));
    }
    let label = format!("restored {}", cmd.label());
    let count = cmd.systems.len() as i64;
    Ok(Effect::new("systems.restored", label, CommandOutput::None)
        .entries(count)
        .undo_with(inverse)
        .emit(MapEvent::MapUpdated { map_id: cmd.map_id }))
}

/// The inverse of a restore: drop exactly these placements and edges. The two commands are
/// each other's inverse, so undo and redo cycle without drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRestored {
    pub map_id: i64,
    pub system_ids: Vec<i64>,
    pub connection_ids: Vec<i64>,
    /// Signatures added to placements that stay: the database cascades only the ones on
    /// placements that go.
    #[serde(default)]
    pub signature_ids: Vec<i64>,
}

pub(super) async fn apply_remove_restored(tx: &mut Tx<'_>, cmd: RemoveRestored) -> Result<Effect> {
    let mut snapshot = capture_systems(tx, cmd.map_id, &cmd.system_ids).await?;
    for id in &cmd.connection_ids {
        if let Some(extra) = capture_connection(tx, cmd.map_id, *id).await? {
            snapshot.absorb(extra);
        }
    }
    let extra = super::signatures::capture_signatures(tx, cmd.map_id, &cmd.signature_ids).await?;
    snapshot.absorb(RestoreSystems {
        map_id: cmd.map_id,
        systems: Vec::new(),
        connections: Vec::new(),
        signatures: extra.signatures,
        jumps: Vec::new(),
    });
    signatures::delete(tx, cmd.map_id, &cmd.signature_ids).await?;
    connections::delete(tx, cmd.map_id, &cmd.connection_ids).await?;
    placements::delete(tx, cmd.map_id, &cmd.system_ids).await?;

    let count = (cmd.system_ids.len() + cmd.connection_ids.len() + cmd.signature_ids.len()) as i64;
    Ok(
        Effect::new("systems.removed", "undid a restore", CommandOutput::None)
            .entries(count)
            .undo_with(MapCommand::RestoreSystems(snapshot))
            .emit(MapEvent::MapUpdated { map_id: cmd.map_id }),
    )
}

/// Snapshot one connection so removing it can be undone, with the signatures linked to it
/// (they outlive it unlinked, and restoring it links them again) and its jump log.
pub(super) async fn capture_connection(
    tx: &mut Tx<'_>,
    map_id: i64,
    connection_id: i64,
) -> Result<Option<RestoreSystems>> {
    let Some(connection) = connections::snapshot(tx, map_id, connection_id).await? else {
        return Ok(None);
    };
    let signatures = signatures::snapshot_around(tx, map_id, &[], &[connection_id]).await?;
    let jumps = jumps::snapshot_through(tx, map_id, &[connection_id]).await?;
    Ok(Some(RestoreSystems {
        map_id,
        systems: Vec::new(),
        connections: vec![connection],
        signatures,
        jumps,
    }))
}
