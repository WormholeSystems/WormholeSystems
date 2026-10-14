//! Map import and export, wire-compatible with the legacy `wormholesystems-map-export`
//! format (version 1), so files move between the two applications in both directions.
//!
//! The file is a JSON envelope with six independently selectable sections. Cross
//! references inside it are portable by construction: EVE solar system ids, wormhole code
//! names ("K162"), signature type names, and entity ids for access rows. Signatures point
//! at connections by array index within the same file.
//!
//! Legacy speaks a slightly different vocabulary for the life-cycle enums (mass `fresh`
//! vs `stable`, ship sizes vs hole sizes, lifetime `healthy` vs `stable`); the `Wire*`
//! enums here are that vocabulary, with conversions. What vector cannot represent is
//! skipped rather than refused: a ghost placement is not exported, a signature without a
//! scanner id is not imported.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::access::{owns_character, require_role, require_role_tx};
use super::command::{CommandOutput, Effect, EventActor, Tx};
use super::error::{MapError, Result};
use super::map::Map;
use super::solar_system::MAX_RALLY_POINTS;
use super::{
    Actor, AliasScheme, ConnectionType, MapLayout, MassStatus, Role, SignatureGroup, SubjectType,
    SystemStatus, TimeStatus, WormholeSize, events_log, ghost, signatures,
};

pub const FORMAT: &str = "wormholesystems-map-export";
pub const VERSION: i64 = 1;

// ---------------------------------------------------------------------------------------
// The wire format
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ExportFile {
    pub format: String,
    pub version: i64,
    pub exported_at: DateTime<Utc>,
    pub map_name: String,
    pub sections: Sections,
}

/// Only the sections that were asked for are present; on import, only the requested ones
/// are read at all, so a broken section nobody selected does not fail the file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Sections {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub settings: Option<SettingsSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub access: Option<Vec<AccessRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub solarsystems: Option<Vec<SolarsystemRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub connections: Option<Vec<ConnectionRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub signatures: Option<Vec<SignatureRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub routes: Option<RoutesSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SettingsSection {
    pub name: String,
    pub layout: MapLayout,
    #[serde(deserialize_with = "lenient_bool")]
    pub allow_layout_override: bool,
    /// A legacy display setting vector does not have; exported as `false`, ignored on
    /// import, kept so the file validates over there.
    #[serde(deserialize_with = "lenient_bool")]
    pub constant_width_enabled: bool,
    pub bookmark_format_wormhole: String,
    pub bookmark_format_kspace: String,
    pub bookmark_format_return: String,
    pub bookmark_alias_scheme: AliasScheme,
    pub bookmark_ignored_alias: Option<String>,
    pub home_solarsystem_id: Option<i64>,
    /// Oldest first. On import only the newest two are kept.
    #[serde(default)]
    pub rally_solarsystem_ids: Option<Vec<i64>>,
    /// The single rally point of files written before a map could hold two. Read when
    /// `rally_solarsystem_ids` is absent, never written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub rally_solarsystem_id: Option<i64>,
}

