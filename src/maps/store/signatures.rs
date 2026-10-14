//! Writes to `signatures`: what was scanned in a placed system.
//!
//! Inserting or linking one fires the `map_*_sync` triggers, which reconcile it with its
//! connection (see [mapping.md](../../../docs/database/mapping.md)). Only updates are
//! stamped by trigger, so an insert that carries a lifetime stamps it itself.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::command::Tx;
use crate::maps::error::Result;
use crate::maps::{MassStatus, Signature, SignatureGroup, TimeStatus, WormholeSize};

/// A whole signature as it stands, keys included: what a removal or a paste snapshots and
/// an undo puts back under the same id. Serialized into the undo history, so the field
/// names are a stored format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRow {
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
    /// When it was first scanned and when its lifetime was last marked, put back as they
    /// were so the age and "EOL since" survive an undo. Absent from snapshots taken before
    /// they were kept, which come back as new.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub time_status_updated_at: Option<DateTime<Utc>>,
}

/// A signature that is not on the map yet.
#[derive(Debug, Clone, Copy)]
pub struct NewSignature<'a> {
    pub solar_system_id: i64,
    pub signature_id: &'a str,
    pub group: SignatureGroup,
    pub signature_type_id: Option<i64>,
    pub name: Option<&'a str>,
    pub size: Option<WormholeSize>,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    pub connection_id: Option<i64>,
}

