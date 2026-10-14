//! Who may see a map and what they may do there: grants, ownership, and the share
//! tokens that let somebody watch without an account.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{acting_on, require_actor};
use super::reference::SearchQuery;
use crate::app::AppState;
use crate::maps::access::{AccessEntry, RevokeAccess, SetAccess};

/// A grantable subject from the access-subject search.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AccessSubject {
    pub subject_type: crate::maps::SubjectType,
    pub subject_id: i64,
    pub name: String,
    pub ticker: Option<String>,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(search_access_subjects))
        .routes(routes!(list_access))
        .routes(routes!(set_access))
        .routes(routes!(revoke_access))
        .routes(routes!(transfer_ownership))
        .routes(routes!(rotate_share_token, revoke_share_token))
}

/// `GET /api/access-subjects/search?q=`, characters, corporations and alliances that can
/// be granted access. Only entities WormholeSystems has already cached are searchable (a character
/// who has signed in, or a corp/alliance one of them belongs to), hence the UI also
/// accepting a raw EVE id.
#[utoipa::path(
    get,
    path = "/api/access-subjects/search",
    tag = "access",
    params(SearchQuery),
    responses((status = 200, body = Vec<AccessSubject>, description = "OK"), super::AuthedErrors),
)]
pub async fn search_access_subjects(
    State(state): State<AppState>,
    creds: Credentials,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Vec<AccessSubject>> {
    require_actor(&state.db, &creds).await?;
    let q = query.q.trim();
    if q.len() < 2 {
        return Ok(Json(Vec::new()));
    }
    let contains = format!("%{q}%");
    let prefix = format!("{q}%");
    let rows = sqlx::query!(
        // The `!` overrides restate what the source tables guarantee: sqlx cannot see
        // through the union and widens every column to nullable.
        r#"select id as "id!", name as "name!",
                  kind as "kind!: crate::maps::SubjectType", ticker
           from (
               select id, name, 'character' as kind, null::text as ticker from characters
               union all
               select id, name, 'corporation', ticker from corporations
               union all
               select id, name, 'alliance', ticker from alliances
           ) s
           where s.name ilike $1 or s.ticker ilike $1
           order by (s.name ilike $2) desc, length(s.name), s.name
           limit 20"#,
        contains,
        prefix,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AccessSubject {
                subject_type: r.kind,
                subject_id: r.id,
                name: r.name,
                ticker: r.ticker,
            })
            .collect(),
    ))
}

/// `GET /api/maps/{id}/access`, who can see this map, and at what role. Viewer+.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/access",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = Vec<AccessEntry>, description = "OK"), super::MapReadErrors),
)]
pub async fn list_access(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<Vec<AccessEntry>> {
    let actor = require_actor(&state.db, &creds).await?;
    let entries = crate::maps::access::list_access(&state.db, actor, map_id).await?;
    Ok(Json(entries))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/access/set",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    request_body = SetAccess,
    responses((status = 200, description = "Done; the body is `null`"), super::MapWriteErrors),
)]
pub async fn set_access(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<SetAccess>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::access::set_access(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/access/revoke",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    request_body = RevokeAccess,
    responses((status = 200, description = "Done; the body is `null`"), super::MapWriteErrors),
)]
pub async fn revoke_access(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RevokeAccess>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::access::revoke_access(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/access/transfer`, hand the map to another character on it.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/access/transfer",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    request_body = crate::maps::access::TransferOwnership,
    responses((status = 200, description = "Done; the body is `null`"), super::MapWriteErrors),
)]
pub async fn transfer_ownership(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<crate::maps::access::TransferOwnership>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::access::transfer_ownership(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/share`, mint a share link, replacing any earlier one.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/share",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, body = String, description = "The new share token"), super::MapReadErrors),
)]
pub async fn rotate_share_token(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<String> {
    let actor = require_actor(&state.db, &creds).await?;
    let token = crate::maps::map::rotate_share_token(&state.db, actor, map_id).await?;
    Ok(Json(token))
}

/// `DELETE /api/maps/{id}/share`, withdraw the share link.
#[utoipa::path(
    delete,
    path = "/api/maps/{id}/share",
    tag = "access",
    params(("id" = i64, Path, description = "The map")),
    responses((status = 200, description = "Done; the body is `null`"), super::MapReadErrors),
)]
pub async fn revoke_share_token(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    crate::maps::map::revoke_share_token(&state.db, actor, map_id).await?;
    Ok(Json(()))
}
