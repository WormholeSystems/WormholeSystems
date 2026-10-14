//! The public EVE Scout feed: Thera and Turnur connections, refreshed on demand.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::acting_on;
use crate::app::AppState;
use crate::integrations::eve_scout::EveScoutConnection;
use crate::maps::eve_scout::AddEveScoutConnections;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(eve_scout))
        .routes(routes!(add_to_map))
}

/// `GET /api/evescout`, public Thera/Turnur connections, proxied and cached for 60s.
/// Upstream failures degrade to an empty list.
#[utoipa::path(
    get,
    path = "/api/evescout",
    tag = "eve scout",
    responses((status = 200, body = Vec<EveScoutConnection>, description = "OK")),
    security(()),
)]
pub async fn eve_scout(State(state): State<AppState>) -> ApiResult<Vec<EveScoutConnection>> {
    Ok(Json(state.eve_scout.connections().await))
}

/// What the client sends to put a hub's holes on the map: the list itself is the server's,
/// fetched fresh so a stale card cannot map holes that have since collapsed.
#[derive(Debug, Clone, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AddEveScoutToMap {
    pub map_id: i64,
    pub hub_solar_system_id: i64,
    /// Where to put the hub if it is not on the map yet; the server picks otherwise.
    #[serde(default)]
    #[ts(optional)]
    pub x: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub y: Option<f64>,
}

/// `POST /api/maps/{id}/evescout/add`, put every public hole out of one hub on the map.
/// Member+. Answers with how many connections it added; the socket drives the refetch.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/evescout/add",
    tag = "eve scout",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddEveScoutToMap,
    responses((status = 200, body = u64, description = "How many connections were added"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_to_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<AddEveScoutToMap>,
) -> ApiResult<u64> {
    let actor = acting_on(&state.db, &creds, map_id, body.map_id).await?;
    let holes = state.eve_scout.holes_from(body.hub_solar_system_id).await;
    let added = crate::maps::run(
        &state.db,
        actor,
        AddEveScoutConnections {
            map_id: body.map_id,
            hub_solar_system_id: body.hub_solar_system_id,
            at: body.x.zip(body.y),
            holes,
        },
    )
    .await?;
    Ok(Json(added))
}
