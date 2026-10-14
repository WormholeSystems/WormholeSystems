//! Async client for the EVE Swagger Interface (ESI) and its SSO.
//!
//! `EsiClient` is framework-agnostic: authenticated calls take an access-token string,
//! and the web layer supplies/persists tokens through [`token::TokenStore`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub mod error;
pub mod jwt;
pub mod scopes;
pub mod sso;
pub mod token;

pub mod character;
pub mod entities;
pub mod skyhooks;
pub mod sovereignty;
pub mod status;
pub mod ui;

pub use {
    character::{CharacterLocation, CharacterOnline, CharacterPublic, CharacterShip},
    entities::{Affiliation, Alliance, Corporation},
    error::{EsiError, Result},
    jwt::Claims,
    scopes::Scope,
    skyhooks::RaidableSkyhook,
    sovereignty::SovereigntySystem,
    sso::{Sso, SsoConfig},
    status::TranquilityStatus,
    token::{Token, TokenStore},
};

pub const BASE_URL: &str = "https://esi.evetech.net";
pub const COMPATIBILITY_DATE: &str = "2026-06-09";

/// Below this many errors left in ESI's error window, nothing more is sent until the
/// window resets. Going over the limit gets every route refused with a 420, and doing that
/// repeatedly gets the application banned.
const ERROR_LIMIT_FLOOR: u64 = 10;
/// How long to stand back when ESI says stop without saying for how long.
const DEFAULT_BACKOFF: Duration = Duration::from_secs(60);
/// The most cached replies kept for conditional requests. A few per tracked character;
/// past this the whole cache is dropped, which costs one full reply each.
const CONDITIONAL_CAP: usize = 10_000;

#[derive(Clone)]
pub struct EsiClient {
    http: reqwest::Client,
    base_url: String,
    compatibility_date: String,
    /// Shared by every clone, since ESI's limits are per application, not per handle.
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Shared {
    /// The last reply to each polled route, by path, for `If-None-Match` and `Expires`.
    replies: Mutex<HashMap<String, CachedReply>>,
    /// When sending may resume, by who is being held back: [`ANYONE`] for the error limit
    /// and public routes, otherwise the access token whose bucket ran out.
    paused: Mutex<HashMap<String, Instant>>,
}

/// The key in [`Shared::paused`] that holds every request back.
const ANYONE: &str = "";

struct CachedReply {
    etag: Option<String>,
    body: String,
    /// Until when ESI would hand back this same reply anyway.
    fresh_until: Option<Instant>,
}

impl EsiClient {
    pub fn new() -> Self {
        Self::with_config(crate::user_agent::client(), BASE_URL, COMPATIBILITY_DATE)
    }

    pub fn with_config(
        http: reqwest::Client,
        base_url: impl Into<String>,
        compatibility_date: impl Into<String>,
    ) -> Self {
        EsiClient {
            http,
            base_url: base_url.into(),
            compatibility_date: compatibility_date.into(),
            shared: Arc::default(),
        }
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .header("X-Compatibility-Date", &self.compatibility_date);
        if let Some(token) = token {
            req = req.bearer_auth(token);
        }
        req
    }

    /// Send a request unless ESI has told us to wait, and take note of what its limit
    /// headers say about the next one.
    async fn send(
        &self,
        req: reqwest::RequestBuilder,
        token: Option<&str>,
    ) -> Result<reqwest::Response> {
        self.check_paused(token)?;
        let resp = req.send().await?;
        self.note_limits(&resp, token);
        Ok(resp)
    }

    /// Fail fast while a pause covering this request is running.
    fn check_paused(&self, token: Option<&str>) -> Result<()> {
        let paused = self.shared.paused.lock().expect("esi pause lock");
        let now = Instant::now();
        let until = [Some(ANYONE), token]
            .into_iter()
            .flatten()
            .filter_map(|key| paused.get(key))
            .filter(|until| **until > now)
            .max();
        match until {
            Some(until) => Err(EsiError::Backoff((*until - now).as_secs().max(1))),
            None => Ok(()),
        }
    }

    /// Pause on a 420 (the error limit, which covers every route) until its window resets,
    /// and before it once few errors are left. Pause on a 429 for its `Retry-After`: that
    /// bucket belongs to one character's token, or to our address for public routes.
    fn note_limits(&self, resp: &reqwest::Response, token: Option<&str>) {
        let header = |name: &str| {
            resp.headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse::<u64>().ok())
        };
        let reset = header("x-esi-error-limit-reset").map(Duration::from_secs);
        let (key, wait) = match resp.status().as_u16() {
            420 => (ANYONE, reset.unwrap_or(DEFAULT_BACKOFF)),
            429 => (
                token.unwrap_or(ANYONE),
                header("retry-after")
                    .map(Duration::from_secs)
                    .unwrap_or(DEFAULT_BACKOFF),
            ),
            _ => match (header("x-esi-error-limit-remain"), reset) {
                (Some(remain), Some(reset)) if remain <= ERROR_LIMIT_FLOOR => (ANYONE, reset),
                _ => return,
            },
        };
        let until = Instant::now() + wait;
        let mut paused = self.shared.paused.lock().expect("esi pause lock");
        // Clear out the ones that have run, so per-token entries do not pile up.
        let now = Instant::now();
        paused.retain(|_, until| *until > now);
        let entry = paused.entry(key.to_string()).or_insert(until);
        if *entry <= until {
            if key == ANYONE {
                eprintln!(
                    "ESI: backing off for {}s ({})",
                    wait.as_secs(),
                    resp.status()
                );
            }
            *entry = until;
        }
    }

