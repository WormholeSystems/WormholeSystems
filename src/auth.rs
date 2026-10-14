use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::PgTokenStore;
use crate::esi::scopes::Scope;
use crate::esi::token::TokenStore;
use crate::esi::{EsiClient, Sso};
use crate::session;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<Auth>,
    pub db: sqlx::PgPool,
    pub hub: crate::maps::MapHub,
    pub user_hub: crate::user_channel::UserHub,
    pub grid: crate::maps::GridConfig,
    /// The latest Tranquility status, for the header indicator.
    pub server: crate::server_status::ServerWatch,
    /// `None` when no Discord application is configured, which disables the bot half.
    pub discord: Option<Arc<crate::config::DiscordConfig>>,
    /// Whether this deployment is served over HTTPS, taken from the callback URL EVE was
    /// given. The session cookie is marked `Secure` when it is, so a browser will not send
    /// it in the clear; local development over plain http would never receive it back.
    pub secure_cookies: bool,
    /// Where the per-viewer map background images live.
    pub backgrounds: crate::maps::BackgroundStore,
    /// Who may sign in. Checked at the SSO callback only, so it decides who gets a session
    /// and whose characters join an account; a session already open runs out as usual.
    pub allowed_logins: crate::allowlist::AffiliationAllowlist,
    /// Who may create maps, independent of who may log in: an instance can admit a whole
    /// alliance while only its leadership creates maps.
    pub map_creators: crate::allowlist::AffiliationAllowlist,
    /// The site-wide banner, if the operator set one.
    pub announcement: Option<Arc<crate::announcement::Announcement>>,
}

pub struct Auth {
    sso: Arc<Sso>,
    esi: EsiClient,
}

impl Auth {
    pub fn new(sso: Arc<Sso>, esi: EsiClient) -> Self {
        Auth { sso, esi }
    }

    pub fn sso(&self) -> &Arc<Sso> {
        &self.sso
    }

    pub fn esi(&self) -> &EsiClient {
        &self.esi
    }
}

#[derive(Deserialize)]
pub struct LoginQuery {
    /// `?link=true` adds the authenticated character to the currently signed-in user
    /// instead of resolving/creating an account.
    #[serde(default)]
    link: bool,
    /// Comma-separated ESI scopes to ask for on top of what the signed-in character has
    /// already granted. Unknown names are ignored rather than rejected.
    #[serde(default)]
    scopes: Option<String>,
    /// Where to land afterwards, so topping up permissions returns to the map that asked.
    #[serde(default)]
    return_to: Option<String>,
}

/// Only our own pages. An absolute URL, or a protocol-relative `//host` that a browser
/// would treat as one, would turn this into an open redirect.
fn safe_return_to(path: &str) -> Option<String> {
    let ok = path.starts_with('/') && !path.starts_with("//") && !path.contains('\\');
    ok.then(|| path.to_string())
}

/// `GET /auth/login`: record a one-time CSRF `state` and redirect to the EVE SSO. With
/// `?link=true` and an active session, the new character links to the current user;
/// `?scopes=` re-consents for more permissions on the character already signed in.
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<LoginQuery>,
) -> Response {
    let actor = match jar.get(session::SESSION_COOKIE) {
        Some(cookie) => session::actor_for_session(&state.db, cookie.value())
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let link_user_id: Option<i64> = if query.link {
        actor.as_ref().map(|actor| actor.user_id)
    } else {
        None
    };

    // Everything the acting character already consented to, plus whatever was asked for.
    // SSO replaces the token wholesale, so leaving the existing scopes out of the request
    // would revoke them.
    let mut scopes: Vec<Scope> = Vec::new();
    if let (Some(requested), Some(actor)) = (query.scopes.as_deref(), actor.as_ref()) {
        for scope in granted_scopes(&state.db, actor.character_id).await {
            scopes.push(scope);
        }
        for name in requested.split(',') {
            if let Some(scope) = Scope::parse(name.trim())
                && !scopes.contains(&scope)
            {
                scopes.push(scope);
            }
        }
    }

    let return_to = query.return_to.as_deref().and_then(safe_return_to);
    let csrf = Uuid::new_v4().to_string();
    if let Err(err) = sqlx::query!(
        "insert into oauth_login_flows (state, link_user_id, redirect_to, expires_at)
         values ($1, $2, $3, now() + interval '10 minutes')",
        csrf,
        link_user_id,
        return_to,
    )
    .execute(&state.db)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not start login: {err}"),
        )
            .into_response();
    }
    let url = if scopes.is_empty() {
        state.auth.sso.authorize_url(&csrf)
    } else {
        state.auth.sso.authorize_url_for(&csrf, &scopes)
    };
    Redirect::to(&url).into_response()
}

