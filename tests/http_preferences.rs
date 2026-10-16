//! Display preferences over HTTP: kept on the account, the defaults until changed, and a
//! change to one leaving the others alone.

mod common;

use axum::http::StatusCode;
use common::app::{app, get, request_json, session_cookie};
use common::world;
use serde_json::json;
use sqlx::PgPool;
use wormholesystems::maps::Actor;

#[sqlx::test]
async fn preferences_start_as_the_defaults_and_keep_a_change(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;

    let fresh = get(app(&pool), "/api/me/preferences", Some(&cookie)).await;
    assert_eq!(fresh.status, StatusCode::OK);
    assert_eq!(fresh.body, json!({ "effect_icons": true }));

    let saved = request_json(
        app(&pool),
        "POST",
        "/api/me/preferences",
        Some(&cookie),
        json!({ "effect_icons": false }),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK, "{:?}", saved.body);
    assert_eq!(saved.body, json!({ "effect_icons": false }));

    // An empty change leaves what was saved.
    let unchanged = request_json(
        app(&pool),
        "POST",
        "/api/me/preferences",
        Some(&cookie),
        json!({}),
    )
    .await;
    assert_eq!(unchanged.body, json!({ "effect_icons": false }));

    let read = get(app(&pool), "/api/me/preferences", Some(&cookie)).await;
    assert_eq!(read.body, json!({ "effect_icons": false }));
}

/// They belong to the account, not the session: another sign-in, on another device, finds
/// the same choice.
#[sqlx::test]
async fn preferences_follow_the_account_to_another_session(pool: PgPool) {
    let w = world(&pool).await;
    let laptop = session_cookie(&pool, w.owner).await;
    request_json(
        app(&pool),
        "POST",
        "/api/me/preferences",
        Some(&laptop),
        json!({ "effect_icons": false }),
    )
    .await;

    let phone = session_cookie(&pool, w.owner).await;
    let read = get(app(&pool), "/api/me/preferences", Some(&phone)).await;
    assert_eq!(read.body, json!({ "effect_icons": false }));

    // Someone else's account is untouched.
    let other_user = common::new_user(&pool).await;
    common::add_character(&pool, other_user, 1002, 2001, None).await;
    let other = session_cookie(
        &pool,
        Actor {
            user_id: other_user,
            character_id: 1002,
        },
    )
    .await;
    let theirs = get(app(&pool), "/api/me/preferences", Some(&other)).await;
    assert_eq!(theirs.body, json!({ "effect_icons": true }));
}

#[sqlx::test]
async fn preferences_need_someone_signed_in(pool: PgPool) {
    world(&pool).await;
    let read = get(app(&pool), "/api/me/preferences", None).await;
    assert_eq!(read.status, StatusCode::UNAUTHORIZED);
    let write = request_json(
        app(&pool),
        "POST",
        "/api/me/preferences",
        None,
        json!({ "effect_icons": false }),
    )
    .await;
    assert_eq!(write.status, StatusCode::UNAUTHORIZED);
}
