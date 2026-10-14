//! Searching one map: systems, aliases, notes and recent kills, ranked together.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, read_map_as};
use super::reference::SearchQuery;
use crate::app::AppState;
use crate::maps::search::MapSearchHit;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(search_map))
}

/// `GET /api/maps/{id}/search?q=`: the map command palette. Matches placed systems by
/// name, alias, occupier and (for members) notes, then falls back to off-map systems the
/// palette can offer to add, ranked by what matched. Viewer+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/search",
    tag = "search",
    params(("id" = i64, Path, description = "The map"), SearchQuery, ShareQuery),
    responses((status = 200, body = Vec<MapSearchHit>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn search_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(query): Query<SearchQuery>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<MapSearchHit>> {
    let role = read_map_as(&state, &creds, map_id, &share).await?.role;
    Ok(Json(
        crate::maps::search::search_map(&state.db, map_id, role, &query.q).await?,
    ))
}
