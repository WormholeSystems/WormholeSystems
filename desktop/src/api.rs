//! The few calls the app makes, with a personal access token. Blocking: the app runs them on
//! GPUI's background executor, and a handful of requests per jump needs nothing more.

use std::time::Duration;

use anyhow::{Context as _, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::scan::PastedSignature;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MassStatus {
    Stable,
    Reduced,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeStatus {
    Stable,
    Eol,
    Critical,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WormholeSize {
    Xl,
    Large,
    Medium,
    Small,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionKind {
    Wormhole,
    Stargate,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapEntry {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub is_archived: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapNaming {
    /// `numeric` or `alphabetical`.
    #[serde(default)]
    pub alias_scheme: Option<String>,
    pub ignored_alias: String,
    pub bookmark_wormhole: String,
    pub bookmark_kspace: String,
    pub bookmark_return: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapInfo {
    pub name: String,
    pub naming: MapNaming,
}

/// A placed system. Ghosts (holes nobody has flown yet) are kept apart as [`Ghost`]s:
/// nothing can be jumped into one, but a jump can say which system it was.
#[derive(Debug, Clone, Deserialize)]
pub struct PlacedSystem {
    pub id: i64,
    pub solar_system_id: i64,
    pub name: String,
    pub alias: Option<String>,
    pub region: String,
    pub wormhole_class_id: Option<i64>,
    pub security_status: f64,
    pub occupying_group: Option<String>,
    #[serde(default)]
    pub is_home: bool,
    #[serde(default)]
    pub position_x: f64,
    #[serde(default)]
    pub position_y: f64,
    #[serde(default)]
    pub statics: Vec<Static>,
}

/// A wormhole a system always has.
#[derive(Debug, Clone, Deserialize)]
pub struct Static {
    pub code: String,
}

/// A hole drawn on the map that nobody has been through, so its system is unknown.
#[derive(Debug, Clone, Deserialize)]
pub struct Ghost {
    pub id: i64,
    pub alias: Option<String>,
    #[serde(default)]
    pub position_x: f64,
    #[serde(default)]
    pub position_y: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connection {
    pub id: i64,
    pub from_system: i64,
    pub to_system: i64,
    pub kind: ConnectionKind,
    pub size: Option<WormholeSize>,
    pub mass_status: Option<MassStatus>,
    pub time_status: Option<TimeStatus>,
    /// Rolling it is off: the hole is kept for the way home.
    #[serde(default)]
    pub preserve_mass: bool,
    #[serde(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Signature {
    /// The row's key, which edits and deletes name.
    #[serde(default)]
    pub id: i64,
    /// The scanner id, `ABC-123`.
    pub signature_id: String,
    pub solar_system_id: i64,
    pub signature_type_id: Option<i64>,
    pub connection_id: Option<i64>,
    /// `wormhole`, `data`, …, `unknown` for one nobody has categorised.
    #[serde(default)]
    pub group: String,
    /// The scanner's type name when no catalog type matched.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub size: Option<WormholeSize>,
    #[serde(default)]
    pub mass_status: Option<MassStatus>,
    #[serde(default)]
    pub time_status: Option<TimeStatus>,
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignatureCategory {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignatureType {
    pub id: i64,
    /// The wormhole code, `H296`; `None` for anything that is not a wormhole.
    pub signature: Option<String>,
    pub name: String,
    pub signature_category_id: i64,
    /// Where a wormhole type leads, as a class id.
    pub target_class: Option<i64>,
    /// The heaviest ship it lets through, in kg; wormhole types only.
    #[serde(default)]
    pub max_jump_mass: Option<f64>,
    /// A note telling two types with the same code apart, shown after it.
    #[serde(default)]
    pub extra: Option<String>,
    /// The system classes it can appear in.
    #[serde(default)]
    pub spawn_areas: Vec<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SignatureCatalog {
    pub categories: Vec<SignatureCategory>,
    pub types: Vec<SignatureType>,
}

/// Everything a bookmark needs from one map, fetched together after a jump.
#[derive(Debug, Clone)]
pub struct MapSnapshot {
    pub map: MapInfo,
    pub systems: Vec<PlacedSystem>,
    pub ghosts: Vec<Ghost>,
    pub connections: Vec<Connection>,
    pub signatures: Vec<Signature>,
}

impl MapSnapshot {
    /// The placed system called `name`. The game writes names the way the SDE has them, but
    /// compare without case anyway.
    pub fn system_named(&self, name: &str) -> Option<&PlacedSystem> {
        self.systems
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// The wormhole between two placed systems, in either direction.
    pub fn connection_between(&self, a: i64, b: i64) -> Option<&Connection> {
        self.connections.iter().find(|c| {
            c.kind == ConnectionKind::Wormhole
                && ((c.from_system == a && c.to_system == b)
                    || (c.from_system == b && c.to_system == a))
        })
    }

    pub fn connection(&self, id: i64) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// The signature a connection was found as, on whichever side.
    pub fn signature_on(&self, connection_id: i64) -> Option<&Signature> {
        self.signatures
            .iter()
            .find(|s| s.connection_id == Some(connection_id))
    }
}

/// What a hotkey changes on a connection. Fields left `None` stay as they are.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass_status: Option<MassStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_status: Option<TimeStatus>,
}

/// A system looked up by name, for one the map does not have yet.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemHit {
    pub id: i64,
    pub name: String,
    pub security: f64,
    pub wormhole_class_id: Option<i64>,
}

/// The per-user map settings the app follows.
#[derive(Debug, Clone, Deserialize)]
pub struct UserSettings {
    #[serde(default)]
    pub suggest_alias: bool,
    /// List a system's statics first when picking a wormhole's type.
    #[serde(default)]
    pub show_statics_first: bool,
}

/// One change to a signature, as the website's row makes it.
#[derive(Debug, Clone)]
pub enum SignatureEdit {
    Group(String),
    /// `None` for not known.
    Type(Option<i64>),
    Link(i64),
    Unlink,
    /// `None` for not known, the website's "fresh".
    Mass(Option<MassStatus>),
    /// `None` for healthy.
    Time(Option<TimeStatus>),
    /// On the signature's connection, not the signature.
    PreserveMass {
        connection: i64,
        on: bool,
    },
}

/// Record a jump: place the system, connect it and link the signature, as one undo.
#[derive(Debug, Clone, Serialize)]
pub struct TrackJump {
    pub map_id: i64,
    pub from_map_solar_system_id: i64,
    pub to_solar_system_id: i64,
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_pk: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<WormholeSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass_status: Option<MassStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_status: Option<TimeStatus>,
}

/// Say which system a ghost turned out to be, with what the jump taught about the hole.
#[derive(Debug, Clone, Serialize)]
pub struct ResolveGhost {
    pub map_id: i64,
    pub map_solar_system_id: i64,
    pub solar_system_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<WormholeSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mass_status: Option<MassStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_status: Option<TimeStatus>,
}

#[derive(Deserialize)]
struct RoutingGraph {
    adjacency: std::collections::HashMap<String, Vec<i64>>,
}

#[derive(Deserialize)]
struct IgnoredSystem {
    solar_system_id: i64,
}

#[derive(Deserialize)]
struct MapView {
    map: MapInfo,
    systems: Vec<serde_json::Value>,
    connections: Vec<Connection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Me {
    pub name: String,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    base: String,
    token: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base: &str, token: &str) -> Client {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .http_status_as_error(false)
            // The system's roots, so a local or company CA it trusts works here too.
            .tls_config(
                ureq::tls::TlsConfig::builder()
                    .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                    .build(),
            )
            .user_agent(concat!(
                "wormholesystems-desktop/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .new_agent();
        Client {
            base: base.trim().trim_end_matches('/').to_string(),
            token: token.trim().to_string(),
            agent,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn read<T: serde::de::DeserializeOwned>(
        path: &str,
        mut response: ureq::http::Response<ureq::Body>,
    ) -> Result<T> {
        let status = response.status();
        if status.is_success() {
            return response
                .body_mut()
                .read_json::<T>()
                .with_context(|| format!("could not read the reply to {path}"));
        }
        let body = response.body_mut().read_json::<ApiErrorBody>().ok();
        let reason = body.and_then(|b| b.message.or(b.error));
        Err(match (status.as_u16(), reason) {
            (401, _) => anyhow!("the server did not accept the token"),
            (403, _) => anyhow!("this token has no access to that map"),
            (_, Some(reason)) => anyhow!("{path}: {reason}"),
            (code, None) => anyhow!("{path}: HTTP {code}"),
        })
    }

    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .agent
            .get(&self.url(path))
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .with_context(|| format!("could not reach {}", self.base))?;
        Self::read(path, response)
    }

    fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let response = self
            .agent
            .post(&self.url(path))
            .header("Authorization", &format!("Bearer {}", self.token))
            .send_json(body)
            .with_context(|| format!("could not reach {}", self.base))?;
        Self::read(path, response)
    }

    /// Who the token belongs to. `/api/me` answers `null` for nobody, which a token never is.
    pub fn me(&self) -> Result<Me> {
        self.get::<Option<Me>>("/api/me")?
            .ok_or_else(|| anyhow!("the server did not accept the token"))
    }

    pub fn maps(&self) -> Result<Vec<MapEntry>> {
        let mut maps: Vec<MapEntry> = self.get("/api/maps")?;
        maps.retain(|m| !m.is_archived);
        Ok(maps)
    }

    pub fn catalog(&self) -> Result<SignatureCatalog> {
        self.get("/api/signature-types")
    }

    pub fn snapshot(&self, map_id: i64) -> Result<MapSnapshot> {
        let view: MapView = self.get(&format!("/api/maps/{map_id}"))?;
        // Ghosts (holes nobody has been through) have no solar system to scan or jump to.
        let mut systems = Vec::new();
        let mut ghosts = Vec::new();
        for s in view.systems {
            match s.get("kind").and_then(|k| k.as_str()) {
                Some("system") => systems.extend(serde_json::from_value(s).ok()),
                Some("ghost") => ghosts.extend(serde_json::from_value(s).ok()),
                _ => {}
            }
        }
        let signatures = self.get(&format!("/api/maps/{map_id}/signatures"))?;
        Ok(MapSnapshot {
            map: view.map,
            systems,
            ghosts,
            connections: view.connections,
            signatures,
        })
    }

    /// Upsert a scan into a system's signatures. Never deletes: what the scan no longer sees
    /// is left for [`Client::remove_signatures`].
    pub fn paste(
        &self,
        map_id: i64,
        solar_system_id: i64,
        signatures: &[PastedSignature],
    ) -> Result<()> {
        self.post::<serde_json::Value>(
            &format!("/api/maps/{map_id}/signatures/paste"),
            &serde_json::json!({
                "map_id": map_id,
                "solar_system_id": solar_system_id,
                "signatures": signatures,
            }),
        )?;
        Ok(())
    }

    /// Delete signatures by row key, with the website's cascade: a hole left without a
    /// signature on that side goes, and so does a far system left with nothing.
    pub fn remove_signatures(&self, map_id: i64, keys: &[i64]) -> Result<()> {
        self.post::<serde_json::Value>(
            &format!("/api/maps/{map_id}/signatures/remove-bulk"),
            &serde_json::json!({ "map_id": map_id, "signature_pks": keys }),
        )?;
        Ok(())
    }

    pub fn grid(&self) -> Result<crate::jump::Grid> {
        self.get("/api/grid-config")
    }

    /// Which systems each system has a stargate to. A jump along one is travel, never a
    /// new hole.
    pub fn stargates(&self) -> Result<std::collections::HashMap<i64, Vec<i64>>> {
        let graph: RoutingGraph = self.get("/api/routing-graph")?;
        Ok(graph
            .adjacency
            .into_iter()
            .filter_map(|(k, v)| Some((k.parse().ok()?, v)))
            .collect())
    }

    pub fn user_settings(&self, map_id: i64) -> Result<UserSettings> {
        self.get(&format!("/api/maps/{map_id}/settings/user"))
    }

    /// The systems the map keeps off: a jump into one is flown, not mapped.
    pub fn ignored(&self, map_id: i64) -> Result<Vec<i64>> {
        let list: Vec<IgnoredSystem> = self.get(&format!("/api/maps/{map_id}/ignored"))?;
        Ok(list.into_iter().map(|i| i.solar_system_id).collect())
    }

    /// The system called exactly `name`, ignoring case.
    pub fn find_system(&self, name: &str) -> Result<Option<SystemHit>> {
        let hits: Vec<SystemHit> = self
            .agent
            .get(&self.url("/api/systems/search"))
            .query("q", name)
            .header("Authorization", &format!("Bearer {}", self.token))
            .call()
            .with_context(|| format!("could not reach {}", self.base))
            .and_then(|r| Self::read("/api/systems/search", r))?;
        Ok(hits.into_iter().find(|h| h.name.eq_ignore_ascii_case(name)))
    }

    pub fn track_jump(&self, cmd: &TrackJump) -> Result<()> {
        self.post::<serde_json::Value>(
            &format!("/api/maps/{}/track-jump", cmd.map_id),
            &serde_json::to_value(cmd)?,
        )?;
        Ok(())
    }

    pub fn resolve_ghost(&self, cmd: &ResolveGhost) -> Result<()> {
        self.post::<serde_json::Value>(
            &format!("/api/maps/{}/systems/resolve-ghost", cmd.map_id),
            &serde_json::to_value(cmd)?,
        )?;
        Ok(())
    }

    /// Set a placement's alias; blank clears it.
    pub fn set_alias(&self, map_id: i64, map_solar_system_id: i64, alias: &str) -> Result<()> {
        let alias = alias.trim();
        self.post::<serde_json::Value>(
            &format!("/api/maps/{map_id}/systems/set-alias"),
            &serde_json::json!({
                "map_id": map_id,
                "map_solar_system_id": map_solar_system_id,
                "alias": (!alias.is_empty()).then_some(alias),
            }),
        )?;
        Ok(())
    }

    pub fn edit_signature(&self, map_id: i64, key: i64, edit: &SignatureEdit) -> Result<()> {
        let mut body = serde_json::json!({ "map_id": map_id, "signature_pk": key });
        let path = match edit {
            SignatureEdit::Group(group) => {
                body["group"] = group.as_str().into();
                "update"
            }
            SignatureEdit::Type(id) => {
                body["signature_type_id"] = serde_json::to_value(id)?;
                "update"
            }
            SignatureEdit::Link(connection) => {
                body["connection_id"] = (*connection).into();
                "link"
            }
            SignatureEdit::Unlink => "unlink",
            SignatureEdit::Mass(mass) => {
                body["mass_status"] = serde_json::to_value(mass)?;
                "update"
            }
            SignatureEdit::Time(time) => {
                body["time_status"] = serde_json::to_value(time)?;
                "update"
            }
            SignatureEdit::PreserveMass { connection, on } => {
                self.post::<serde_json::Value>(
                    &format!("/api/maps/{map_id}/connections/set-status"),
                    &serde_json::json!({
                        "map_id": map_id,
                        "connection_id": connection,
                        "preserve_mass": on,
                    }),
                )?;
                return Ok(());
            }
        };
        self.post::<serde_json::Value>(&format!("/api/maps/{map_id}/signatures/{path}"), &body)?;
        Ok(())
    }

    pub fn set_status(&self, map_id: i64, connection_id: i64, change: &StatusChange) -> Result<()> {
        let mut body = serde_json::to_value(change)?;
        body["map_id"] = map_id.into();
        body["connection_id"] = connection_id.into();
        self.post::<serde_json::Value>(
            &format!("/api/maps/{map_id}/connections/set-status"),
            &body,
        )?;
        Ok(())
    }
}
