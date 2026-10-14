//! The map's ignore list: systems everyone routes around and never maps by flying.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, acting_on, read_map_as};
use crate::app::AppState;
use crate::maps::ignored::{
    AddIgnoredSystem, ClearIgnoredSystems, IgnoredSystem, RemoveIgnoredSystem,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_ignored_systems))
        .routes(routes!(add_ignored_system))
        .routes(routes!(remove_ignored_system))
        .routes(routes!(clear_ignored_systems))
}

/// `GET /api/maps/{id}/ignored`: the systems this map keeps out of the way.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/ignored",
    tag = "ignored systems",
    params(("id" = i64, Path, description = "The map"), ShareQuery),
    responses((status = 200, body = Vec<IgnoredSystem>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn list_ignored_systems(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<IgnoredSystem>> {
    read_map_as(&state, &creds, map_id, &share).await?;
    let rows = crate::maps::ignored::read_ignored_systems(&state.db, map_id).await?;
    Ok(Json(rows))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/ignored/add",
    tag = "ignored systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddIgnoredSystem,
    responses((status = 200, body = IgnoredSystem, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_ignored_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddIgnoredSystem>,
) -> ApiResult<IgnoredSystem> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let row = crate::maps::ignored::add_ignored_system(&state.db, actor, cmd).await?;
    Ok(Json(row))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/ignored/remove",
    tag = "ignored systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveIgnoredSystem,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_ignored_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveIgnoredSystem>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::ignored::remove_ignored_system(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/ignored/clear",
    tag = "ignored systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = ClearIgnoredSystems,
    responses((status = 200, body = u64, description = "How many systems were removed from the list"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn clear_ignored_systems(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<ClearIgnoredSystems>,
) -> ApiResult<u64> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let count = crate::maps::ignored::clear_ignored_systems(&state.db, actor, cmd).await?;
    Ok(Json(count))
}
