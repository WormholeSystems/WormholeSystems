//! Personal access tokens over HTTP: minting, the one-time secret, bearer authentication
//! on the API, and the ways a token stops working.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::app::{TestResponse, app, get, request_json, send, session_cookie};
use common::{member_with_role, world};
use serde_json::json;
use sqlx::PgPool;
use wormholesystems::maps::Role;

async fn bearer_get(app: Router, path: &str, token: &str) -> TestResponse {
    let req = Request::builder()
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    send(app, req).await
}

async fn bearer_json(
    app: Router,
    method: &str,
    path: &str,
    token: &str,
    body: serde_json::Value,
) -> TestResponse {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    send(app, req).await
}

async fn mint(pool: &PgPool, cookie: &str, name: &str) -> (i64, String) {
    let created = request_json(
        app(pool),
        "POST",
        "/api/me/tokens",
        Some(cookie),
        json!({ "name": name }),
    )
    .await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    (
        created.body["token"]["id"].as_i64().unwrap(),
        created.body["plaintext"].as_str().unwrap().to_string(),
    )
}

#[sqlx::test]
async fn the_secret_is_shown_once_and_only_its_hash_is_kept(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;

    let (id, plaintext) = mint(&pool, &cookie, "my script").await;
    assert!(plaintext.starts_with("wst_"));

    let stored: String =
        sqlx::query_scalar("select token_hash from personal_access_tokens where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(stored, plaintext);
    assert_eq!(stored, wormholesystems::tokens::hash(&plaintext));

    let listed = get(app(&pool), "/api/me/tokens", Some(&cookie)).await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.body[0]["name"], "my script");
    assert!(listed.body[0].get("plaintext").is_none());
    assert!(listed.body[0].get("token_hash").is_none());
    assert!(listed.body[0]["last_used_at"].is_null());
}

#[sqlx::test]
async fn a_blank_name_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let refused = request_json(
        app(&pool),
        "POST",
        "/api/me/tokens",
        Some(&cookie),
        json!({ "name": "  " }),
    )
    .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn a_bearer_token_acts_as_its_user(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let (id, plaintext) = mint(&pool, &cookie, "script").await;

    let me = bearer_get(app(&pool), "/api/me", &plaintext).await;
    assert_eq!(me.status, StatusCode::OK);
    assert_eq!(me.body["character_id"], w.owner.character_id);

    let maps = bearer_get(app(&pool), "/api/maps", &plaintext).await;
    assert_eq!(maps.status, StatusCode::OK);
    assert_eq!(maps.body[0]["id"], w.map_id);
    assert_eq!(maps.body[0]["name"], "Chain");

    let used: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("select last_used_at from personal_access_tokens where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used.is_some());
}

#[sqlx::test]
async fn a_bad_token_is_refused_rather_than_treated_as_a_guest(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;

    let wrong = bearer_get(app(&pool), "/api/me", "wst_nonsense").await;
    assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);

    // A public map is readable by a guest, but not by a client waving a bad token.
    sqlx::query("update maps set is_public = true where id = $1")
        .bind(w.map_id)
        .execute(&pool)
        .await
        .unwrap();
    let path = format!("/api/maps/{}", w.map_id);
    let guest = get(app(&pool), &path, None).await;
    assert_eq!(guest.status, StatusCode::OK);
    let bad = bearer_get(app(&pool), &path, "wst_nonsense").await;
    assert_eq!(bad.status, StatusCode::UNAUTHORIZED);

    let (id, plaintext) = mint(&pool, &cookie, "script").await;
    assert_eq!(
        bearer_get(app(&pool), "/api/me", &plaintext).await.status,
        StatusCode::OK
    );

    sqlx::query(
        "update personal_access_tokens set expires_at = now() - interval '1 minute' where id = $1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let expired = bearer_get(app(&pool), "/api/me", &plaintext).await;
    assert_eq!(expired.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn revoking_locks_the_token_out_and_only_the_owner_may_revoke(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let (id, plaintext) = mint(&pool, &cookie, "script").await;

    let stranger = member_with_role(&pool, w.owner, w.map_id, 1002, 2002, Role::Viewer).await;
    let stranger_cookie = session_cookie(&pool, stranger).await;
    let path = format!("/api/me/tokens/{id}");
    let not_theirs = request_json(
        app(&pool),
        "DELETE",
        &path,
        Some(&stranger_cookie),
        json!({}),
    )
    .await;
    assert_eq!(not_theirs.status, StatusCode::NOT_FOUND);
    assert_eq!(
        bearer_get(app(&pool), "/api/me", &plaintext).await.status,
        StatusCode::OK
    );

    let revoked = request_json(app(&pool), "DELETE", &path, Some(&cookie), json!({})).await;
    assert_eq!(revoked.status, StatusCode::OK);
    assert_eq!(
        bearer_get(app(&pool), "/api/me", &plaintext).await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test]
async fn tokens_are_managed_from_a_session_only(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let (_, plaintext) = mint(&pool, &cookie, "script").await;

    let listed = bearer_get(app(&pool), "/api/me/tokens", &plaintext).await;
    assert_eq!(listed.status, StatusCode::UNAUTHORIZED);
    let minted = bearer_json(
        app(&pool),
        "POST",
        "/api/me/tokens",
        &plaintext,
        json!({ "name": "another" }),
    )
    .await;
    assert_eq!(minted.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn expiry_is_days_from_now(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let created = request_json(
        app(&pool),
        "POST",
        "/api/me/tokens",
        Some(&cookie),
        json!({ "name": "short lived", "expires_in_days": 30 }),
    )
    .await;
    assert_eq!(created.status, StatusCode::OK);
    let expires: chrono::DateTime<chrono::Utc> =
        serde_json::from_value(created.body["token"]["expires_at"].clone()).unwrap();
    let days = (expires - chrono::Utc::now()).num_days();
    assert!((29..=30).contains(&days), "{days}");

    let past = request_json(
        app(&pool),
        "POST",
        "/api/me/tokens",
        Some(&cookie),
        json!({ "name": "already dead", "expires_in_days": 0 }),
    )
    .await;
    assert_eq!(past.status, StatusCode::BAD_REQUEST);
}
