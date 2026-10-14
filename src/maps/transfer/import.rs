//! Reading a parsed file into a map, or into a new one.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::allowlist::AffiliationAllowlist;
use crate::maps::access::owns_character;
use crate::maps::command::{CommandOutput, Effect, EventActor, MapTx, Tx};
use crate::maps::error::{MapError, Result};
use crate::maps::map::Map;
use crate::maps::solar_system::MAX_RALLY_POINTS;
use crate::maps::store;
use crate::maps::{
    Actor, AliasScheme, ConnectionType, MapLayout, Role, SignatureGroup, SubjectType, SystemStatus,
    signatures,
};

use super::format::{
    AccessRow, ConnectionRow, ParsedExport, RoutesSection, Sections, SettingsSection, SignatureRow,
    SolarsystemRow, WireLifetime, WireMassStatus, WireShipSize, group_for_category_code,
};

/// What happened to one section of an import.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SectionCounts {
    pub created: i64,
    pub updated: i64,
    pub skipped: i64,
}

impl SectionCounts {
    fn changed(&self) -> i64 {
        self.created + self.updated
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ImportSummary {
    pub settings: SectionCounts,
    pub access: SectionCounts,
    pub systems: SectionCounts,
    pub connections: SectionCounts,
    pub signatures: SectionCounts,
    pub routes: SectionCounts,
}

impl ImportSummary {
    fn changed(&self) -> i64 {
        self.settings.changed()
            + self.access.changed()
            + self.systems.changed()
            + self.connections.changed()
            + self.signatures.changed()
            + self.routes.changed()
    }
}

/// Merge a parsed export into the map. Manager+. Rows with a natural key (system id,
/// signature id, access subject, watchlist entry) are updated in place; connections have
/// none, so an equivalent edge in either direction means the file's is skipped. The owner
/// grant is never touched. One transaction, recorded in the journal as a single
/// non-undoable entry.
pub async fn import_map(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    parsed: &ParsedExport,
) -> Result<ImportSummary> {
    let mut tx = MapTx::begin(pool, map_id, EventActor::Character(actor), Role::Manager).await?;
    let summary = import_into_tx(&mut tx, map_id, parsed).await?;
    finish_import(&mut tx, map_id, parsed, &summary).await?;
    tx.commit().await?;
    Ok(summary)
}

/// Create a fresh map owned by the acting character and load the payload into it. When the
/// file carries a routes section, the seeded trade-hub watchlist makes way for the file's
/// list. An explicit `name` wins over the file's. Creating a map this way is limited to
/// `creators` just like creating an empty one.
pub async fn import_map_as_new(
    pool: &PgPool,
    actor: Actor,
    creators: &AffiliationAllowlist,
    parsed: ParsedExport,
    name: Option<String>,
) -> Result<Map> {
    if !owns_character(pool, actor.user_id, actor.character_id).await? {
        return Err(MapError::Forbidden);
    }
    if !crate::maps::map::may_create_map(pool, actor, creators).await? {
        return Err(MapError::Forbidden);
    }
    let name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| parsed.map_name.clone());

    let mut parsed = parsed;
    if let Some(settings) = &mut parsed.sections.settings {
        settings.name = name.clone();
    }

    let mut tx = pool.begin().await?;
    let map = crate::maps::map::insert_map(&mut tx, &name, None, actor.character_id).await?;
    if parsed.sections.routes.is_some() {
        sqlx::query!("delete from map_watchlist where map_id = $1", map.id)
            .execute(&mut *tx)
            .await?;
    }
    let mut tx = MapTx::adopt(tx, map.id, EventActor::Character(actor));
    let summary = import_into_tx(&mut tx, map.id, &parsed).await?;
    finish_import(&mut tx, map.id, &parsed, &summary).await?;
    let map = crate::maps::map::fetch_map(&mut **tx, map.id).await?;
    tx.commit().await?;
    Ok(map)
}

/// Settle the ghosts, write the journal entry and queue everything an import can touch for
/// open clients. `HistoryChanged` makes them refetch the view, signatures, watchlist and
/// history in one go.
async fn finish_import(
    tx: &mut MapTx,
    map_id: i64,
    parsed: &ParsedExport,
    summary: &ImportSummary,
) -> Result<()> {
    tx.emit(crate::maps::MapEvent::HistoryChanged { map_id });
    if parsed.sections.settings.is_some() {
        tx.emit(crate::maps::MapEvent::MapUpdated { map_id });
    }
    if parsed.sections.access.is_some() {
        tx.emit(crate::maps::MapEvent::AccessChanged { map_id });
    }
    let effect = Effect::new("map.imported", "imported a map export", CommandOutput::None)
        .entries(summary.changed().max(1));
    tx.settle(effect).await?;
    Ok(())
}

async fn import_into_tx(
    tx: &mut Tx<'_>,
    map_id: i64,
    parsed: &ParsedExport,
) -> Result<ImportSummary> {
    let mut summary = ImportSummary::default();
    let known = known_solarsystem_ids(tx, &parsed.sections).await?;

    if let Some(settings) = &parsed.sections.settings {
        import_settings(tx, map_id, settings, &mut summary.settings).await?;
    }
    if let Some(access) = &parsed.sections.access {
        import_access(tx, map_id, access, &mut summary.access).await?;
    }
    if let Some(systems) = &parsed.sections.solarsystems {
        import_solarsystems(tx, map_id, systems, &known, &mut summary.systems).await?;
    }
    // Home and rally live on placements, so they can only be marked once the systems the
    // file names are actually on the map.
    if let Some(settings) = &parsed.sections.settings {
        import_home_and_rally(tx, map_id, settings).await?;
    }

    let mut connection_ids_by_index = HashMap::new();
    if let Some(connections) = &parsed.sections.connections {
        connection_ids_by_index =
            import_connections(tx, map_id, connections, &mut summary.connections).await?;
    }
    if let Some(signatures) = &parsed.sections.signatures {
        import_signatures(
            tx,
            map_id,
            signatures,
            &connection_ids_by_index,
            &mut summary.signatures,
        )
        .await?;
    }
    if let Some(routes) = &parsed.sections.routes {
        import_routes(tx, map_id, routes, &known, &mut summary.routes).await?;
    }

    Ok(summary)
}

/// Every solar system id the payload references that exists in the local static data, so
/// unknown ids are skipped instead of hitting foreign keys.
async fn known_solarsystem_ids(tx: &mut Tx<'_>, sections: &Sections) -> Result<HashSet<i64>> {
    let mut ids: Vec<i64> = Vec::new();
    if let Some(settings) = &sections.settings {
        ids.extend(settings.home_solarsystem_id);
        ids.extend(settings.rally_ids());
    }
    if let Some(systems) = &sections.solarsystems {
        ids.extend(systems.iter().map(|s| s.solarsystem_id));
    }
    if let Some(routes) = &sections.routes {
        ids.extend(routes.route_solarsystems.iter().map(|r| r.solarsystem_id));
        ids.extend(routes.ignored_solarsystems.iter().map(|r| r.solarsystem_id));
    }

    let found = sqlx::query_scalar!("select id from solar_systems where id = any($1)", &ids,)
        .fetch_all(&mut **tx)
        .await?;
    Ok(found.into_iter().collect())
}

async fn import_settings(
    tx: &mut Tx<'_>,
    map_id: i64,
    settings: &SettingsSection,
    counts: &mut SectionCounts,
) -> Result<()> {
    sqlx::query!(
        "update maps
         set name = $2, layout = $3, allow_layout_override = $4, alias_scheme = $5,
             ignored_alias = $6, bookmark_wormhole = $7, bookmark_kspace = $8,
             bookmark_return = $9
         where id = $1",
        map_id,
        settings.name.trim(),
        settings.layout as MapLayout,
        settings.allow_layout_override,
        settings.bookmark_alias_scheme as AliasScheme,
        settings.bookmark_ignored_alias.as_deref().unwrap_or(""),
        settings.bookmark_format_wormhole,
        settings.bookmark_format_kspace,
        settings.bookmark_format_return,
    )
    .execute(&mut **tx)
    .await?;
    counts.updated += 1;
    Ok(())
}

/// Mark the file's home and rally systems, clearing the marks when the file has none (or
/// names a system that is not on the map). `coalesce(... , false)` keeps ghosts out: a
/// null system id never equals anything.
async fn import_home_and_rally(
    tx: &mut Tx<'_>,
    map_id: i64,
    settings: &SettingsSection,
) -> Result<()> {
    sqlx::query!(
        "update map_solar_systems
         set is_home = coalesce(solar_system_id = $2, false)
         where map_id = $1 and is_home is distinct from coalesce(solar_system_id = $2, false)",
        map_id,
        settings.home_solarsystem_id,
    )
    .execute(&mut **tx)
    .await?;
    // Duplicates would take two of the slots for one system.
    let mut rally: Vec<i64> = Vec::new();
    for id in settings.rally_ids() {
        if !rally.contains(&id) {
            rally.push(id);
        }
    }
    sqlx::query!(
        "update map_solar_systems set rallied_at = null
         where map_id = $1 and rallied_at is not null",
        map_id,
    )
    .execute(&mut **tx)
    .await?;
    // The newest of the placed ones, a microsecond apart in file order so the oldest stays
    // the first to make way for the next.
    sqlx::query!(
        "update map_solar_systems mss
         set rallied_at = now() + picked.ord * interval '1 microsecond'
         from (select placed.id, wanted.ord
               from unnest($2::bigint[]) with ordinality as wanted (solar_system_id, ord)
               join map_solar_systems placed
                   on placed.map_id = $1 and placed.solar_system_id = wanted.solar_system_id
               order by wanted.ord desc
               limit $3) picked
         where mss.id = picked.id",
        map_id,
        &rally,
        MAX_RALLY_POINTS as i64,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn import_access(
    tx: &mut Tx<'_>,
    map_id: i64,
    entries: &[AccessRow],
    counts: &mut SectionCounts,
) -> Result<()> {
    for entry in entries {
        let existing = sqlx::query_scalar!(
            r#"select role as "role: Role" from map_access
               where map_id = $1 and subject_id = $2"#,
            map_id,
            entry.entity_id,
        )
        .fetch_optional(&mut **tx)
        .await?;

        // The map's owner keeps their grant exactly as it is.
        if existing == Some(Role::Owner) {
            counts.skipped += 1;
            continue;
        }

        sqlx::query!(
            "insert into map_access (map_id, subject_type, subject_id, role, expires_at)
             values ($1, $2, $3, $4, $5)
             on conflict (map_id, subject_id)
             do update set subject_type = excluded.subject_type, role = excluded.role,
                 expires_at = excluded.expires_at",
            map_id,
            entry.entity_type as SubjectType,
            entry.entity_id,
            entry.permission as Role,
            entry.expires_at,
        )
        .execute(&mut **tx)
        .await?;

        match existing {
            Some(_) => counts.updated += 1,
            None => counts.created += 1,
        }
    }
    Ok(())
}

async fn import_solarsystems(
    tx: &mut Tx<'_>,
    map_id: i64,
    entries: &[SolarsystemRow],
    known: &HashSet<i64>,
    counts: &mut SectionCounts,
) -> Result<()> {
    // Keyed by system (last entry wins), unknown systems counted out up front.
    let mut importable: BTreeMap<i64, &SolarsystemRow> = BTreeMap::new();
    for entry in entries {
        if known.contains(&entry.solarsystem_id) {
            importable.insert(entry.solarsystem_id, entry);
        } else {
            counts.skipped += 1;
        }
    }

    let ids: Vec<i64> = importable.keys().copied().collect();
    let existing_details: HashSet<i64> = sqlx::query_scalar!(
        "select solar_system_id from map_solar_system_details
         where map_id = $1 and solar_system_id = any($2)",
        map_id,
        &ids,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .collect();
    let existing_placements: HashSet<i64> = sqlx::query_scalar!(
        r#"select solar_system_id as "solar_system_id!" from map_solar_systems
           where map_id = $1 and solar_system_id = any($2)"#,
        map_id,
        &ids,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .collect();

    for (id, entry) in &importable {
        sqlx::query!(
            "insert into map_solar_system_details
                 (map_id, solar_system_id, status, occupying_group, notes)
             values ($1, $2, $3, $4, $5)
             on conflict (map_id, solar_system_id)
             do update set status = excluded.status, occupying_group = excluded.occupying_group,
                 notes = excluded.notes, updated_at = now()",
            map_id,
            id,
            entry.status as SystemStatus,
            entry.occupier_alias.as_deref(),
            entry.notes.as_deref(),
        )
        .execute(&mut **tx)
        .await?;

        let placed = if let (Some(x), Some(y)) = (entry.position_x, entry.position_y) {
            let row = store::placements::NewPlacement {
                is_pinned: entry.pinned.unwrap_or(false),
                ..store::placements::NewPlacement::system(*id, x, y, entry.alias.as_deref())
            };
            store::placements::upsert_by_system(tx, map_id, &row).await?;
            true
        } else {
            false
        };

        let existed = if placed {
            existing_placements.contains(id)
        } else {
            existing_details.contains(id)
        };
        if existed {
            counts.updated += 1;
        } else {
            counts.created += 1;
        }
    }
    Ok(())
}

async fn import_connections(
    tx: &mut Tx<'_>,
    map_id: i64,
    entries: &[ConnectionRow],
    counts: &mut SectionCounts,
) -> Result<HashMap<i64, i64>> {
    let placements: HashMap<i64, i64> = sqlx::query!(
        r#"select id, solar_system_id as "solar_system_id!" from map_solar_systems
           where map_id = $1 and solar_system_id is not null"#,
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| (r.solar_system_id, r.id))
    .collect();

    // An edge exists in one direction only; the pair is normalized so the check matches
    // either way round.
    let mut existing: HashMap<(i64, i64), i64> = sqlx::query!(
        "select id, from_system, to_system from map_connections where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| {
        (
            (
                r.from_system.min(r.to_system),
                r.from_system.max(r.to_system),
            ),
            r.id,
        )
    })
    .collect();

    let mut ids_by_index = HashMap::new();
    for (index, entry) in entries.iter().enumerate() {
        let (Some(&from), Some(&to)) = (
            placements.get(&entry.from_solarsystem_id),
            placements.get(&entry.to_solarsystem_id),
        ) else {
            counts.skipped += 1;
            continue;
        };
        if from == to {
            counts.skipped += 1;
            continue;
        }

        let pair = (from.min(to), from.max(to));
        if let Some(&id) = existing.get(&pair) {
            ids_by_index.insert(index as i64, id);
            counts.skipped += 1;
            continue;
        }

        // A stargate edge carries no wormhole life-cycle state, whatever the file says.
        let (mass, time, size) = match entry.kind {
            ConnectionType::Wormhole => (
                entry.mass_status.to_status(),
                entry.lifetime.to_status(),
                Some(entry.ship_size.to_size()),
            ),
            ConnectionType::Stargate => (None, None, None),
        };

        let row = store::connections::NewConnection {
            from_system: from,
            to_system: to,
            kind: entry.kind,
            mass_status: mass,
            time_status: time,
            size,
            preserve_mass: entry.preserve_mass,
            time_status_updated_at: entry.lifetime_updated_at,
            created_at: entry.connected_at,
        };
        let id = store::connections::insert(tx, map_id, &row).await?;

        existing.insert(pair, id);
        ids_by_index.insert(index as i64, id);
        counts.created += 1;
    }
    Ok(ids_by_index)
}

async fn import_signatures(
    tx: &mut Tx<'_>,
    map_id: i64,
    entries: &[SignatureRow],
    connection_ids_by_index: &HashMap<i64, i64>,
    counts: &mut SectionCounts,
) -> Result<()> {
    let placed: HashSet<i64> = sqlx::query_scalar!(
        r#"select solar_system_id as "solar_system_id!" from map_solar_systems
           where map_id = $1 and solar_system_id is not null"#,
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .collect();

    let existing: HashSet<(i64, String)> = sqlx::query!(
        "select solar_system_id, signature_id from signatures where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| (r.solar_system_id, r.signature_id))
    .collect();

    // The catalog types the file names, resolved by name in one query. Same seed data on
    // both sides, so names line up.
    let names: Vec<String> = entries
        .iter()
        .filter_map(|e| e.type_name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let types: HashMap<String, (i64, i64)> = sqlx::query!(
        "select id, name, signature_category_id from signature_types where name = any($1)",
        &names,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|r| (r.name, (r.id, r.signature_category_id)))
    .collect();

    // A connection takes one signature per endpoint system. A file that names the same
    // connection twice from one system describes one hole scanned twice, so the first
    // claim stands and the rest come in as loose signatures.
    // Seeded with the links the map already has, keyed by the scanner id holding each, so
    // re-importing a file over the same map leaves a signature holding its own link.
    let mut claimed: HashMap<(i64, i64), String> = sqlx::query!(
        "select connection_id, solar_system_id, signature_id from signatures
         where map_id = $1 and connection_id is not null",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .filter_map(|r| {
        r.connection_id
            .map(|c| ((c, r.solar_system_id), r.signature_id))
    })
    .collect();

    for entry in entries {
        // This app requires the scanner id (it is the natural key), and the system must be
        // on the map for the signature to hang off.
        let Some(signature_id) = entry
            .signature_id
            .as_deref()
            .map(str::trim)
            .filter(|id| id.len() == 7)
        else {
            counts.skipped += 1;
            continue;
        };
        if !placed.contains(&entry.solarsystem_id) {
            counts.skipped += 1;
            continue;
        }

        let group = entry
            .category
            .as_deref()
            .and_then(group_for_category_code)
            .unwrap_or(SignatureGroup::Unknown);
        // A type from the wrong category (or one this database does not know) is dropped
        // rather than mislinked.
        let signature_type_id = entry
            .type_name
            .as_deref()
            .and_then(|name| types.get(name))
            .filter(|(_, category)| Some(*category) == signatures::category_id_for(group))
            .map(|(id, _)| *id);

        // Only a wormhole signature carries life-cycle state and a connection link.
        let (size, mass, time, connection_id) = if group == SignatureGroup::Wormhole {
            (
                entry.ship_size.map(WireShipSize::to_size),
                entry.mass_status.and_then(WireMassStatus::to_status),
                entry.lifetime.and_then(WireLifetime::to_status),
                entry
                    .connection_index
                    .and_then(|i| connection_ids_by_index.get(&i).copied())
                    .filter(|id| match claimed.entry((*id, entry.solarsystem_id)) {
                        Entry::Occupied(held) => held.get() == signature_id,
                        Entry::Vacant(free) => {
                            free.insert(signature_id.to_owned());
                            true
                        }
                    }),
            )
        } else {
            (None, None, None, None)
        };

        let row = store::signatures::NewSignature {
            solar_system_id: entry.solarsystem_id,
            signature_id,
            group,
            signature_type_id,
            name: entry.raw_type_name.as_deref(),
            size,
            mass_status: mass,
            time_status: time,
            connection_id,
        };
        let id = store::signatures::upsert_by_scanner_id(tx, map_id, &row).await?;

        // The stamp trigger writes now() whenever the lifetime changes; the file knows
        // better, so its stamp is put back. Changing nothing else, this does not restamp.
        if let Some(stamp) = entry.lifetime_updated_at
            && time.is_some()
        {
            sqlx::query!(
                "update signatures set time_status_updated_at = $2 where id = $1",
                id,
                stamp,
            )
            .execute(&mut **tx)
            .await?;
        }

        if existing.contains(&(entry.solarsystem_id, signature_id.to_string())) {
            counts.updated += 1;
        } else {
            counts.created += 1;
        }
    }
    Ok(())
}

async fn import_routes(
    tx: &mut Tx<'_>,
    map_id: i64,
    routes: &RoutesSection,
    known: &HashSet<i64>,
    counts: &mut SectionCounts,
) -> Result<()> {
    let existing: HashSet<i64> = sqlx::query_scalar!(
        "select solar_system_id from map_watchlist where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .collect();

    for entry in &routes.route_solarsystems {
        if !known.contains(&entry.solarsystem_id) {
            counts.skipped += 1;
            continue;
        }
        sqlx::query!(
            "insert into map_watchlist (map_id, solar_system_id, is_pinned)
             values ($1, $2, $3)
             on conflict (map_id, solar_system_id)
             do update set is_pinned = excluded.is_pinned",
            map_id,
            entry.solarsystem_id,
            entry.is_pinned,
        )
        .execute(&mut **tx)
        .await?;
        if existing.contains(&entry.solarsystem_id) {
            counts.updated += 1;
        } else {
            counts.created += 1;
        }
    }

    let ignored: HashSet<i64> = sqlx::query_scalar!(
        "select solar_system_id from map_ignored_solar_systems where map_id = $1",
        map_id,
    )
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .collect();
    for entry in &routes.ignored_solarsystems {
        if !known.contains(&entry.solarsystem_id) {
            counts.skipped += 1;
            continue;
        }
        if ignored.contains(&entry.solarsystem_id) {
            counts.updated += 1;
            continue;
        }
        sqlx::query!(
            "insert into map_ignored_solar_systems (map_id, solar_system_id) values ($1, $2)",
            map_id,
            entry.solarsystem_id,
        )
        .execute(&mut **tx)
        .await?;
        counts.created += 1;
    }
    Ok(())
}
