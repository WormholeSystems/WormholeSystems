//! Error reports over the Sentry protocol, so any of Sentry, GlitchTip or Bugsink can take
//! them. Off unless `SENTRY_DSN` is set. When on, panics (a background job's included: the
//! hook sees every thread) and `tracing` errors become events, and the lines logged before
//! one ride along as breadcrumbs.
//!
//! What leaves is scrubbed first: no default PII, no request headers, cookies or query
//! strings, and anything shaped like a token, an SSO `code` or `state` is blanked out of
//! the messages, whichever way it got there.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;
use sentry::protocol::{Context, Event, Value};
use sentry::{Breadcrumb, ClientInitGuard, ClientOptions};

/// `wormholesystems-api@0.1.0`, plus the commit when the build was given one (the images
/// are, through the `GIT_SHA` build argument).
pub fn release() -> String {
    let version = env!("CARGO_PKG_VERSION");
    match option_env!("GIT_SHA").filter(|sha| !sha.is_empty()) {
        Some(sha) => format!("wormholesystems-api@{version}+{sha}"),
        None => format!("wormholesystems-api@{version}"),
    }
}

/// Start the client when `SENTRY_DSN` is set. The guard flushes what is queued when the
/// process ends, so it lives as long as `main`.
pub fn init() -> Option<ClientInitGuard> {
    let dsn = nonempty("SENTRY_DSN")?;
    let dsn = match dsn.parse() {
        Ok(dsn) => dsn,
        Err(err) => {
            // Logging is not up yet; this is the one thing worth saying before it is.
            eprintln!("SENTRY_DSN is not a valid DSN ({err}), error reporting is off");
            return None;
        }
    };
    let mut options = ClientOptions::new()
        .release(release())
        .send_default_pii(false)
        // Traces are OpenTelemetry's job here; this is for errors.
        .traces_sample_rate(0.0)
        .before_send(|event| Some(scrub_event(event)))
        .before_breadcrumb(|crumb| Some(scrub_breadcrumb(crumb)));
    options.dsn = Some(dsn);
    options.environment = nonempty("SENTRY_ENVIRONMENT").map(Cow::Owned);
    Some(sentry::init(options))
}

/// The `tracing` side: an `error!` is an event, `warn!` and `info!` are breadcrumbs, and
/// spans are left alone.
pub fn layer<S>() -> sentry::integrations::tracing::SentryLayer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    sentry::integrations::tracing::layer().span_filter(|_| false)
}

fn nonempty(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub fn scrub_event(mut event: Event<'static>) -> Event<'static> {
    if let Some(request) = event.request.as_mut() {
        if let Some(url) = request.url.as_mut() {
            url.set_query(None);
            url.set_fragment(None);
        }
        request.query_string = None;
        request.cookies = None;
        request.data = None;
        request.headers.clear();
        request.env.clear();
    }
    event.user = None;
    event.server_name = None;
    // The device context carries the host name.
    event.contexts.remove("device");
    if let Some(message) = event.message.as_mut() {
        redact_in_place(message);
    }
    if let Some(entry) = event.logentry.as_mut() {
        redact_in_place(&mut entry.message);
        entry.params.iter_mut().for_each(redact_value);
    }
    for exception in event.exception.values.iter_mut() {
        if let Some(value) = exception.value.as_mut() {
            redact_in_place(value);
        }
    }
    event.extra.values_mut().for_each(redact_value);
    for context in event.contexts.values_mut() {
        if let Context::Other(fields) = context {
            fields.values_mut().for_each(redact_value);
        }
    }
    event.breadcrumbs.values = std::mem::take(&mut event.breadcrumbs.values)
        .into_iter()
        .map(scrub_breadcrumb)
        .collect();
    event
}

pub fn scrub_breadcrumb(mut crumb: Breadcrumb) -> Breadcrumb {
    if let Some(message) = crumb.message.as_mut() {
        redact_in_place(message);
    }
    crumb.data.values_mut().for_each(redact_value);
    crumb
}

fn redact_value(value: &mut Value) {
    match value {
        Value::String(s) => redact_in_place(s),
        Value::Array(items) => items.iter_mut().for_each(redact_value),
        Value::Object(fields) => fields.values_mut().for_each(redact_value),
        _ => {}
    }
}

fn redact_in_place(text: &mut String) {
    if let Cow::Owned(clean) = redact(text) {
        *text = clean;
    }
}

/// Query parameters and JSON fields that carry a credential or the SSO handshake.
static PARAM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"\b(code|state|token|access_token|refresh_token|client_secret|password)(=|"\s*:\s*")([^&\s"']+)"#,
    )
    .expect("valid regex")
});
static BEARER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([Bb]earer|[Bb]asic)\s+[A-Za-z0-9._~+/=-]+").expect("valid regex")
});
/// Our personal access tokens, which say what they are.
static PAT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"wst_[0-9a-f]{16,}").expect("valid regex"));

/// `text` with every credential-shaped piece replaced, borrowed when there was none.
pub fn redact(text: &str) -> Cow<'_, str> {
    let text = PARAM.replace_all(text, "$1$2[redacted]");
    let text = match BEARER.replace_all(&text, "$1 [redacted]") {
        Cow::Owned(s) => Cow::Owned(s),
        Cow::Borrowed(_) => text,
    };
    match PAT.replace_all(&text, "wst_[redacted]") {
        Cow::Owned(s) => Cow::Owned(s),
        Cow::Borrowed(_) => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sentry::protocol::{Exception, Request};

    #[test]
    fn credentials_are_blanked_wherever_they_are() {
        assert_eq!(
            redact("GET /auth/callback?code=abc123&state=xyz HTTP/1.1"),
            "GET /auth/callback?code=[redacted]&state=[redacted] HTTP/1.1"
        );
        assert_eq!(
            redact(r#"token endpoint 400: {"refresh_token": "eyJ.x.y"}"#),
            r#"token endpoint 400: {"refresh_token": "[redacted]"}"#
        );
        assert_eq!(
            redact("Authorization: Bearer eyJhbGciOi.payload.sig"),
            "Authorization: Bearer [redacted]"
        );
        assert_eq!(
            redact("unknown token wst_0123456789abcdef0123456789abcdef"),
            "unknown token wst_[redacted]"
        );
        assert!(matches!(
            redact("character 9001 went offline"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn an_event_loses_its_request_details_and_host() {
        let mut event = Event {
            message: Some("callback failed for /auth/callback?code=c&state=s".into()),
            server_name: Some("a1b2c3".into()),
            request: Some(Request {
                url: "https://map.example.com/auth/callback?code=c&state=s"
                    .parse()
                    .ok(),
                query_string: Some("code=c&state=s".into()),
                cookies: Some("ws_session=secret".into()),
                headers: [("Authorization".to_string(), "Bearer x".to_string())].into(),
                ..Default::default()
            }),
            ..Default::default()
        };
        event.exception.values.push(Exception {
            ty: "Error".into(),
            value: Some("refresh_token=abc".into()),
            ..Default::default()
        });
        let event = scrub_event(event);
        let request = event.request.unwrap();
        assert_eq!(
            request.url.unwrap().as_str(),
            "https://map.example.com/auth/callback"
        );
        assert!(request.query_string.is_none());
        assert!(request.cookies.is_none());
        assert!(request.headers.is_empty());
        assert!(event.server_name.is_none());
        assert_eq!(
            event.message.as_deref(),
            Some("callback failed for /auth/callback?code=[redacted]&state=[redacted]")
        );
        assert_eq!(
            event.exception.values[0].value.as_deref(),
            Some("refresh_token=[redacted]")
        );
    }
}
