//! The map's event log and the moves through it: undo, redo, and jumping to any step.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{acting_on, require_actor};
use crate::app::AppState;
use crate::maps::events_log::{GotoMapEvent, MapHistory, MapIdBody};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_map_events))
        .routes(routes!(undo_map_event))
        .routes(routes!(redo_map_event))
        .routes(routes!(goto_map_event))
}

/// `GET /api/maps/{id}/events`: the map's history tree and where it currently sits. Viewer+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/events",
    tag = "history",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = MapHistory, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_map_events(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<MapHistory> {
    let actor = require_actor(&state.db, &creds).await?;
    let history = crate::maps::events_log::list_history(&state.db, actor, map_id).await?;
    Ok(Json(history))
}

/// `POST /api/maps/{id}/events/undo`, step back to the previous point in the history.
/// Member+. Moving the cursor can touch anything the steps it crosses did, so it publishes
/// `HistoryChanged` and clients refetch rather than trying to reconstruct a targeted event.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/events/undo",
    tag = "history",
    params(("id" = i64, Path, description = "The map")),
    request_body = MapIdBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn undo_map_event(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<MapIdBody>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::events_log::undo(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/events/redo`, step forward onto the most recent next point. Member+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/events/redo",
    tag = "history",
    params(("id" = i64, Path, description = "The map")),
    request_body = MapIdBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn redo_map_event(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<MapIdBody>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::events_log::redo(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/events/goto`, move the map onto any step, including one on a
/// branch that was left behind by an undo. Member+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/events/goto",
    tag = "history",
    params(("id" = i64, Path, description = "The map")),
    request_body = GotoMapEvent,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn goto_map_event(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<GotoMapEvent>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::events_log::goto(&state.db, actor, cmd).await?;
    Ok(Json(()))
}
