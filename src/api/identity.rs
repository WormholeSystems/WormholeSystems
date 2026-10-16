//! Who is signed in, which character they are flying, and the ESI calls that act on
//! that character alone (autopilot waypoints). The work is [`crate::identity`]'s.

use super::extract::Credentials;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::{require_actor, session_actor, session_id};
use super::{ApiError, ApiResult};
use crate::app::AppState;
use crate::identity::characters::{
    self, CharacterError, CharacterRef, CharacterStatus, CharacterSummary, ScopeStatus, Waypoint,
};
use crate::identity::preferences::{self, UpdateUserPreferences, UserPreferences};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me))
        .routes(routes!(me_status))
        .routes(routes!(my_preferences, update_my_preferences))
        .routes(routes!(my_characters))
        .routes(routes!(my_scopes))
        .routes(routes!(my_discord))
        .routes(routes!(unlink_discord))
        .routes(routes!(switch_character))
        .routes(routes!(remove_character))
        .routes(routes!(preferred_character))
        .routes(routes!(set_waypoint))
        .routes(routes!(set_waypoint_all))
}

/// `GET /api/me`, who's signed in, if anyone.
#[utoipa::path(
    get,
    path = "/api/me",
    tag = "identity",
    responses((status = 200, body = Option<CharacterSummary>, description = "The acting character, or `null` when nobody is signed in"), super::AuthedErrors),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn me(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Option<CharacterSummary>> {
    let Some(actor) = session_actor(&state.db, &creds).await? else {
        return Ok(Json(None));
    };
    Ok(Json(
        characters::summary(&state.db, actor, &state.map_creators).await?,
    ))
}

/// `GET /api/me/preferences`: how the site looks to this person, on every map.
#[utoipa::path(
    get,
    path = "/api/me/preferences",
    tag = "identity",
    responses((status = 200, body = UserPreferences, description = "OK"), super::AuthedErrors),
)]
pub async fn my_preferences(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<UserPreferences> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(preferences::get(&state.db, actor.user_id).await?))
}

/// `POST /api/me/preferences`: change some of them; the rest stay as they are.
#[utoipa::path(
    post,
    path = "/api/me/preferences",
    tag = "identity",
    request_body = UpdateUserPreferences,
    responses((status = 200, body = UserPreferences, description = "The preferences now"), super::AuthedErrors),
)]
pub async fn update_my_preferences(
    State(state): State<AppState>,
    creds: Credentials,
    Json(change): Json<UpdateUserPreferences>,
) -> ApiResult<UserPreferences> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(
        preferences::update(&state.db, actor.user_id, change).await?,
    ))
}

/// `GET /api/me/status`, live status of the active character (online / system / ship).
#[utoipa::path(
    get,
    path = "/api/me/status",
    tag = "identity",
    responses((status = 200, body = Option<CharacterStatus>, description = "The live status, or `null` when it is not tracked yet"), super::AuthedErrors),
)]
pub async fn me_status(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Option<CharacterStatus>> {
    let Some(actor) = session_actor(&state.db, &creds).await? else {
        return Ok(Json(None));
    };
    Ok(Json(
        characters::status(&state.db, actor.character_id).await?,
    ))
}

/// `GET /api/me/discord`: the linked Discord account, if any.
#[utoipa::path(
    get,
    path = "/api/me/discord",
    tag = "identity",
    responses((status = 200, body = Option<crate::discord::DiscordAccount>, description = "The linked account, or `null`"), super::AuthedErrors),
)]
pub async fn my_discord(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Option<crate::discord::DiscordAccount>> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(
        crate::discord::account_for(&state.db, actor.user_id).await,
    ))
}

/// `POST /api/me/discord/unlink`, forget it, and stop what depended on it.
#[utoipa::path(
    post,
    path = "/api/me/discord/unlink",
    tag = "identity",
    responses((status = 200, description = "Done; the body is `null`"), super::AuthedErrors),
)]
pub async fn unlink_discord(State(state): State<AppState>, creds: Credentials) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    crate::discord::link::unlink(&state.db, actor.user_id).await;
    Ok(Json(()))
}

/// `GET /api/me/scopes`, every ESI permission the app can use, and whether the acting
/// character has granted it. Always the full list, so the UI can show what is missing.
#[utoipa::path(
    get,
    path = "/api/me/scopes",
    tag = "identity",
    responses((status = 200, body = Vec<ScopeStatus>, description = "OK"), super::AuthedErrors),
)]
pub async fn my_scopes(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<ScopeStatus>> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(
        characters::scopes(&state.db, actor.character_id).await,
    ))
}

/// `GET /api/me/characters`: the user's characters, marking the active and preferred ones.
#[utoipa::path(
    get,
    path = "/api/me/characters",
    tag = "identity",
    responses((status = 200, body = Vec<CharacterRef>, description = "OK"), super::AuthedErrors),
)]
pub async fn my_characters(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<CharacterRef>> {
    let actor = require_actor(&state.db, &creds).await?;
    Ok(Json(characters::list(&state.db, actor).await?))
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterIdBody {
    pub character_id: i64,
}

/// `POST /api/me/switch-character`, switch the session's active character.
#[utoipa::path(
    post,
    path = "/api/me/switch-character",
    tag = "identity",
    request_body = CharacterIdBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), super::SharedReadErrors),
)]
pub async fn switch_character(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<CharacterIdBody>,
) -> ApiResult<()> {
    let Some(session_id) = session_id(&creds) else {
        return Err(ApiError::unauthorized());
    };
    let ok =
        crate::session::set_active_character(&state.db, &session_id, body.character_id).await?;
    if !ok {
        return Err(ApiError::bad_request("that character isn't yours"));
    }
    Ok(Json(()))
}

