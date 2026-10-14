//! The map's watchlist: systems somebody wants a standing route to.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, acting_on, read_map_as};
use crate::app::AppState;
use crate::maps::watchlist::{
    AddWatchlistEntry, RemoveWatchlistEntry, SetWatchlistPinned, WatchlistEntry,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_watchlist))
        .routes(routes!(add_watchlist_entry))
        .routes(routes!(set_watchlist_pinned))
        .routes(routes!(remove_watchlist_entry))
}

/// `GET /api/maps/{id}/watchlist`: the map's tracked destinations.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/watchlist",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map"), ShareQuery),
    responses((status = 200, body = Vec<WatchlistEntry>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn list_watchlist(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<WatchlistEntry>> {
    read_map_as(&state, &creds, map_id, &share).await?;
    let entries = crate::maps::watchlist::read_watchlist(&state.db, map_id).await?;
    Ok(Json(entries))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/add",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddWatchlistEntry,
    responses((status = 200, body = WatchlistEntry, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_watchlist_entry(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddWatchlistEntry>,
) -> ApiResult<WatchlistEntry> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let entry = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/set-pinned",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = SetWatchlistPinned,
    responses((status = 200, body = WatchlistEntry, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn set_watchlist_pinned(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<SetWatchlistPinned>,
) -> ApiResult<WatchlistEntry> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let entry = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/remove",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveWatchlistEntry,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_watchlist_entry(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveWatchlistEntry>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}
