//! Who is signed in, which character they are flying, and the ESI calls that act on
//! that character alone (autopilot waypoints). Nothing here is about a map.

use super::extract::Credentials;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::{require_actor, session_actor, session_id};
use super::{ApiError, ApiResult};
use crate::app::AppState;

/// The signed-in character, for the UI's auth state.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterSummary {
    pub character_id: i64,
    pub name: String,
    /// Whether this character may create a map here. An instance can limit that to named
    /// affiliations, and a button that only produces a 403 is worse than no button.
    pub can_create_map: bool,
}

/// Live status of the active character, for the navbar readout.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterStatus {
    pub online: bool,
    pub solar_system: Option<String>,
    pub ship_type_id: Option<i64>,
    pub ship_name: Option<String>,
    pub ship_type: Option<String>,
}

/// One of the user's characters, for the switcher.
#[derive(Clone, Debug, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CharacterRef {
    pub character_id: i64,
    pub name: String,
    pub is_active: bool,
    /// The one new sessions start as, chosen on the characters page.
    pub is_preferred: bool,
    pub online: bool,
    /// Where the character is right now, when online and tracked. Drives the paste
    /// system-mismatch warning.
    pub solar_system_id: Option<i64>,
}

/// One ESI permission and whether the acting character has consented to it.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ScopeStatus {
    pub scope: String,
    pub granted: bool,
}

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(me))
        .routes(routes!(me_status))
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
    responses((status = 200, body = Option<CharacterSummary>, description = "The acting character, or `null` when nobody is signed in"), (status = 401, response = super::Unauthorized)),
    security((), ("bearer" = []), ("session" = [])),
)]
pub async fn me(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Option<CharacterSummary>> {
    let Some(actor) = session_actor(&state.db, &creds).await? else {
        return Ok(Json(None));
    };
    let row = sqlx::query!(
        "select name from characters where id = $1",
        actor.character_id
    )
    .fetch_optional(&state.db)
    .await?;
    let Some(row) = row else {
        return Ok(Json(None));
    };
    let can_create_map =
        crate::maps::map::may_create_map(&state.db, actor, &state.map_creators).await?;
    Ok(Json(Some(CharacterSummary {
        character_id: actor.character_id,
        name: row.name,
        can_create_map,
    })))
}

/// `GET /api/me/status`, live status of the active character (online / system / ship).
#[utoipa::path(
    get,
    path = "/api/me/status",
    tag = "identity",
    responses((status = 200, body = Option<CharacterStatus>, description = "The live status, or `null` when it is not tracked yet"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn me_status(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Option<CharacterStatus>> {
    let Some(actor) = session_actor(&state.db, &creds).await? else {
        return Ok(Json(None));
    };
    let row = sqlx::query!(
        r#"select s.online,
                  s.ship_type_id,
                  ss.name as "solar_system?",
                  s.ship_name,
                  t.name as "ship_type?"
           from character_status s
           left join solar_systems ss on ss.id = s.solar_system_id
           left join types t on t.id = s.ship_type_id
           where s.character_id = $1"#,
        actor.character_id,
    )
    .fetch_optional(&state.db)
    .await?;
    Ok(Json(row.map(|r| CharacterStatus {
        online: r.online,
        solar_system: r.solar_system,
        ship_type_id: r.ship_type_id,
        ship_name: r.ship_name,
        ship_type: r.ship_type,
    })))
}

/// `GET /api/me/discord`: the linked Discord account, if any.
#[utoipa::path(
    get,
    path = "/api/me/discord",
    tag = "identity",
    responses((status = 200, body = Option<crate::discord::DiscordAccount>, description = "The linked account, or `null`"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, description = "Done; the body is `null`"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, body = Vec<crate::api::ScopeStatus>, description = "OK"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn my_scopes(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<crate::api::ScopeStatus>> {
    let actor = require_actor(&state.db, &creds).await?;
    let granted = crate::auth::granted_scopes(&state.db, actor.character_id).await;
    Ok(Json(
        crate::esi::scopes::Scope::ALL
            .iter()
            .map(|scope| crate::api::ScopeStatus {
                scope: scope.as_str().to_string(),
                granted: granted.contains(scope),
            })
            .collect(),
    ))
}

/// `GET /api/me/characters`: the user's characters, marking the active and preferred ones.
#[utoipa::path(
    get,
    path = "/api/me/characters",
    tag = "identity",
    responses((status = 200, body = Vec<CharacterRef>, description = "OK"), (status = 401, response = super::Unauthorized)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn my_characters(
    State(state): State<AppState>,
    creds: Credentials,
) -> ApiResult<Vec<CharacterRef>> {
    let actor = require_actor(&state.db, &creds).await?;
    let rows = sqlx::query!(
        r#"select c.id, c.name, c.is_preferred, coalesce(s.online, false) as "online!",
                  case when s.online then s.solar_system_id end as "solar_system_id?"
           from characters c
           left join character_status s on s.character_id = c.id
           where c.user_id = $1 order by c.name"#,
        actor.user_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| CharacterRef {
                character_id: r.id,
                name: r.name,
                is_active: r.id == actor.character_id,
                is_preferred: r.is_preferred,
                online: r.online,
                solar_system_id: r.solar_system_id,
            })
            .collect(),
    ))
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
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
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
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound)),
    security(("bearer" = []), ("session" = [])),
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
    security(("bearer" = []), ("session" = [])),
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
    let character_id = body.character_id;

    let count = sqlx::query_scalar!(
        "select count(*) from characters where user_id = $1",
        actor.user_id,
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);
    if count <= 1 {
        return Err(ApiError::bad_request("can't remove your only character"));
    }

    let owns = sqlx::query_scalar!(
        "select exists(select 1 from characters where id = $1 and user_id = $2)",
        character_id,
        actor.user_id,
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);
    if !owns {
        return Err(ApiError::bad_request("that character isn't yours"));
    }

    // Switch away before deleting the active one (sessions FK-cascade on the character).
    if character_id == actor.character_id
        && let Some(other) = sqlx::query_scalar!(
            "select id from characters where user_id = $1 and id <> $2
             order by is_preferred desc, id limit 1",
            actor.user_id,
            character_id,
        )
        .fetch_optional(&state.db)
        .await?
    {
        crate::session::set_active_character(&state.db, &session_id, other).await?;
    }

    sqlx::query!(
        "delete from characters where id = $1 and user_id = $2",
        character_id,
        actor.user_id,
    )
    .execute(&state.db)
    .await?;

    // Removing the preferred character leaves the account without one, so hand the flag on.
    crate::session::ensure_preferred_character(&state.db, actor.user_id).await?;
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

/// Set an in-game autopilot waypoint for one character via ESI. Maps a missing waypoint
/// scope to 409 (the character must re-consent) and other ESI failures to 502.
async fn esi_waypoint(
    state: &AppState,
    character_id: i64,
    destination_id: i64,
    add_to_beginning: bool,
    clear_other_waypoints: bool,
) -> Result<(), ApiError> {
    use axum::http::StatusCode;
    let store = crate::db::PgTokenStore::new(state.db.clone());
    let token = state
        .sso
        .access_token(&store, character_id, crate::esi::Scope::WriteWaypoint)
        .await
        .map_err(|err| match err {
            crate::esi::EsiError::MissingScope(_) => ApiError {
                status: StatusCode::CONFLICT,
                message: "that character has not granted the waypoint scope".into(),
            },
            other => ApiError {
                status: StatusCode::BAD_GATEWAY,
                message: other.to_string(),
            },
        })?;
    state
        .esi
        .set_waypoint(
            &token,
            destination_id,
            add_to_beginning,
            clear_other_waypoints,
        )
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: err.to_string(),
        })
}

/// ESI accepts a solar system, station, or structure id as a waypoint destination.
/// We can only vouch for the first two (structures are per-character ACL'd and not
/// seeded), so unknown ids are passed through to ESI rather than rejected here.
async fn validate_waypoint_destination(
    state: &AppState,
    destination_id: i64,
) -> Result<(), ApiError> {
    let exists = sqlx::query_scalar!(
        "select exists(select 1 from solar_systems where id = $1)
             or exists(select 1 from stations where id = $1)",
        destination_id,
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);
    if exists {
        Ok(())
    } else {
        Err(ApiError::bad_request("unknown destination"))
    }
}

/// `POST /api/waypoints`, set a destination/waypoint for one of the caller's characters.
#[utoipa::path(
    post,
    path = "/api/waypoints",
    tag = "identity",
    request_body = SetWaypointBody,
    responses((status = 200, description = "Done; the body is `null`"), (status = 400, response = super::BadRequest), (status = 401, response = super::Unauthorized), (status = 404, response = super::NotFound), (status = 409, response = super::Conflict), (status = 502, response = super::UpstreamFailed)),
    security(("bearer" = []), ("session" = [])),
)]
pub async fn set_waypoint(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<SetWaypointBody>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    let owns = sqlx::query_scalar!(
        "select exists(select 1 from characters where id = $1 and user_id = $2)",
        body.character_id,
        actor.user_id,
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(false);
    if !owns {
        return Err(ApiError::bad_request("that character isn't yours"));
    }
    validate_waypoint_destination(&state, body.destination_id).await?;
    esi_waypoint(
        &state,
        body.character_id,
        body.destination_id,
        body.add_to_beginning.unwrap_or(false),
        body.clear_other_waypoints.unwrap_or(true),
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
    security(("bearer" = []), ("session" = [])),
)]
pub async fn set_waypoint_all(
    State(state): State<AppState>,
    creds: Credentials,
    Json(body): Json<SetWaypointAllBody>,
) -> ApiResult<()> {
    let actor = require_actor(&state.db, &creds).await?;
    validate_waypoint_destination(&state, body.destination_id).await?;
    let ids = sqlx::query_scalar!(
        "select c.id from characters c
         join character_status s on s.character_id = c.id and s.online
         where c.user_id = $1",
        actor.user_id,
    )
    .fetch_all(&state.db)
    .await?;
    if ids.is_empty() {
        return Err(ApiError::bad_request("no characters online"));
    }
    let mut ok = 0usize;
    let mut last_err = None;
    for id in ids {
        match esi_waypoint(
            &state,
            id,
            body.destination_id,
            body.add_to_beginning.unwrap_or(false),
            body.clear_other_waypoints.unwrap_or(true),
        )
        .await
        {
            Ok(()) => ok += 1,
            Err(err) => last_err = Some(err),
        }
    }
    if ok == 0 {
        return Err(last_err.expect("at least one attempt"));
    }
    Ok(Json(()))
}
