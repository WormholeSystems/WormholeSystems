//! Writing a map out, and how much each section would carry.

use std::collections::{BTreeMap, HashMap};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::maps::access::require_role;
use crate::maps::error::{MapError, Result};
use crate::maps::{Actor, Role, SignatureGroup, SystemStatus};

use super::format::{
    AccessRow, ConnectionRow, ExportFile, FORMAT, IgnoredRow, RouteRow, RoutesSection, SectionSet,
    Sections, SettingsSection, SignatureRow, SolarsystemRow, VERSION, WireLifetime, WireMassStatus,
    WireShipSize, category_code,
};

/// Build the export payload for the selected sections. Manager+. Secrets (share token,
/// webhooks, per-user settings) and the owner grant never leave the map. Ghost placements
/// and the connections touching them are skipped: the format cannot say "a system nobody
/// has identified yet".
pub async fn export_map(
    pool: &PgPool,
    actor: Actor,
    map_id: i64,
    sections: SectionSet,
) -> Result<ExportFile> {
    require_role(pool, map_id, actor.user_id, Role::Manager).await?;

    let map_name = sqlx::query_scalar!("select name from maps where id = $1", map_id)
        .fetch_optional(pool)
        .await?
        .ok_or(MapError::NotFound)?;

    let mut out = Sections::default();

    if sections.settings {
        out.settings = Some(export_settings(pool, map_id).await?);
    }
    if sections.access {
        out.access = Some(export_access(pool, map_id).await?);
    }
    if sections.solarsystems {
        out.solarsystems = Some(export_solarsystems(pool, map_id).await?);
    }

    // Signatures name their connection by its position in this very file, so the two
    // sections are built together.
    let mut index_by_connection = None;
    if sections.connections {
        let (rows, indexes) = export_connections(pool, map_id).await?;
        out.connections = Some(rows);
        index_by_connection = Some(indexes);
    }
    if sections.signatures {
        out.signatures = Some(export_signatures(pool, map_id, index_by_connection.as_ref()).await?);
    }
    if sections.routes {
        out.routes = Some(export_routes(pool, map_id).await?);
    }

    Ok(ExportFile {
        format: FORMAT.into(),
        version: VERSION,
        exported_at: Utc::now(),
        map_name,
        sections: out,
    })
}

async fn export_settings(pool: &PgPool, map_id: i64) -> Result<SettingsSection> {
    let row = sqlx::query!(
        r#"select name, layout, allow_layout_override, alias_scheme, ignored_alias,
                  bookmark_wormhole, bookmark_kspace, bookmark_return,
                  (select solar_system_id from map_solar_systems
                   where map_id = m.id and is_home) as "home?",
                  array(select solar_system_id from map_solar_systems
                        where map_id = m.id and rallied_at is not null
                        order by rallied_at, id) as "rally!: Vec<i64>"
           from maps m where m.id = $1"#,
        map_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MapError::NotFound)?;

    Ok(SettingsSection {
        name: row.name,
        layout: row.layout,
        allow_layout_override: row.allow_layout_override,
        constant_width_enabled: false,
        bookmark_format_wormhole: row.bookmark_wormhole,
        bookmark_format_kspace: row.bookmark_kspace,
        bookmark_format_return: row.bookmark_return,
        bookmark_alias_scheme: row.alias_scheme,
        bookmark_ignored_alias: Some(row.ignored_alias).filter(|a| !a.is_empty()),
        home_solarsystem_id: row.home,
        rally_solarsystem_ids: Some(row.rally),
        rally_solarsystem_id: None,
    })
}

async fn export_access(pool: &PgPool, map_id: i64) -> Result<Vec<AccessRow>> {
    let rows = sqlx::query!(
        // The view widens every column to nullable; the table it reads does not.
        r#"select a.subject_type as "subject_type!", a.subject_id as "subject_id!",
                  coalesce(c.name, corp.name, al.name) as "name?",
                  a.role as "role!", a.expires_at
           from map_access_live a
           left join characters c
             on a.subject_type = 'character' and c.id = a.subject_id
           left join corporations corp
             on a.subject_type = 'corporation' and corp.id = a.subject_id
           left join alliances al
             on a.subject_type = 'alliance' and al.id = a.subject_id
           where a.map_id = $1 and a.role <> 'owner'
           order by a.id"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AccessRow {
            entity_type: r.subject_type,
            entity_id: r.subject_id,
            entity_name: r.name,
            permission: r.role,
            expires_at: r.expires_at,
        })
        .collect())
}

