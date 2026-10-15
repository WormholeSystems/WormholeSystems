//! Linking a WormholeSystems account to a Discord one.
//!
//! Standard OAuth2 with the `identify` scope, which is all WormholeSystems needs to direct-message
//! the right person and know who a slash command is speaking for.
//!
//! The handshake goes through [`crate::oauth_flow`], the same single-use state the EVE login
//! uses, so expiry and replay are handled in one place. Its states are marked as Discord
//! ones, so the EVE callback will not take them and this one will not take EVE's.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::app::AppState;
use crate::oauth_flow::{self, FlowKind};
use crate::session;

use super::{API, DiscordUser};

/// `GET /discord/connect`, start the link. Requires a WormholeSystems session.
pub async fn connect(State(state): State<AppState>, jar: CookieJar) -> Response {
    let Some(config) = state.discord.as_ref() else {
        return (StatusCode::NOT_FOUND, "Discord is not configured").into_response();
    };
    let Some(cookie) = jar.get(session::SESSION_COOKIE) else {
        return Redirect::to("/").into_response();
    };
    let actor = session::actor_for_session(&state.db, cookie.value()).await;
    let Ok(Some(actor)) = actor else {
        return Redirect::to("/").into_response();
    };

    // The flow row carries the user, and the callback only links to them when it comes
    // back to this browser, still signed in as them.
    let Ok(csrf) = oauth_flow::start(
        &state.db,
        FlowKind::DiscordLink,
        Some(actor.user_id),
        Some("/settings/discord"),
    )
    .await
    else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "could not start").into_response();
    };

    let url = reqwest::Url::parse_with_params(
        "https://discord.com/oauth2/authorize",
        [
            ("client_id", config.client_id.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "identify"),
            ("state", csrf.as_str()),
        ],
    )
    .expect("authorize URL");
    let jar = jar.add(oauth_flow::browser_cookie(
        FlowKind::DiscordLink,
        &csrf,
        state.secure_cookies,
    ));
    (jar, Redirect::to(url.as_str())).into_response()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

/// `GET /discord/callback`, finish the link.
pub async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response {
    if state.discord.is_none() {
        return (StatusCode::NOT_FOUND, "Discord is not configured").into_response();
    }
    // Only the browser that started the link may finish it, checked before the state is
    // consumed so a forged callback cannot spend somebody else's.
    if !oauth_flow::started_here(&jar, FlowKind::DiscordLink, &query.state) {
        return (StatusCode::BAD_REQUEST, "that link request expired").into_response();
    }
    let cleared = oauth_flow::clear_cookie(jar.clone(), FlowKind::DiscordLink);
    (cleared, finish_link(&state, &jar, query).await).into_response()
}

async fn finish_link(state: &AppState, jar: &CookieJar, query: CallbackQuery) -> Response {
    let Some(config) = state.discord.as_ref() else {
        return (StatusCode::NOT_FOUND, "Discord is not configured").into_response();
    };
    // Single-use: consume the row, rejecting an unknown or expired handshake.
    let flow = oauth_flow::consume(&state.db, FlowKind::DiscordLink, &query.state).await;
    let Ok(Some(flow)) = flow else {
        return (StatusCode::BAD_REQUEST, "that link request expired").into_response();
    };
    let Some(user_id) = flow.link_user_id else {
        return (StatusCode::BAD_REQUEST, "that link request expired").into_response();
    };
    if !oauth_flow::signed_in_as(&state.db, jar, user_id).await {
        return (
            StatusCode::BAD_REQUEST,
            "that link was started from an account you are not signed in to",
        )
            .into_response();
    }

    let http = crate::user_agent::client();
    let token: Result<TokenResponse, _> = async {
        http.post(format!("{API}/oauth2/token"))
            .form(&[
                ("client_id", config.client_id.as_str()),
                ("client_secret", config.client_secret.as_str()),
                ("grant_type", "authorization_code"),
                ("code", query.code.as_str()),
                ("redirect_uri", config.redirect_uri.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    .await;
    let Ok(token) = token else {
        return (StatusCode::BAD_GATEWAY, "Discord refused the code").into_response();
    };

    let user: Result<DiscordUser, _> = async {
        http.get(format!("{API}/users/@me"))
            .bearer_auth(&token.access_token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    .await;
    let Ok(user) = user else {
        return (StatusCode::BAD_GATEWAY, "Discord would not say who you are").into_response();
    };

    // One Discord account per WormholeSystems user and the reverse: both are how "which person is
    // this" is answered, and two answers is no answer.
    let stored = sqlx::query!(
        "insert into discord_accounts (user_id, discord_user_id, username, display_name, avatar)
         values ($1, $2, $3, $4, $5)
         on conflict (user_id) do update set
             discord_user_id = excluded.discord_user_id,
             username = excluded.username,
             display_name = excluded.display_name,
             avatar = excluded.avatar,
             updated_at = now()",
        user_id,
        user.id,
        user.username,
        user.global_name,
        user.avatar,
    )
    .execute(&state.db)
    .await;
    if stored.is_err() {
        return (
            StatusCode::CONFLICT,
            "that Discord account is already linked to someone else",
        )
            .into_response();
    }
    Redirect::to("/settings/discord?linked=1").into_response()
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

/// Forget the link, and stop anything that depended on it.
pub async fn unlink(pool: &sqlx::PgPool, user_id: i64) {
    let _ = sqlx::query!("delete from discord_accounts where user_id = $1", user_id)
        .execute(pool)
        .await;
    // A direct message with nobody to send it to is not a failure to retry, it is an alert
    // that cannot work until they link again.
    let stranded = sqlx::query!(
        "update map_alerts set
             is_active = false, disabled_at = now(), disabled_reason = 'discord_unlinked',
             updated_at = now()
         where created_by_user_id = $1 and is_active and delivery = 'discord_dm'
         returning id, map_id",
        user_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for row in stranded {
        crate::alerts::log(
            pool,
            Some(row.id),
            row.map_id,
            Some(user_id),
            "disabled",
            Some("discord_unlinked"),
        )
        .await;
    }
}
