//! `POST /api/sentry-tunnel`: the browser's error reports, passed on to the web DSN's
//! server. Same-origin, so the Content-Security-Policy keeps `connect-src 'self'` and an
//! ad blocker does not drop reports on their way to a known Sentry host.
//!
//! It forwards to exactly one place, the project `PUBLIC_SENTRY_DSN` names, and only an
//! envelope addressed to that same DSN. Anything else is refused, so it can never be used
//! to make this server send requests somewhere of the caller's choosing. Without the
//! variable it is a 404, as if it were not there.

use std::str::FromStr;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use sentry::types::Dsn;

use crate::app::AppState;

/// A browser error report is a few kilobytes; a megabyte leaves room for a long stack and
/// breadcrumbs without letting anyone stream something large through.
pub const BODY_LIMIT: usize = 1024 * 1024;

/// The configured destination and the client that talks to it.
pub struct Tunnel {
    dsn: Dsn,
    http: reqwest::Client,
}

impl Tunnel {
    pub fn new(dsn: &str) -> Result<Tunnel, String> {
        let dsn = Dsn::from_str(dsn).map_err(|err| err.to_string())?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            // Never follow a redirect: the destination is the DSN's, and nothing else.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|err| err.to_string())?;
        Ok(Tunnel { dsn, http })
    }

    /// Whether `envelope` is addressed to our DSN. The first line of an envelope is a JSON
    /// header, and the SDK puts the DSN there when it reports through a tunnel.
    pub fn accepts(&self, envelope: &[u8]) -> bool {
        #[derive(serde::Deserialize)]
        struct Header {
            dsn: String,
        }
        let line = envelope.split(|b| *b == b'\n').next().unwrap_or_default();
        let Ok(header) = serde_json::from_slice::<Header>(line) else {
            return false;
        };
        let Ok(dsn) = Dsn::from_str(&header.dsn) else {
            return false;
        };
        dsn.scheme() == self.dsn.scheme()
            && dsn.host() == self.dsn.host()
            && dsn.port() == self.dsn.port()
            && dsn.path() == self.dsn.path()
            && dsn.project_id() == self.dsn.project_id()
            && dsn.public_key() == self.dsn.public_key()
    }

    async fn forward(&self, envelope: Bytes) -> StatusCode {
        // Built from our DSN, never from the one in the envelope.
        let url = self.dsn.envelope_api_url();
        let auth = format!(
            "Sentry sentry_version=7, sentry_client=wormholesystems-tunnel, sentry_key={}",
            self.dsn.public_key()
        );
        let sent = self
            .http
            .post(url.as_str())
            .header(header::CONTENT_TYPE, "application/x-sentry-envelope")
            .header("X-Sentry-Auth", auth)
            .body(envelope)
            .send()
            .await;
        match sent {
            Ok(response) => {
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            Err(err) => {
                tracing::warn!("sentry tunnel: could not reach {}: {err}", self.dsn.host());
                StatusCode::BAD_GATEWAY
            }
        }
    }
}

/// `POST /api/sentry-tunnel`. Outside the API router, like the health check: not in the
/// OpenAPI document, not compressed, no session needed (a report can come from the login
/// page).
pub async fn handle(State(state): State<AppState>, envelope: Bytes) -> StatusCode {
    let Some(tunnel) = state.sentry_tunnel.as_deref() else {
        return StatusCode::NOT_FOUND;
    };
    if !tunnel.accepts(&envelope) {
        return StatusCode::BAD_REQUEST;
    }
    tunnel.forward(envelope).await
}

#[cfg(test)]
mod tests {
    use super::*;

    const DSN: &str = "https://public@errors.example.com/42";

    fn envelope(dsn: &str) -> Vec<u8> {
        format!("{{\"dsn\":\"{dsn}\",\"sent_at\":\"2026-01-01T00:00:00Z\"}}\n{{\"type\":\"event\"}}\n{{}}")
            .into_bytes()
    }

    #[test]
    fn only_the_configured_project_is_forwarded() {
        let tunnel = Tunnel::new(DSN).unwrap();
        assert!(tunnel.accepts(&envelope(DSN)));
        for other in [
            "https://public@evil.example.com/42",
            "https://public@errors.example.com/43",
            "https://public@errors.example.com:8443/42",
            "http://public@errors.example.com/42",
            "https://other@errors.example.com/42",
            "https://public@errors.example.com/prefix/42",
        ] {
            assert!(!tunnel.accepts(&envelope(other)), "{other}");
        }
        assert!(!tunnel.accepts(b"{\"event_id\":\"x\"}\n{}"));
        assert!(!tunnel.accepts(b"not json"));
        assert!(!tunnel.accepts(b""));
    }
}
