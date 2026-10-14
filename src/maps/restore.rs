//! Snapshot and restore: the inverse of every removal. A snapshot captures placements and
//! everything that cascaded with them (connections, signatures), and restoring replays it
//! under the original ids so the pieces still line up.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::command::{CommandOutput, Effect, MapCommand, Tx};
use super::error::Result;
use super::{ConnectionType, MapEvent, MassStatus, SignatureGroup, TimeStatus, WormholeSize};

/// One removed placement, with everything needed to put it back exactly as it was.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredSystem {
    pub id: i64,
    pub solar_system_id: Option<i64>,
    pub position_x: f64,
    pub position_y: f64,
    pub alias: Option<String>,
    pub is_home: bool,
    pub is_rally: bool,
    pub is_pinned: bool,
    /// Set on a ghost, which is not allowed back on the map without naming its scan and
    /// the system it hung off. `None` on a system somebody placed.
    #[serde(default)]
    pub raised_by_signature_id: Option<i64>,
    #[serde(default)]
    pub hangs_off_id: Option<i64>,
    /// Comes back with the signatures, so the panel still says how old they are.
    #[serde(default)]
    pub signatures_pasted_at: Option<DateTime<Utc>>,
}

/// A connection that cascaded away with its endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredConnection {
    pub id: i64,
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionType,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub size: Option<WormholeSize>,
    pub preserve_mass: bool,
}

/// A signature that cascaded away with its system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredSignature {
    pub id: i64,
    pub solar_system_id: i64,
    pub signature_id: String,
    pub group: SignatureGroup,
    pub signature_type_id: Option<i64>,
    pub name: Option<String>,
    pub size: Option<WormholeSize>,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub connection_id: Option<i64>,
}

