//! The map as a whole: the list, creating and deleting one, reading the graph, the
//! per-user settings that hang off it, and who is currently flying it.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, acting_on, read_map_as, require_actor, session_actor};
use crate::app::AppState;
use crate::maps::map::{MapEntry, UpdateMap};
use crate::maps::presence::MapCharacter;
use crate::maps::{Map, MapView};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(my_maps, create_map))
        .routes(routes!(fetch_map, delete_map))
        .routes(routes!(fetch_shared_map))
        .routes(routes!(update_map))
        .routes(routes!(map_characters))
}

/// `GET /api/maps`, every map the signed-in character can access, with their role.
#[utoipa::path(
    get,
    path = "/api/maps",
    tag = "maps",
    responses((status = 200, body = Vec<MapEntry>, description = "OK"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn my_maps(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<MapEntry>> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(
        crate::maps::map::list_entries(&state.db, actor.user_id).await?,
    ))
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateMapBody {
    pub name: String,
    /// Blank counts as absent.
    #[serde(default)]
    #[ts(optional)]
    pub description: Option<String>,
}

/// `POST /api/maps`, create a map owned by the active character.
#[utoipa::path(
    post,
    path = "/api/maps",
    tag = "maps",
    request_body = CreateMapBody,
    responses((status = 200, body = Map, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn create_map(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<CreateMapBody>,
) -> ApiResult<Map> {
    let actor = require_actor(&state.db, &creds).await?;
    let map = crate::maps::map::create_map(
        &state.db,
        actor,
        &state.map_creators,
        crate::maps::map::CreateMap {
            name: body.name,
            description: body.description.filter(|d| !d.trim().is_empty()),
        },
    )
    .await?;
    Ok(Json(map))
}

/// `DELETE /api/maps/{id}`, delete a map (owner only).
#[utoipa::path(
    delete,
    path = "/api/maps/{id}",
    tag = "maps",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn delete_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    crate::maps::map::delete_map(&state.db, actor, crate::maps::map::DeleteMap { map_id }).await?;
    Ok(Json(()))
}

/// `GET /api/maps/{id}`: the full map view (map + systems + connections).
#[utoipa::path(
    get,
    path = "/api/maps/{id}",
    tag = "maps",
    params(("id" = i64, Path, description = "The map"), ShareQuery),
    responses((status = 200, body = MapView, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn fetch_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<MapView> {
    let reader = read_map_as(&state, &creds, map_id, &share).await?;
    let view =
        crate::maps::map::read_map(&state.db, reader, crate::maps::map::GetMap { map_id }).await?;
    Ok(Json(view))
}

/// `GET /api/share/{token}`: the map a share link leads to, for whoever holds it. A token
/// that matches nothing, a withdrawn one and a never-shared map all answer `NotFound`, so
/// the response gives a guesser nothing to work from.
#[utoipa::path(
    get,
    path = "/api/share/{token}",
    tag = "maps",
    params(("token" = String, Path, description = "The share token")),
    responses((status = 200, body = MapView, description = "OK"), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn fetch_shared_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(token): Path<String>,
) -> ApiResult<MapView> {
    let actor = session_actor(&state.db, &creds).await?;
    let view = crate::maps::map::read_shared_map(&state.db, &token, actor).await?;
    Ok(Json(view))
}

/// `GET /api/maps/{id}/characters`, presence: online characters of users who opted into
/// tracking on this map, holding the location scope, whose user has member-or-better
/// access. Member+ may view (viewers never see pilot data).
#[utoipa::path(
    get,
    path = "/api/maps/{id}/characters",
    tag = "maps",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<MapCharacter>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn map_characters(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<MapCharacter>> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(
        crate::maps::presence::characters_on(&state.db, map_id, actor.user_id).await?,
    ))
}

/// `POST /api/maps/{id}/update`, rename a map or change its description/image. Manager+.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/update",
    tag = "maps",
    params(("id" = i64, Path, description = "The map")),
    request_body = UpdateMap,
    responses((status = 200, body = Map, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn update_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<UpdateMap>,
) -> ApiResult<Map> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let map = crate::maps::map::update_map(&state.db, actor, cmd).await?;
    Ok(Json(map))
}
