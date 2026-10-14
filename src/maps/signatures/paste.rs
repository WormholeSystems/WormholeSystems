//! A scanner paste: every signature in a system at once, upserted.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::maps::command::{CommandOutput, Effect, MapCommand, Tx, run};
use crate::maps::error::Result;
use crate::maps::store;
use crate::maps::store::signatures::NewSignature;
use crate::maps::{Actor, SignatureGroup};
use crate::realtime::MapEvent;

use super::{
    capture_signatures, category_id_for, ensure_system_placed, is_wormhole, validate_signature_id,
};

/// One row of a parsed scanner paste. `group` is `None` when the scanner line carried no
/// classification (legacy: keep whatever the row already has). The catalog type is only
/// ever client-matched for site categories; wormhole types never come from a paste.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct PastedSignature {
    pub signature_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub group: Option<SignatureGroup>,
    #[serde(default)]
    #[ts(optional)]
    pub signature_type_id: Option<i64>,
    #[serde(default)]
    #[ts(optional)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct PasteSignatures {
    pub map_id: i64,
    pub solar_system_id: i64,
    pub signatures: Vec<PastedSignature>,
}

/// Upsert a pasted in-game scan into a system's signatures. Legacy semantics, so this never
/// deletes anything (the panel diffs client-side and calls [`remove_signatures`]):
///
/// - group: pasted value, else keep
/// - catalog type: an existing wormhole type always survives; a site row takes the
///   pasted match, or is cleared when the paste carries only an unmatched raw name
/// - connection link: kept unless the paste recategorizes the row to a non-wormhole group
/// - raw name: refreshed for sites, preserved for wormholes
/// - `size` / `mass_status` / `time_status` / `created_at`: never touched
pub async fn paste_signatures(pool: &PgPool, actor: Actor, cmd: PasteSignatures) -> Result<()> {
    run(pool, actor, cmd).await
}

/// What one pasted row writes over an existing signature: the legacy merge rules listed on
/// [`paste_signatures`]. `pasted_type` is the pasted catalog type, already checked against
/// the pasted group's category; `kept_type_fits` says whether the existing catalog type
/// still belongs to the merged group.
struct PastedFields {
    group: SignatureGroup,
    type_id: Option<i64>,
    name: Option<String>,
    clear_link: bool,
}

fn merge_paste(
    existing_group: SignatureGroup,
    existing_type: Option<i64>,
    existing_name: Option<String>,
    pasted: &PastedSignature,
    pasted_type: Option<i64>,
    kept_type_fits: bool,
) -> PastedFields {
    let group = pasted.group.unwrap_or(existing_group);
    let type_id = if is_wormhole(group) {
        existing_type.or(pasted_type)
    } else if pasted_type.is_some() {
        pasted_type
    } else if pasted.name.is_some() {
        // A site with only an unmatched raw name clears a stale type.
        None
    } else {
        existing_type.filter(|_| kept_type_fits)
    };
    let name = if is_wormhole(group) {
        existing_name.or_else(|| pasted.name.clone())
    } else {
        pasted.name.clone().or(existing_name)
    };
    // Recategorizing a hole into a site drops the link (legacy rule); the connection
    // itself stays on the map.
    let clear_link = pasted.group.is_some_and(|g| !is_wormhole(g));
    PastedFields {
        group,
        type_id,
        name,
        clear_link,
    }
}

