//! Reference reads: the fixed universe and the caches over it. None of it belongs to a
//! map, so none of it is authorized against one; a few need a session, no more.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::require_actor;
use super::{ApiError, ApiResult};
use crate::app::AppState;
use crate::maps::view::{SystemSearchResult, search_systems_named, systems_for};
use crate::maps::{EffectModifier, GridConfig};

/// One organisation in a system's threat top list.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ThreatEntity {
    pub id: i64,
    pub name: String,
    pub entity_type: String,
    pub kills: i32,
}

/// A wormhole system's threat analysis, for the threat card.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ThreatAnalysis {
    pub threat_level: crate::maps::ThreatLevel,
    pub threat_analyzed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub entities: Vec<ThreatEntity>,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(instance))
        .routes(routes!(grid_config))
        .routes(routes!(effect_modifiers))
        .routes(routes!(search_systems))
        .routes(routes!(resolve_systems))
        .routes(routes!(routing_graph))
        .routes(routes!(threat_analysis))
        .routes(routes!(server_status))
        .routes(routes!(skyhooks))
        .routes(routes!(reference_counts))
}

/// How much of New Eden this install has loaded. The landing page states these, so they
/// are the real contents of this database rather than numbers written into a page.
#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ReferenceCounts {
    pub solar_systems: i64,
    pub wormhole_systems: i64,
    pub stargates: i64,
    pub wormhole_types: i64,
}

/// `GET /api/reference-counts`: how much static data is seeded. Changes only when a new
/// SDE build is loaded.
#[utoipa::path(
    get,
    path = "/api/reference-counts",
    tag = "reference",
    responses((status = 200, body = ReferenceCounts, description = "OK")),
    security(()),
)]
pub async fn reference_counts(State(state): State<AppState>) -> ApiResult<ReferenceCounts> {
    let row = sqlx::query!(
        r#"select
             (select count(*) from solar_systems) as "solar_systems!",
             (select count(*) from wormhole_systems) as "wormhole_systems!",
             (select count(*) from stargates) as "stargates!",
             (select count(*) from wormhole_types) as "wormhole_types!""#
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(ReferenceCounts {
        solar_systems: row.solar_systems,
        wormhole_systems: row.wormhole_systems,
        stargates: row.stargates,
        wormhole_types: row.wormhole_types,
    }))
}

/// What this deployment can actually do, so the interface can say what is switched off
/// rather than offering something that will quietly never arrive. Self-hosters configure
/// Discord separately from the rest, and most of them do not configure it at all.
#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Instance {
    pub discord: DiscordCapability,
    /// The operator's site-wide banner, or `null`. Always the configured one: whether this
    /// visitor already dismissed it is a cookie the frontend reads while rendering.
    pub announcement: Option<crate::announcement::Announcement>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct DiscordCapability {
    /// Whether an account can be linked at all: the OAuth half of the application.
    pub linking: bool,
    /// Whether the bot can post to a channel or DM. Needs a bot token on top of the rest;
    /// a plain webhook destination works without it.
    pub bot: bool,
}

/// `GET /api/instance`: what this deployment has switched on, and what its operator wants
/// everyone to read. No actor check; it says nothing an anonymous visitor could not infer
/// from the page.
#[utoipa::path(
    get,
    path = "/api/instance",
    tag = "reference",
    responses((status = 200, body = Instance, description = "OK")),
    security(()),
)]
pub async fn instance(State(state): State<AppState>) -> ApiResult<Instance> {
    Ok(Json(Instance {
        discord: DiscordCapability {
            linking: state.discord.is_some(),
            bot: state
                .discord
                .as_ref()
                .is_some_and(|d| d.bot_token.is_some()),
        },
        announcement: state.announcement.as_deref().cloned(),
    }))
}

