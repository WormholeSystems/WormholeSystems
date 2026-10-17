//! Map business logic: the authorized, validated actions a user takes on a map.
//!
//! The application layer over the [database spec](../../docs/database/), built to the
//! behaviour spec in [`docs/features/maps.md`](../../docs/features/maps.md). Plain async
//! functions over a `PgPool`, so the tests drive them directly.

pub mod access;
pub mod alerts;
pub mod background;
pub mod command;
pub mod connection;
pub mod error;
pub mod eve_scout;
pub mod ghost;
pub mod history;
pub mod ignored;
pub mod jumps;
pub mod map;
pub mod personal_watchlist;
pub mod presence;
pub mod restore;
pub mod search;
pub mod signatures;
pub mod solar_system;
mod store;
pub mod tracking;
pub mod transfer;
pub mod user_settings;
pub mod view;
pub mod watchlist;

pub use command::{Command, CommandOutput, EventActor, MapCommand, execute, run};

pub use background::{BackgroundMode, BackgroundStore};
pub use connection::MapConnection;
pub use error::{MapError, Result};
pub use map::Map;
pub use signatures::Signature;
pub use solar_system::MapSolarSystem;
pub use view::{EffectModifier, MapSystemView, Sovereignty, Static};

/// Deserializer for `Option<Option<T>>` "absent = leave, null = clear" fields: a present
/// field (including `null`) becomes `Some(inner)`; pair with `#[serde(default)]` so an
/// absent field stays `None`. Without this, serde collapses JSON `null` to the *outer*
/// `None` and a clear silently becomes a no-op.
pub fn double_option<'de, T, D>(de: D) -> std::result::Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    serde::Deserialize::deserialize(de).map(Some)
}

