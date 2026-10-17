//! The map's watchlist: systems somebody wants a standing route to.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, acting_on, read_map_as, require_actor};
use crate::app::AppState;
use crate::maps::personal_watchlist::{
    AddPersonalWatchlistEntry, RemovePersonalWatchlistEntry, SetPersonalWatchlistPinned,
};
use crate::maps::watchlist::{
    AddWatchlistEntry, RemoveWatchlistEntry, SetWatchlistPinned, WatchlistEntry,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_watchlist))
        .routes(routes!(add_watchlist_entry))
        .routes(routes!(set_watchlist_pinned))
        .routes(routes!(remove_watchlist_entry))
        .routes(routes!(list_personal_watchlist))
        .routes(routes!(add_personal_watchlist_entry))
        .routes(routes!(set_personal_watchlist_pinned))
        .routes(routes!(remove_personal_watchlist_entry))
}

/// `GET /api/maps/{id}/watchlist`: the map's tracked destinations.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/watchlist",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map"), ShareQuery),
    responses((status = 200, body = Vec<WatchlistEntry>, description = "OK"), super::SharedReadErrors),
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
    responses((status = 200, body = WatchlistEntry, description = "OK"), super::MapWriteErrors),
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
    responses((status = 200, body = WatchlistEntry, description = "OK"), super::MapWriteErrors),
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
    responses((status = 200, description = "Done; the body is `null`"), super::MapWriteErrors),
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

/// `GET /api/maps/{id}/watchlist/personal`: the caller's own watchlist on this map.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/watchlist/personal",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<WatchlistEntry>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_personal_watchlist(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<WatchlistEntry>> {
    let actor = require_actor(&state.db, &creds).await?;
    let entries = crate::maps::personal_watchlist::list(&state.db, actor, map_id).await?;
    Ok(Json(entries))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/personal/add",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddPersonalWatchlistEntry,
    responses((status = 200, body = WatchlistEntry, description = "OK"), super::MapWriteErrors),
)]
pub async fn add_personal_watchlist_entry(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<AddPersonalWatchlistEntry>,
) -> ApiResult<WatchlistEntry> {
    let actor = require_actor(&state.db, &creds).await?;
    let entry = crate::maps::personal_watchlist::add(&state.db, actor, map_id, body).await?;
    personal_changed(&state, actor.user_id, map_id);
    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/personal/set-pinned",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = SetPersonalWatchlistPinned,
    responses((status = 200, body = WatchlistEntry, description = "OK"), super::MapWriteErrors),
)]
pub async fn set_personal_watchlist_pinned(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SetPersonalWatchlistPinned>,
) -> ApiResult<WatchlistEntry> {
    let actor = require_actor(&state.db, &creds).await?;
    let entry = crate::maps::personal_watchlist::set_pinned(&state.db, actor, map_id, body).await?;
    personal_changed(&state, actor.user_id, map_id);
    Ok(Json(entry))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/watchlist/personal/remove",
    tag = "watchlist",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemovePersonalWatchlistEntry,
    responses((status = 200, description = "Done; the body is `null`"), super::MapWriteErrors),
)]
pub async fn remove_personal_watchlist_entry(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<RemovePersonalWatchlistEntry>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    crate::maps::personal_watchlist::remove(&state.db, actor, map_id, body).await?;
    personal_changed(&state, actor.user_id, map_id);
    Ok(Json(()))
}

/// Tell the user's other tabs, which hold their own copy of the list.
fn personal_changed(state: &AppState, user_id: i64, map_id: i64) {
    state.user_hub.publish(
        user_id,
        crate::realtime::UserEvent::PersonalWatchlistChanged { map_id },
    );
}
