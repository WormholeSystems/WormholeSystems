//! Scan results: the signatures on each system, the paste that replaces them wholesale,
//! and the links that tie one to a connection.

use super::extract::Credentials;
use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::{ShareQuery, acting_on, read_map_as};
use super::{ApiError, ApiResult};
use crate::app::AppState;
use crate::maps::Signature;
use crate::maps::signatures::{
    AddSignature, LinkSignature, PasteSignatures, RemoveSignature, RemoveSignatures,
    UnlinkSignature, UpdateSignature,
};

/// A cosmic-signature category from the seeded catalog.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SignatureCategoryInfo {
    pub id: i64,
    pub name: String,
    pub code: String,
}

/// A cosmic-signature type from the seeded catalog. `signature` is the wormhole code
/// (wormhole types only); `target_class` its destination class; `spawn_areas` the system
/// classes this type can appear in.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SignatureTypeInfo {
    pub id: i64,
    pub signature: Option<String>,
    pub name: String,
    pub signature_category_id: i64,
    pub target_class: Option<i32>,
    pub extra: Option<String>,
    pub spawn_areas: Vec<i32>,
    /// Wormhole physics (joined from `wormhole_types` by code; wormhole types only).
    pub total_mass: Option<i64>,
    pub max_jump_mass: Option<i64>,
    pub lifetime_hours: Option<f64>,
    pub signature_strength: Option<f64>,
}

/// The full signature catalog, served once and cached client-side.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SignatureCatalog {
    pub categories: Vec<SignatureCategoryInfo>,
    pub types: Vec<SignatureTypeInfo>,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(signature_catalog))
        .routes(routes!(list_signatures))
        .routes(routes!(add_signature))
        .routes(routes!(paste_signatures))
        .routes(routes!(update_signature))
        .routes(routes!(link_signature))
        .routes(routes!(unlink_signature))
        .routes(routes!(remove_signature))
        .routes(routes!(remove_signatures_bulk))
}

/// `GET /api/maps/{id}/signatures`, all signatures on the map.
#[utoipa::path(
    get,
    path = "/api/maps/{id}/signatures",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map"), ShareQuery),
    responses((status = 200, body = Vec<Signature>, description = "OK"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn list_signatures(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Query(share): Query<ShareQuery>,
) -> ApiResult<Vec<Signature>> {
    read_map_as(&state, &creds, map_id, &share).await?;
    let sigs = crate::maps::signatures::read_signatures(&state.db, map_id).await?;
    Ok(Json(sigs))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/add",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = AddSignature,
    responses((status = 200, body = Signature, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn add_signature(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<AddSignature>,
) -> ApiResult<Signature> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let sig = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(sig))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/paste",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = PasteSignatures,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn paste_signatures(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<PasteSignatures>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/update",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = UpdateSignature,
    responses((status = 200, body = Signature, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn update_signature(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<UpdateSignature>,
) -> ApiResult<Signature> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let sig = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(sig))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/link",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = LinkSignature,
    responses((status = 200, body = Signature, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn link_signature(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<LinkSignature>,
) -> ApiResult<Signature> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let sig = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(sig))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/unlink",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = UnlinkSignature,
    responses((status = 200, body = Signature, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn unlink_signature(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<UnlinkSignature>,
) -> ApiResult<Signature> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    let sig = crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(sig))
}

#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/remove",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveSignature,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_signature(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveSignature>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `POST /api/maps/{id}/signatures/remove-bulk`: the panel's "delete missing
/// signatures" path, with the legacy connection + orphan-endpoint cascade.
#[utoipa::path(
    post,
    path = "/api/maps/{id}/signatures/remove-bulk",
    tag = "signatures",
    params(("id" = i64, Path, description = "The map")),
    request_body = RemoveSignatures,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 403, response = super::Forbidden), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn remove_signatures_bulk(
    State(state): State<AppState>,
    creds: Credentials,
    Path(map_id): Path<i64>,
    Json(cmd): Json<RemoveSignatures>,
) -> ApiResult<()> {
    let actor = acting_on(&state.db, &creds, map_id, cmd.map_id).await?;
    crate::maps::run(&state.db, actor, cmd).await?;
    Ok(Json(()))
}

/// `GET /api/signature-types`: the seeded signature catalog (categories + types with
/// spawn areas). Reference data, so no actor/role check; cacheable for a day.
#[utoipa::path(
    get,
    path = "/api/signature-types",
    tag = "signatures",
    responses((status = 200, body = SignatureCatalog, description = "OK")),
    security(()),
)]
pub async fn signature_catalog(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let categories = sqlx::query_as!(
        SignatureCategoryInfo,
        "select id, name, code from signature_categories order by id",
    )
    .fetch_all(&state.db)
    .await?;
    let types = sqlx::query!(
        r#"select st.id, st.signature, st.name, st.signature_category_id, st.target_class,
                  st.extra,
                  coalesce(array_agg(sa.wormhole_class_id order by sa.wormhole_class_id)
                           filter (where sa.wormhole_class_id is not null), '{}') as "spawn_areas!",
                  wt.total_mass, wt.max_mass_per_jump, wt.lifetime_hours, wt.signature_strength
           from signature_types st
           left join wormhole_types wt on wt.code = st.signature
           left join signature_type_spawn_areas sa on sa.signature_type_id = st.id
           group by st.id, wt.code
           order by st.id"#,
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(|r| SignatureTypeInfo {
        id: r.id,
        signature: r.signature,
        name: r.name,
        signature_category_id: r.signature_category_id,
        target_class: r.target_class,
        extra: r.extra,
        spawn_areas: r.spawn_areas,
        total_mass: r.total_mass,
        max_jump_mass: r.max_mass_per_jump,
        lifetime_hours: r.lifetime_hours,
        signature_strength: r.signature_strength,
    })
    .collect();
    Ok((
        [(axum::http::header::CACHE_CONTROL, "public, max-age=86400")],
        Json(SignatureCatalog { categories, types }),
    ))
}
