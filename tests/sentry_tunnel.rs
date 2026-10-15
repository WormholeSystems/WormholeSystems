//! The browser's error reports go through `/api/sentry-tunnel` to the web DSN, and nowhere
//! else: not without a DSN, not to another host or project, not in bulk.

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{Request, StatusCode};
use axum::routing::post;
use sqlx::PgPool;
use tower::ServiceExt;
use wormholesystems::telemetry::tunnel::{BODY_LIMIT, Tunnel};

/// A stand-in Sentry that keeps what it was sent, and the DSN that points at it.
async fn collector() -> (String, Arc<Mutex<Vec<(String, Bytes)>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let keep = seen.clone();
    let app = Router::new().route(
        "/api/{project}/envelope/",
        post(
            move |axum::extract::Path(project): axum::extract::Path<String>, body: Bytes| {
                keep.lock().unwrap().push((project, body));
                async { StatusCode::OK }
            },
        ),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://public@{addr}/7"), seen)
}

fn envelope(dsn: &str) -> String {
    format!("{{\"dsn\":\"{dsn}\"}}\n{{\"type\":\"event\"}}\n{{\"message\":\"boom\"}}")
}

async fn post_envelope(app: Router, body: impl Into<Body>) -> StatusCode {
    app.oneshot(
        Request::post("/api/sentry-tunnel")
            .header("content-type", "application/x-sentry-envelope")
            .body(body.into())
            .unwrap(),
    )
    .await
    .unwrap()
    .status()
}

fn app_with(pool: &PgPool, dsn: Option<&str>) -> Router {
    let mut state = common::app::state(pool);
    state.sentry_tunnel = dsn.map(|dsn| Arc::new(Tunnel::new(dsn).unwrap()));
    wormholesystems::app::router(state)
}

#[sqlx::test]
async fn without_a_dsn_there_is_no_tunnel(pool: PgPool) {
    let status = post_envelope(app_with(&pool, None), envelope("http://k@a/1")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_report_for_the_dsn_is_forwarded(pool: PgPool) {
    let (dsn, seen) = collector().await;
    let status = post_envelope(app_with(&pool, Some(&dsn)), envelope(&dsn)).await;
    assert_eq!(status, StatusCode::OK);
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, "7");
    assert_eq!(seen[0].1, envelope(&dsn).as_bytes());
}

#[sqlx::test]
async fn anything_else_is_refused(pool: PgPool) {
    let (dsn, seen) = collector().await;
    let other_project = dsn.replace("/7", "/8");
    for body in [
        envelope(&other_project),
        envelope("http://public@169.254.169.254/7"),
        "no header at all".to_string(),
    ] {
        let status = post_envelope(app_with(&pool, Some(&dsn)), body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let huge = format!("{}{}", envelope(&dsn), " ".repeat(BODY_LIMIT));
    let status = post_envelope(app_with(&pool, Some(&dsn)), huge).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(seen.lock().unwrap().is_empty());
}
