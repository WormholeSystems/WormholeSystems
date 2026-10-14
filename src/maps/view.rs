//! The map read model: everything a node displays, assembled by the map read and the
//! system pickers from joins across the SDE, intel and sovereignty tables.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::error::Result;

/// A static wormhole a system always has, plus the class it leads to (`dest_class` is the
/// `wormhole_class_id` encoding; `None` for the few codes with no fixed destination) and
/// the hole physics for the static tooltip.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Static {
    pub code: String,
    pub dest_class: Option<i32>,
    /// Total mass the hole can pass before collapsing, kg.
    pub total_mass: Option<i64>,
    /// Max mass of a single ship per jump, kg.
    pub max_jump_mass: Option<i64>,
    pub lifetime_hours: Option<f64>,
    /// Scan signature strength in percent (higher = easier to scan).
    pub signature_strength: Option<f64>,
}

/// The statics of every wormhole among `ids`, grouped by system. A separate query rather
/// than a join wherever systems are read: statics are one-to-many, so joining would
/// multiply the system rows.
pub async fn statics_for(
    exec: impl sqlx::PgExecutor<'_>,
    ids: &[i64],
) -> sqlx::Result<std::collections::HashMap<i64, Vec<Static>>> {
    let mut out: std::collections::HashMap<i64, Vec<Static>> = std::collections::HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let rows = sqlx::query!(
        "select wss.solar_system_id, wt.code, wt.dest_class,
                wt.total_mass, wt.max_mass_per_jump, wt.lifetime_hours, wt.signature_strength
         from wormhole_system_statics wss
         join wormhole_types wt on wt.code = wss.wormhole_code
         where wss.solar_system_id = any($1)
         order by wt.dest_class nulls last, wt.code",
        ids,
    )
    .fetch_all(exec)
    .await?;
    for row in rows {
        out.entry(row.solar_system_id).or_default().push(Static {
            code: row.code,
            dest_class: row.dest_class,
            total_mass: row.total_mass,
            max_jump_mass: row.max_mass_per_jump,
            lifetime_hours: row.lifetime_hours,
            signature_strength: row.signature_strength,
        });
    }
    Ok(out)
}

/// One buff/debuff a wormhole effect applies, for the node's effect popover. `kind` is the
/// effect strength tier; `stat` is what it modifies; `value` is the (already-formatted) amount.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EffectModifier {
    pub kind: String,
    pub stat: String,
    pub value: String,
}

