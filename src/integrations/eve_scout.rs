//! EVE Scout's public feed: the wormholes out of Thera and Turnur, read on demand and kept
//! for a minute.
//!
//! What it says is turned into the map's own vocabulary here, once, so the card and the
//! command that maps a hub's holes read the same typed statuses.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::maps::eve_scout::{EveScoutHole, HUBS, size_from_eve_scout};
use crate::maps::{MassStatus, TimeStatus};

/// How long one read of the feed is served for. Scouts update it by hand, so a minute is
/// fresh, and it keeps a room full of open cards from each asking EVE Scout.
const CACHE_FOR: Duration = Duration::from_secs(60);

/// The whole request, connection included. The feed is a nice-to-have; a slow one should
/// cost the card a moment, not hold it open.
const TIMEOUT: Duration = Duration::from_secs(10);

/// A public wormhole out of Thera or Turnur, as EVE Scout's scouts have it. Oriented
/// hub-first rather than in EVE Scout's in/out terms, and statuses normalized to WormholeSystems's
/// own vocabulary.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EveScoutConnection {
    pub hub_solar_system_id: i64,
    /// `Thera` or `Turnur`.
    pub hub: String,
    pub hub_signature: String,
    pub solar_system_id: i64,
    pub signature: String,
    pub mass_status: MassStatus,
    pub time_status: TimeStatus,
    /// The wormhole code, e.g. `J377`. Absent while a scout has only half-scanned it.
    #[ts(optional)]
    pub wormhole_type: Option<String>,
    /// `frigate` / `medium` / `large` / `capital`.
    #[ts(optional)]
    pub max_ship_size: Option<String>,
    #[ts(optional)]
    pub remaining_hours: Option<f64>,
    /// When a scout last touched it, for the card's freshness stamp.
    #[ts(optional)]
    pub updated_at: Option<String>,
}

impl EveScoutConnection {
    /// The row in the vocabulary of the command that maps it.
    pub fn hole(&self) -> EveScoutHole {
        EveScoutHole {
            solar_system_id: self.solar_system_id,
            hub_signature: self.hub_signature.clone(),
            signature: self.signature.clone(),
            wormhole_type: self.wormhole_type.clone(),
            mass_status: self.mass_status,
            time_status: self.time_status,
            size: self.max_ship_size.as_deref().and_then(size_from_eve_scout),
        }
    }
}

/// The feed, with its last read. One per process, in the app state.
pub struct EveScout {
    url: String,
    cache: Mutex<Option<(Instant, Vec<EveScoutConnection>)>>,
}

impl EveScout {
    pub fn new(url: impl Into<String>) -> Self {
        EveScout {
            url: url.into(),
            cache: Mutex::new(None),
        }
    }

    /// Every public hole out of both hubs. A failed read is an empty list, cached like any
    /// other, so an outage at EVE Scout costs one attempt a minute rather than one per
    /// viewer.
    pub async fn connections(&self) -> Vec<EveScoutConnection> {
        if let Some((at, edges)) = self.cache.lock().expect("cache lock").as_ref()
            && at.elapsed() < CACHE_FOR
        {
            return edges.clone();
        }
        let edges = match self.fetch().await {
            Ok(edges) => edges,
            Err(err) => {
                tracing::warn!("eve scout: {err}");
                Vec::new()
            }
        };
        *self.cache.lock().expect("cache lock") = Some((Instant::now(), edges.clone()));
        edges
    }

    /// The holes out of one hub, ready for the command that maps them.
    pub async fn holes_from(&self, hub_solar_system_id: i64) -> Vec<EveScoutHole> {
        self.connections()
            .await
            .iter()
            .filter(|c| c.hub_solar_system_id == hub_solar_system_id)
            .map(EveScoutConnection::hole)
            .collect()
    }

    async fn fetch(&self) -> Result<Vec<EveScoutConnection>, String> {
        let body: serde_json::Value = crate::user_agent::client()
            .get(&self.url)
            .timeout(TIMEOUT)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?;
        let rows = body.as_array().ok_or("the feed is not a list")?;
        Ok(rows.iter().filter_map(parse).collect())
    }
}

