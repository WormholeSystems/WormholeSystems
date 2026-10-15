//! Prometheus metrics, served on a listener of their own at `METRICS_ADDR` and never on the
//! public port. Off when that is unset: the macros below then record into nothing, so the
//! instrumented code costs next to nothing either way.
//!
//! Labels stay bounded: routes are the router's templates (`/api/maps/{id}`), never the
//! path asked for, statuses are classes, and nothing carries an id.

use std::future::Future;
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::{MatchedPath, Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::get;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use sqlx::PgPool;

/// Seconds, from a fast API read to a slow ESI call or a long background pass.
const BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0,
];

/// Install the recorder and serve `/metrics` on `addr` until shutdown.
///
/// Fails like the main listener does: an operator who asked for metrics on an address that
/// cannot be bound wants to hear it at startup.
pub async fn serve(addr: &str, pool: PgPool) {
    let handle = install();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("could not listen on {addr} for metrics: {err}"));
    tracing::info!("metrics on http://{addr}/metrics");

    // Histograms keep their samples until this drains them.
    let upkeep = handle.clone();
    crate::background::spawn_every("metrics upkeep", Duration::from_secs(5), move || {
        upkeep.run_upkeep();
        std::future::ready(Ok::<(), std::convert::Infallible>(()))
    });

    let app = Router::new()
        .route("/metrics", get(render))
        .with_state((handle, pool));
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(crate::background::stopped())
            .await
            .ok();
    });
}

/// Start recording, process-wide. Once per process: a second call panics.
pub fn install() -> PrometheusHandle {
    PrometheusBuilder::new()
        .set_buckets_for_metric(Matcher::Suffix("_seconds".into()), BUCKETS)
        .expect("the buckets are not empty")
        .install_recorder()
        .expect("no other metrics recorder is installed")
}

/// The pool is read at scrape time; everything else was recorded as it happened.
async fn render(State((handle, pool)): State<(PrometheusHandle, PgPool)>) -> String {
    let idle = pool.num_idle() as f64;
    gauge!("ws_db_pool_connections", "state" => "idle").set(idle);
    gauge!("ws_db_pool_connections", "state" => "in_use").set(f64::from(pool.size()) - idle);
    gauge!("ws_db_pool_max_connections").set(pool.options().get_max_connections());
    handle.render()
}

/// Middleware: every request counted and timed by its route template and status class.
pub async fn track_http(request: Request, next: Next) -> Response {
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());
    let method = method_label(request.method());
    let started = Instant::now();
    let response = next.run(request).await;
    let labels = [
        ("method", method.to_string()),
        ("route", route),
        ("status", status_class(response.status()).to_string()),
    ];
    counter!("ws_http_requests_total", &labels).increment(1);
    histogram!("ws_http_request_duration_seconds", &labels).record(started.elapsed());
    response
}

fn method_label(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::PATCH => "PATCH",
        Method::DELETE => "DELETE",
        Method::HEAD => "HEAD",
        Method::OPTIONS => "OPTIONS",
        _ => "other",
    }
}

pub fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        _ => "5xx",
    }
}

/// Run an open socket, counted in `ws_open_sockets` for as long as it lives. A guard
/// rather than a decrement at the end, so a socket dropped mid-await is still let go of.
pub async fn track_socket(kind: &'static str, socket: impl Future<Output = ()>) {
    struct Open(metrics::Gauge);
    impl Drop for Open {
        fn drop(&mut self) {
            self.0.decrement(1.0);
        }
    }
    let open = gauge!("ws_open_sockets", "kind" => kind);
    open.increment(1.0);
    let _open = Open(open);
    socket.await;
}

/// One pass of a background job: how long it took and whether it failed.
pub fn job_pass(job: &'static str, took: Duration, failed: bool) {
    histogram!("ws_background_job_duration_seconds", "job" => job).record(took);
    if failed {
        counter!("ws_background_job_failures_total", "job" => job).increment(1);
    }
}

/// Ticks a job missed because its previous pass was still running.
pub fn ticks_skipped(job: &'static str, missed: u64) {
    if missed > 0 {
        counter!("ws_background_ticks_skipped_total", "job" => job).increment(missed);
    }
}

/// One outgoing ESI request, or one that was not sent. `outcome` is a fixed word.
pub fn esi_request(outcome: &'static str, took: Option<Duration>) {
    counter!("ws_esi_requests_total", "outcome" => outcome).increment(1);
    if let Some(took) = took {
        histogram!("ws_esi_request_duration_seconds").record(took);
    }
}

/// What an ESI reply's status says, as a label.
pub fn esi_outcome(status: u16) -> &'static str {
    match status {
        200..=299 => "ok",
        304 => "not_modified",
        420 => "error_limited",
        429 => "rate_limited",
        400..=499 => "client_error",
        _ => "server_error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_and_esi_replies_become_a_few_words() {
        assert_eq!(status_class(StatusCode::OK), "2xx");
        assert_eq!(status_class(StatusCode::SWITCHING_PROTOCOLS), "1xx");
        assert_eq!(status_class(StatusCode::NOT_FOUND), "4xx");
        assert_eq!(status_class(StatusCode::BAD_GATEWAY), "5xx");
        assert_eq!(esi_outcome(304), "not_modified");
        assert_eq!(esi_outcome(420), "error_limited");
        assert_eq!(esi_outcome(429), "rate_limited");
        assert_eq!(esi_outcome(403), "client_error");
        assert_eq!(esi_outcome(503), "server_error");
        assert_eq!(
            method_label(&Method::from_bytes(b"PROPFIND").unwrap()),
            "other"
        );
    }
}
