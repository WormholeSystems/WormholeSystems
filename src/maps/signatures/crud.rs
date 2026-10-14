//! Adding, editing, removing and reading signatures one at a time or in bulk.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::maps::access::require_role;
use crate::maps::command::{CommandOutput, Effect, MapCommand, Tx, run};
use crate::maps::error::{MapError, Result};
use crate::maps::store;
use crate::maps::store::signatures::NewSignature;
use crate::maps::{Actor, MapEvent, MassStatus, Role, SignatureGroup, TimeStatus, WormholeSize};

use super::{
    Signature, capture_signatures, ensure_system_placed, fetch_signature_tx, is_wormhole,
    validate_signature_id, validate_type_for_group,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddSignature {
    pub map_id: i64,
    pub solar_system_id: i64,
    pub signature_id: String,
    pub group: SignatureGroup,
    #[serde(default)]
    #[ts(optional)]
    pub signature_type_id: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
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

impl AddSignature {
    pub fn validate(&self) -> Result<()> {
        validate_signature_id(self.signature_id.trim())?;
        // Wormhole state only belongs on a wormhole signature.
        if !is_wormhole(self.group)
            && (self.size.is_some() || self.mass_status.is_some() || self.time_status.is_some())
        {
            return Err(MapError::Validation(
                "only a wormhole signature can carry size / mass / time".into(),
            ));
        }
        Ok(())
    }
}

/// Record a scanned signature in a placed system. The system must be on this map. A
/// duplicate `signature_id` in the same system → `Conflict`. Linking to a connection is a
/// separate step ([`link_signature`]).
pub async fn add_signature(pool: &PgPool, actor: Actor, cmd: AddSignature) -> Result<Signature> {
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_add_signature(
    tx: &mut Tx<'_>,
    cmd: AddSignature,
) -> Result<Effect> {
    cmd.validate()?;
    ensure_system_placed(tx, cmd.map_id, cmd.solar_system_id).await?;
    if let Some(type_id) = cmd.signature_type_id {
        validate_type_for_group(tx, type_id, cmd.group).await?;
    }

    let exists = sqlx::query_scalar!(
        "select exists(
             select 1 from signatures
             where map_id = $1 and solar_system_id = $2 and signature_id = $3
         )",
        cmd.map_id,
        cmd.solar_system_id,
        cmd.signature_id.trim(),
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(false);
    if exists {
        return Err(MapError::Conflict("signature already scanned here".into()));
    }

    let sig = store::signatures::insert(
        tx,
        cmd.map_id,
        &NewSignature {
            solar_system_id: cmd.solar_system_id,
            signature_id: cmd.signature_id.trim(),
            group: cmd.group,
            signature_type_id: cmd.signature_type_id,
            name: cmd.name.as_deref(),
            size: cmd.size,
            mass_status: cmd.mass_status,
            time_status: cmd.time_status,
            connection_id: None,
        },
    )
    .await?;
    let inverse = MapCommand::RemoveSignature(RemoveSignature {
        map_id: cmd.map_id,
        signature_pk: sig.id,
    });
    let event = MapEvent::SignatureChanged {
        map_id: cmd.map_id,
        solar_system_id: sig.solar_system_id,
    };
    Ok(Effect::new(
        "signatures.added",
        format!("added signature {}", sig.signature_id),
        CommandOutput::Signature(Box::new(sig)),
    )
    .undo_with(inverse)
    .emit(event))
}

/// A partial edit of a signature. `None` leaves a field unchanged; `Some(None)` clears it.
/// Changing the group clears the catalog type, the connection link, and any wormhole state
/// (matching the legacy category select), unless a new type is supplied in the same call.
/// Editing a linked wormhole's state propagates to its connection via the DB trigger.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateSignature {
    pub map_id: i64,
    pub signature_pk: i64,
    /// `Some` renames the scanner id (7 chars; duplicate in the system → `Conflict`).
    #[serde(default)]
    #[ts(optional)]
    pub signature_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub group: Option<SignatureGroup>,
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub signature_type_id: Option<Option<i64>>,
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub name: Option<Option<String>>,
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub size: Option<Option<WormholeSize>>,
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub mass_status: Option<Option<MassStatus>>,
    #[serde(default, deserialize_with = "crate::maps::double_option")]
    #[ts(optional)]
    pub time_status: Option<Option<TimeStatus>>,
}

pub async fn update_signature(
    pool: &PgPool,
    actor: Actor,
    cmd: UpdateSignature,
) -> Result<Signature> {
    run(pool, actor, cmd).await
}

/// What an update writes: the current row with the command's partial edit merged in, per
/// the legacy rules documented on [`UpdateSignature`].
struct UpdatedFields {
    group: SignatureGroup,
    type_id: Option<i64>,
    name: Option<String>,
    size: Option<WormholeSize>,
    mass_status: Option<MassStatus>,
    time_status: Option<TimeStatus>,
    connection_id: Option<i64>,
}

fn merge_update(current: &Signature, cmd: &UpdateSignature) -> Result<UpdatedFields> {
    let group = cmd.group.unwrap_or(current.group);
    let group_changed = group != current.group;

    // A group change invalidates the old catalog type and the link; the same call may
    // re-supply a type, but never a stale one.
    let type_id = if group_changed {
        cmd.signature_type_id.flatten()
    } else {
        cmd.signature_type_id.unwrap_or(current.signature_type_id)
    };
    let name = cmd.name.clone().unwrap_or_else(|| current.name.clone());
    let (size, mass_status, time_status) = if group_changed && !is_wormhole(group) {
        (None, None, None)
    } else {
        (
            cmd.size.unwrap_or(current.size),
            cmd.mass_status.unwrap_or(current.mass_status),
            cmd.time_status.unwrap_or(current.time_status),
        )
    };
    let connection_id = if group_changed {
        None
    } else {
        current.connection_id
    };

    if !is_wormhole(group) && (size.is_some() || mass_status.is_some() || time_status.is_some()) {
        return Err(MapError::Validation(
            "only a wormhole signature can carry size / mass / time".into(),
        ));
    }
    Ok(UpdatedFields {
        group,
        type_id,
        name,
        size,
        mass_status,
        time_status,
        connection_id,
    })
}

pub(in crate::maps) async fn apply_update_signature(
    tx: &mut Tx<'_>,
    cmd: UpdateSignature,
) -> Result<Effect> {
    let current = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;

    let signature_id = match &cmd.signature_id {
        Some(v) => {
            let v = v.trim();
            validate_signature_id(v)?;
            if v != current.signature_id {
                let taken = sqlx::query_scalar!(
                    "select exists(
                         select 1 from signatures
                         where map_id = $1 and solar_system_id = $2 and signature_id = $3
                     )",
                    cmd.map_id,
                    current.solar_system_id,
                    v,
                )
                .fetch_one(&mut **tx)
                .await?
                .unwrap_or(false);
                if taken {
                    return Err(MapError::Conflict("signature already scanned here".into()));
                }
            }
            v.to_string()
        }
        None => current.signature_id.clone(),
    };

    let merged = merge_update(&current, &cmd)?;
    if let Some(type_id) = merged.type_id {
        validate_type_for_group(tx, type_id, merged.group).await?;
    }

    sqlx::query!(
        r#"update signatures
           set signature_id = $1, "group" = $2, signature_type_id = $3, name = $4, size = $5,
               mass_status = $6, time_status = $7, connection_id = $8, updated_at = now()
           where id = $9 and map_id = $10"#,
        signature_id,
        merged.group,
        merged.type_id,
        merged.name.as_deref(),
        merged.size,
        merged.mass_status,
        merged.time_status,
        merged.connection_id,
        cmd.signature_pk,
        cmd.map_id,
    )
    .execute(&mut **tx)
    .await?;
    // Re-read: a linked signature's size follows its identified type, which the sync
    // trigger may have just enforced over the edit.
    let sig = fetch_signature_tx(tx, cmd.map_id, cmd.signature_pk).await?;
    let inverse = MapCommand::UpdateSignature(UpdateSignature {
        map_id: cmd.map_id,
        signature_pk: cmd.signature_pk,
        signature_id: Some(current.signature_id.clone()),
        group: Some(current.group),
        signature_type_id: Some(current.signature_type_id),
        name: Some(current.name),
        size: Some(current.size),
        mass_status: Some(current.mass_status),
        time_status: Some(current.time_status),
    });
    let mut events = vec![MapEvent::SignatureChanged {
        map_id: cmd.map_id,
        solar_system_id: sig.solar_system_id,
    }];
    if let Some(connection_id) = sig.connection_id {
        events.push(MapEvent::ConnectionChanged {
            map_id: cmd.map_id,
            connection_id,
        });
    }
    Ok(Effect::new(
        "signatures.updated",
        format!("edited signature {}", sig.signature_id),
        CommandOutput::Signature(Box::new(sig)),
    )
    .undo_with(inverse)
    .emit_all(events))
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveSignature {
    pub map_id: i64,
    pub signature_pk: i64,
}

/// What a delete touched, so the caller can publish the right events.
#[derive(Debug, Clone)]
pub struct RemovedSignature {
    pub solar_system_id: i64,
    pub removed_connection_id: Option<i64>,
    /// Ghosts the removed connection stranded. A real system is left where it is.
    pub removed_placement_ids: Vec<i64>,
}

/// Delete a signature. Legacy cascade: if it was the last signature on its side of a
/// linked connection, the connection goes too (the other end's signature, if any, is
/// unlinked by the FK). Real endpoint systems are left in place; that cleanup belongs to
/// the bulk path ([`remove_signatures`]). A ghost is not one of those: it exists only as
/// the far side of the hole this signature described, so it goes with the connection.
pub async fn remove_signature(
    pool: &PgPool,
    actor: Actor,
    cmd: RemoveSignature,
) -> Result<RemovedSignature> {
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_remove_signature(
    tx: &mut Tx<'_>,
    cmd: RemoveSignature,
) -> Result<Effect> {
    let restore = capture_signatures(tx, cmd.map_id, &[cmd.signature_pk]).await?;
    // The node this scan raised goes with it, by the foreign key. Snapshotted first, so
    // one undo brings the scan and what it drew back together.
    let raised = crate::maps::ghost::raised_by(tx, cmd.map_id, &[cmd.signature_pk]).await?;
    let raised_snapshot = capture_raised(tx, cmd.map_id, &raised).await?;

    let sig = store::signatures::delete(tx, cmd.map_id, &[cmd.signature_pk])
        .await?
        .pop()
        .ok_or(MapError::NotFound)?;

    let mut removed_connection_id = None;
    if let Some(conn_id) = sig.connection_id
        && delete_connection_if_side_empty(tx, cmd.map_id, conn_id, sig.solar_system_id)
            .await?
            .is_some()
    {
        removed_connection_id = Some(conn_id);
    }

    let mut events = vec![MapEvent::SignatureChanged {
        map_id: cmd.map_id,
        solar_system_id: sig.solar_system_id,
    }];
    if let Some(connection_id) = removed_connection_id {
        events.push(MapEvent::ConnectionChanged {
            map_id: cmd.map_id,
            connection_id,
        });
    }
    events.extend(raised.iter().map(|id| MapEvent::SystemRemoved {
        map_id: cmd.map_id,
        map_solar_system_id: *id,
    }));
    Ok(Effect::new(
        "signatures.removed",
        "removed a signature",
        CommandOutput::Removal(Box::new(RemovedSignature {
            solar_system_id: sig.solar_system_id,
            removed_connection_id,
            removed_placement_ids: raised,
        })),
    )
    .undo_with(undo_with_raised(
        cmd.map_id,
        raised_snapshot,
        MapCommand::RestoreSignatures(restore),
    ))
    .emit_all(events))
}

async fn capture_raised(
    tx: &mut Tx<'_>,
    map_id: i64,
    raised: &[i64],
) -> Result<Option<crate::maps::restore::RestoreSystems>> {
    if raised.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        crate::maps::restore::capture_systems(tx, map_id, raised).await?,
    ))
}

/// Undo of a delete that took nodes with it. The nodes go back first: their snapshot
/// carries the edge, and the scan cannot name an edge that is not there yet. It carries
/// the scan too, so the second step is usually the one that finds nothing left to do.
fn undo_with_raised(
    map_id: i64,
    raised: Option<crate::maps::restore::RestoreSystems>,
    inverse: MapCommand,
) -> MapCommand {
    let Some(raised) = raised else { return inverse };
    MapCommand::Sequence(crate::maps::command::Sequence {
        map_id,
        steps: vec![MapCommand::RestoreSystems(raised), inverse],
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RemoveSignatures {
    pub map_id: i64,
    pub signature_pks: Vec<i64>,
}

/// What the bulk delete touched, so the caller can publish the right events.
#[derive(Debug, Clone)]
pub struct BulkRemoveOutcome {
    /// Solar systems that lost signatures.
    pub systems: Vec<i64>,
    pub removed_connection_ids: Vec<i64>,
    /// Placements (`map_solar_systems.id`) removed by the orphan cleanup.
    pub removed_placement_ids: Vec<i64>,
}

/// Bulk delete (the panel's "delete missing signatures" path), with the full legacy
/// cascade: each linked connection whose side is left without signatures is deleted, and
/// its far-end placement, if not pinned / home / rally and left with no connections, is
/// removed from the map (taking its own signatures with it). The system the signatures
/// were in always stays.
pub async fn remove_signatures(
    pool: &PgPool,
    actor: Actor,
    cmd: RemoveSignatures,
) -> Result<BulkRemoveOutcome> {
    run(pool, actor, cmd).await
}

pub(in crate::maps) async fn apply_remove_signatures(
    tx: &mut Tx<'_>,
    cmd: RemoveSignatures,
) -> Result<Effect> {
    let restore = capture_signatures(tx, cmd.map_id, &cmd.signature_pks).await?;
    let raised = crate::maps::ghost::raised_by(tx, cmd.map_id, &cmd.signature_pks).await?;
    let raised_snapshot = capture_raised(tx, cmd.map_id, &raised).await?;
    let removed = store::signatures::delete(tx, cmd.map_id, &cmd.signature_pks).await?;

    let mut systems: Vec<i64> = removed.iter().map(|r| r.solar_system_id).collect();
    systems.sort_unstable();
    systems.dedup();

    // Each (connection, side) pair at most once, then the legacy side-empty rule.
    let mut sides: Vec<(i64, i64)> = removed
        .iter()
        .filter_map(|r| r.connection_id.map(|c| (c, r.solar_system_id)))
        .collect();
    sides.sort_unstable();
    sides.dedup();

    let mut removed_connection_ids = Vec::new();
    let mut endpoint_candidates: Vec<i64> = Vec::new();
    for (conn_id, side_system) in sides {
        if let Some(far_side) =
            delete_connection_if_side_empty(tx, cmd.map_id, conn_id, side_system).await?
        {
            removed_connection_ids.push(conn_id);
            endpoint_candidates.extend(far_side);
        }
    }
    endpoint_candidates.sort_unstable();
    endpoint_candidates.dedup();

    // Orphan cleanup: the far ends of the deleted connections that are unpinned, unmarked,
    // and now connection-less disappear from the map. Never the system the signatures were
    // in: that is where the user is working, and losing its last hole doesn't make it stale.
    let mut removed_placement_ids = raised;
    for placement_id in endpoint_candidates {
        if store::placements::delete_if_orphan(tx, cmd.map_id, placement_id).await? {
            removed_placement_ids.push(placement_id);
        }
    }

    let count = cmd.signature_pks.len() as i64;
    let mut events: Vec<MapEvent> = systems
        .iter()
        .map(|solar_system_id| MapEvent::SignatureChanged {
            map_id: cmd.map_id,
            solar_system_id: *solar_system_id,
        })
        .collect();
    events.extend(
        removed_connection_ids
            .iter()
            .map(|connection_id| MapEvent::ConnectionChanged {
                map_id: cmd.map_id,
                connection_id: *connection_id,
            }),
    );
    events.extend(
        removed_placement_ids
            .iter()
            .map(|id| MapEvent::SystemRemoved {
                map_id: cmd.map_id,
                map_solar_system_id: *id,
            }),
    );
    Ok(Effect::new(
        "signatures.removed",
        format!("removed {count} signatures"),
        CommandOutput::BulkRemoval(Box::new(BulkRemoveOutcome {
            systems,
            removed_connection_ids,
            removed_placement_ids,
        })),
    )
    .entries(count)
    .undo_with(undo_with_raised(
        cmd.map_id,
        raised_snapshot,
        MapCommand::RestoreSignatures(restore),
    ))
    .emit_all(events))
}

/// Delete `conn_id` if no signature in `side_system` still references it (the legacy
/// same-side survivor rule). Returns `None` when a survivor kept it alive, otherwise the
/// placement at the far end from `side_system`.
async fn delete_connection_if_side_empty(
    tx: &mut Tx<'_>,
    map_id: i64,
    conn_id: i64,
    side_system: i64,
) -> Result<Option<Option<i64>>> {
    let survivors = sqlx::query_scalar!(
        "select count(*) from signatures
         where connection_id = $1 and solar_system_id = $2",
        conn_id,
        side_system,
    )
    .fetch_one(&mut **tx)
    .await?
    .unwrap_or(0);
    if survivors > 0 {
        return Ok(None);
    }
    let Some(endpoints) = store::connections::delete(tx, map_id, &[conn_id])
        .await?
        .pop()
    else {
        return Ok(None);
    };
    // A ghost has no solar system, so it is always the far end.
    let far_side = sqlx::query_scalar!(
        "select id from map_solar_systems
         where map_id = $1 and id in ($2, $3) and solar_system_id is distinct from $4",
        map_id,
        endpoints.from_system,
        endpoints.to_system,
        side_system,
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(Some(far_side))
}

/// Every signature on a map. Viewer+.
pub async fn list_signatures(pool: &PgPool, actor: Actor, map_id: i64) -> Result<Vec<Signature>> {
    require_role(pool, map_id, actor.user_id, Role::Viewer).await?;
    read_signatures(pool, map_id).await
}

/// The scan, for a caller whose right to read the map has already been settled.
pub async fn read_signatures(pool: &PgPool, map_id: i64) -> Result<Vec<Signature>> {
    let sigs = sqlx::query_as!(
        Signature,
        r#"select id, map_id, solar_system_id, signature_id, "group",
                  signature_type_id, name, size,
                  mass_status,
                  time_status,
                  time_status_updated_at, connection_id, created_at, updated_at
           from signatures where map_id = $1 order by solar_system_id, signature_id"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(sigs)
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    fn wormhole_sig() -> Signature {
        Signature {
            id: 1,
            map_id: 1,
            solar_system_id: 31000001,
            signature_id: "ABC-123".into(),
            group: SignatureGroup::Wormhole,
            signature_type_id: Some(100),
            name: Some("K162".into()),
            size: Some(WormholeSize::Large),
            mass_status: Some(MassStatus::Reduced),
            time_status: Some(TimeStatus::Eol),
            time_status_updated_at: None,
            connection_id: Some(7),
            created_at: chrono::DateTime::UNIX_EPOCH,
            updated_at: chrono::DateTime::UNIX_EPOCH,
        }
    }

    fn edit() -> UpdateSignature {
        UpdateSignature {
            map_id: 1,
            signature_pk: 1,
            ..Default::default()
        }
    }

    #[test]
    fn an_empty_edit_changes_nothing() {
        let sig = wormhole_sig();
        let merged = merge_update(&sig, &edit()).unwrap();
        assert_eq!(merged.group, sig.group);
        assert_eq!(merged.type_id, sig.signature_type_id);
        assert_eq!(merged.name, sig.name);
        assert_eq!(merged.size, sig.size);
        assert_eq!(merged.mass_status, sig.mass_status);
        assert_eq!(merged.time_status, sig.time_status);
        assert_eq!(merged.connection_id, sig.connection_id);
    }

    #[test]
    fn leaving_the_wormhole_group_clears_type_link_and_state() {
        let cmd = UpdateSignature {
            group: Some(SignatureGroup::Data),
            ..edit()
        };
        let merged = merge_update(&wormhole_sig(), &cmd).unwrap();
        assert_eq!(merged.group, SignatureGroup::Data);
        assert_eq!(merged.type_id, None);
        assert_eq!(merged.connection_id, None);
        assert_eq!(
            (merged.size, merged.mass_status, merged.time_status),
            (None, None, None)
        );
        // The raw name survives a recategorize; only the catalog fields reset.
        assert_eq!(merged.name, Some("K162".into()));
    }

    #[test]
    fn a_group_change_may_bring_its_own_type_but_never_keeps_the_stale_one() {
        let cmd = UpdateSignature {
            group: Some(SignatureGroup::Data),
            signature_type_id: Some(Some(200)),
            ..edit()
        };
        assert_eq!(
            merge_update(&wormhole_sig(), &cmd).unwrap().type_id,
            Some(200)
        );

        let without_type = UpdateSignature {
            group: Some(SignatureGroup::Data),
            signature_type_id: None,
            ..edit()
        };
        assert_eq!(
            merge_update(&wormhole_sig(), &without_type)
                .unwrap()
                .type_id,
            None
        );
    }

    #[test]
    fn becoming_a_wormhole_still_drops_the_link_but_keeps_supplied_state() {
        let mut site = wormhole_sig();
        site.group = SignatureGroup::Data;
        site.size = None;
        site.mass_status = None;
        site.time_status = None;
        site.connection_id = None;
        let cmd = UpdateSignature {
            group: Some(SignatureGroup::Wormhole),
            size: Some(Some(WormholeSize::Small)),
            ..edit()
        };
        let merged = merge_update(&site, &cmd).unwrap();
        assert_eq!(merged.size, Some(WormholeSize::Small));
        assert_eq!(merged.connection_id, None);
    }

    #[test]
    fn a_present_null_clears_a_field_where_absent_keeps_it() {
        let cmd = UpdateSignature {
            name: Some(None),
            ..edit()
        };
        assert_eq!(merge_update(&wormhole_sig(), &cmd).unwrap().name, None);
    }

    #[test]
    fn wormhole_state_on_a_site_is_refused() {
        let mut site = wormhole_sig();
        site.group = SignatureGroup::Data;
        site.connection_id = None;
        let cmd = UpdateSignature {
            size: Some(Some(WormholeSize::Small)),
            ..edit()
        };
        assert!(matches!(
            merge_update(&site, &cmd),
            Err(MapError::Validation(_))
        ));
    }
}