impl SettingsSection {
    /// The file's rally points, whichever way it spells them.
    fn rally_ids(&self) -> Vec<i64> {
        match &self.rally_solarsystem_ids {
            Some(ids) => ids.clone(),
            None => self.rally_solarsystem_id.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AccessRow {
    pub entity_type: SubjectType,
    pub entity_id: i64,
    /// Carried so the receiving side can show who this is without an ESI lookup.
    pub entity_name: Option<String>,
    /// `viewer` / `member` / `manager`; the owner row never leaves a map.
    pub permission: Role,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SolarsystemRow {
    pub solarsystem_id: i64,
    pub alias: Option<String>,
    /// Null positions mean intel without a placement: the system is not on the canvas,
    /// but its status and notes are kept.
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    #[serde(default, deserialize_with = "lenient_bool_opt")]
    pub pinned: Option<bool>,
    pub status: SystemStatus,
    pub occupier_alias: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ConnectionRow {
    pub from_solarsystem_id: i64,
    pub to_solarsystem_id: i64,
    /// The wormhole code name ("K162"), when one is known.
    pub wormhole: Option<String>,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub kind: ConnectionType,
    pub mass_status: WireMassStatus,
    pub ship_size: WireShipSize,
    pub lifetime: WireLifetime,
    pub lifetime_updated_at: Option<DateTime<Utc>>,
    pub connected_at: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "lenient_bool")]
    pub preserve_mass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SignatureRow {
    pub solarsystem_id: i64,
    /// The in-game scanner id ("ABC-123"). Legacy allows signatures without one; vector
    /// does not, so those are skipped on import.
    pub signature_id: Option<String>,
    /// The category code ("wormhole", "data", ...). Unmatched codes import as `unknown`.
    pub category: Option<String>,
    /// The catalog type by name; both applications seed the catalog from the same data,
    /// so names resolve on either side.
    pub type_name: Option<String>,
    pub raw_type_name: Option<String>,
    pub wormhole: Option<String>,
    /// Index into this file's `connections` array, when the signature is one end of one.
    pub connection_index: Option<i64>,
    pub mass_status: Option<WireMassStatus>,
    pub ship_size: Option<WireShipSize>,
    pub lifetime: Option<WireLifetime>,
    pub lifetime_updated_at: Option<DateTime<Utc>>,
}

/// The wire spelling of a group is the legacy category code, which hyphenates the one
/// two-word category where the enum uses an underscore.
fn category_code(group: SignatureGroup) -> Option<&'static str> {
    match group {
        SignatureGroup::Unknown => None,
        SignatureGroup::FactionWarfare => Some("faction-warfare"),
        group => Some(group.as_str()),
    }
}

fn group_for_category_code(code: &str) -> Option<SignatureGroup> {
    match code {
        "faction-warfare" => Some(SignatureGroup::FactionWarfare),
        code => SignatureGroup::from_db(code),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RoutesSection {
    pub route_solarsystems: Vec<RouteRow>,
    /// The map's ignore list: routed around by everyone, never placed by jump tracking.
    pub ignored_solarsystems: Vec<IgnoredRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RouteRow {
    pub solarsystem_id: i64,
    #[serde(deserialize_with = "lenient_bool")]
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IgnoredRow {
    pub solarsystem_id: i64,
}

/// A boolean as legacy writes it: PHP lets database `0`/`1` slip through `json_encode`
/// unconverted, and the legacy importer accepts both, so this side does too.
fn lenient_bool<'de, D: serde::Deserializer<'de>>(de: D) -> std::result::Result<bool, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Loose {
        Bool(bool),
        Int(i64),
    }
    match Loose::deserialize(de)? {
        Loose::Bool(b) => Ok(b),
        Loose::Int(0) => Ok(false),
        Loose::Int(1) => Ok(true),
        Loose::Int(n) => Err(serde::de::Error::custom(format!(
            "invalid boolean value {n}"
        ))),
    }
}

fn lenient_bool_opt<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> std::result::Result<Option<bool>, D::Error> {
    #[derive(Deserialize)]
    struct Wrap(#[serde(deserialize_with = "lenient_bool")] bool);
    Ok(Option::<Wrap>::deserialize(de)?.map(|w| w.0))
}

/// Legacy's mass vocabulary. Vector's `stable` is legacy's `fresh`, and vector says
/// "unknown" with a null.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WireMassStatus {
    Fresh,
    Reduced,
    Critical,
    Unknown,
}

impl WireMassStatus {
    fn to_status(self) -> Option<MassStatus> {
        match self {
            WireMassStatus::Fresh => Some(MassStatus::Stable),
            WireMassStatus::Reduced => Some(MassStatus::Reduced),
            WireMassStatus::Critical => Some(MassStatus::Critical),
            WireMassStatus::Unknown => None,
        }
    }

    fn from_status(status: Option<MassStatus>) -> Self {
        match status {
            Some(MassStatus::Stable) => WireMassStatus::Fresh,
            Some(MassStatus::Reduced) => WireMassStatus::Reduced,
            Some(MassStatus::Critical) => WireMassStatus::Critical,
            None => WireMassStatus::Unknown,
        }
    }
}

/// Legacy names hole sizes by the ship class that fits through.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WireShipSize {
    Frigate,
    Medium,
    Large,
    Xlarge,
}

impl WireShipSize {
    fn to_size(self) -> WormholeSize {
        match self {
            WireShipSize::Frigate => WormholeSize::Small,
            WireShipSize::Medium => WormholeSize::Medium,
            WireShipSize::Large => WormholeSize::Large,
            WireShipSize::Xlarge => WormholeSize::Xl,
        }
    }

    /// An unknown size exports as `large`, legacy's own column default.
    fn from_size(size: Option<WormholeSize>) -> Self {
        match size {
            Some(WormholeSize::Small) => WireShipSize::Frigate,
            Some(WormholeSize::Medium) => WireShipSize::Medium,
            Some(WormholeSize::Large) | None => WireShipSize::Large,
            Some(WormholeSize::Xl) => WireShipSize::Xlarge,
        }
    }
}

/// Legacy's lifetime vocabulary has no "unknown": `healthy` is both vector's `stable` and
/// its null, and imports as the null (nothing remarkable to show).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum WireLifetime {
    Healthy,
    Eol,
    Critical,
    Expired,
}

impl WireLifetime {
    fn to_status(self) -> Option<TimeStatus> {
        match self {
            WireLifetime::Healthy => None,
            WireLifetime::Eol => Some(TimeStatus::Eol),
            WireLifetime::Critical => Some(TimeStatus::Critical),
            WireLifetime::Expired => Some(TimeStatus::Expired),
        }
    }

    fn from_status(status: Option<TimeStatus>) -> Self {
        match status {
            Some(TimeStatus::Eol) => WireLifetime::Eol,
            Some(TimeStatus::Critical) => WireLifetime::Critical,
            Some(TimeStatus::Expired) => WireLifetime::Expired,
            Some(TimeStatus::Stable) | None => WireLifetime::Healthy,
        }
    }
}

// ---------------------------------------------------------------------------------------
// Section selection and file parsing
// ---------------------------------------------------------------------------------------

pub const SECTION_NAMES: [&str; 6] = [
    "settings",
    "access",
    "solarsystems",
    "connections",
    "signatures",
    "routes",
];

/// Which sections an export or import touches.
#[derive(Debug, Clone, Copy, Default)]
pub struct SectionSet {
    pub settings: bool,
    pub access: bool,
    pub solarsystems: bool,
    pub connections: bool,
    pub signatures: bool,
    pub routes: bool,
}

impl SectionSet {
    pub fn from_names(names: &[String]) -> Result<Self> {
        let mut set = SectionSet::default();
        for name in names {
            match name.as_str() {
                "settings" => set.settings = true,
                "access" => set.access = true,
                "solarsystems" => set.solarsystems = true,
                "connections" => set.connections = true,
                "signatures" => set.signatures = true,
                "routes" => set.routes = true,
                other => {
                    return Err(MapError::Validation(format!("unknown section \"{other}\"")));
                }
            }
        }
        if !set.any() {
            return Err(MapError::Validation("select at least one section".into()));
        }
        Ok(set)
    }

    fn any(self) -> bool {
        self.settings
            || self.access
            || self.solarsystems
            || self.connections
            || self.signatures
            || self.routes
    }
}

/// A validated export file, holding only the requested sections.
#[derive(Debug, Clone)]
pub struct ParsedExport {
    pub map_name: String,
    pub sections: Sections,
}

/// Decode and validate an uploaded export, keeping only the requested sections. Every way
/// the file can be wrong comes back as `Validation` with a sentence worth showing.
pub fn parse_export(
    content: &str,
    sections: SectionSet,
    for_new_map: bool,
) -> Result<ParsedExport> {
    let invalid = |msg: String| MapError::Validation(msg);

    let value: serde_json::Value =
        serde_json::from_str(content).map_err(|_| invalid("this file is not valid JSON".into()))?;

    if value.get("format").and_then(|f| f.as_str()) != Some(FORMAT) {
        return Err(invalid(
            "this file is not a wormholesystems map export".into(),
        ));
    }
    if value.get("version").and_then(|v| v.as_i64()) != Some(VERSION) {
        return Err(invalid(
            "this file was exported by an incompatible version of the application".into(),
        ));
    }

    let map_name = value
        .get("map_name")
        .and_then(|n| n.as_str())
        .map(str::trim)
        .filter(|n| !n.is_empty() && n.chars().count() <= 255)
        .ok_or_else(|| invalid("the file does not contain a valid map name".into()))?
        .to_string();

    let available = value
        .get("sections")
        .and_then(|s| s.as_object())
        .ok_or_else(|| invalid("the file does not contain any sections".into()))?;

    let requested = [
        ("settings", sections.settings),
        ("access", sections.access),
        ("solarsystems", sections.solarsystems),
        ("connections", sections.connections),
        ("signatures", sections.signatures),
        ("routes", sections.routes),
    ];
    for (name, wanted) in requested {
        if wanted && !available.contains_key(name) {
            return Err(invalid(format!(
                "the file does not contain the \"{name}\" section"
            )));
        }
    }

    // A fresh map has no placements, so connections and signatures would have nothing to
    // attach to without the systems that carry them.
    if for_new_map && !sections.solarsystems && (sections.connections || sections.signatures) {
        return Err(invalid(
            "importing connections or signatures into a new map requires the solar systems section"
                .into(),
        ));
    }

    fn section<T: serde::de::DeserializeOwned>(
        available: &serde_json::Map<String, serde_json::Value>,
        name: &str,
        wanted: bool,
    ) -> Result<Option<T>> {
        if !wanted {
            return Ok(None);
        }
        serde_json::from_value(available[name].clone())
            .map(Some)
            .map_err(|e| {
                MapError::Validation(format!("the \"{name}\" section contains invalid data: {e}"))
            })
    }

    let parsed = Sections {
        settings: section(available, "settings", sections.settings)?,
        access: section(available, "access", sections.access)?,
        solarsystems: section(available, "solarsystems", sections.solarsystems)?,
        connections: section(available, "connections", sections.connections)?,
        signatures: section(available, "signatures", sections.signatures)?,
        routes: section(available, "routes", sections.routes)?,
    };

    // The owner row never travels; a file claiming one is not one of ours.
    if let Some(access) = &parsed.access
        && access.iter().any(|row| row.permission == Role::Owner)
    {
        return Err(invalid("an export never contains an owner grant".into()));
    }

    Ok(ParsedExport {
        map_name,
        sections: parsed,
    })
}

// ---------------------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------------------

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
        r#"select a.subject_type, a.subject_id,
                  coalesce(c.name, corp.name, al.name) as "name?",
                  a.role, a.expires_at
           from map_access a
           left join characters c
             on a.subject_type = 'character' and c.id = a.subject_id
           left join corporations corp
             on a.subject_type = 'corporation' and corp.id = a.subject_id
           left join alliances al
             on a.subject_type = 'alliance' and al.id = a.subject_id
           where a.map_id = $1 and a.role <> 'owner'
             and (a.expires_at is null or a.expires_at > now())
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
            ship_size: WireShipSize::from_size(row.size),
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
// Import
// ---------------------------------------------------------------------------------------

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
    let mut tx = pool.begin().await?;
    require_role_tx(&mut tx, map_id, actor.user_id, Role::Manager).await?;
    let summary = import_into_tx(&mut tx, map_id, parsed).await?;
    let events = finish_import(&mut tx, map_id, actor, &summary).await?;
    tx.commit().await?;
    publish_after_import(map_id, parsed, events);
    Ok(summary)
}

/// Create a fresh map owned by the acting character and load the payload into it. When the
/// file carries a routes section, the seeded trade-hub watchlist makes way for the file's
/// list. An explicit `name` wins over the file's.
pub async fn import_map_as_new(
    pool: &PgPool,
    actor: Actor,
    parsed: ParsedExport,
    name: Option<String>,
) -> Result<Map> {
    if !owns_character(pool, actor.user_id, actor.character_id).await? {
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
    let map = super::map::insert_map(&mut tx, &name, None, actor.character_id).await?;
    if parsed.sections.routes.is_some() {
        sqlx::query!("delete from map_watchlist where map_id = $1", map.id)
            .execute(&mut *tx)
            .await?;
    }
    let summary = import_into_tx(&mut tx, map.id, &parsed).await?;
    let events = finish_import(&mut tx, map.id, actor, &summary).await?;
    let map = super::map::fetch_map(&mut *tx, map.id).await?;
    tx.commit().await?;
    publish_after_import(map.id, &parsed, events);
    Ok(map)
}

/// Reconcile ghosts and write the journal entry, inside the import's transaction.
async fn finish_import(
    tx: &mut Tx<'_>,
    map_id: i64,
    actor: Actor,
    summary: &ImportSummary,
) -> Result<Vec<super::MapEvent>> {
    let (_, ghost_events) = ghost::reconcile(tx, map_id).await?;
    let effect = Effect::new("map.imported", "imported a map export", CommandOutput::None)
        .entries(summary.changed().max(1));
    events_log::record(tx, map_id, EventActor::Character(actor), &effect).await?;
    Ok(ghost_events)
}

/// Everything an import can touch, told to open clients after commit. `HistoryChanged`
/// makes them refetch the view, signatures, watchlist and history in one go.
fn publish_after_import(map_id: i64, parsed: &ParsedExport, ghost_events: Vec<super::MapEvent>) {
    let hub = super::hub();
    hub.publish(super::MapEvent::HistoryChanged { map_id });
    if parsed.sections.settings.is_some() {
        hub.publish(super::MapEvent::MapUpdated { map_id });
    }
    if parsed.sections.access.is_some() {
        hub.publish(super::MapEvent::AccessChanged { map_id });
    }
    for event in ghost_events {
        hub.publish(event);
    }
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
            sqlx::query!(
                "insert into map_solar_systems
                     (map_id, solar_system_id, position_x, position_y, alias, is_pinned)
                 values ($1, $2, $3, $4, $5, $6)
                 on conflict (map_id, solar_system_id)
                 do update set position_x = excluded.position_x, position_y = excluded.position_y,
                     alias = excluded.alias, is_pinned = excluded.is_pinned",
                map_id,
                id,
                x,
                y,
                entry.alias.as_deref(),
                entry.pinned.unwrap_or(false),
            )
            .execute(&mut **tx)
            .await?;
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

        let id = sqlx::query_scalar!(
            "insert into map_connections
                 (map_id, from_system, to_system, type, mass_status, time_status, size,
                  time_status_updated_at, preserve_mass, created_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, coalesce($10, now()))
             returning id",
            map_id,
            from,
            to,
            entry.kind as ConnectionType,
            mass as Option<MassStatus>,
            time as Option<TimeStatus>,
            size as Option<WormholeSize>,
            entry.lifetime_updated_at,
            entry.preserve_mass,
            entry.connected_at,
        )
        .fetch_one(&mut **tx)
        .await?;

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
        // Vector requires the scanner id (it is the natural key), and the system must be
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

        let id = sqlx::query_scalar!(
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
            entry.solarsystem_id,
            signature_id,
            group as SignatureGroup,
            signature_type_id,
            entry.raw_type_name.as_deref(),
            size as Option<WormholeSize>,
            mass as Option<MassStatus>,
            time as Option<TimeStatus>,
            connection_id,
        )
        .fetch_one(&mut **tx)
        .await?;

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
               (select count(*) from map_access a
                where a.map_id = $1 and a.role <> 'owner'
                  and (a.expires_at is null or a.expires_at > now())) as "access!",
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
