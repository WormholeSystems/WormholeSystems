//! The alerts API: what a map watches for, and what happened to those watches.
//!
//! Manager+ throughout: an alert carries a webhook URL or a channel id, which is a key to
//! somebody's Discord server. The alert rows themselves live in [`crate::maps::alerts`];
//! these handlers check the role, validate the request, and answer.

use super::extract::Credentials;
use axum::extract::{Path, State};
use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app::AppState;
use crate::maps::alerts as store;
use crate::maps::{MapError, Role};

pub use crate::maps::alerts::{MapAlert, SaveAlert};

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

/// One line of an alert's history.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapAlertEvent {
    pub id: i64,
    #[ts(optional)]
    pub map_alert_id: Option<i64>,
    #[ts(optional)]
    pub alert_name: Option<String>,
    #[ts(optional)]
    pub actor: Option<String>,
    pub kind: String,
    #[ts(optional)]
    pub detail: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
    responses((status = 200, body = Vec<MapAlert>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_alerts(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapAlert>> {
    require_manager(&state, &creds, map_id).await?;
    Ok(Json(store::list(&state.db, map_id).await?))
}

/// A registered destination, named once and pointed at by any number of alerts.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapWebhook {
    pub id: i64,
    pub name: String,
    /// Enough of the URL to tell two destinations apart, never enough to use one: the URL is
    /// a bearer token for somebody's channel, and every manager reads this list.
    pub summary: String,
    /// How many alerts would stop working if this were deleted.
    pub alert_count: i64,
}

/// A registered role, so alerts ping "Scouts" rather than 1189734502938472.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapWebhookRole {
    pub id: i64,
    pub name: String,
    pub discord_role_id: String,
}

fn webhook_summary(url: &str) -> String {
    let rest = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = rest.split('/').next().unwrap_or(rest);
    // The id is stable and public-ish; the token after it is the secret.
    let id = rest
        .split('/')
        .nth(3)
        .filter(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    match id {
        Some(id) => format!("{host}/…/{id}"),
        None => host.to_string(),
    }
}

/// `GET /api/maps/{id}/webhooks`: the map's destinations. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/webhooks",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapWebhook>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_webhooks(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapWebhook>> {
    require_manager(&state, &creds, map_id).await?;
    let rows = sqlx::query!(
        r#"select w.id, w.name, w.url,
                  (select count(*) from map_alerts a where a.map_webhook_id = w.id) as "alert_count!"
           from map_webhooks w where w.map_id = $1 order by w.name"#,
        map_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| MapWebhook {
                id: row.id,
                name: row.name,
                summary: webhook_summary(&row.url),
                alert_count: row.alert_count,
            })
            .collect(),
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SaveWebhook {
    pub name: String,
    /// Write-only. Absent on an update keeps the stored one.
    #[serde(default)]
    #[ts(optional)]
    pub url: Option<String>,
}

/// `POST /api/maps/{id}/webhooks`, register a destination. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/webhooks",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveWebhook,
    responses((status = 200, body = MapWebhook, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn create_webhook(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SaveWebhook>,
) -> ApiResult<MapWebhook> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("give the destination a name"));
    }
    let Some(url) = body.url.as_deref().map(str::trim).filter(|u| !u.is_empty()) else {
        return Err(ApiError::bad_request("paste the channel's webhook URL"));
    };
    if !url.starts_with("https://discord.com/api/webhooks/") {
        return Err(ApiError::bad_request("that is not a Discord webhook URL"));
    }
    let id = sqlx::query_scalar!(
        "insert into map_webhooks (map_id, name, url) values ($1, $2, $3)
         on conflict (map_id, name) do update set url = excluded.url, updated_at = now()
         returning id",
        map_id,
        body.name.trim(),
        url,
    )
    .fetch_one(&state.db)
    .await?;
    crate::alerts::log(
        &state.db,
        None,
        map_id,
        Some(user_id),
        "destination",
        Some(body.name.trim()),
    )
    .await;
    Ok(Json(MapWebhook {
        id,
        name: body.name.trim().to_string(),
        summary: webhook_summary(url),
        alert_count: 0,
    }))
}

/// `DELETE /api/maps/{id}/webhooks/{webhook_id}`. Manager+. Alerts pointing at it are
/// deleted too, rather than left enabled with nowhere to post.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/webhooks/{webhook_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("webhook_id" = i64, Path, description = "The webhook")),
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn delete_webhook(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, webhook_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    let user_id = require_manager(&state, &creds, map_id).await?;
    let name = sqlx::query_scalar!(
        "delete from map_webhooks where id = $1 and map_id = $2 returning name",
        webhook_id,
        map_id,
    )
    .fetch_optional(&state.db)
    .await?;
    let Some(name) = name else {
        return Err(ApiError::from(MapError::NotFound));
    };
    crate::alerts::log(
        &state.db,
        None,
        map_id,
        Some(user_id),
        "destination_deleted",
        Some(&name),
    )
    .await;
    Ok(Json(()))
}

/// `GET /api/maps/{id}/roles`: the map's named Discord roles. Manager+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/roles",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapWebhookRole>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_roles(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapWebhookRole>> {
    require_manager(&state, &creds, map_id).await?;
    let rows = sqlx::query!(
        "select id, name, discord_role_id from map_webhook_roles
         where map_id = $1 order by name",
        map_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| MapWebhookRole {
                id: row.id,
                name: row.name,
                discord_role_id: row.discord_role_id,
            })
            .collect(),
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SaveRole {
    pub name: String,
    pub discord_role_id: String,
}

/// `POST /api/maps/{id}/roles`, register a role to ping. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/roles",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveRole,
    responses((status = 200, body = MapWebhookRole, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn create_role(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(body): Json<SaveRole>,
) -> ApiResult<MapWebhookRole> {
    require_manager(&state, &creds, map_id).await?;
    let role_id = body.discord_role_id.trim();
    if body.name.trim().is_empty() {
        return Err(ApiError::bad_request("give the role a name"));
    }
    // Discord ids are snowflakes: decimal, and long. Anything else is a copy-paste slip.
    if role_id.len() < 5 || !role_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::bad_request(
            "a Discord role id is a long number: right-click the role with developer mode on",
        ));
    }
    let id = store::register_role(&state.db, map_id, body.name.trim(), role_id).await?;
    Ok(Json(MapWebhookRole {
        id,
        name: body.name.trim().to_string(),
        discord_role_id: role_id.to_string(),
    }))
}

/// `DELETE /api/maps/{id}/roles/{role_id}`. Manager+.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/roles/{role_id}",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map"), ("role_id" = i64, Path, description = "The role")),
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn delete_role(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, role_id)): Path<(i64, i64)>,
) -> ApiResult<()> {
    require_manager(&state, &creds, map_id).await?;
    let deleted = sqlx::query!(
        "delete from map_webhook_roles where id = $1 and map_id = $2",
        role_id,
        map_id,
    )
    .execute(&state.db)
    .await?;
    if deleted.rows_affected() == 0 {
        return Err(ApiError::from(MapError::NotFound));
    }
    Ok(Json(()))
}