/// The ESI scopes a character has consented to, across every token we hold for them.
pub async fn granted_scopes(db: &sqlx::PgPool, character_id: i64) -> Vec<Scope> {
    let names = sqlx::query_scalar!(
        "select distinct s.name
         from esi_tokens t
         join esi_token_scopes ts on ts.token_id = t.id
         join esi_scopes s on s.id = ts.scope_id
         where t.character_id = $1",
        character_id,
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    names.iter().filter_map(|n| Scope::parse(n)).collect()
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

/// `GET /auth/callback`: validate the handshake, persist the character and token, open a
/// session, set the cookie, and redirect into the app.
pub async fn callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let auth = &state.auth;

    // The `state` is single-use and short-lived: consume it, rejecting unknown/expired.
    let flow = match sqlx::query!(
        "delete from oauth_login_flows where state = $1 and expires_at > now()
         returning link_user_id, redirect_to",
        query.state,
    )
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(flow)) => flow,
        Ok(None) => return (StatusCode::BAD_REQUEST, "invalid or expired state").into_response(),
        Err(err) => return server_error(err),
    };

    let (token, claims) = match auth.sso.exchange_code(&query.code).await {
        Ok(pair) => pair,
        Err(err) => {
            return (StatusCode::BAD_GATEWAY, format!("login failed: {err}")).into_response();
        }
    };

    // Corp/alliance drive access checks, so they're required and refreshed on every login.
    // We also cache the entity rows (name/ticker) the character's deferred FKs reference.
    let affiliation = auth
        .esi
        .affiliation(&[claims.character_id])
        .await
        .ok()
        .and_then(|a| a.into_iter().next());
    let Some(affiliation) = affiliation else {
        return (
            StatusCode::BAD_GATEWAY,
            "could not resolve character affiliation",
        )
            .into_response();
    };

    // Before anything is stored: a refused character gets no account, no token and no
    // session, and is not added to the account it was being linked to either.
    if !state.allowed_logins.allows(
        claims.character_id,
        Some(affiliation.corporation_id),
        affiliation.alliance_id,
    ) {
        return Redirect::to(refused_destination(flow.link_user_id.is_some())).into_response();
    }

    let Ok(corp) = auth.esi.corporation(affiliation.corporation_id).await else {
        return (StatusCode::BAD_GATEWAY, "could not resolve corporation").into_response();
    };
    let corporation = session::Entity {
        id: affiliation.corporation_id,
        name: corp.name,
        ticker: corp.ticker,
    };
    let alliance = match affiliation.alliance_id {
        Some(alliance_id) => match auth.esi.alliance(alliance_id).await {
            Ok(a) => Some(session::Entity {
                id: alliance_id,
                name: a.name,
                ticker: a.ticker,
            }),
            Err(_) => {
                return (StatusCode::BAD_GATEWAY, "could not resolve alliance").into_response();
            }
        },
        None => None,
    };

    let user_id = match session::persist_identity(
        &state.db,
        &claims,
        corporation,
        alliance,
        flow.link_user_id,
    )
    .await
    {
        Ok(user_id) => user_id,
        Err(err) => return server_error(err),
    };

    // Persist the ESI token for this character (refresh token is the sensitive credential).
    if let Err(err) = PgTokenStore::new(state.db.clone())
        .save(claims.character_id, &token)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not store token: {err}"),
        )
            .into_response();
    }

    let session_id = match session::create_session(&state.db, user_id, claims.character_id).await {
        Ok(id) => id,
        Err(err) => return server_error(err),
    };

    let cookie = Cookie::build((session::SESSION_COOKIE, session_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(state.secure_cookies)
        .build();
    let destination = flow.redirect_to.unwrap_or_else(|| "/".to_string());
    (jar.add(cookie), Redirect::to(&destination)).into_response()
}

/// Where a character the instance does not let in is sent, with a flag the page turns into
/// an explanation. Adding an alt starts from the characters page and returns there; a
/// plain login has no page of its own to go back to but the login page.
fn refused_destination(linking: bool) -> &'static str {
    if linking {
        "/settings/characters?refused=1"
    } else {
        "/login?refused=1"
    }
}

/// `GET /auth/logout`: end the session and clear the cookie.
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get(session::SESSION_COOKIE) {
        let _ = session::delete_session(&state.db, cookie.value()).await;
    }
    (
        jar.remove(Cookie::from(session::SESSION_COOKIE)),
        Redirect::to("/"),
    )
        .into_response()
}

fn server_error(err: sqlx::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("database error: {err}"),
    )
        .into_response()
}
