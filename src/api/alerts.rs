//! The alerts API: what a map watches for, and what happened to those watches.
//!
//! Manager+ throughout: an alert carries a webhook URL or a channel id, which is a key to
//! somebody's Discord server. The alerts, destinations and roles live in
//! [`crate::maps::alerts`]; these handlers check the role and answer.

use super::extract::Credentials;
use axum::extract::{Path, State};
use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::AppState;
use crate::maps::Role;
use crate::maps::alerts as store;

pub use crate::maps::alerts::{
    MapAlert, MapAlertEvent, MapWebhook, MapWebhookRole, SaveAlert, SaveRole, SaveWebhook,
};

use super::{ApiError, ApiResult};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_alerts, create_alert))
        .routes(routes!(list_alert_events))
        .routes(routes!(update_alert, delete_alert))
        .routes(routes!(set_alert_active))
        .routes(routes!(list_webhooks, create_webhook))
        .routes(routes!(delete_webhook))
        .routes(routes!(list_roles, create_role))
        .routes(routes!(delete_role))
}

async fn require_manager(
    state: &AppState,
    creds: &Credentials,
    map_id: i64,
) -> Result<i64, ApiError> {
    let actor = super::extract::require_role_on_map(state, creds, map_id, Role::Manager).await?;
    Ok(actor.user_id)
}

/// `GET /api/maps/{id}/alerts`, every alert on the map. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/alerts",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapAlert>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_alerts(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapAlert>> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(store::list(&state.db, map_id).await?))
}

/// `GET /api/maps/{id}/webhooks`: the map's destinations. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/webhooks",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapWebhook>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_webhooks(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapWebhook>> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(store::list_webhooks(&state.db, map_id).await?))
}

/// `POST /api/maps/{id}/webhooks`, register a destination. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/webhooks",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveWebhook,
    responses((status = 200, body = MapWebhook, description = "OK"), super::MapWriteErrors),
)]
pub async fn create_webhook(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SaveWebhook>,
) -> ApiResult<MapWebhook> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    Ok(Json(
        store::create_webhook(&state.db, map_id, user_id, &body).await?,
    ))
}

/// `DELETE /api/maps/{id}/webhooks/{webhook_id}`. Manager+. Alerts pointing at it are
/// deleted too, rather than left enabled with nowhere to post.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/webhooks/{webhook_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("webhook_id" = i64, Path, description = "The webhook")),
    responses((status = 200, description = "Done; the body is `null`"), super::MapReadErrors),
)]
pub async fn delete_webhook(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, webhook_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    store::delete_webhook(&state.db, map_id, webhook_id, user_id).await?;
    Ok(Json(()))
}

/// `GET /api/maps/{id}/roles`: the map's named Discord roles. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/roles",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapWebhookRole>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_roles(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapWebhookRole>> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(store::list_roles(&state.db, map_id).await?))
}

/// `POST /api/maps/{id}/roles`, register a role to ping. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/roles",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveRole,
    responses((status = 200, body = MapWebhookRole, description = "OK"), super::MapWriteErrors),
)]
pub async fn create_role(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SaveRole>,
) -> ApiResult<MapWebhookRole> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(store::create_role(&state.db, map_id, &body).await?))
}

/// `DELETE /api/maps/{id}/roles/{role_id}`. Manager+.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/roles/{role_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("role_id" = i64, Path, description = "The role")),
    responses((status = 200, description = "Done; the body is `null`"), super::MapReadErrors),
)]
pub async fn delete_role(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, role_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    require_manager(&state, &creds, map_id).await?;
    store::delete_role(&state.db, map_id, role_id).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/alerts`, create one. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/alerts",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveAlert,
    responses((status = 200, body = MapAlert, description = "OK"), super::MapWriteErrors),
)]
pub async fn create_alert(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SaveAlert>,
) -> ApiResult<MapAlert> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    store::check(&state.db, map_id, &body).await?;
    Ok(Json(
        store::create(&state.db, map_id, user_id, &body).await?,
    ))
}

/// `PUT /api/maps/{id}/alerts/{alert_id}`, replace its settings. Manager+.
#[utoipa::path(
    put,
    path = "/api/maps/{id}/alerts/{alert_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("alert_id" = i64, Path, description = "The alert")),
    request_body = SaveAlert,
    responses((status = 200, body = MapAlert, description = "OK"), super::MapWriteErrors),
)]
pub async fn update_alert(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, alert_id)): Path<(i64, i64)>,
    Json(body): Json<SaveAlert>,
) -> ApiResult<MapAlert> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    store::check(&state.db, map_id, &body).await?;
    Ok(Json(
        store::update(&state.db, map_id, alert_id, user_id, &body).await?,
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetAlertActive {
    pub is_active: bool,
}

/// `POST /api/maps/{id}/alerts/{alert_id}/active`, turn it on or off by hand. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/alerts/{alert_id}/active",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("alert_id" = i64, Path, description = "The alert")),
    request_body = SetAlertActive,
    responses((status = 200, body = MapAlert, description = "OK"), super::MapWriteErrors),
)]
pub async fn set_alert_active(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, alert_id)): Path<(i64, i64)>,
    Json(body): Json<SetAlertActive>,
) -> ApiResult<MapAlert> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    Ok(Json(
        store::set_active(&state.db, map_id, alert_id, user_id, body.is_active, None).await?,
    ))
}

/// `DELETE /api/maps/{id}/alerts/{alert_id}`. Manager+.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/alerts/{alert_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("alert_id" = i64, Path, description = "The alert")),
    responses((status = 200, description = "Done; the body is `null`"), super::MapReadErrors),
)]
pub async fn delete_alert(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, alert_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    store::delete(&state.db, map_id, alert_id, user_id).await?;
    Ok(Json(()))
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct EventsQuery {
    /// How many of the newest lines to return.
    #[serde(default)]
    pub limit: Option<i64>,
}

/// `GET /api/maps/{id}/alerts/events`: the audit trail. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/alerts/events",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), EventsQuery),
    responses((status = 200, body = Vec<MapAlertEvent>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_alert_events(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(query): Query<EventsQuery>,
) -> ApiResult<Vec<MapAlertEvent>> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(
        store::list_events(&state.db, map_id, query.limit).await?,
    ))
}