/// `POST /api/me/preferred-character`, choose which character new sessions start as.
#[utoipa::path(
    post,
    path = "/api/me/preferred-character",
    tag = "identity",
    request_body = CharacterIdBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), super::SharedReadErrors),
)]
pub async fn preferred_character(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<CharacterIdBody>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    let ok = crate::session::set_preferred_character(&state.db, actor.user_id, body.character_id)
        .await?;
    if !ok {
        return Err(ApiError::bad_request("that character isn't yours"));
    }
    Ok(Json(()))
}

/// `POST /api/me/remove-character`, remove one of the user's characters. Refuses to remove
/// the last one. If it's the active character, the session switches to another first (so
/// the session isn't cascade-deleted).
#[utoipa::path(
    post,
    path = "/api/me/remove-character",
    tag = "identity",
    request_body = CharacterIdBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict)),
)]
pub async fn remove_character(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<CharacterIdBody>,
) -> ApiResult<()> {
    let Some(session_id) = session_id(&creds) else {
        return Err(ApiError::unauthorized());
    };
    let actor = require_actor(&state.db, &creds).await?;
    characters::remove(&state.db, &session_id, actor, body.character_id).await?;
    Ok(Json(()))
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetWaypointBody {
    pub character_id: i64,
    pub destination_id: i64,
    #[serde(default)]
    #[ts(optional)]
    pub add_to_beginning: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub clear_other_waypoints: Option<bool>,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SetWaypointAllBody {
    pub destination_id: i64,
    #[serde(default)]
    #[ts(optional)]
    pub add_to_beginning: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub clear_other_waypoints: Option<bool>,
}

fn esi(state: &AppState) -> characters::Esi<'_> {
    characters::Esi {
        sso: &state.sso,
        esi: &state.esi,
    }
}

/// Something the user can fix is a 400, a missing waypoint scope a 409 (the character
/// has to re-consent), and ESI failing a 502.
impl From<CharacterError> for ApiError {
    fn from(err: CharacterError) -> Self {
        use axum::http::StatusCode;
        match err {
            CharacterError::Invalid(message) => ApiError::bad_request(message),
            CharacterError::MissingScope | CharacterError::SignedOut => ApiError {
                status: StatusCode::CONFLICT,
                message: err.to_string(),
            },
            // What ESI said goes to the log: it can name the token, the route and our own
            // backoff state, none of which the caller can act on.
            CharacterError::Upstream(err) => {
                tracing::warn!("waypoint: ESI failed: {err}");
                ApiError {
                    status: StatusCode::BAD_GATEWAY,
                    message: "EVE did not take the waypoint, try again in a moment".into(),
                }
            }
            CharacterError::Db(err) => ApiError::from(err),
        }
    }
}

/// `POST /api/waypoints`, set a destination/waypoint for one of the caller's characters.
#[utoipa::path(
    post,
    path = "/api/waypoints",
    tag = "identity",
    request_body = SetWaypointBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict), (status = 502, response = super::UpstreamFailed)),
)]
pub async fn set_waypoint(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<SetWaypointBody>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    characters::set_waypoint(
        &state.db,
        esi(&state),
        actor.user_id,
        body.character_id,
        Waypoint {
            destination_id: body.destination_id,
            add_to_beginning: body.add_to_beginning.unwrap_or(false),
            clear_other_waypoints: body.clear_other_waypoints.unwrap_or(true),
        },
    )
    .await?;
    Ok(Json(()))
}

/// `POST /api/waypoints/all`, set the destination for every online character of the
/// caller. Best-effort: characters without the scope are skipped; fails only when none
/// succeed.
#[utoipa::path(
    post,
    path = "/api/waypoints/all",
    tag = "identity",
    request_body = SetWaypointAllBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 409, response = super::Conflict), (status = 502, response = super::UpstreamFailed)),
)]
pub async fn set_waypoint_all(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<SetWaypointAllBody>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    characters::set_waypoint_all(
        &state.db,
        esi(&state),
        actor.user_id,
        Waypoint {
            destination_id: body.destination_id,
            add_to_beginning: body.add_to_beginning.unwrap_or(false),
            clear_other_waypoints: body.clear_other_waypoints.unwrap_or(true),
        },
    )
    .await?;
    Ok(Json(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_esi_failure_does_not_reach_the_caller() {
        let err = ApiError::from(CharacterError::Upstream(crate::esi::EsiError::Store(
            "refresh token wst_secret rejected".into(),
        )));
        assert_eq!(err.status, axum::http::StatusCode::BAD_GATEWAY);
        assert!(!err.message.contains("wst_secret"), "{}", err.message);
    }
}
