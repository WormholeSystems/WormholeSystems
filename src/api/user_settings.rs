//! The per-user settings kept against a map. What they hold and how they are stored is
//! [`crate::maps::user_settings`]; this is the reading and the writing of them.

use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::maps::Role;
use crate::maps::user_settings::{MapUserSettings, UpdateMapUserSettings};

use super::ApiResult;
use super::extract::Credentials;
use crate::app::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(map_user_settings, update_map_user_settings))
}

/// `GET /api/maps/{id}/settings/user`: the caller's per-map preferences (defaults when
/// no row exists yet). Requires any access to the map.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/settings/user",
    tag = "user settings",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = MapUserSettings, description = "OK"), super::MapReadErrors),
)]
pub async fn map_user_settings(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<MapUserSettings> {
    let actor = super::extract::require_role_on_map(&state, &creds, map_id, Role::Viewer).await?;
    Ok(Json(
        crate::maps::user_settings::load(&state.db, map_id, actor.user_id).await?,
    ))
}

/// `POST /api/maps/{id}/settings/user`, partial update (upsert) of the caller's per-map
/// preferences.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/settings/user",
    tag = "user settings",
    params(("id" = i64, Path, description = "The map")),
    request_body = UpdateMapUserSettings,
    responses((status = 200, body = MapUserSettings, description = "OK"), super::MapWriteErrors),
)]
pub async fn update_map_user_settings(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<UpdateMapUserSettings>,
) -> ApiResult<MapUserSettings> {
    let actor = super::extract::require_role_on_map(&state, &creds, map_id, Role::Viewer).await?;
    Ok(Json(
        crate::maps::user_settings::update(&state.db, map_id, actor.user_id, &body).await?,
    ))
}