pub(in crate::maps) async fn apply_paste_signatures(
    tx: &mut Tx<'_>,
    cmd: PasteSignatures,
) -> Result<Effect> {
    ensure_system_placed(tx, cmd.map_id, cmd.solar_system_id).await?;
    // Snapshot the whole system: a paste both edits existing rows and adds new ones.
    let existing_ids: Vec<i64> = sqlx::query_scalar!(
        "select id from signatures where map_id = $1 and solar_system_id = $2",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .fetch_all(&mut **tx)
    .await?;
    let mut restore = capture_signatures(tx, cmd.map_id, &existing_ids).await?;
    restore.replaces_system = Some(cmd.solar_system_id);

    // Only a paste moves this: it says when the list last matched a scan, which a hand
    // edit does not. Undo puts the previous time back with the previous list.
    let placement = sqlx::query!(
        "select id, signatures_pasted_at from map_solar_systems
         where map_id = $1 and solar_system_id = $2
         for update",
        cmd.map_id,
        cmd.solar_system_id,
    )
    .fetch_one(&mut **tx)
    .await?;
    restore.pasted_at = placement.signatures_pasted_at;
    sqlx::query!(
        "update map_solar_systems set signatures_pasted_at = now() where id = $1",
        placement.id,
    )
    .execute(&mut **tx)
    .await?;

    for s in &cmd.signatures {
        let sid = s.signature_id.trim();
        if sid.is_empty() {
            continue;
        }
        validate_signature_id(sid)?;

        // A pasted type only counts if it belongs to the pasted group's category.
        let mut pasted_type = None;
        if let (Some(type_id), Some(group)) = (s.signature_type_id, s.group)
            && type_category(tx, type_id).await? == category_id_for(group)
        {
            pasted_type = Some(type_id);
        }

        let existing = sqlx::query!(
            r#"select id, "group", signature_type_id, name
               from signatures
               where map_id = $1 and solar_system_id = $2 and signature_id = $3
               for update"#,
            cmd.map_id,
            cmd.solar_system_id,
            sid,
        )
        .fetch_optional(&mut **tx)
        .await?;

        let Some(existing) = existing else {
            // A paste only names what the scanner shows; the hole's state is for somebody
            // to fill in.
            store::signatures::insert(
                tx,
                cmd.map_id,
                &NewSignature {
                    solar_system_id: cmd.solar_system_id,
                    signature_id: sid,
                    group: s.group.unwrap_or_default(),
                    signature_type_id: pasted_type,
                    name: s.name.as_deref(),
                    size: None,
                    mass_status: None,
                    time_status: None,
                    connection_id: None,
                },
            )
            .await?;
            continue;
        };

        let kept_type_fits = match existing.signature_type_id {
            Some(kept) => {
                let group = s.group.unwrap_or(existing.group);
                type_category(tx, kept).await? == category_id_for(group)
            }
            None => false,
        };
        let merged = merge_paste(
            existing.group,
            existing.signature_type_id,
            existing.name,
            s,
            pasted_type,
            kept_type_fits,
        );

        sqlx::query!(
            r#"update signatures
               set "group" = $1, signature_type_id = $2, name = $3,
                   connection_id = case when $4 then null else connection_id end,
                   updated_at = now()
               where id = $5"#,
            merged.group,
            merged.type_id,
            merged.name.as_deref(),
            merged.clear_link,
            existing.id,
        )
        .execute(&mut **tx)
        .await?;
    }

    let count = cmd.signatures.len() as i64;
    Ok(Effect::new(
        "signatures.pasted",
        format!("pasted {count} signatures"),
        CommandOutput::None,
    )
    .entries(count)
    .undo_with(MapCommand::RestoreSignatures(restore))
    .emit(MapEvent::SignatureChanged {
        map_id: cmd.map_id,
        solar_system_id: cmd.solar_system_id,
    })
    .emit(MapEvent::SystemDetailsChanged {
        map_id: cmd.map_id,
        map_solar_system_id: placement.id,
    }))
}

/// A catalog type's category id, or `None` for an unknown type id.
async fn type_category(tx: &mut Tx<'_>, type_id: i64) -> Result<Option<i64>> {
    Ok(sqlx::query_scalar!(
        "select signature_category_id from signature_types where id = $1",
        type_id,
    )
    .fetch_optional(&mut **tx)
    .await?)
}

#[cfg(test)]
mod merge_tests {
    use super::*;

    fn pasted(group: Option<SignatureGroup>, name: Option<&str>) -> PastedSignature {
        PastedSignature {
            signature_id: "ABC-123".into(),
            group,
            signature_type_id: None,
            name: name.map(Into::into),
        }
    }

    #[test]
    fn a_wormholes_existing_type_and_name_survive_a_repaste() {
        let merged = merge_paste(
            SignatureGroup::Wormhole,
            Some(100),
            Some("K162".into()),
            &pasted(Some(SignatureGroup::Wormhole), Some("Fresh name")),
            Some(200),
            true,
        );
        assert_eq!(merged.type_id, Some(100));
        assert_eq!(merged.name, Some("K162".into()));
        assert!(!merged.clear_link);
    }

    #[test]
    fn a_scanned_down_wormhole_takes_the_pasted_type_when_it_had_none() {
        let merged = merge_paste(
            SignatureGroup::Wormhole,
            None,
            None,
            &pasted(Some(SignatureGroup::Wormhole), None),
            Some(200),
            false,
        );
        assert_eq!(merged.type_id, Some(200));
    }

    #[test]
    fn a_site_takes_the_pasted_match_and_refreshes_its_name() {
        let merged = merge_paste(
            SignatureGroup::Data,
            Some(100),
            Some("Old site".into()),
            &pasted(Some(SignatureGroup::Data), Some("New site")),
            Some(200),
            true,
        );
        assert_eq!(merged.type_id, Some(200));
        assert_eq!(merged.name, Some("New site".into()));
    }

    #[test]
    fn a_site_with_only_a_raw_name_clears_a_stale_type() {
        let merged = merge_paste(
            SignatureGroup::Data,
            Some(100),
            None,
            &pasted(Some(SignatureGroup::Data), Some("Unmatched name")),
            None,
            true,
        );
        assert_eq!(merged.type_id, None);
        assert_eq!(merged.name, Some("Unmatched name".into()));
    }

    #[test]
    fn a_bare_repaste_keeps_the_old_type_only_while_it_fits_the_group() {
        let bare = pasted(Some(SignatureGroup::Relic), None);
        let kept = merge_paste(SignatureGroup::Relic, Some(100), None, &bare, None, true);
        assert_eq!(kept.type_id, Some(100));
        let recategorized = merge_paste(SignatureGroup::Data, Some(100), None, &bare, None, false);
        assert_eq!(recategorized.type_id, None);
        assert_eq!(recategorized.group, SignatureGroup::Relic);
    }

    #[test]
    fn recategorizing_a_hole_into_a_site_drops_the_link() {
        let merged = merge_paste(
            SignatureGroup::Wormhole,
            Some(100),
            None,
            &pasted(Some(SignatureGroup::Gas), None),
            None,
            false,
        );
        assert!(merged.clear_link);
        // A paste that names no group at all leaves the link alone.
        let unnamed = merge_paste(
            SignatureGroup::Wormhole,
            Some(100),
            None,
            &pasted(None, None),
            None,
            true,
        );
        assert!(!unnamed.clear_link);
    }
}
