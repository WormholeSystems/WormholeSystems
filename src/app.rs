//! The server process: what it is made of and how it starts.

use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;

use crate::auth::AppState;

/// `GET /healthz`: up, and the database answers. What Docker's healthcheck asks, so it is
/// cheap, needs no session and says nothing beyond a word.
///
/// The query has a deadline of its own: a pool with every connection taken would otherwise
/// hold this for the pool's acquire timeout, longer than the healthcheck waits.
pub async fn healthz(State(state): State<AppState>) -> (StatusCode, &'static str) {
    let ping = sqlx::query_scalar!("select 1 as \"one!\"").fetch_one(&state.db);
    match tokio::time::timeout(Duration::from_secs(3), ping).await {
        Ok(Ok(_)) => (StatusCode::OK, "ok"),
        Ok(Err(err)) => {
            tracing::warn!("health check: database error: {err}");
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable")
        }
        Err(_) => {
            tracing::warn!("health check: database did not answer in time");
            (StatusCode::SERVICE_UNAVAILABLE, "database unavailable")
        }
    }
}
