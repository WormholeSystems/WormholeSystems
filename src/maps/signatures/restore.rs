//! Snapshots of signatures, and putting them back: the undo of a removal or a paste.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::MapEvent;
use crate::maps::command::{CommandOutput, Effect, MapCommand, Tx};
use crate::maps::error::Result;
use crate::maps::store;
use crate::maps::store::signatures::SignatureRow;

use super::RemoveSignatures;

/// A snapshot of signatures, the inverse of any signature removal or paste.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSignatures {
    pub map_id: i64,
    pub signatures: Vec<SignatureRow>,
    /// When set, the system is reset to exactly this snapshot (used by paste undo, which
    /// must also drop rows the paste created).
    pub replaces_system: Option<i64>,
    /// The system's paste time before the paste, put back with `replaces_system`.
    #[serde(default)]
    pub pasted_at: Option<DateTime<Utc>>,
}

/// Snapshot signature rows so a removal or paste can be undone.
pub(in crate::maps) async fn capture_signatures(
    tx: &mut Tx<'_>,
    map_id: i64,
    ids: &[i64],
) -> Result<RestoreSignatures> {
    let signatures = store::signatures::snapshot(tx, map_id, ids).await?;
    Ok(RestoreSignatures {
        map_id,
        signatures,
        replaces_system: None,
        pasted_at: None,
    })
}

pub(in crate::maps) async fn apply_restore_signatures(
    tx: &mut Tx<'_>,
    cmd: RestoreSignatures,
) -> Result<Effect> {
    // Paste undo: clear whatever the paste left behind before replaying the snapshot.
    let mut events = Vec::new();
    if let Some(solar_system_id) = cmd.replaces_system {
        sqlx::query!(
            "delete from signatures where map_id = $1 and solar_system_id = $2",
            cmd.map_id,
            solar_system_id,
        )
        .execute(&mut **tx)
        .await?;
        let placement = sqlx::query_scalar!(
            "update map_solar_systems set signatures_pasted_at = $1
             where map_id = $2 and solar_system_id = $3
             returning id",
            cmd.pasted_at,
            cmd.map_id,
            solar_system_id,
        )
        .fetch_optional(&mut **tx)
        .await?;
        events.extend(
            placement.map(|map_solar_system_id| MapEvent::SystemDetailsChanged {
                map_id: cmd.map_id,
                map_solar_system_id,
            }),
        );
    }
    for s in &cmd.signatures {
        store::signatures::reinsert_overwriting(tx, cmd.map_id, s).await?;
    }
    let count = cmd.signatures.len() as i64;
    let ids: Vec<i64> = cmd.signatures.iter().map(|s| s.id).collect();
    let mut systems: Vec<i64> = cmd
        .signatures
        .iter()
        .map(|s| s.solar_system_id)
        .chain(cmd.replaces_system)
        .collect();
    systems.sort_unstable();
    systems.dedup();
    events.extend(
        systems
            .into_iter()
            .map(|solar_system_id| MapEvent::SignatureChanged {
                map_id: cmd.map_id,
                solar_system_id,
            }),
    );
    Ok(Effect::new(
        "signatures.restored",
        format!("restored {count} signatures"),
        CommandOutput::None,
    )
    .entries(count)
    .undo_with(MapCommand::RemoveSignatures(RemoveSignatures {
        map_id: cmd.map_id,
        signature_pks: ids,
    }))
    .emit_all(events))
}