/// Normalize one EVE Scout signature. Tolerant of shape drift: unknown fields default to
/// healthy/fresh, and a row without both endpoints, or with neither end at a hub, is
/// dropped rather than guessed at.
pub fn parse(sig: &serde_json::Value) -> Option<EveScoutConnection> {
    let in_id = sig.get("in_system_id")?.as_i64()?;
    let out_id = sig.get("out_system_id")?.as_i64()?;
    let text = |key: &str| {
        sig.get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.is_empty())
    };
    let (hub_solar_system_id, hub) = HUBS
        .iter()
        .find(|(id, _)| *id == in_id || *id == out_id)
        .map(|(id, name)| (*id, (*name).to_string()))?;
    let hub_is_in = hub_solar_system_id == in_id;
    let (hub_signature, signature) = if hub_is_in {
        (text("in_signature"), text("out_signature"))
    } else {
        (text("out_signature"), text("in_signature"))
    };
    let mass_status = match sig.get("mass").and_then(|v| v.as_str()).unwrap_or("") {
        m if m.contains("crit") => MassStatus::Critical,
        m if m.contains("reduced") || m.contains("destab") => MassStatus::Reduced,
        _ => MassStatus::Stable,
    };
    let time_status = match sig.get("life").and_then(|v| v.as_str()) {
        Some(l) if l.contains("expired") => TimeStatus::Expired,
        Some(l) if l.contains("crit") => TimeStatus::Critical,
        Some(l) if l.contains("eol") => TimeStatus::Eol,
        Some(_) => TimeStatus::Stable,
        // No life field: derive from the remaining lifetime when present.
        None => match sig.get("remaining_hours").and_then(|v| v.as_f64()) {
            Some(h) if h < 1.0 => TimeStatus::Critical,
            Some(h) if h < 4.0 => TimeStatus::Eol,
            _ => TimeStatus::Stable,
        },
    };
    Some(EveScoutConnection {
        hub_solar_system_id,
        hub,
        hub_signature: hub_signature.unwrap_or_default(),
        solar_system_id: if hub_is_in { out_id } else { in_id },
        signature: signature.unwrap_or_default(),
        mass_status,
        time_status,
        wormhole_type: text("wh_type"),
        max_ship_size: text("max_ship_size"),
        remaining_hours: sig.get("remaining_hours").and_then(|v| v.as_f64()),
        updated_at: text("updated_at").or_else(|| text("created_at")),
    })
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::maps::{MassStatus, TimeStatus};

    #[test]
    fn normalizes_statuses_and_drops_incomplete_rows() {
        let sig = serde_json::json!({
            "in_system_id": 30000142, "out_system_id": 31000005,
            "mass": "destab", "life": "eol"
        });
        let edge = parse(&sig).unwrap();
        assert_eq!(edge.mass_status, MassStatus::Reduced);
        assert_eq!(edge.time_status, TimeStatus::Eol);

        let sig = serde_json::json!({
            "in_system_id": 30000142, "out_system_id": 31000005,
            "remaining_hours": 0.5
        });
        let edge = parse(&sig).unwrap();
        assert_eq!(edge.mass_status, MassStatus::Stable);
        assert_eq!(edge.time_status, TimeStatus::Critical);

        let sig = serde_json::json!({
            "in_system_id": 30000142, "out_system_id": 31000005, "life": "expired"
        });
        assert_eq!(parse(&sig).unwrap().time_status, TimeStatus::Expired);

        let sig = serde_json::json!({ "in_system_id": 30000142 });
        assert!(parse(&sig).is_none());
    }

    /// The statuses go out as the same words they always did.
    #[test]
    fn statuses_keep_their_wire_words() {
        let edge = parse(&serde_json::json!({
            "in_system_id": 30000142, "out_system_id": 31000005,
            "mass": "critical", "life": "eol"
        }))
        .unwrap();
        let json = serde_json::to_value(&edge).unwrap();
        assert_eq!(json["mass_status"], "critical");
        assert_eq!(json["time_status"], "eol");
    }

    /// Whichever end is the hub, the row comes back hub-first.
    #[test]
    fn orients_the_hub_first_either_way() {
        let a = parse(&serde_json::json!({
            "in_system_id": 31001882, "out_system_id": 30002086,
            "in_signature": "BCC-784", "out_signature": "DZT-829", "wh_type": "J377"
        }))
        .unwrap();
        assert_eq!(a.hub, "Turnur");
        assert_eq!(a.hub_solar_system_id, 30002086);
        assert_eq!(a.solar_system_id, 31001882);
        assert_eq!(a.hub_signature, "DZT-829");
        assert_eq!(a.signature, "BCC-784");
        assert_eq!(a.wormhole_type.as_deref(), Some("J377"));

        let b = parse(&serde_json::json!({
            "in_system_id": 31000005, "out_system_id": 30000142,
            "in_signature": "AAA-111", "out_signature": "BBB-222"
        }))
        .unwrap();
        assert_eq!(b.hub, "Thera");
        assert_eq!(b.solar_system_id, 30000142);
        assert_eq!(b.hub_signature, "AAA-111");
        assert_eq!(b.signature, "BBB-222");

        // Neither end is a hub: EVE Scout only publishes Thera and Turnur, so this is drift.
        assert!(
            parse(&serde_json::json!({
                "in_system_id": 30000142, "out_system_id": 30002187
            }))
            .is_none()
        );
    }
}
