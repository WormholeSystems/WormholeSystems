//! The map as a whole: the list, creating and deleting one, reading the graph, the
//! per-user settings that hang off it, and who is currently flying it.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{ShareQuery, acting_on, read_map_as, require_actor, session_actor};
use crate::auth::AppState;
use crate::maps::map::UpdateMap;
use crate::maps::{Map, MapView};

/// A map in the user's list, with their role on it.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapEntry {
    pub id: i64,
    pub name: String,
    #[ts(optional)]
    pub description: Option<String>,
    pub role: String,
    pub system_count: i64,
    pub connection_count: i64,
    /// How many people can see it, counting every grant however it was made.
    pub member_count: i64,
    /// Tracked pilots currently online in one of its systems.
    pub pilots_online: i64,
    /// Whether this user keeps it in the top bar.
    pub is_pinned: bool,
    /// When the chain last changed. `None` for a map nobody has touched yet.
    #[ts(optional)]
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    /// Hidden from this user's list. Per-user, so archiving does not touch anyone else.
    pub is_archived: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// An online, tracked character on the map (presence), for the node pilot rows and the
/// pilots card.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MapCharacter {
    pub character_id: i64,
    pub name: String,
    pub corporation_ticker: String,
    pub solar_system_id: Option<i64>,
    pub ship_type_id: Option<i64>,
    /// The name the pilot gave the hull, which is not the hull's type.
    pub ship_name: Option<String>,
    pub ship_type: Option<String>,
    /// The hull's inventory group, so the client can tell a covert-ops scanner from a
    /// combat ship without shipping a table of hull names.
    pub ship_group_id: Option<i64>,
    /// Docked in a station or structure: on the map, but not on grid.
    pub is_docked: bool,
    /// One of the viewer's own characters, so their alts can be told apart at a glance.
    pub is_mine: bool,
}

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
    let maps = crate::maps::map::list_maps(&state.db, actor.user_id).await?;
    let ids: Vec<i64> = maps.iter().map(|(m, _)| m.id).collect();

    // The counts in one pass rather than a query per card. Each is a scalar subquery so a
    // map with nothing on it still comes back as a zero rather than falling out of a join.
    let stats = sqlx::query!(
        r#"select m.id,
                  (select count(*) from map_solar_systems s where s.map_id = m.id)
                      as "systems!",
                  (select count(*) from map_connections c where c.map_id = m.id)
                      as "connections!",
                  (select count(*) from map_access a where a.map_id = m.id) as "members!",
                  coalesce((select mus.is_archived from map_user_settings mus
                             where mus.map_id = m.id and mus.user_id = $2), false)
                      as "archived!",
                  coalesce((select mus.is_pinned from map_user_settings mus
                             where mus.map_id = m.id and mus.user_id = $2), false)
                      as "pinned!",
                  (select max(e.created_at) from map_events e where e.map_id = m.id)
                      as "last_activity?",
                  (select count(*)
                     from map_user_settings mus
                     join characters c on c.user_id = mus.user_id
                     join character_status cs on cs.character_id = c.id and cs.online
                     join map_solar_systems ms
                          on ms.map_id = m.id and ms.solar_system_id = cs.solar_system_id
                    where mus.map_id = m.id and mus.tracking_allowed) as "pilots!"
           from maps m
           where m.id = any($1)"#,
        &ids,
        actor.user_id,
    )
    .fetch_all(&state.db)
    .await?;

    let by_id: std::collections::HashMap<i64, _> = stats.into_iter().map(|r| (r.id, r)).collect();
    Ok(Json(
        maps.into_iter()
            .map(|(m, role)| {
                let s = by_id.get(&m.id);
                MapEntry {
                    id: m.id,
                    name: m.name,
                    description: m.description,
                    role: role.as_str().to_string(),
                    system_count: s.map_or(0, |s| s.systems),
                    connection_count: s.map_or(0, |s| s.connections),
                    member_count: s.map_or(0, |s| s.members),
                    pilots_online: s.map_or(0, |s| s.pilots),
                    last_activity: s.and_then(|s| s.last_activity),
                    is_archived: s.is_some_and(|s| s.archived),
                    is_pinned: s.is_some_and(|s| s.pinned),
                    created_at: m.created_at,
                }
            })
            .collect(),
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
    if token.is_empty() {
        return Err(crate::maps::MapError::NotFound.into());
    }
    let map_id = sqlx::query_scalar!("select id from maps where share_token = $1", token)
        .fetch_optional(&state.db)
        .await?
        .ok_or(crate::maps::MapError::NotFound)?;

    let actor = session_actor(&state.db, &creds).await?;
    let reader =
        crate::maps::access::reader_for(&state.db, map_id, actor, Some(token.as_str())).await?;
    let view =
        crate::maps::map::read_map(&state.db, reader, crate::maps::map::GetMap { map_id }).await?;
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
    // Member+: a viewer does not get to see where anyone is.
    let actor =
        super::extract::require_role_on_map(&state, &creds, map_id, crate::maps::Role::Member)
            .await?;
    let rows = sqlx::query!(
        r#"select c.id as character_id, c.name, co.ticker as corporation_ticker,
                  s.solar_system_id, s.ship_type_id, s.ship_name, t.name as "ship_type?",
                  t.group_id as "ship_group_id?", s.is_docked as "is_docked!",
                  (c.user_id = $2) as "is_mine!"
           from characters c
           join users u on u.id = c.user_id
           join map_user_settings mus
               on mus.map_id = $1 and mus.user_id = u.id and mus.tracking_allowed
           join character_status s on s.character_id = c.id and s.online
           join corporations co on co.id = c.corporation_id
           left join types t on t.id = s.ship_type_id
           where exists (
                     select 1 from esi_tokens et
                     join esi_token_scopes ets on ets.token_id = et.id
                     join esi_scopes sc on sc.id = ets.scope_id
                     where et.character_id = c.id
                       and sc.name = 'esi-location.read_location.v1'
                 )
             and exists (
                     select 1 from map_access_live ma
                     where ma.map_id = $1
                       and ma.role <> 'viewer'
                       and ma.subject_id in (
                           select id from characters where user_id = u.id
                           union all
                           select corporation_id from characters where user_id = u.id
                           union all
                           select alliance_id from characters
                           where user_id = u.id and alliance_id is not null
                       )
                 )
           order by c.name"#,
        map_id,
        actor.user_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| MapCharacter {
                character_id: r.character_id,
                name: r.name,
                corporation_ticker: r.corporation_ticker,
                solar_system_id: r.solar_system_id,
                ship_type_id: r.ship_type_id,
                ship_name: r.ship_name,
                ship_type: r.ship_type,
                ship_group_id: r.ship_group_id,
                is_docked: r.is_docked,
                is_mine: r.is_mine,
            })
            .collect(),
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
