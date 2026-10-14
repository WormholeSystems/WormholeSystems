//! Cosmic signatures scanned in a system, and linking a wormhole signature to a
//! connection. All mutations are Member+; reads are Viewer+ (see
//! [access.md](../../docs/database/access.md)).
//!
//! A signature may reference a catalog type (`signature_types`); `name` holds the raw
//! scanner type name when nothing matched. Only `wormhole`-group signatures carry
//! `mass_status` / `time_status` / `size` or a connection. Linking sets `connection_id`,
//! which fires the `map_*_sync` triggers (migration 0009): the connection and its
//! signatures reconcile to the worst state per field, then stay in lock-step.
//!
//! Paste ([`paste_signatures`]) is upsert-only: it never deletes. Rows that vanished from a
//! scan go through [`remove_signatures`], which cascades: a connection dies with its last
//! same-side signature, and endpoints left unpinned, unmarked and connection-less leave the
//! map.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{MassStatus, SignatureGroup, TimeStatus, WormholeSize};

use super::command::Tx;
use super::error::{MapError, Result};

mod crud;
mod expiry;
mod link;
mod paste;
mod restore;

pub use crud::*;
pub use expiry::*;
pub use link::*;
pub use paste::*;
pub use restore::*;

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Signature {
    pub id: i64,
    pub map_id: i64,
    pub solar_system_id: i64,
    /// The in-game scanner id, e.g. `ABC-123`. Unique per `(map, system)`.
    pub signature_id: String,
    pub group: SignatureGroup,
    /// The matched catalog type ([`signature_types`](../../docs/database/static.md)).
    pub signature_type_id: Option<i64>,
    /// The raw scanner type name when no catalog type matched.
    pub name: Option<String>,
    pub size: Option<WormholeSize>,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    /// When `time_status` last changed (DB trigger), for "EOL since" displays.
    pub time_status_updated_at: Option<DateTime<Utc>>,
    /// The connection this signature is one end of, if linked. Only `wormhole` sigs.
    pub connection_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Only this group may carry life-cycle state (`size`/`mass`/`time`) and a connection link.
fn is_wormhole(group: SignatureGroup) -> bool {
    group == SignatureGroup::Wormhole
}

/// The `signature_categories` id a group maps to; `Unknown` has no catalog category.
pub(super) fn category_id_for(group: SignatureGroup) -> Option<i64> {
    match group {
        SignatureGroup::Wormhole => Some(1),
        SignatureGroup::Data => Some(2),
        SignatureGroup::Relic => Some(3),
        SignatureGroup::Combat => Some(4),
        SignatureGroup::Gas => Some(5),
        SignatureGroup::Ore => Some(6),
        SignatureGroup::Homefront => Some(7),
        SignatureGroup::FactionWarfare => Some(8),
        SignatureGroup::Unknown => None,
    }
}

/// Scanner ids are exactly 7 chars (`ABC-123`).
fn validate_signature_id(id: &str) -> Result<()> {
    if id.len() != 7 {
        return Err(MapError::Validation(
            "signature id must be 7 characters (ABC-123)".into(),
        ));
    }
    Ok(())
}

/// `Validation` unless the catalog type exists and belongs to the group's category.
async fn validate_type_for_group(
    tx: &mut Tx<'_>,
    type_id: i64,
    group: SignatureGroup,
) -> Result<()> {
    let category = sqlx::query_scalar!(
        "select signature_category_id from signature_types where id = $1",
        type_id,
    )
    .fetch_optional(&mut **tx)
    .await?;
    if category.is_none() || category != category_id_for(group) {
        return Err(MapError::Validation(
            "signature type does not belong to the signature's category".into(),
        ));
    }
    Ok(())
}

async fn fetch_signature_tx(tx: &mut Tx<'_>, map_id: i64, signature_pk: i64) -> Result<Signature> {
    sqlx::query_as!(
        Signature,
        r#"select id, map_id, solar_system_id, signature_id, "group",
                  signature_type_id, name, size,
                  mass_status,
                  time_status,
                  time_status_updated_at, connection_id, created_at, updated_at
           from signatures where id = $1 and map_id = $2"#,
        signature_pk,
        map_id,
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(MapError::NotFound)
}

/// `Validation` if the solar system isn't currently placed on the map (signatures hang off
/// a placement via the `(map_id, solar_system_id)` foreign key).
async fn ensure_system_placed(tx: &mut Tx<'_>, map_id: i64, solar_system_id: i64) -> Result<()> {
    let placed = sqlx::query_scalar!(
        "select exists(select 1 from map_solar_systems where map_id = $1 and solar_system_id = $2)",
        map_id,
        solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if !placed {
        return Err(MapError::Validation(
            "the signature's system is not placed on this map".into(),
        ));
    }
    Ok(())
}