/// `GET /api/grid-config`: the server-owned map canvas geometry.
#[utoipa::path(
    get,
    path = "/api/grid-config",
    tag = "reference",
    responses((status = 200, body = GridConfig, description = "OK")),
    security(()),
)]
pub async fn grid_config(State(_state): State<AppState>) -> ApiResult<GridConfig> {
    Ok(Json(crate::maps::grid()))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EffectsQuery {
    /// The effect, as the map names it (`Pulsar`, `Wolf-Rayet Star`, ...).
    pub name: String,
    /// The wormhole class the effect applies at.
    pub class: i32,
}

/// `GET /api/effects?name=&class=`: the buffs/debuffs a wormhole effect applies at a
/// system's class. Reference data, so no actor/role check.
#[utoipa::path(
    get,
    path = "/api/effects",
    tag = "reference",
    params(EffectsQuery),
    responses((status = 200, body = Vec<EffectModifier>, description = "OK")),
    security(()),
)]
pub async fn effect_modifiers(
    State(state): State<AppState>,
    Query(query): Query<EffectsQuery>,
) -> ApiResult<Vec<EffectModifier>> {
    let mods = crate::maps::view::effect_modifiers(&state.db, &query.name, query.class).await?;
    Ok(Json(mods))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchQuery {
    /// What to look for; at least two characters.
    pub q: String,
}

/// `GET /api/systems/search?q=`, search the SDE solar systems by name. Prefix matches rank
/// first, then shorter names, then alphabetical. Returns nothing for queries under 2 chars.
#[utoipa::path(
    get,
    path = "/api/systems/search",
    tag = "reference",
    params(SearchQuery),
    responses((status = 200, body = Vec<SystemSearchResult>, description = "OK"), super::AuthedErrors),
)]
pub async fn search_systems(
    State(state): State<AppState>,
    creds: Credentials,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Vec<SystemSearchResult>> {
    require_actor(&state.db, &creds).await?;
    Ok(Json(search_systems_named(&state.db, &query.q).await?))
}

/// The static half of the routing graph, typed and built once: k-space stargate
/// adjacency (Zarzakh excluded since its gates are faction-gated), per-system security
/// for the safer/less-secure cost functions, and the station indexes the planner offers.
#[derive(Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RoutingGraph {
    pub adjacency: std::collections::HashMap<i64, Vec<i64>>,
    pub security: std::collections::HashMap<i64, f64>,
    pub jove: Vec<i64>,
    pub stations: Vec<i64>,
    pub services: Vec<StationGroup>,
    pub corporations: Vec<StationGroup>,
}

/// A named set of stations: a service, or the corporation that owns them.
#[derive(Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StationGroup {
    pub id: i64,
    pub name: String,
    /// The owning corporation's faction; absent for service groups.
    #[ts(optional)]
    pub faction: Option<FactionRef>,
    pub stations: Vec<StationRef>,
}

#[derive(Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FactionRef {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, serde::Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StationRef {
    pub id: i64,
    pub name: String,
    pub solar_system_id: i64,
}

/// The graph is immutable SDE data, so it is built and serialized once per process
/// rather than recomputed for every cold client load. `Bytes` clones are refcounted,
/// so a request never copies the multi-megabyte payload.
static ROUTING_GRAPH_JSON: tokio::sync::OnceCell<axum::body::Bytes> =
    tokio::sync::OnceCell::const_new();

/// `GET /api/routing-graph`. Static reference data: cacheable for a day.
#[utoipa::path(
    get,
    path = "/api/routing-graph",
    tag = "reference",
    responses((status = 200, body = RoutingGraph, description = "OK")),
    security(()),
)]
pub async fn routing_graph(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let body = match ROUTING_GRAPH_JSON.get() {
        Some(body) => body.clone(),
        None => {
            let graph = build_routing_graph(&state.db).await?;
            let body = axum::body::Bytes::from(
                serde_json::to_string(&graph).expect("routing graph serializes"),
            );
            // An empty adjacency means the SDE is not seeded yet. Serving it is fine;
            // pinning it for the process lifetime would keep the planner empty until
            // a restart, so only a real graph is cached.
            if !graph.adjacency.is_empty() {
                let _ = ROUTING_GRAPH_JSON.set(body.clone());
            }
            body
        }
    };
    Ok((
        [
            (axum::http::header::CACHE_CONTROL, "public, max-age=86400"),
            (axum::http::header::CONTENT_TYPE, "application/json"),
        ],
        body,
    ))
}

