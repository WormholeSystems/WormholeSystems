//! Connections between mapped systems, the jump log kept against them, and the sweep
//! that clears out the ones nobody has been through in hours.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::{ShareQuery, acting_on, read_map_as, require_actor};
use super::{ApiError, ApiResult};
use crate::app::AppState;
use crate::maps::MapConnection;
use crate::maps::connection::{
    AddConnection, CleanStaleConnections, RemoveConnection, SetConnectionStatus, StaleConnection,
};
use crate::maps::jumps::{
    AddConnectionJump, ConnectionJump, RemoveConnectionJump, UpdateConnectionJump,
};

/// A ship type matched by the manual-jump ship search.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ShipSearchResult {
    pub id: i64,
    pub name: String,
    pub group_name: String,
    /// Hull mass in kg.
    pub mass: Option<f64>,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(search_ships))
        .routes(routes!(add_connection))
        .routes(routes!(set_connection_status))
        .routes(routes!(remove_connection))
        .routes(routes!(list_connection_jumps))
        .routes(routes!(add_connection_jump))
        .routes(routes!(update_connection_jump))
        .routes(routes!(remove_connection_jump))
        .routes(routes!(list_stale_connections))
        .routes(routes!(clean_stale_connections))
}

/// `POST /api/maps/{id}/connections/add`
///
/// The ghost guard sits here, not in the command: raising a ghost creates the one connection
/// an unmapped hole may have, and that goes through the same command from the inside.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/add",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddConnection,
    responses((status = 200, body = MapConnection, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_connection(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddConnection>,
) -> ApiResult<MapConnection> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let ghost_endpoint = sqlx::query_scalar!(
        r#"select exists(
               select 1 from map_solar_systems
               where map_id = $1 and id in ($2, $3) and solar_system_id is null
           ) as "ghost!""#,
        map_id,
        cmd.from_system,
        cmd.to_system,
    )
    .fetch_one(&state.db)
    .await?;
    if ghost_endpoint {
        return Err(ApiError::bad_request(
            "assign a system to that hole before connecting anything to it",
        ));
    }
    let conn = crate::maps::connection::add_connection(&state.db, actor, cmd).await?;
    Ok(Json(conn))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/set-status",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = SetConnectionStatus,
    responses((status = 200, body = MapConnection, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn set_connection_status(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<SetConnectionStatus>,
) -> ApiResult<MapConnection> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let conn = crate::maps::connection::set_connection_status(&state.db, actor, cmd).await?;
    Ok(Json(conn))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/remove",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveConnection,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_connection(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveConnection>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::connection::remove_connection(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `GET /api/maps/{id}/connections/{cid}/jumps`: the latest 10 jump-log rows.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/connections/{cid}/jumps",
    tag = "connections",
    params(("id" = i64, Path, description = "The map"), ("cid" = i64, Path, description = "The connection"), ShareQuery),
    responses((status = 200, body = Vec<ConnectionJump>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn list_connection_jumps(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, connection_id)): Path<(i64, i64)>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<ConnectionJump>> {
    read_map_as(&state, &creds, map_id, &share).await?;
    let jumps = crate::maps::jumps::read_jumps(&state.db, map_id, connection_id).await?;
    Ok(Json(jumps))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/jumps/add",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddConnectionJump,
    responses((status = 200, body = ConnectionJump, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_connection_jump(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddConnectionJump>,
) -> ApiResult<ConnectionJump> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let jump = crate::maps::jumps::add_jump(&state.db, actor, cmd).await?;
    Ok(Json(jump))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/jumps/update",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = UpdateConnectionJump,
    responses((status = 200, body = ConnectionJump, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn update_connection_jump(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<UpdateConnectionJump>,
) -> ApiResult<ConnectionJump> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let jump = crate::maps::jumps::update_jump(&state.db, actor, cmd).await?;
    Ok(Json(jump))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/jumps/remove",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveConnectionJump,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_connection_jump(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveConnectionJump>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::jumps::remove_jump(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `GET /api/maps/{id}/connections/stale`, edges that have been critical or expired for
/// over an hour.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/connections/stale",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<StaleConnection>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn list_stale_connections(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<StaleConnection>> {
    let actor = require_actor(&state.db, &creds).await?;
    let rows = crate::maps::connection::list_stale_connections(&state.db, actor, map_id).await?;
    Ok(Json(rows))
}

/// `POST /api/maps/{id}/connections/clean-stale`, sweep them, and the placements they
/// orphan, as one undoable change.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/connections/clean-stale",
    tag = "connections",
    params(("id" = i64, Path, description = "The map")),
    request_body = CleanStaleConnections,
    responses((status = 200, body = u64, description = "How many connections were removed"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn clean_stale_connections(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<CleanStaleConnections>,
) -> ApiResult<u64> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let removed = crate::maps::connection::clean_stale_connections(&state.db, actor, cmd).await?;
    Ok(Json(removed))
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ShipSearchQuery {
    /// Part of a ship type name.
    pub q: String,
}

/// `GET /api/ships/search?q=`, published ship types (SDE category 6) by name, with
/// hull mass for the manual-jump form. Reference data, no actor check.
#[utoipa::path(
    get,
    path = "/api/ships/search",
    tag = "connections",
    params(ShipSearchQuery),
    responses((status = 200, body = Vec<ShipSearchResult>, description = "OK")),
    security(()),
)]
pub async fn search_ships(
    State(state): State<AppState>,
    Query(query): Query<ShipSearchQuery>,
) -> ApiResult<Vec<ShipSearchResult>> {
    let q = query.q.trim();
    if q.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let results = sqlx::query_as!(
        ShipSearchResult,
        r#"select t.id, t.name, g.name as group_name, t.mass
           from types t
           join groups g on g.id = t.group_id
           where g.category_id = 6 and t.published and t.name ilike '%' || $1 || '%'
           order by t.name limit 25"#,
        q,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(results))
}
