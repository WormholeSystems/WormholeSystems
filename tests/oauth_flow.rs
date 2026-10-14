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

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/auth/callback?code=x&state={discord}"),
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let refused = common::app::get(
        callbacks(&pool),
        &format!("/discord/callback?code=x&state={eve}"),
        None,
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    assert_eq!(flows(&pool).await, 2);
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