async fn export_solarsystems(pool: &PgPool, map_id: i64) -> Result<Vec<SolarsystemRow>> {
    // Intel and placement are separate rows that mostly overlap: a system can be placed
    // without anyone having classified it, and intel outlives its placement. Merged here,
    // keyed by system, so each system exports exactly once.
    let mut rows: BTreeMap<i64, SolarsystemRow> = BTreeMap::new();

    let details = sqlx::query!(
        "select solar_system_id, status, occupying_group, notes
         from map_solar_system_details where map_id = $1",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    for d in details {
        rows.insert(
            d.solar_system_id,
            SolarsystemRow {
                solarsystem_id: d.solar_system_id,
                alias: None,
                position_x: None,
                position_y: None,
                pinned: None,
                status: d.status,
                occupier_alias: d.occupying_group,
                notes: d.notes,
            },
        );
    }

    let placements = sqlx::query!(
        r#"select solar_system_id as "solar_system_id!", alias, position_x, position_y, is_pinned
           from map_solar_systems where map_id = $1 and solar_system_id is not null"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;
    for p in placements {
        let row = rows
            .entry(p.solar_system_id)
            .or_insert_with(|| SolarsystemRow {
                solarsystem_id: p.solar_system_id,
                alias: None,
                position_x: None,
                position_y: None,
                pinned: None,
                status: SystemStatus::Unknown,
                occupier_alias: None,
                notes: None,
            });
        row.alias = p.alias;
        row.position_x = Some(p.position_x);
        row.position_y = Some(p.position_y);
        row.pinned = Some(p.is_pinned);
    }

    Ok(rows.into_values().collect())
}

async fn export_connections(
    pool: &PgPool,
    map_id: i64,
) -> Result<(Vec<ConnectionRow>, HashMap<i64, i64>)> {
    // The best available name for each connection's hole comes off a linked signature's
    // catalog type. The non-K162 side names the hole itself, so it wins when both ends are
    // scanned.
    let mut codes: HashMap<i64, String> = HashMap::new();
    let code_rows = sqlx::query!(
        r#"select s.connection_id as "connection_id!", st.signature as "code!"
           from signatures s
           join signature_types st on st.id = s.signature_type_id
           where s.map_id = $1 and s.connection_id is not null and st.signature is not null
           order by s.id"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;
    for row in code_rows {
        let entry = codes
            .entry(row.connection_id)
            .or_insert_with(|| row.code.clone());
        if entry == "K162" && row.code != "K162" {
            *entry = row.code;
        }
    }

    let rows = sqlx::query!(
        r#"select c.id, f.solar_system_id as "from_ss?", t.solar_system_id as "to_ss?",
                  c.type as kind, c.mass_status, c.time_status, c.size,
                  c.time_status_updated_at, c.created_at, c.preserve_mass
           from map_connections c
           join map_solar_systems f on f.id = c.from_system
           join map_solar_systems t on t.id = c.to_system
           where c.map_id = $1
           order by c.id"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;

    let mut exported = Vec::new();
    let mut indexes = HashMap::new();
    for row in rows {
        let (Some(from_ss), Some(to_ss)) = (row.from_ss, row.to_ss) else {
            continue;
        };
        indexes.insert(row.id, exported.len() as i64);
        exported.push(ConnectionRow {
            from_solarsystem_id: from_ss,
            to_solarsystem_id: to_ss,
            wormhole: codes.get(&row.id).cloned(),
            kind: row.kind,
            mass_status: WireMassStatus::from_status(row.mass_status),
            ship_size: Some(WireShipSize::from_size(row.size)),
            lifetime: WireLifetime::from_status(row.time_status),
            lifetime_updated_at: row.time_status_updated_at,
            connected_at: Some(row.created_at),
            preserve_mass: row.preserve_mass,
        });
    }
    Ok((exported, indexes))
}

async fn export_signatures(
    pool: &PgPool,
    map_id: i64,
    index_by_connection: Option<&HashMap<i64, i64>>,
) -> Result<Vec<SignatureRow>> {
    let rows = sqlx::query!(
        r#"select s.solar_system_id, s.signature_id, s."group" as sig_group,
                  st.name as "type_name?", st.signature as "code?", s.name,
                  s.size, s.mass_status, s.time_status, s.time_status_updated_at,
                  s.connection_id
           from signatures s
           left join signature_types st on st.id = s.signature_type_id
           where s.map_id = $1
           order by s.id"#,
        map_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SignatureRow {
            solarsystem_id: r.solar_system_id,
            signature_id: Some(r.signature_id),
            category: category_code(r.sig_group).map(str::to_string),
            type_name: r.type_name,
            raw_type_name: r.name,
            wormhole: r.code.filter(|_| r.sig_group == SignatureGroup::Wormhole),
            connection_index: match (r.connection_id, index_by_connection) {
                (Some(id), Some(indexes)) => indexes.get(&id).copied(),
                _ => None,
            },
            mass_status: r.mass_status.map(|m| WireMassStatus::from_status(Some(m))),
            ship_size: r.size.map(|s| WireShipSize::from_size(Some(s))),
            lifetime: Some(WireLifetime::from_status(r.time_status)),
            lifetime_updated_at: r.time_status_updated_at,
        })
        .collect())
}

async fn export_routes(pool: &PgPool, map_id: i64) -> Result<RoutesSection> {
    let rows = sqlx::query!(
        "select solar_system_id, is_pinned from map_watchlist where map_id = $1 order by id",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    let ignored = sqlx::query_scalar!(
        "select solar_system_id from map_ignored_solar_systems where map_id = $1 order by id",
        map_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(RoutesSection {
        route_solarsystems: rows
            .into_iter()
            .map(|r| RouteRow {
                solarsystem_id: r.solar_system_id,
                is_pinned: r.is_pinned,
            })
            .collect(),
        ignored_solarsystems: ignored
            .into_iter()
            .map(|solarsystem_id| IgnoredRow { solarsystem_id })
            .collect(),
    })
}

// ---------------------------------------------------------------------------------------
// The counts the transfer settings page shows
// ---------------------------------------------------------------------------------------

/// How much of the map each section would carry, for the export UI. Manager+.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct TransferCounts {
    pub access: i64,
    pub systems: i64,
    pub connections: i64,
    pub signatures: i64,
    pub routes: i64,
}

pub async fn transfer_counts(pool: &PgPool, actor: Actor, map_id: i64) -> Result<TransferCounts> {
    require_role(pool, map_id, actor.user_id, Role::Manager).await?;
    let row = sqlx::query!(
        r#"select
               (select count(*) from map_access_live a
                where a.map_id = $1 and a.role <> 'owner') as "access!",
               (select count(*) from (
                    select solar_system_id from map_solar_system_details where map_id = $1
                    union
                    select solar_system_id from map_solar_systems
                    where map_id = $1 and solar_system_id is not null
                ) systems) as "systems!",
               (select count(*) from map_connections c
                join map_solar_systems f on f.id = c.from_system
                join map_solar_systems t on t.id = c.to_system
                where c.map_id = $1
                  and f.solar_system_id is not null
                  and t.solar_system_id is not null) as "connections!",
               (select count(*) from signatures s where s.map_id = $1) as "signatures!",
               (select count(*) from map_watchlist w where w.map_id = $1)
               + (select count(*) from map_ignored_solar_systems i where i.map_id = $1)
                 as "routes!""#,
        map_id,
    )
    .fetch_one(pool)
    .await?;
    Ok(TransferCounts {
        access: row.access,
        systems: row.systems,
        connections: row.connections,
        signatures: row.signatures,
        routes: row.routes,
    })
}