/// `POST /api/maps/{id}/alerts`, create one. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/alerts",
    tag = "alerts",
    params(("id" = i64, Path, description = "The map")),
    request_body = SaveAlert,
    responses((status = 200, body = MapAlert, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, body = MapAlert, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, body = MapAlert, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, body = Vec<MapAlertEvent>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_alert_events(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(query): Query<EventsQuery>,
) -> ApiResult<Vec<MapAlertEvent>> {
    require_manager(&state, &creds, map_id).await?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let rows = sqlx::query!(
        r#"select e.id, e.map_alert_id, a.name as "alert_name?", e.kind, e.detail, e.created_at,
                  (select c.name from characters c
                    where c.user_id = e.actor_user_id
                    order by c.id limit 1) as "actor?"
           from map_alert_events e
           left join map_alerts a on a.id = e.map_alert_id
           where e.map_id = $1
           order by e.id desc
           limit $2"#,
        map_id,
        limit,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| MapAlertEvent {
                id: row.id,
                map_alert_id: row.map_alert_id,
                alert_name: row.alert_name,
                actor: row.actor,
                kind: row.kind,
                detail: row.detail,
                created_at: row.created_at,
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_webhook_is_summarised_without_its_token() {
        let url = "https://discord.com/api/webhooks/123456/verysecrettoken";
        let shown = webhook_summary(url);
        assert!(shown.contains("discord.com"));
        assert!(shown.contains("123456"));
        assert!(!shown.contains("verysecrettoken"));
    }
}