/// The inverse of a removal: placements plus everything that cascaded with them. Internal,
/// produced by the remove commands and never routed by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSystems {
    pub map_id: i64,
    pub systems: Vec<RestoredSystem>,
    pub connections: Vec<RestoredConnection>,
    pub signatures: Vec<RestoredSignature>,
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
    sqlx::query!(
        "delete from signatures where map_id = $1 and id = any($2)",
        map_id,
        &ids,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Snapshot placements and their cascade so a removal can be undone.
pub(super) async fn capture_systems(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<RestoreSystems> {
    let systems: Vec<RestoredSystem> = sqlx::query_as!(
        RestoredSystem,
        "select id, solar_system_id, position_x, position_y, alias, is_home, is_rally, is_pinned,
                raised_by_signature_id, hangs_off_id, signatures_pasted_at
         from map_solar_systems where map_id = $1 and id = any($2)",
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?;
    // Ghosts hold no signatures, having no system to hold them against.
    let system_ids: Vec<i64> = systems.iter().filter_map(|s| s.solar_system_id).collect();

    let connections = sqlx::query_as!(
        RestoredConnection,
        r#"select id, from_system, to_system, type as kind,
                  mass_status,
                  time_status,
                  size, preserve_mass
           from map_connections
           where map_id = $1 and (from_system = any($2) or to_system = any($2))"#,
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    // Both sides of the hole: this system's own scan, and the signature in the system
    // across each dying connection.
    let connection_ids: Vec<i64> = connections.iter().map(|c| c.id).collect();
    let signatures = sqlx::query_as!(
        RestoredSignature,
        r#"select id, solar_system_id, signature_id, "group",
                  signature_type_id, name, size,
                  mass_status,
                  time_status, connection_id
           from signatures
           where map_id = $1
             and (solar_system_id = any($2) or connection_id = any($3))"#,
        map_id,
        &system_ids,
        &connection_ids,
    )
    .fetch_all(&mut **tx)
    .await?;

    Ok(RestoreSystems {
        map_id,
        systems,
        connections,
        signatures,
    })
}

pub(super) async fn apply_restore_systems(tx: &mut Tx<'_>, cmd: RestoreSystems) -> Result<Effect> {
    // Original ids are reused so the restored connections and signatures still line up.
    for s in &cmd.systems {
        sqlx::query!(
            "insert into map_solar_systems
                 (id, map_id, solar_system_id, position_x, position_y, alias,
                  is_home, is_rally, is_pinned, raised_by_signature_id, hangs_off_id,
                  signatures_pasted_at)
             overriding system value
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             on conflict (id) do nothing",
            s.id,
            cmd.map_id,
            s.solar_system_id,
            s.position_x,
            s.position_y,
            s.alias.as_deref(),
            s.is_home,
            s.is_rally,
            s.is_pinned,
            s.raised_by_signature_id,
            s.hangs_off_id,
            s.signatures_pasted_at,
        )
        .execute(&mut **tx)
        .await?;
    }
    for c in &cmd.connections {
        sqlx::query!(
            "insert into map_connections
                 (id, map_id, from_system, to_system, type, mass_status, time_status, size,
                  preserve_mass)
             overriding system value
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             on conflict (id) do nothing",
            c.id,
            cmd.map_id,
            c.from_system,
            c.to_system,
            c.kind,
            c.mass_status,
            c.time_status,
            c.size,
            c.preserve_mass,
        )
        .execute(&mut **tx)
        .await?;
    }
    for s in &cmd.signatures {
        sqlx::query!(
            r#"insert into signatures
                   (id, map_id, solar_system_id, signature_id, "group", signature_type_id,
                    name, size, mass_status, time_status, connection_id)
               overriding system value
               values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               on conflict (id) do nothing"#,
            s.id,
            cmd.map_id,
            s.solar_system_id,
            s.signature_id,
            s.group,
            s.signature_type_id,
            s.name.as_deref(),
            s.size,
            s.mass_status,
            s.time_status,
            s.connection_id,
        )
        .execute(&mut **tx)
        .await?;
    }

    // Undoing a restore drops exactly what was put back, connections included: a stale-clean
    // can restore edges whose endpoints were never removed, and cascading from the
    // placements alone would leave those behind.
    let inverse = MapCommand::RemoveRestored(RemoveRestored {
        map_id: cmd.map_id,
        system_ids: cmd.systems.iter().map(|s| s.id).collect(),
        connection_ids: cmd.connections.iter().map(|c| c.id).collect(),
        signature_ids: Vec::new(),
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
        if let Some(extra) = capture_connection(tx, cmd.map_id, *id).await?
            && !snapshot.connections.iter().any(|c| c.id == *id)
        {
            snapshot.connections.extend(extra.connections);
        }
    }
    let extra = super::signatures::capture_signatures(tx, cmd.map_id, &cmd.signature_ids).await?;
    for s in extra.signatures {
        if !snapshot.signatures.iter().any(|known| known.id == s.id) {
            snapshot.signatures.push(s);
        }
    }
    sqlx::query!(
        "delete from signatures where map_id = $1 and id = any($2)",
        cmd.map_id,
        &cmd.signature_ids,
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "delete from map_connections where map_id = $1 and id = any($2)",
        cmd.map_id,
        &cmd.connection_ids,
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        "delete from map_solar_systems where map_id = $1 and id = any($2)",
        cmd.map_id,
        &cmd.system_ids,
    )
    .execute(&mut **tx)
    .await?;

    let count = (cmd.system_ids.len() + cmd.connection_ids.len() + cmd.signature_ids.len()) as i64;
    Ok(
        Effect::new("systems.removed", "undid a restore", CommandOutput::None)
            .entries(count)
            .undo_with(MapCommand::RestoreSystems(snapshot))
            .emit(MapEvent::MapUpdated { map_id: cmd.map_id }),
    )
}

/// Snapshot one connection so removing it can be undone.
pub(super) async fn capture_connection(
    tx: &mut Tx<'_>,
    map_id: i64,
    connection_id: i64,
) -> Result<Option<RestoreSystems>> {
    let connection = sqlx::query_as!(
        RestoredConnection,
        r#"select id, from_system, to_system, type as kind,
                  mass_status,
                  time_status,
                  size, preserve_mass
           from map_connections where id = $1 and map_id = $2"#,
        connection_id,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(connection.map(|c| RestoreSystems {
        map_id,
        systems: Vec::new(),
        connections: vec![c],
        signatures: Vec::new(),
    }))
}
