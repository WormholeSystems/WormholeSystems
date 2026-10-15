//! With an OTLP endpoint configured, a request's span reaches it named after the route
//! template, and the path asked for does not. Its own binary: it installs the process's
//! subscriber.
#![cfg(feature = "otel")]

mod common;

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{Request, StatusCode};
use axum::routing::post;
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn a_request_span_is_exported_by_its_route(pool: PgPool) {
    let seen: Arc<Mutex<Vec<Bytes>>> = Arc::default();
    let keep = seen.clone();
    let collector = Router::new().route(
        "/v1/traces",
        post(move |body: Bytes| {
            keep.lock().unwrap().push(body);
            async { StatusCode::OK }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, collector).await.unwrap() });
    // SAFETY: set before anything reads the environment, and this binary has one test.
    unsafe { std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", &endpoint) };

    let telemetry = wormholesystems::telemetry::init();
    common::app::app(&pool)
        .oneshot(
            Request::get("/api/maps/424242")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Dropping it flushes the batch; the collector answers on another worker meanwhile.
    tokio::task::spawn_blocking(move || drop(telemetry))
        .await
        .unwrap();

    let seen = seen.lock().unwrap();
    let body: Vec<u8> = seen.iter().flat_map(|b| b.to_vec()).collect();
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("GET /api/maps/{id}"), "{text}");
    assert!(text.contains("wormholesystems-api"), "{text}");
    assert!(!text.contains("424242"), "{text}");
}
