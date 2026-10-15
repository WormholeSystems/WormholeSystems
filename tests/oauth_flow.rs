//! The OAuth `state` table shared by the EVE login and Discord linking: single use, short
//! lived, and only good for the kind of handshake that minted it.

mod common;

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use sqlx::PgPool;
use wormholesystems::config::DiscordConfig;
use wormholesystems::oauth_flow::{FlowKind, consume, start};

async fn flows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from oauth_login_flows")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Both callbacks on the real handlers, with Discord configured so its callback gets as
/// far as reading the state.
fn callbacks(pool: &PgPool) -> Router {
    let mut state = common::app::state(pool);
    state.discord = Some(Arc::new(DiscordConfig {
        client_id: "test".into(),
        client_secret: "test".into(),
        redirect_uri: "http://localhost/discord/callback".into(),
        public_key: "00".into(),
        bot_token: None,
    }));
    Router::new()
        .route("/auth/login", get(wormholesystems::auth::login))
        .route("/auth/callback", get(wormholesystems::auth::callback))
        .route(
            "/discord/callback",
            get(wormholesystems::discord::link::callback),
        )
        .with_state(state)
}

#[sqlx::test]
async fn a_state_is_consumed_once_and_carries_what_the_start_left(pool: PgPool) {
    let user = common::new_user(&pool).await;
    let state = start(&pool, FlowKind::EveLogin, Some(user), Some("/maps/1"))
        .await
        .unwrap();

    let flow = consume(&pool, FlowKind::EveLogin, &state)
        .await
        .unwrap()
        .expect("the flow it just started");
    assert_eq!(flow.link_user_id, Some(user));
    assert_eq!(flow.redirect_to.as_deref(), Some("/maps/1"));

    // A replay finds nothing.
    assert!(
        consume(&pool, FlowKind::EveLogin, &state)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test]
async fn a_state_is_only_good_for_the_kind_that_started_it(pool: PgPool) {
    let user = common::new_user(&pool).await;
    let eve = start(&pool, FlowKind::EveLogin, None, None).await.unwrap();
    let discord = start(&pool, FlowKind::DiscordLink, Some(user), None)
        .await
        .unwrap();

    assert!(
        consume(&pool, FlowKind::DiscordLink, &eve)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        consume(&pool, FlowKind::EveLogin, &discord)
            .await
            .unwrap()
            .is_none()
    );

    // The wrong callback leaves them alone, so each still works where it belongs.
    assert!(
        consume(&pool, FlowKind::EveLogin, &eve)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        consume(&pool, FlowKind::DiscordLink, &discord)
            .await
            .unwrap()
            .is_some()
    );
}

#[sqlx::test]
async fn each_callback_refuses_the_other_ones_state(pool: PgPool) {
    let user = common::new_user(&pool).await;
    let eve = start(&pool, FlowKind::EveLogin, None, None).await.unwrap();
    let discord = start(&pool, FlowKind::DiscordLink, Some(user), None)
        .await
        .unwrap();

    // Each carrying its own browser cookie, so it is the kind that refuses them.
    let refused = common::app::get(
        callbacks(&pool),
        &format!("/auth/callback?code=x&state={discord}"),
        Some(&format!("ws_oauth_eve={discord}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/discord/callback?code=x&state={eve}"),
        Some(&format!("ws_oauth_discord={eve}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    assert_eq!(flows(&pool).await, 2);
}

/// The `state` a start handler put into the provider URL it redirected to.
fn state_in(location: &str) -> String {
    let url = reqwest::Url::parse(location).unwrap();
    url.query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .unwrap()
}

/// The `name=value` part of the `Set-Cookie` for `name`.
fn cookie_pair(set_cookies: &[String], name: &str) -> String {
    set_cookies
        .iter()
        .find(|c| c.starts_with(&format!("{name}=")))
        .map(|c| c.split(';').next().unwrap().to_string())
        .unwrap_or_else(|| panic!("no {name} cookie in {set_cookies:?}"))
}

#[sqlx::test]
async fn a_callback_without_the_starting_browsers_cookie_is_refused(pool: PgPool) {
    let state = start(&pool, FlowKind::EveLogin, None, None).await.unwrap();

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/auth/callback?code=x&state={state}"),
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    // Not consumed: a forged callback cannot spend the real browser's login.
    assert_eq!(flows(&pool).await, 1);
}

#[sqlx::test]
async fn a_callback_whose_cookie_names_another_state_is_refused(pool: PgPool) {
    let user = common::new_user(&pool).await;
    let mine = start(&pool, FlowKind::EveLogin, None, None).await.unwrap();
    let theirs = start(&pool, FlowKind::EveLogin, None, None).await.unwrap();
    let discord_mine = start(&pool, FlowKind::DiscordLink, Some(user), None)
        .await
        .unwrap();
    let discord_theirs = start(&pool, FlowKind::DiscordLink, Some(user), None)
        .await
        .unwrap();

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/auth/callback?code=x&state={theirs}"),
        Some(&format!("ws_oauth_eve={mine}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/discord/callback?code=x&state={discord_theirs}"),
        Some(&format!("ws_oauth_discord={discord_mine}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    assert_eq!(flows(&pool).await, 4);
}

#[sqlx::test]
async fn a_link_finished_by_another_account_is_refused(pool: PgPool) {
    let w = common::world(&pool).await;
    let other = common::new_user(&pool).await;
    common::add_character(&pool, other, 1002, 2002, None).await;
    let other_session = common::app::session_cookie(
        &pool,
        wormholesystems::maps::Actor {
            user_id: other,
            character_id: 1002,
        },
    )
    .await;

    // Both handshakes were started for the owner, but the browser finishing them is
    // signed in as somebody else.
    let eve = start(&pool, FlowKind::EveLogin, Some(w.owner.user_id), None)
        .await
        .unwrap();
    let refused = common::app::get(
        callbacks(&pool),
        &format!("/auth/callback?code=x&state={eve}"),
        Some(&format!("ws_oauth_eve={eve}; {other_session}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let discord = start(&pool, FlowKind::DiscordLink, Some(w.owner.user_id), None)
        .await
        .unwrap();
    let refused = common::app::get(
        callbacks(&pool),
        &format!("/discord/callback?code=x&state={discord}"),
        Some(&format!("ws_oauth_discord={discord}")),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert_eq!(flows(&pool).await, 0);
}

#[sqlx::test]
async fn the_browser_that_started_a_login_gets_past_the_state_check(pool: PgPool) {
    let w = common::world(&pool).await;
    let session = common::app::session_cookie(&pool, w.owner).await;

    for (start_path, signed_in) in [("/auth/login", false), ("/auth/login?link=true", true)] {
        let cookie = signed_in.then_some(session.as_str());
        let started = common::app::get(callbacks(&pool), start_path, cookie).await;
        assert_eq!(started.status, StatusCode::SEE_OTHER);
        let state = state_in(started.location().unwrap());
        let handshake = cookie_pair(&started.set_cookies(), "ws_oauth_eve");
        assert_eq!(handshake, format!("ws_oauth_eve={state}"));
        let set = started.set_cookies().join("\n");
        assert!(set.contains("HttpOnly") && set.contains("SameSite=Lax"));

        let cookies = match cookie {
            Some(session) => format!("{handshake}; {session}"),
            None => handshake,
        };
        let finished = common::app::get(
            callbacks(&pool),
            &format!("/auth/callback?code=x&state={state}"),
            Some(&cookies),
        )
        .await;
        // The stub SSO cannot exchange the code: that is as far as a test login gets, and
        // it is past every check on the state.
        assert_eq!(finished.status, StatusCode::BAD_GATEWAY);
        assert_eq!(flows(&pool).await, 0);
        assert!(
            finished
                .set_cookies()
                .iter()
                .any(|c| c.starts_with("ws_oauth_eve=;") || c.starts_with("ws_oauth_eve=\"\"")),
            "the handshake cookie is cleared: {:?}",
            finished.set_cookies()
        );
    }
}

#[sqlx::test]
async fn starting_a_flow_clears_the_abandoned_ones(pool: PgPool) {
    sqlx::query(
        "insert into oauth_login_flows (state, kind, expires_at)
         values ('abandoned', 'eve_login', now() - interval '1 minute')",
    )
    .execute(&pool)
    .await
    .unwrap();

    start(&pool, FlowKind::EveLogin, None, None).await.unwrap();

    let left: Vec<String> = sqlx::query_scalar("select state from oauth_login_flows")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left.len(), 1);
    assert_ne!(left[0], "abandoned");
}
