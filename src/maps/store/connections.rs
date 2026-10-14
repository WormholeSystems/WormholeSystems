//! Writes to `map_connections`: the edges between placements.
//!
//! Nothing here touches the linked signatures by hand. The `map_*_sync` triggers keep the
//! two in lock-step (see [mapping.md](../../../docs/database/mapping.md)).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::command::Tx;
use crate::maps::error::Result;
use crate::maps::{ConnectionType, MassStatus, TimeStatus, WormholeSize};

/// A whole connection as it stands, keys included: what a removal snapshots and an undo
/// puts back under the same id. Serialized into the undo history, so the field names are a
/// stored format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionRow {
    pub id: i64,
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionType,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub size: Option<WormholeSize>,
    pub preserve_mass: bool,
}

/// A connection that is not on the map yet.
#[derive(Debug, Clone, Copy)]
pub struct NewConnection {
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionType,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub size: Option<WormholeSize>,
    pub preserve_mass: bool,
    /// Only an import knows when the lifetime was last marked; anything else starts unset.
    pub time_status_updated_at: Option<DateTime<Utc>>,
    /// `None` is now. An import keeps the age the hole had where it came from.
    pub created_at: Option<DateTime<Utc>>,
}

impl NewConnection {
    /// A freshly drawn edge: nothing known about it yet but, perhaps, its size.
    pub fn drawn(
        from_system: i64,
        to_system: i64,
        kind: ConnectionType,
        size: Option<WormholeSize>,
    ) -> Self {
        NewConnection {
            from_system,
            to_system,
            kind,
            mass_status: None,
            time_status: None,
            size,
            preserve_mass: false,
            time_status_updated_at: None,
            created_at: None,
        }
    }
}

/// Add a connection. Returns its id.
pub(crate) async fn insert(tx: &mut Tx<'_>, map_id: i64, row: &NewConnection) -> Result<i64> {
    Ok(sqlx::query_scalar!(
        "insert into map_connections
             (map_id, from_system, to_system, type, mass_status, time_status, size,
              time_status_updated_at, preserve_mass, created_at)
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9, coalesce($10, now()))
         returning id",
        map_id,
        row.from_system,
        row.to_system,
        row.kind,
        row.mass_status,
        row.time_status,
        row.size,
        row.time_status_updated_at,
        row.preserve_mass,
        row.created_at,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// One connection as it stands, for a snapshot.
pub(crate) async fn snapshot(
    tx: &mut Tx<'_>,
    map_id: i64,
    connection_id: i64,
) -> Result<Option<ConnectionRow>> {
    Ok(sqlx::query_as!(
        ConnectionRow,
        r#"select id, from_system, to_system, type as kind,
                  mass_status,
                  time_status,
                  size, preserve_mass
           from map_connections where id = $1 and map_id = $2"#,
        connection_id,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?)
}

/// Every connection with an end at one of these placements, for a snapshot of what goes
/// with them.
pub(crate) async fn snapshot_touching(
    tx: &mut Tx<'_>,
    map_id: i64,
    placement_ids: &[i64],
) -> Result<Vec<ConnectionRow>> {
    Ok(sqlx::query_as!(
        ConnectionRow,
        r#"select id, from_system, to_system, type as kind,
                  mass_status,
                  time_status,
                  size, preserve_mass
           from map_connections
           where map_id = $1 and (from_system = any($2) or to_system = any($2))"#,
        map_id,
        placement_ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Put a snapshotted connection back under its own id, so the signatures restored with it
/// still line up. A row that is somehow back already is left alone.
pub(crate) async fn reinsert(tx: &mut Tx<'_>, map_id: i64, row: &ConnectionRow) -> Result<()> {
    sqlx::query!(
        "insert into map_connections
             (id, map_id, from_system, to_system, type, mass_status, time_status, size,
              preserve_mass)
         overriding system value
         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         on conflict (id) do nothing",
        row.id,
        map_id,
        row.from_system,
        row.to_system,
        row.kind,
        row.mass_status,
        row.time_status,
        row.size,
        row.preserve_mass,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// The ends of a connection that was just deleted.
#[derive(Debug, Clone, Copy)]
pub struct DeletedConnection {
    pub from_system: i64,
    pub to_system: i64,
}

/// Delete these connections. Their signatures survive unlinked, by the foreign key. Returns
/// the ends of the ones that were there.
pub(crate) async fn delete(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<Vec<DeletedConnection>> {
    Ok(sqlx::query_as!(
        DeletedConnection,
        "delete from map_connections where map_id = $1 and id = any($2)
         returning from_system, to_system",
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}
