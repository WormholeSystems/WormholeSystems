//! The per-viewer background image over HTTP: an upload becomes a URL in the settings,
//! the file comes back as itself, a non-image is refused, and removal clears both.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use common::app::{app, get, request_json, session_cookie};
use common::world;
use sqlx::PgPool;
use tower::ServiceExt;

const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01";

fn multipart(name: &str, bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = "ws-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"x.png\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

async fn upload(
    pool: &PgPool,
    cookie: &str,
    map_id: i64,
    name: &str,
    bytes: &[u8],
) -> common::app::TestResponse {
    let (content_type, body) = multipart(name, bytes);
    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/maps/{map_id}/background-image"))
        .header(header::CONTENT_TYPE, content_type)
        .header(header::COOKIE, cookie)
        .body(Body::from(body))
        .unwrap();
    common::app::send(app(pool), req).await
}

async fn fetch_image(
    pool: &PgPool,
    cookie: &str,
    map_id: i64,
) -> (StatusCode, Option<String>, Vec<u8>) {
    let req = Request::builder()
        .uri(format!("/api/maps/{map_id}/background-image"))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();
    let response = app(pool).oneshot(req).await.unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, content_type, bytes.to_vec())
}

#[sqlx::test]
async fn an_upload_becomes_a_url_and_the_file_comes_back_as_itself(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;
    let settings_path = format!("/api/maps/{}/settings/user", w.map_id);

    let before = get(app(&pool), &settings_path, Some(&cookie)).await;
    assert_eq!(before.body["background_image_url"], serde_json::Value::Null);
    assert_eq!(before.body["background_image_mode"], "grid");
    let (missing, _, _) = fetch_image(&pool, &cookie, w.map_id).await;
    assert_eq!(missing, StatusCode::NOT_FOUND);

    let saved = upload(&pool, &cookie, w.map_id, "image", PNG).await;
    assert_eq!(saved.status, StatusCode::OK, "{:?}", saved.body);
    let url = saved.body["background_image_url"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(url.starts_with(&format!("/api/maps/{}/background-image?v=", w.map_id)));

    let (status, content_type, bytes) = fetch_image(&pool, &cookie, w.map_id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type.as_deref(), Some("image/png"));
    assert_eq!(bytes, PNG);

    // A replacement is a different URL, so a cached copy of the old one is never shown.
    let replaced = upload(&pool, &cookie, w.map_id, "image", PNG).await;
    assert_eq!(replaced.status, StatusCode::OK);
    assert_ne!(replaced.body["background_image_url"].as_str().unwrap(), url);

    let mode = request_json(
        app(&pool),
        "POST",
        &settings_path,
        Some(&cookie),
        serde_json::json!({ "background_image_mode": "viewport" }),
    )
    .await;
    assert_eq!(mode.body["background_image_mode"], "viewport");
    assert!(mode.body["background_image_url"].is_string());

    let removed = request_json(
        app(&pool),
        "DELETE",
        &format!("/api/maps/{}/background-image", w.map_id),
        Some(&cookie),
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(removed.status, StatusCode::OK);
    assert_eq!(
        removed.body["background_image_url"],
        serde_json::Value::Null
    );
    let (gone, _, _) = fetch_image(&pool, &cookie, w.map_id).await;
    assert_eq!(gone, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_file_that_is_not_an_image_or_the_wrong_part_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let cookie = session_cookie(&pool, w.owner).await;

    let refused = upload(&pool, &cookie, w.map_id, "image", b"<svg xmlns='x'/>").await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);

    let misnamed = upload(&pool, &cookie, w.map_id, "file", PNG).await;
    assert_eq!(misnamed.status, StatusCode::BAD_REQUEST);

    let settings = get(
        app(&pool),
        &format!("/api/maps/{}/settings/user", w.map_id),
        Some(&cookie),
    )
    .await;
    assert_eq!(
        settings.body["background_image_url"],
        serde_json::Value::Null
    );
}

#[sqlx::test]
async fn a_stranger_can_neither_upload_nor_see_one(pool: PgPool) {
    let w = world(&pool).await;
    let stranger_user = common::new_user(&pool).await;
    common::add_character(&pool, stranger_user, 4242, 9001, None).await;
    let cookie = session_cookie(
        &pool,
        wormholesystems::maps::Actor {
            user_id: stranger_user,
            character_id: 4242,
        },
    )
    .await;

    let refused = upload(&pool, &cookie, w.map_id, "image", PNG).await;
    assert_ne!(refused.status, StatusCode::OK);
    let (status, _, _) = fetch_image(&pool, &cookie, w.map_id).await;
    assert_ne!(status, StatusCode::OK);
}