    /// Send a request, returning the response or an [`EsiError::Api`] that captures the body
    /// on a non-success status, which `reqwest`'s `error_for_status` discards.
    async fn send_checked(
        &self,
        req: reqwest::RequestBuilder,
        token: Option<&str>,
    ) -> Result<reqwest::Response> {
        let resp = self.send(req, token).await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(EsiError::Api {
                status: status.as_u16(),
                body,
            });
        }
        Ok(resp)
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str, token: Option<&str>) -> Result<T> {
        let resp = self
            .send_checked(self.request(reqwest::Method::GET, path, token), token)
            .await?;
        Ok(resp.json().await?)
    }

    /// [`get_json`](Self::get_json) for a route polled on a schedule. Inside the reply's
    /// cache window the stored reply is returned without asking, since ESI would only
    /// repeat it; after that the request carries the stored `ETag`, and a `304` (half the
    /// rate-limit cost of a `200`) reuses the stored body.
    async fn get_json_polled<T: DeserializeOwned>(&self, path: &str, token: &str) -> Result<T> {
        let mut req = self.request(reqwest::Method::GET, path, Some(token));
        {
            let replies = self.shared.replies.lock().expect("esi replies lock");
            if let Some(reply) = replies.get(path) {
                if reply.fresh_until.is_some_and(|t| t > Instant::now()) {
                    return Ok(serde_json::from_str(&reply.body)?);
                }
                if let Some(etag) = &reply.etag {
                    req = req.header(reqwest::header::IF_NONE_MATCH, etag);
                }
            }
        }

        let resp = self.send(req, Some(token)).await?;
        let status = resp.status();
        let fresh_until = fresh_until(resp.headers());
        if status == reqwest::StatusCode::NOT_MODIFIED {
            let mut replies = self.shared.replies.lock().expect("esi replies lock");
            if let Some(reply) = replies.get_mut(path) {
                reply.fresh_until = fresh_until;
                return Ok(serde_json::from_str(&reply.body)?);
            }
            // Only when the cache was dropped between asking and the answer.
            return Err(EsiError::Api {
                status: status.as_u16(),
                body: String::new(),
            });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(EsiError::Api {
                status: status.as_u16(),
                body,
            });
        }
        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let body = resp.text().await?;
        let value = serde_json::from_str(&body)?;
        if etag.is_some() || fresh_until.is_some() {
            let mut replies = self.shared.replies.lock().expect("esi replies lock");
            if replies.len() >= CONDITIONAL_CAP {
                replies.clear();
            }
            replies.insert(
                path.to_string(),
                CachedReply {
                    etag,
                    body,
                    fresh_until,
                },
            );
        }
        Ok(value)
    }

    async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        token: Option<&str>,
    ) -> Result<T> {
        let resp = self
            .send_checked(
                self.request(reqwest::Method::POST, path, token).json(body),
                token,
            )
            .await?;
        Ok(resp.json().await?)
    }
}

/// How long a reply stays current, from `Cache-Control: max-age` or else `Expires`, which
/// is measured against the reply's own `Date` so a skewed local clock does not matter.
fn fresh_until(headers: &reqwest::header::HeaderMap) -> Option<Instant> {
    let text = |name| headers.get(name).and_then(|v| v.to_str().ok());
    let max_age = text(reqwest::header::CACHE_CONTROL).and_then(|cc| {
        cc.split(',')
            .filter_map(|d| d.trim().strip_prefix("max-age="))
            .find_map(|secs| secs.parse::<u64>().ok())
    });
    let remaining = match max_age {
        Some(secs) => Duration::from_secs(secs),
        None => {
            let date = |name| text(name).and_then(|v| DateTime::parse_from_rfc2822(v).ok());
            let expires = date(reqwest::header::EXPIRES)?;
            let now = date(reqwest::header::DATE)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|| DateTime::<Utc>::from(SystemTime::now()));
            (expires.with_timezone(&Utc) - now).to_std().ok()?
        }
    };
    // Nothing ESI polls is cached for long; a bogus header should not freeze a route.
    (!remaining.is_zero()).then(|| Instant::now() + remaining.min(Duration::from_secs(3600)))
}

impl Default for EsiClient {
    fn default() -> Self {
        Self::new()
    }
}
