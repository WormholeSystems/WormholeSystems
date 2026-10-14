//! Writes to `map_connection_jumps`: the jump log a connection's mass estimate is read from.
//!
//! A connection's jumps go with it by the foreign key, so whatever snapshots a connection
//! snapshots its jumps too, and puts them back after it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::command::Tx;
use crate::maps::error::Result;

/// A whole jump as it stands, keys included: what a removal snapshots and an undo puts back
/// under the same id. Serialized into the undo history, so the field names are a stored
/// format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpRow {
    pub id: i64,
    pub connection_id: Option<i64>,
    pub character_id: Option<i64>,
    pub from_solar_system_id: i64,
    pub to_solar_system_id: i64,
    pub ship_type_id: Option<i64>,
    pub ship_name: Option<String>,
    pub mass: i64,
    pub is_manual: bool,
    pub created_at: DateTime<Utc>,
}

/// These jumps as they stand, for a snapshot.
pub(crate) async fn snapshot(tx: &mut Tx<'_>, map_id: i64, ids: &[i64]) -> Result<Vec<JumpRow>> {
    Ok(sqlx::query_as!(
        JumpRow,
        "select id, connection_id, character_id, from_solar_system_id, to_solar_system_id,
                ship_type_id, ship_name, mass, is_manual, created_at
         from map_connection_jumps where map_id = $1 and id = any($2)
         order by id",
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Every jump through these connections, for a snapshot of what goes with them.
pub(crate) async fn snapshot_through(
    tx: &mut Tx<'_>,
    map_id: i64,
    connection_ids: &[i64],
) -> Result<Vec<JumpRow>> {
    Ok(sqlx::query_as!(
        JumpRow,
        "select id, connection_id, character_id, from_solar_system_id, to_solar_system_id,
                ship_type_id, ship_name, mass, is_manual, created_at
         from map_connection_jumps where map_id = $1 and connection_id = any($2)
         order by id",
        map_id,
        connection_ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Put a snapshotted jump back under its own id. A row that is somehow back already is left
/// alone, and a pilot whose character has gone since is forgotten, as the foreign key would
/// have done had the jump stayed.
pub(crate) async fn reinsert(tx: &mut Tx<'_>, map_id: i64, row: &JumpRow) -> Result<()> {
    sqlx::query!(
        "insert into map_connection_jumps
             (id, map_id, connection_id, character_id, from_solar_system_id,
              to_solar_system_id, ship_type_id, ship_name, mass, is_manual, created_at)
         overriding system value
         values ($1, $2, $3, (select id from characters where id = $4), $5, $6, $7, $8, $9,
                 $10, $11)
         on conflict (id) do nothing",
        row.id,
        map_id,
        row.connection_id,
        row.character_id,
        row.from_solar_system_id,
        row.to_solar_system_id,
        row.ship_type_id,
        row.ship_name.as_deref(),
        row.mass,
        row.is_manual,
        row.created_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
