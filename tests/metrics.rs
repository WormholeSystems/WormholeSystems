//! The Prometheus page labels requests by route template, never by the path asked for, so
//! an id never becomes a label. One test: the recorder is per process.

mod common;

use axum::body::Body;
use axum::http::Request;
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
async fn requests_are_counted_by_route_template(pool: PgPool) {
    let handle = wormholesystems::telemetry::metrics::install();
    for path in ["/api/healthz", "/api/maps/987654", "/nowhere/987654"] {
        common::app::app(&pool)
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
    }
    let page = handle.render();
    let line = |route: &str, status: &str| {
        format!("ws_http_requests_total{{method=\"GET\",route=\"{route}\",status=\"{status}\"}} 1")
    };
    assert!(page.contains(&line("/api/healthz", "2xx")), "{page}");
    assert!(page.contains(&line("/api/maps/{id}", "4xx")), "{page}");
    assert!(page.contains(&line("unmatched", "4xx")), "{page}");
    assert!(!page.contains("987654"), "{page}");
    assert!(
        page.contains("ws_http_request_duration_seconds_bucket"),
        "{page}"
    );
}