/// A Rust enum stored as `text` (per the schema's [enum convention](../../docs/database/README.md)).
/// Generates the variants, `as_str` / `from_db`, and (for the `as "pg_type"` form) the sqlx
/// glue so the enum binds and decodes directly in queries. Columns typed plain `text` use
/// the form without a pg type and go through `as_str` / `from_db` by hand. Variant order is
/// the `Ord` order: used for `Role`.
macro_rules! text_enum {
    ($(#[$m:meta])* $vis:vis enum $name:ident as $pg:literal { $($(#[$vm:meta])* $variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$m])*
        // snake_case rather than lowercase: it agrees with the string each variant maps to,
        // which lowercase does not once a variant is two words (`LessSecure`). serde and
        // sqlx are told the same thing, so the wire and the database never disagree.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema, sqlx::Type)]
        #[serde(rename_all = "snake_case")]
        #[sqlx(type_name = $pg, rename_all = "snake_case")]
        #[ts(export)]
        $vis enum $name { $($(#[$vm])* $variant),+ }

        crate::maps::text_enum!(@impl $name { $($variant => $s),+ });
    };
    ($(#[$m:meta])* $vis:vis enum $name:ident { $($(#[$vm:meta])* $variant:ident => $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
        #[serde(rename_all = "snake_case")]
        #[ts(export)]
        $vis enum $name { $($(#[$vm])* $variant),+ }

        crate::maps::text_enum!(@impl $name { $($variant => $s),+ });
    };
    (@impl $name:ident { $($variant:ident => $s:literal),+ }) => {
        impl $name {
            /// The label, for the places that are writing it out rather than binding it.
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $s),+ }
            }
            pub fn from_db(s: &str) -> Option<Self> {
                match s { $($s => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

pub(crate) use text_enum;

text_enum! {
    /// A map access role, ordered `Viewer < Member < Manager < Owner`.
    pub enum Role as "map_role" {
        Viewer => "viewer",
        Member => "member",
        Manager => "manager",
        Owner => "owner",
    }
}

text_enum! {
    /// What an access grant targets. EVE ids are globally unique across the three.
    pub enum SubjectType as "subject_type" {
        Character => "character",
        Corporation => "corporation",
        Alliance => "alliance",
    }
}

text_enum! {
    /// The kind of edge between two placed systems.
    pub enum ConnectionType as "connection_type" {
        Wormhole => "wormhole",
        Stargate => "stargate",
    }
}

text_enum! {
    /// How a chain names itself: the sequence an alias suggestion walks.
    pub enum AliasScheme as "alias_scheme" {
        Numeric => "numeric",
        Alphabetical => "alphabetical",
    }
}

text_enum! {
    /// Where the nodes get their positions. `Manual` keeps whatever they were dragged to;
    /// `Tree` derives them, and is the map's choice unless it hands it to each viewer.
    pub enum MapLayout as "map_layout" {
        Manual => "manual",
        Tree => "tree",
    }
}

text_enum! {
    /// What a route is optimised for.
    pub enum RoutePreference as "route_preference" {
        Shorter => "shorter",
        Safer => "safer",
        LessSecure => "less_secure",
    }
}

text_enum! {
    /// Which part of the chain the killmails card shows.
    pub enum KillmailScope as "killmail_scope" {
        All => "all",
        Jspace => "jspace",
        Kspace => "kspace",
        /// Only the system selected on the map, which the card sends with each request.
        SelectedSystem => "selected_system",
    }
}

text_enum! {
    /// A wormhole's remaining mass, worst last. Variant order is the severity order the
    /// [reconcile-on-link merge](../../docs/database/mapping.md) relies on: `max` is the
    /// worst (= "massed"). Kept in lock-step across a connection and its signatures by the
    /// `map_*_sync` DB triggers (migration 0009).
    pub enum MassStatus as "mass_status" {
        Stable => "stable",
        Reduced => "reduced",
        Critical => "critical",
    }
}

text_enum! {
    /// A wormhole's remaining lifetime, worst last. `Eol` ≈ "<4h"; `Critical` ≈ "<1h"
    /// (super-EOL); `Expired` is the in-game "Expired, closure imminent", past any reliable
    /// lifetime. Same severity-ordering / merge semantics as [`MassStatus`].
    pub enum TimeStatus as "time_status" {
        Stable => "stable",
        Eol => "eol",
        Critical => "critical",
        Expired => "expired",
    }
}

text_enum! {
    /// Max ship-mass class that can transit a wormhole. Ordered most-permissive →
    /// most-restrictive, so `max` (= `Small`) is the "weakest"/worst: the conservative
    /// pick when two ends disagree (they shouldn't: both ends of a hole share a size).
    pub enum WormholeSize as "wormhole_size" {
        Xl => "xl",
        Large => "large",
        Medium => "medium",
        Small => "small",
    }
}

text_enum! {
    /// A cosmic-signature group, mirroring [`signature_categories`](../../docs/database/static.md).
    /// Only `Wormhole` signatures carry connection links and the wormhole life-cycle state.
    pub enum SignatureGroup as "signature_group" {
        Wormhole => "wormhole",
        Data => "data",
        Relic => "relic",
        Gas => "gas",
        Combat => "combat",
        Ore => "ore",
        Homefront => "homefront",
        FactionWarfare => "faction_warfare",
        Unknown => "unknown",
    }
}

impl Default for SignatureGroup {
    /// An unresolved signature is `Unknown` until its group is scanned.
    fn default() -> Self {
        Self::Unknown
    }
}

text_enum! {
    /// A placed system's intel status (`map_solar_system_details.status`), set by users.
    /// Matches the legacy vocabulary: `active` = recent activity seen, `empty` = scanned
    /// and found empty.
    pub enum SystemStatus as "system_status" {
        Unknown => "unknown",
        Friendly => "friendly",
        Hostile => "hostile",
        Active => "active",
        Unscanned => "unscanned",
        Empty => "empty",
    }
}

impl Default for SystemStatus {
    /// A freshly placed system is `Unknown` until someone classifies it.
    fn default() -> Self {
        Self::Unknown
    }
}

text_enum! {
    /// A wormhole system's kill-activity threat level, from the daily analysis.
    pub enum ThreatLevel as "threat_level" {
        Unknown => "unknown",
        High => "high",
        Critical => "critical",
    }
}

#[cfg(test)]
mod tests {
    use super::Role;

    /// `Role` is compared with `>=` all over the access layer, and the frontend keeps its own
    /// copy of this order in `lib/map/roles.ts`. Reordering the variants would quietly change
    /// who can manage a map, and both sides would still compile.
    #[test]
    fn roles_run_least_to_most_privileged() {
        assert!(Role::Viewer < Role::Member);
        assert!(Role::Member < Role::Manager);
        assert!(Role::Manager < Role::Owner);
    }
}

/// A user acting as one of their characters. `user_id` drives authorization (effective
/// role across all their characters); `character_id` attributes ownership on creation.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Actor {
    pub user_id: i64,
    pub character_id: i64,
}

/// The graph as seen by a viewer: the map plus its placed systems and connections.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapView {
    pub map: Map,
    /// The caller's effective role on this map, for client-side permission gating.
    pub role: Role,
    /// Whether the *active* character is itself covered by a grant. Access is per-user
    /// (any of their characters), so this can be false while `role` is set: the map is
    /// readable through a different character, but anything tied to the active one
    /// (tracking, waypoints) will not behave as the pilot expects.
    pub character_has_access: bool,
    pub systems: Vec<MapSystemView>,
    pub connections: Vec<MapConnection>,
}

/// Map canvas geometry. Server-owned (built from env in `crate::config`), fetched by the
/// client so layout has a single source of truth. Node height is `2 * cell_size`;
/// dimensions are world-space px.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct GridConfig {
    pub cell_size: f64,
    pub world_width: f64,
    pub world_height: f64,
    pub viewport_height: f64,
}

/// The canvas geometry for this process, installed at startup. Global so commands can
/// place nodes without threading it through. Unset in tests, which get the defaults the
/// client also falls back to.
static GRID: std::sync::OnceLock<GridConfig> = std::sync::OnceLock::new();

pub fn set_grid(grid: GridConfig) {
    let _ = GRID.set(grid);
}

pub fn grid() -> GridConfig {
    *GRID.get_or_init(GridConfig::default)
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            // Match the legacy map: 20-unit grid cells, so the node (2 cells) is 40 tall.
            cell_size: 20.0,
            world_width: 4000.0,
            world_height: 2000.0,
            viewport_height: 1400.0,
        }
    }
}
