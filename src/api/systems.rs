//! Systems on a map: placing, moving, removing, and the per-system details (alias,
//! status, occupier, home, rally, pinned, notes) that are all the same shape.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{acting_on, require_actor};
use crate::app::AppState;
use crate::maps::MapSolarSystem;
use crate::maps::solar_system::{
    AddSystem, ClearMap, MoveSystem, MoveSystems, RemoveSystem, RemoveSystems, SetAlias, SetHome,
    SetNotes, SetOccupier, SetPinned, SetRally, SetStatus, SystemDetails,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(clear_map))
        .routes(routes!(add_system))
        .routes(routes!(resolve_ghost_system))
        .routes(routes!(move_systems))
        .routes(routes!(move_system))
        .routes(routes!(remove_systems))
        .routes(routes!(remove_system))
        .routes(routes!(set_alias))
        .routes(routes!(set_status))
        .routes(routes!(set_occupier))
        .routes(routes!(set_home))
        .routes(routes!(set_rally))
        .routes(routes!(set_pinned))
        .routes(routes!(set_notes))
        .routes(routes!(system_details))
}

/// `POST /api/maps/{id}/systems/resolve-ghost`, say which system a ghost turned out to
/// be. Merging into an existing placement removes the ghost, so that goes out too.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/resolve-ghost",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = crate::maps::ghost::ResolveGhostSystem,
    responses((status = 200, body = MapSolarSystem, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn resolve_ghost_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<crate::maps::ghost::ResolveGhostSystem>,
) -> ApiResult<MapSolarSystem> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let placed = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(placed))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/add",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddSystem,
    responses((status = 200, body = MapSolarSystem, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddSystem>,
) -> ApiResult<MapSolarSystem> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let placed = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(placed))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/move-one",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = MoveSystem,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn move_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<MoveSystem>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/move",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = MoveSystems,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn move_systems(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<MoveSystems>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/remove-one",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveSystem,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_system(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveSystem>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/systems/remove",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveSystems,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_systems(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveSystems>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/clear",
    tag = "systems",
    params(("id" = i64, Path, description = "The map")),
    request_body = ClearMap,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn clear_map(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<ClearMap>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[rustfmt::skip]
macro_rules! detail_handler {
    ($name:ident, $path:literal, $cmd:ty, $action:path) => {
        #[utoipa::path(
            post,
            path = $path,
            tag = "systems",
            params(("id" = i64, Path, description = "The map")),
            request_body = $cmd,
            responses(
                (status = 200, description = "Done; the body is `null`"),
                (status = 400, response = super::BadRequest),
                (status = 401, response = super::Unauthorized),
                (status = 403, response = super::Forbidden),
                (status = 404, response = super::NotFound),
                (status = 409, response = super::Conflict),
            ),
            security(("bearer" = []), ("session" = [])),
        )]
        pub async fn $name(
            State(state): State<AppState>,
            creds: Credentials,
            Path(map_id): Path<i64>,
            Json(cmd): Json<$cmd>,
        ) -> ApiResult<()> {
            let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
            $action(&state.db, actor, cmd).await?;
            Ok(Json(()))
        }
    };
}

detail_handler!(
    set_alias,
    "/api/maps/{id}/systems/set-alias",
    SetAlias,
    crate::maps::solar_system::set_alias
);

detail_handler!(
    set_status,
    "/api/maps/{id}/systems/set-status",
    SetStatus,
    crate::maps::solar_system::set_status
);

detail_handler!(
    set_occupier,
    "/api/maps/{id}/systems/set-occupier",
    SetOccupier,
    crate::maps::solar_system::set_occupier
);

detail_handler!(
    set_notes,
    "/api/maps/{id}/systems/set-notes",
    SetNotes,
    crate::maps::solar_system::set_notes
);

detail_handler!(
    set_home,
    "/api/maps/{id}/systems/set-home",
    SetHome,
    crate::maps::solar_system::set_home
);

detail_handler!(
    set_rally,
    "/api/maps/{id}/systems/set-rally",
    SetRally,
    crate::maps::solar_system::set_rally
);

detail_handler!(
    set_pinned,
    "/api/maps/{id}/systems/set-pinned",
    SetPinned,
    crate::maps::solar_system::set_pinned
);

/// `GET /api/maps/{id}/systems/{mss}/details`, member-gated intel (notes). 403 for viewers.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/systems/{mss}/details",
    tag = "systems",
    params(("id" = i64, Path, description = "The map"), ("mss" = i64, Path, description = "The placement (map solar system id)")),
    responses((status = 200, body = SystemDetails, description = "OK"), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn system_details(
    State(state): State<AppState>,
    creds: Credentials,
    Path((map_id, mss)): Path<(i64, i64)>,
) -> ApiResult<SystemDetails> {
    let actor = require_actor(&state.db, &creds).await?;
    let details = crate::maps::solar_system::system_details(&state.db, actor, map_id, mss).await?;
    Ok(Json(details))
}
