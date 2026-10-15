//! The wire format: the file's envelope and sections, the legacy vocabulary for the
//! life-cycle enums, and reading a file back in.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::maps::error::{MapError, Result};
use crate::maps::{
    AliasScheme, ConnectionType, MapLayout, MassStatus, Role, SignatureGroup, SubjectType,
    SystemStatus, TimeStatus, WormholeSize,
};

pub const FORMAT: &str = "wormholesystems-map-export";
pub const VERSION: i64 = 1;

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
    /// A legacy display setting this app does not have; exported as `false`, ignored on
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
    pub(super) fn rally_ids(&self) -> Vec<i64> {
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
    /// Legacy writes null for a hole nobody has sized yet. This side always writes one.
    #[serde(default)]
    pub ship_size: Option<WireShipSize>,
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
    /// The in-game scanner id ("ABC-123"). Legacy allows signatures without one; this app
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
pub(super) fn category_code(group: SignatureGroup) -> Option<&'static str> {
    match group {
        SignatureGroup::Unknown => None,
        SignatureGroup::FactionWarfare => Some("faction-warfare"),
        group => Some(group.as_str()),
    }
}

pub(super) fn group_for_category_code(code: &str) -> Option<SignatureGroup> {
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

/// Legacy's mass vocabulary. Our `stable` is legacy's `fresh`, and we say
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
    pub(super) fn to_status(self) -> Option<MassStatus> {
        match self {
            WireMassStatus::Fresh => Some(MassStatus::Stable),
            WireMassStatus::Reduced => Some(MassStatus::Reduced),
            WireMassStatus::Critical => Some(MassStatus::Critical),
            WireMassStatus::Unknown => None,
        }
    }

    pub(super) fn from_status(status: Option<MassStatus>) -> Self {
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
    pub(super) fn to_size(self) -> WormholeSize {
        match self {
            WireShipSize::Frigate => WormholeSize::Small,
            WireShipSize::Medium => WormholeSize::Medium,
            WireShipSize::Large => WormholeSize::Large,
            WireShipSize::Xlarge => WormholeSize::Xl,
        }
    }

    /// An unknown size exports as `large`, legacy's own column default.
    pub(super) fn from_size(size: Option<WormholeSize>) -> Self {
        match size {
            Some(WormholeSize::Small) => WireShipSize::Frigate,
            Some(WormholeSize::Medium) => WireShipSize::Medium,
            Some(WormholeSize::Large) | None => WireShipSize::Large,
            Some(WormholeSize::Xl) => WireShipSize::Xlarge,
        }
    }
}

/// Legacy's lifetime vocabulary has no "unknown": `healthy` is both our `stable` and
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
    pub(super) fn to_status(self) -> Option<TimeStatus> {
        match self {
            WireLifetime::Healthy => None,
            WireLifetime::Eol => Some(TimeStatus::Eol),
            WireLifetime::Critical => Some(TimeStatus::Critical),
            WireLifetime::Expired => Some(TimeStatus::Expired),
        }
    }

    pub(super) fn from_status(status: Option<TimeStatus>) -> Self {
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
