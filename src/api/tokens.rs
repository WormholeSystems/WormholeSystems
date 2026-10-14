//! The account's personal access tokens. Session-only on purpose: see
//! [`require_session_actor`](super::extract::require_session_actor).

use axum::Json;
use axum::extract::{Path, State};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::ApiResult;
use super::extract::{Credentials, require_session_actor};
use crate::app::AppState;
use crate::tokens::{CreateToken, CreatedToken, PersonalAccessToken};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(my_tokens, create_token))
        .routes(routes!(revoke_token))
}

/// `GET /api/me/tokens`, every token on the account, never the secrets.
#[utoipa::path(
    get,
    path = "/api/me/tokens",
    tag = "tokens",
    responses((status = 200, body = Vec<PersonalAccessToken>, description = "OK"), (status = 401, response = super::Unauthorized)),
    security(("session" = [])),
)]
pub async fn my_tokens(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<PersonalAccessToken>> {
    let actor = require_session_actor(&state.db, &creds).await?;
    Ok(Json(crate::tokens::list(&state.db, actor.user_id).await?))
}

/// `POST /api/me/tokens`, mint one. The plaintext is in this response and nowhere else.
#[utoipa::path(
    post,
    path = "/api/me/tokens",
    tag = "tokens",
    request_body = CreateToken,
    responses((status = 200, body = CreatedToken, description = "OK"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized)),
    security(("session" = [])),
)]
pub async fn create_token(
    State(state): State<AppState>,
    creds: Credentials,
    Json(cmd): Json<CreateToken>,
) -> ApiResult<CreatedToken> {
    let actor = require_session_actor(&state.db, &creds).await?;
    Ok(Json(
        crate::tokens::create(&state.db, actor.user_id, cmd).await?,
    ))
}

/// `DELETE /api/me/tokens/{id}`, and whatever used it is locked out on its next request.
#[utoipa::path(
    delete,
    path = "/api/me/tokens/{id}",
    tag = "tokens",
    params(("id" = i64, Path, description = "The token")),
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security(("session" = [])),
)]
pub async fn revoke_token(
    State(state): State<AppState>,
    creds: Credentials,
    Path(token_id): Path<i64>,
) -> ApiResult<()> {
    let actor = require_session_actor(&state.db, &creds).await?;
    crate::tokens::revoke(&state.db, actor.user_id, token_id).await?;
    Ok(Json(()))
}