/// The modifiers a wormhole effect applies at a given class. Reference data, no auth needed.
/// The modifier table is keyed by class 1..6; special classes map to the strength tier the
/// game uses for them (C13 has C6-strength effects, drifter systems C14-18 have C2 strength).
pub async fn effect_modifiers(
    pool: &PgPool,
    effect_name: &str,
    wormhole_class_id: i32,
) -> Result<Vec<EffectModifier>> {
    let effective_class = match wormhole_class_id {
        13 => 6,
        14..=18 => 2,
        c => c,
    };
    let rows = sqlx::query_as!(
        EffectModifier,
        "select kind, stat, value from wormhole_effect_modifiers
         where effect_name = $1 and wormhole_class_id = $2
         order by stat, kind",
        effect_name,
        effective_class,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Who holds sovereignty in a system. The variant *is* the holder kind, so the node knows
/// which EVE image endpoint to use for the icon; only alliances/corps carry a ticker.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Sovereignty {
    Alliance {
        id: i64,
        name: String,
        ticker: String,
    },
    Corporation {
        id: i64,
        name: String,
        ticker: String,
    },
    Faction {
        id: i64,
        name: String,
    },
}

/// Build the holder from the columns every sovereignty query selects. Lives here rather
/// than beside one of those queries because four of them read the same four columns.
pub fn sovereignty_of(
    kind: Option<&str>,
    id: Option<i64>,
    name: Option<String>,
    ticker: Option<String>,
) -> Option<Sovereignty> {
    match (kind, id, name) {
        (Some("alliance"), Some(id), Some(name)) => Some(Sovereignty::Alliance {
            id,
            name,
            ticker: ticker.unwrap_or_default(),
        }),
        (Some("corporation"), Some(id), Some(name)) => Some(Sovereignty::Corporation {
            id,
            name,
            ticker: ticker.unwrap_or_default(),
        }),
        (Some("faction"), Some(id), Some(name)) => Some(Sovereignty::Faction { id, name }),
        _ => None,
    }
}

/// A placed system enriched with everything a map node displays. Read-only, built by
/// `get_map` from joins across the SDE + intel + sovereignty tables. Mutations use the lean
/// [`MapSolarSystem`].
///
/// Two shapes, because a node is either a system somebody placed or a hole somebody
/// scanned: a ghost has no security, statics, or intel, and saying that in the type makes
/// the check impossible to skip.
// The two variants are deliberately lopsided: a ghost is a node and nothing else. Boxing
// the larger one would put the wire type behind an indirection to save bytes on a value
// that is built once per node and serialised immediately.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MapSystemView {
    System {
        // Placement (map_solar_systems).
        id: i64,
        map_id: i64,
        solar_system_id: i64,
        position_x: f64,
        position_y: f64,
        alias: Option<String>,
        is_home: bool,
        is_rally: bool,
        is_pinned: bool,
        /// When a scan was last pasted into this system; `None` if nobody has yet. Hand
        /// edits leave it alone.
        signatures_pasted_at: Option<DateTime<Utc>>,
        // Intel (map_solar_system_details; defaults when no row exists yet).
        status: super::SystemStatus,
        occupying_group: Option<String>,
        // Reference (solar_systems / regions / constellations).
        name: String,
        security_status: f64,
        /// Absent on the systems the SDE gives no class, which is not the same as a ghost
        /// having none.
        wormhole_class_id: Option<i32>,
        region: String,
        region_id: i64,
        constellation_id: i64,
        constellation: String,
        // Wormhole reference (wormhole_systems / statics).
        effect_name: Option<String>,
        is_shattered: bool,
        /// Kill-activity threat (wormhole systems only; `None` for k-space).
        threat_level: Option<super::ThreatLevel>,
        statics: Vec<Static>,
        // Sovereignty (system_sovereignty → alliance/corp/faction).
        sovereignty: Option<Sovereignty>,
    },
    /// The far side of a scanned hole. It draws, moves and is named like any other node,
    /// which is all it has: the rest is looked up from a system it is not yet.
    Ghost {
        id: i64,
        map_id: i64,
        position_x: f64,
        position_y: f64,
        alias: Option<String>,
        is_home: bool,
        is_rally: bool,
        is_pinned: bool,
        status: super::SystemStatus,
    },
}

impl MapSystemView {
    pub fn id(&self) -> i64 {
        match self {
            MapSystemView::System { id, .. } | MapSystemView::Ghost { id, .. } => *id,
        }
    }

    /// `None` while the node is still a hole.
    pub fn solar_system_id(&self) -> Option<i64> {
        match self {
            MapSystemView::System {
                solar_system_id, ..
            } => Some(*solar_system_id),
            MapSystemView::Ghost { .. } => None,
        }
    }

    pub fn alias(&self) -> Option<&str> {
        match self {
            MapSystemView::System { alias, .. } | MapSystemView::Ghost { alias, .. } => {
                alias.as_deref()
            }
        }
    }

    pub fn position(&self) -> (f64, f64) {
        match self {
            MapSystemView::System {
                position_x,
                position_y,
                ..
            }
            | MapSystemView::Ghost {
                position_x,
                position_y,
                ..
            } => (*position_x, *position_y),
        }
    }

    /// `None` on a hole nobody has been through.
    pub fn name(&self) -> Option<&str> {
        match self {
            MapSystemView::System { name, .. } => Some(name),
            MapSystemView::Ghost { .. } => None,
        }
    }
}