async fn build_routing_graph(db: &sqlx::PgPool) -> Result<RoutingGraph, ApiError> {
    const ZARZAKH: i64 = 30100000;
    let rows = sqlx::query!(
        "select solar_system_id, destination_system_id from stargates
         where solar_system_id <> $1 and destination_system_id <> $1",
        ZARZAKH,
    )
    .fetch_all(db)
    .await?;
    let mut adjacency: std::collections::HashMap<i64, Vec<i64>> = std::collections::HashMap::new();
    for r in rows {
        adjacency
            .entry(r.solar_system_id)
            .or_default()
            .push(r.destination_system_id);
    }
    let security: std::collections::HashMap<i64, f64> =
        sqlx::query!("select id, security_status from solar_systems")
            .fetch_all(db)
            .await?
            .into_iter()
            .map(|r| (r.id, r.security_status))
            .collect();
    let jove: Vec<i64> = sqlx::query_scalar!("select solar_system_id from jove_observatories")
        .fetch_all(db)
        .await?;
    let stations: Vec<i64> = sqlx::query_scalar!("select distinct solar_system_id from stations")
        .fetch_all(db)
        .await?;
    // The legacy "essential" station services, each carrying its concrete stations so a
    // result can name the station. Security Offices (27) are a known quirk: only
    // CONCORD-owned stations below 0.5 displayed security actually run one, despite the
    // operation listing the service everywhere.
    const ESSENTIAL_SERVICES: [i64; 6] = [5, 10, 13, 14, 15, 27];
    const SECURITY_OFFICE: i64 = 27;
    const CONCORD_CORPORATION: i64 = 1000125;
    let rows = sqlx::query!(
        r#"select ss.id as "service_id", ss.name as "service_name",
                  st.id as "station_id", st.name as "station_name",
                  st.solar_system_id, sys.security_status
           from station_services ss
           join station_operation_services sos on sos.service_id = ss.id
           join stations st on st.operation_id = sos.operation_id
           join solar_systems sys on sys.id = st.solar_system_id
           where ss.id = any($1)
             and (ss.id <> $2 or st.owner_corporation_id = $3)
           order by ss.name, st.name"#,
        &ESSENTIAL_SERVICES,
        SECURITY_OFFICE,
        CONCORD_CORPORATION,
    )
    .fetch_all(db)
    .await?;
    let rows = rows.into_iter().filter(|row| {
        row.service_id != SECURITY_OFFICE
            || crate::util::security::ccp_round_security(row.security_status) < 0.5
    });
    let services = group_stations(rows.map(|row| {
        (
            row.service_id,
            row.service_name,
            StationRef {
                id: row.station_id,
                name: row.station_name.unwrap_or_default(),
                solar_system_id: row.solar_system_id,
            },
        )
    }));

    // The NPC corporations that own stations, in the same shape as the services above, so
    // "the nearest Quafe Company station" is the same question as "the nearest repair shop"
    // and the client answers both with one relaxation over the graph it already has.
    let rows = sqlx::query!(
        r#"select st.owner_corporation_id as "corporation_id!", c.name as "corporation_name!",
                  c.faction_id, f.name as "faction_name?",
                  st.id as "station_id", st.name as "station_name", st.solar_system_id
           from stations st
           join corporations c on c.id = st.owner_corporation_id
           left join factions f on f.id = c.faction_id
           where st.owner_corporation_id is not null
           order by c.name, st.name"#,
    )
    .fetch_all(db)
    .await?;
    let mut faction_of = std::collections::HashMap::new();
    for row in &rows {
        if let (Some(id), Some(name)) = (row.faction_id, row.faction_name.clone()) {
            faction_of.insert(row.corporation_id, FactionRef { id, name });
        }
    }
    let mut corporations = group_stations(rows.into_iter().map(|row| {
        (
            row.corporation_id,
            row.corporation_name,
            StationRef {
                id: row.station_id,
                name: row.station_name.unwrap_or_default(),
                solar_system_id: row.solar_system_id,
            },
        )
    }));
    for group in &mut corporations {
        group.faction = faction_of.remove(&group.id);
    }

    Ok(RoutingGraph {
        adjacency,
        security,
        jove,
        stations,
        services,
        corporations,
    })
}

