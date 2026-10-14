//! Request plumbing every area shares: who is calling, what they are allowed to be
//! reading, and the small checks that guard a command before it reaches an action.

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use super::ApiError;
use crate::app::AppState;
use crate::maps::Actor;

/// What a request carries to say who it is from: the session cookie a browser sends, or
/// the bearer token a script sends. One extractor for both, so a handler asks who is
/// calling and never which way they proved it.
pub struct Credentials {
    jar: CookieJar,
    bearer: Option<String>,
}

impl<S: Send + Sync> FromRequestParts<S> for Credentials {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await?;
        let bearer = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_string);
        Ok(Credentials { jar, bearer })
    }
}

impl Credentials {
    /// The cookies alone, for the handlers that read something other than the session.
    pub fn jar(&self) -> &CookieJar {
        &self.jar
    }
}

/// The raw session id from the cookie, if any.
pub(crate) fn session_id(creds: &Credentials) -> Option<String> {
    creds
        .jar
        .get(crate::session::SESSION_COOKIE)
        .map(|c| c.value().to_string())
}

/// The acting character, or `None` if not signed in. A bearer token is tried before the
/// cookie, and a bad one is an error rather than a guest: a script with a stale token has
/// to be told, not shown the public half of the site.
pub(crate) async fn session_actor(
    db: &sqlx::PgPool,
    creds: &Credentials,
) -> Result<Option<Actor>, ApiError> {
    if let Some(token) = &creds.bearer {
        return crate::tokens::actor_for_token(db, token)
            .await?
            .map(Some)
            .ok_or_else(ApiError::unauthorized);
    }
    let Some(session_id) = session_id(creds) else {
        return Ok(None);
    };
    Ok(crate::session::actor_for_session(db, &session_id).await?)
}

/// The auth guard: the acting character, or 401.
pub(crate) async fn require_actor(
    db: &sqlx::PgPool,
    creds: &Credentials,
) -> Result<Actor, ApiError> {
    session_actor(db, creds)
        .await?
        .ok_or_else(ApiError::unauthorized)
}

/// The session-only guard, for managing the tokens themselves: a bearer token that could
/// mint another would never really expire.
pub(crate) async fn require_session_actor(
    db: &sqlx::PgPool,
    creds: &Credentials,
) -> Result<Actor, ApiError> {
    if creds.bearer.is_some() {
        return Err(ApiError::unauthorized());
    }
    require_actor(db, creds).await
}

/// Command bodies carry `map_id` (the action contracts authorize on it); it must agree
/// with the path so a URL can't act on a different map than it names.
fn check_map_id(path_id: i64, body_id: i64) -> Result<(), ApiError> {
    if path_id == body_id {
        Ok(())
    } else {
        Err(ApiError::bad_request("map id in body does not match URL"))
    }
}

/// The caller, for a command that names its own map: the id in the body has to agree with
/// the one in the URL before the session is resolved, so a URL can never act on a map it
/// does not name. Every mutating handler starts here.
pub(crate) async fn acting_on(
    db: &sqlx::PgPool,
    creds: &Credentials,
    path_id: i64,
    body_id: i64,
) -> Result<Actor, ApiError> {
    check_map_id(path_id, body_id)?;
    require_actor(db, creds).await
}

/// The role guard: the acting character, holding at least `min_role` on the map, or
/// 401/403. For reads and the handlers that mutate outside the command pipeline; the
/// pipeline itself re-checks inside its transaction.
pub(crate) async fn require_role_on_map(
    state: &crate::app::AppState,
    creds: &Credentials,
    map_id: i64,
    min_role: crate::maps::Role,
) -> Result<Actor, ApiError> {
    let actor = require_actor(&state.db, creds).await?;
    crate::maps::access::require_role(&state.db, map_id, actor.user_id, min_role).await?;
    Ok(actor)
}

/// The share token, when the caller has one.
#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ShareQuery {
    /// A share token, for reading a map that was opened with a link rather than a grant.
    #[serde(default)]
    pub share: Option<String>,
}

/// The token the share route left behind, so following a link once does not mean carrying
/// the token in every address afterwards.
pub(crate) fn share_cookie(creds: &Credentials, map_id: i64) -> Option<String> {
    creds
        .jar
        .get(&format!("map_share_{map_id}"))
        .map(|c| c.value().to_string())
        .filter(|token| !token.is_empty())
}

/// Who is reading: a grant if the session has one, otherwise whatever the map has been
/// opened up to. Guests never get further than viewer.
pub(crate) async fn read_map_as(
    state: &AppState,
    creds: &Credentials,
    map_id: i64,
    share: &ShareQuery,
) -> Result<crate::maps::access::Reader, ApiError> {
    let actor = session_actor(&state.db, creds).await?;
    let token = share
        .share
        .clone()
        .or_else(|| share_cookie(creds, map_id))
        .filter(|token| !token.is_empty());
    Ok(crate::maps::access::reader_for(&state.db, map_id, actor, token.as_deref()).await?)
}
