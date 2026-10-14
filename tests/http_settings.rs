//! Per-map user settings over HTTP: the parts a bad body has to be refused on.

mod common;

use axum::http::StatusCode;
use common::app::{app, get, request_json, session_cookie};
use common::world;
use serde_json::json;
use sqlx::PgPool;

/// Only the caller's own characters can map for them: a stranger's id, or one that does
/// not exist, is refused rather than stored.
#[sqlx::test]
async fn tracked_pilots_must_be_the_callers_own(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let path = format!("/api/maps/{}/settings/user", w.map_id);

    let refused = request_json(
        app(&pool),
        "POST",
        &path,
        Some(&cookie),
        json!({ "tracked_character_ids": [w.owner.character_id, 9999] }),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let saved = request_json(
        app(&pool),
        "POST",
        &path,
        Some(&cookie),
        json!({ "tracked_character_ids": [w.owner.character_id] }),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK);
    assert_eq!(
        saved.body["tracked_character_ids"],
        json!([w.owner.character_id])
    );

    let read = get(app(&pool), &path, Some(&cookie)).await;
    assert_eq!(
        read.body["tracked_character_ids"],
        json!([w.owner.character_id])
    );

    // Clearing the set is a real value, not an absent field.
    let cleared = request_json(
        app(&pool),
        "POST",
        &path,
        Some(&cookie),
        json!({ "tracked_character_ids": [] }),
    )
    .await;
    assert_eq!(cleared.body["tracked_character_ids"], json!([]));
}

/// The defaults a user with no row is shown are the ones a fresh row gets from the columns,
/// so nothing about a map changes under someone the moment they first change a setting.
#[sqlx::test]
async fn the_missing_row_and_a_bare_row_read_the_same(pool: PgPool) {
    use wormholesystems::maps::user_settings::{MapUserSettings, insert_defaults, load};

    let w = world(&pool).await;
    let missing = load(&pool, w.map_id, w.owner.user_id).await.unwrap();
    insert_defaults(&pool, w.map_id, w.owner.user_id)
        .await
        .unwrap();
    let bare = load(&pool, w.map_id, w.owner.user_id).await.unwrap();

    let json = |s: &MapUserSettings| serde_json::to_value(s).unwrap();
    assert_eq!(json(&missing), json(&MapUserSettings::default()));
    assert_eq!(json(&bare), json(&missing));
}

/// A partial update leaves everything it does not name at the default, on a first change
/// as on any later one.
#[sqlx::test]
async fn a_first_change_sets_only_what_it_names(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let path = format!("/api/maps/{}/settings/user", w.map_id);
    let before = get(app(&pool), &path, Some(&cookie)).await.body;

    let saved = request_json(
        app(&pool),
        "POST",
        &path,
        Some(&cookie),
        json!({ "security_penalty": 80, "is_pinned": true }),
    )
    .await;
    assert_eq!(saved.status, StatusCode::OK);
    let mut expected = before.clone();
    expected["security_penalty"] = json!(80);
    expected["is_pinned"] = json!(true);
    assert_eq!(saved.body, expected);
}