/// Record a new signature and hand it back.
pub(crate) async fn insert(
    tx: &mut Tx<'_>,
    map_id: i64,
    row: &NewSignature<'_>,
) -> Result<Signature> {
    Ok(sqlx::query_as!(
        Signature,
        r#"insert into signatures
               (map_id, solar_system_id, signature_id, "group", signature_type_id, name,
                size, mass_status, time_status, connection_id, time_status_updated_at)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                   case when $9::time_status is not null then now() end)
           returning id, map_id, solar_system_id, signature_id, "group",
                     signature_type_id, name, size,
                     mass_status,
                     time_status,
                     time_status_updated_at, connection_id, created_at, updated_at"#,
        map_id,
        row.solar_system_id,
        row.signature_id,
        row.group,
        row.signature_type_id,
        row.name,
        row.size,
        row.mass_status,
        row.time_status,
        row.connection_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// Record a signature, or overwrite the one with the same scanner id in that system. An
/// import's way in. Returns its id.
pub(crate) async fn upsert_by_scanner_id(
    tx: &mut Tx<'_>,
    map_id: i64,
    row: &NewSignature<'_>,
) -> Result<i64> {
    Ok(sqlx::query_scalar!(
        r#"insert into signatures
               (map_id, solar_system_id, signature_id, "group", signature_type_id, name,
                size, mass_status, time_status, connection_id,
                time_status_updated_at)
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                   case when $9::time_status is not null then now() end)
           on conflict (map_id, solar_system_id, signature_id)
           do update set "group" = excluded."group",
               signature_type_id = excluded.signature_type_id, name = excluded.name,
               size = excluded.size, mass_status = excluded.mass_status,
               time_status = excluded.time_status, connection_id = excluded.connection_id,
               updated_at = now()
           returning id"#,
        map_id,
        row.solar_system_id,
        row.signature_id,
        row.group,
        row.signature_type_id,
        row.name,
        row.size,
        row.mass_status,
        row.time_status,
        row.connection_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// These signatures as they stand, for a snapshot.
pub(crate) async fn snapshot(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<Vec<SignatureRow>> {
    Ok(sqlx::query_as!(
        SignatureRow,
        r#"select id, solar_system_id, signature_id, "group",
                  signature_type_id, name, size,
                  mass_status,
                  time_status, connection_id, created_at as "created_at?",
                  time_status_updated_at
           from signatures where map_id = $1 and id = any($2)"#,
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Every signature scanned in these systems or linked to these connections, for a snapshot
/// of what goes with them.
pub(crate) async fn snapshot_around(
    tx: &mut Tx<'_>,
    map_id: i64,
    solar_system_ids: &[i64],
    connection_ids: &[i64],
) -> Result<Vec<SignatureRow>> {
    Ok(sqlx::query_as!(
        SignatureRow,
        r#"select id, solar_system_id, signature_id, "group",
                  signature_type_id, name, size,
                  mass_status,
                  time_status, connection_id, created_at as "created_at?",
                  time_status_updated_at
           from signatures
           where map_id = $1
             and (solar_system_id = any($2) or connection_id = any($3))"#,
        map_id,
        solar_system_ids,
        connection_ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}

/// Put a snapshotted signature back under its own id. A row that is still there is left
/// alone: this is the undo of something that took it away.
pub(crate) async fn reinsert(tx: &mut Tx<'_>, map_id: i64, row: &SignatureRow) -> Result<()> {
    sqlx::query!(
        r#"insert into signatures
               (id, map_id, solar_system_id, signature_id, "group", signature_type_id,
                name, size, mass_status, time_status, connection_id, created_at,
                time_status_updated_at)
           overriding system value
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, coalesce($12, now()), $13)
           on conflict (id) do nothing"#,
        row.id,
        map_id,
        row.solar_system_id,
        row.signature_id,
        row.group,
        row.signature_type_id,
        row.name.as_deref(),
        row.size,
        row.mass_status,
        row.time_status,
        row.connection_id,
        row.created_at,
        row.time_status_updated_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Put a snapshotted signature back under its own id, and if it is still there, put its
/// scanned state back over whatever it holds now. The undo of an edit as much as a removal.
pub(crate) async fn reinsert_overwriting(
    tx: &mut Tx<'_>,
    map_id: i64,
    row: &SignatureRow,
) -> Result<()> {
    sqlx::query!(
        r#"insert into signatures
               (id, map_id, solar_system_id, signature_id, "group", signature_type_id,
                name, size, mass_status, time_status, connection_id, created_at,
                time_status_updated_at)
           overriding system value
           values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, coalesce($12, now()), $13)
           on conflict (id) do update set
               "group" = excluded."group",
               signature_type_id = excluded.signature_type_id,
               name = excluded.name,
               size = excluded.size,
               mass_status = excluded.mass_status,
               time_status = excluded.time_status,
               connection_id = excluded.connection_id"#,
        row.id,
        map_id,
        row.solar_system_id,
        row.signature_id,
        row.group,
        row.signature_type_id,
        row.name.as_deref(),
        row.size,
        row.mass_status,
        row.time_status,
        row.connection_id,
        row.created_at,
        row.time_status_updated_at,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Link a signature to a connection, or with `None` let it go. Linking fires the sync.
/// Either way the signature has been touched, as it is when a link lets it go to make room
/// for another, so the edit stamp moves whether or not the sync changed its state.
pub(crate) async fn set_connection(
    tx: &mut Tx<'_>,
    map_id: i64,
    signature_pk: i64,
    connection_id: Option<i64>,
) -> Result<()> {
    sqlx::query!(
        "update signatures set connection_id = $1, updated_at = now()
         where id = $2 and map_id = $3",
        connection_id,
        signature_pk,
        map_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Where a signature that was just deleted had been, and what it was linked to.
#[derive(Debug, Clone, Copy)]
pub struct DeletedSignature {
    pub solar_system_id: i64,
    pub connection_id: Option<i64>,
}

/// Delete these signatures. A ghost one of them raised goes with it, by the foreign key.
/// Returns what the ones that were there had been.
pub(crate) async fn delete(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<Vec<DeletedSignature>> {
    Ok(sqlx::query_as!(
        DeletedSignature,
        "delete from signatures where map_id = $1 and id = any($2)
         returning solar_system_id, connection_id",
        map_id,
        ids,
    )
    .fetch_all(&mut **tx)
    .await?)
}