/// Fold `(group id, group name, station)` rows, already ordered by group, into one entry
/// per group.
fn group_stations(rows: impl Iterator<Item = (i64, String, StationRef)>) -> Vec<StationGroup> {
    let mut out: Vec<StationGroup> = Vec::new();
    let mut current: Option<StationGroup> = None;
    for (id, name, station) in rows {
        match &mut current {
            Some(group) if group.id == id => group.stations.push(station),
            _ => {
                if let Some(group) = current.take() {
                    out.push(group);
                }
                current = Some(StationGroup {
                    id,
                    name,
                    faction: None,
                    stations: vec![station],
                });
            }
        }
    }
    if let Some(group) = current {
        out.push(group);
    }
    out
}

/// `GET /api/systems/resolve?ids=a,b,c`, resolve solar system ids to display data for
/// route rows. Capped at 200 ids.
#[utoipa::path(
    get,
    path = "/api/systems/resolve",
    tag = "reference",
    params(ResolveQuery),
    responses((status = 200, body = Vec<SystemSearchResult>, description = "OK"), (status = 400, response = super::BadRequest)),
    security(()),
)]
pub async fn resolve_systems(
    State(state): State<AppState>,
    Query(query): Query<ResolveQuery>,
) -> ApiResult<Vec<SystemSearchResult>> {
    let ids: Vec<i64> = query
        .ids
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .take(200)
        .collect();
    Ok(Json(systems_for(&state.db, &ids).await?))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ResolveQuery {
    /// Comma-separated solar system ids, at most 200.
    pub ids: String,
}

/// `GET /api/threat/{solar_system_id}`: a wormhole system's threat analysis. 404 for
/// k-space systems (threat is only computed for wormhole space).
#[utoipa::path(
    get,
    path = "/api/threat/{solar_system_id}",
    tag = "reference",
    params(("solar_system_id" = i64, Path, description = "The EVE solar system")),
    responses((status = 200, body = ThreatAnalysis, description = "OK"), super::SharedReadErrors),
)]
pub async fn threat_analysis(
    State(state): State<AppState>,
    creds: Credentials,
    Path(solar_system_id): Path<i64>,
) -> ApiResult<ThreatAnalysis> {
    require_actor(&state.db, &creds).await?;
    let row = sqlx::query!(
        r#"select threat_level as "threat_level: crate::maps::ThreatLevel", threat_analyzed_at
           from wormhole_systems where solar_system_id = $1"#,
        solar_system_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(crate::maps::MapError::NotFound)?;
    let entities = sqlx::query!(
        "select entity_id, entity_type, name, kills from wormhole_system_threats
         where solar_system_id = $1 order by kills desc",
        solar_system_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(ThreatAnalysis {
        threat_level: row.threat_level,
        threat_analyzed_at: row.threat_analyzed_at,
        entities: entities
            .into_iter()
            .map(|e| ThreatEntity {
                id: e.entity_id,
                name: e.name,
                entity_type: e.entity_type,
                kills: e.kills,
            })
            .collect(),
    }))
}

/// `GET /api/skyhooks`, every skyhook currently or shortly raidable. Public EVE data, so
/// no per-map gating; a session is still required, like the rest of the API.
#[utoipa::path(
    get,
    path = "/api/skyhooks",
    tag = "reference",
    responses((status = 200, body = Vec<crate::skyhooks::Skyhook>, description = "OK"), super::AuthedErrors),
)]
pub async fn skyhooks(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<crate::skyhooks::Skyhook>> {
    require_actor(&state.db, &creds).await?;
    Ok(Json(crate::skyhooks::list(&state.db).await?))
}

/// `GET /api/server-status`, what Tranquility is doing. Public: the header shows it
/// signed in or not, and it is the same figure ESI serves to anyone.
#[utoipa::path(
    get,
    path = "/api/server-status",
    tag = "reference",
    responses((status = 200, body = crate::server_status::ServerStatus, description = "OK")),
    security(()),
)]
pub async fn server_status(
    State(state): State<AppState>,
) -> ApiResult<crate::server_status::ServerStatus> {
    Ok(Json(state.server.current()))
}
