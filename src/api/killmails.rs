//! Kills on the systems of one map.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, read_map_as};
use crate::app::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(map_killmails))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct KillmailsQuery {
    /// The solar system selected on the map. Only read when the card is set to follow the
    /// selection; the server keeps no idea of what a viewer has selected.
    #[serde(default)]
    pub solar_system_id: Option<i64>,
}

/// `GET /api/maps/{id}/killmails`, recent kills in this map's systems, newest first.
/// Viewer+, like reading the graph: a killmail is public record on zKillboard anyway.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/killmails",
    tag = "killmails",
    params(("id" = i64, Path, description = "The map"), KillmailsQuery, ShareQuery),
    responses((status = 200, body = Vec<crate::killmails::MapKillmail>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn map_killmails(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(query): Query<KillmailsQuery>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<crate::killmails::MapKillmail>> {
    let reader = read_map_as(&state, &creds, map_id, &share).await?;
    // Which kills to show is a per-user preference, and a watcher has nowhere to keep one.
    let scope = match reader.actor {
        Some(actor) => sqlx::query_scalar!(
            "select killmail_filter from map_user_settings where map_id = $1 and user_id = $2",
            map_id,
            actor.user_id,
        )
        .fetch_optional(&state.db)
        .await?
        .unwrap_or(crate::maps::KillmailScope::All),
        None => crate::maps::KillmailScope::All,
    };
    let Some(filter) = crate::killmails::KillmailFilter::new(scope, query.solar_system_id) else {
        return Ok(Json(Vec::new()));
    };

    Ok(Json(
        crate::killmails::list_for_map(&state.db, map_id, filter, crate::killmails::CARD_LIMIT)
            .await?,
    ))
}
