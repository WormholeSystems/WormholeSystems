//! `GET /api/instance`: what the deployment has switched on, and the operator's banner.

mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use common::app::{get, state};
use sqlx::PgPool;
use wormholesystems::announcement::{Announcement, AnnouncementVars};

// Compression is for the whole API; the OpenAPI document is just a reply big enough to get it.
#[sqlx::test]
async fn replies_are_compressed_when_the_client_asks(pool: PgPool) {
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    let request = |encoding: Option<&str>| {
        let mut req = Request::get("/api/openapi.json");
        if let Some(encoding) = encoding {
            req = req.header(header::ACCEPT_ENCODING, encoding);
        }
        req.body(Body::empty()).unwrap()
    };
    let app = common::app::app(&pool);

    for (asked, got) in [
        (Some("gzip"), Some("gzip")),
        (Some("zstd"), Some("zstd")),
        (None, None),
    ] {
        let res = app.clone().oneshot(request(asked)).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()
                .get(header::CONTENT_ENCODING)
                .map(|v| v.to_str().unwrap()),
            got,
        );
    }
}

#[sqlx::test]
async fn no_announcement_is_null(pool: PgPool) {
    let res = get(common::app::app(&pool), "/api/instance", None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body["announcement"].is_null());
}

// Anonymous visitors get it too: the landing page shows it.
#[sqlx::test]
async fn the_announcement_is_served_to_everyone(pool: PgPool) {
    let mut state = state(&pool);
    state.announcement = Announcement::from_vars(AnnouncementVars {
        level: Some("warning"),
        title: Some("Moving to a new address"),
        message: Some("This page moves on October 1st."),
        link_url: Some("https://example.com"),
        link_label: Some("Read more"),
        dismissible: Some("false"),
    })
    .map(Arc::new);
    let app = wormholesystems::api::router().with_state(state);

    let res = get(app, "/api/instance", None).await;
    let a = &res.body["announcement"];
    assert_eq!(a["level"], "warning");
    assert_eq!(a["title"], "Moving to a new address");
    assert_eq!(a["message"], "This page moves on October 1st.");
    assert_eq!(a["link"]["url"], "https://example.com");
    assert_eq!(a["link"]["label"], "Read more");
    assert_eq!(a["dismissible"], false);
    assert_eq!(a["id"].as_str().map(str::len), Some(12));
}

// What Docker's healthcheck asks: up while the database answers, and 503 once it does not.
#[sqlx::test]
async fn healthz_follows_the_database(pool: PgPool) {
    use axum::extract::State;

    let (status, _) = wormholesystems::app::healthz(State(state(&pool))).await;
    assert_eq!(status, StatusCode::OK);

    pool.close().await;
    let (status, _) = wormholesystems::app::healthz(State(state(&pool))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

// The proxies only forward /api/ and friends, so an uptime check from outside asks this one.
#[sqlx::test]
async fn healthz_answers_under_api_too(pool: PgPool) {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let res = common::app::app(&pool)
        .oneshot(
            Request::get("/api/healthz")
                .header("accept-encoding", "gzip")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().get("content-encoding").is_none());
}
