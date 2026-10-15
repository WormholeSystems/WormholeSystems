//! What the process says about itself: its logs, and the opt-in hooks that send errors,
//! metrics and traces somewhere. Every hook is off until its variable is set, so a default
//! install keeps everything on the machine.

pub mod metrics;
#[cfg(feature = "otel")]
pub mod otel;
pub mod sentry;
pub mod tunnel;

use std::io::IsTerminal;
use std::sync::atomic::{AtomicBool, Ordering};

use tracing_subscriber::layer::{Layered, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, Registry};

/// How a log line is written. `LOG_FORMAT=json` is one object per line, for a log shipper;
/// anything else is the human-readable default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

impl LogFormat {
    pub fn parse(value: Option<&str>) -> LogFormat {
        match value.map(str::trim) {
            Some(v) if v.eq_ignore_ascii_case("json") => LogFormat::Json,
            _ => LogFormat::Pretty,
        }
    }
}

/// What has to outlive `main`'s work so the hooks can flush on the way out.
#[must_use = "dropping it stops the hooks before the process is done"]
pub struct Telemetry {
    _sentry: Option<::sentry::ClientInitGuard>,
    #[cfg(feature = "otel")]
    _otel: Option<otel::Tracing>,
}

/// Install the subscriber and whichever hooks are configured. Logs go to stderr, so the few
/// subcommands whose stdout is read by a script (`sde-status`) keep it to themselves.
/// `RUST_LOG` narrows or widens it per module; unset means `info`.
pub fn init() -> Telemetry {
    // Before the configuration is read, so `.env` has to be loaded here for a host-side run
    // to see LOG_FORMAT, RUST_LOG and the hooks' variables in it.
    dotenvy::dotenv().ok();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let format = LogFormat::parse(std::env::var("LOG_FORMAT").ok().as_deref());
    let sentry = sentry::init();
    let mut layers: Vec<BoxedLayer> = vec![log_layer(format)];
    if sentry.is_some() {
        layers.push(sentry::layer().boxed());
    }
    #[cfg(feature = "otel")]
    let otel = otel::init().map(|(guard, layer)| {
        layers.push(layer.boxed());
        guard
    });
    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .init();
    if sentry.is_some() {
        tracing::info!("error reporting on, release {}", sentry::release());
    }
    #[cfg(feature = "otel")]
    if otel.is_some() {
        EXPORTING_TRACES.store(true, Ordering::Relaxed);
        tracing::info!("trace export on");
    }
    Telemetry {
        _sentry: sentry,
        #[cfg(feature = "otel")]
        _otel: otel,
    }
}

/// The span each request runs in: its method and route template, never the path asked
/// for, so an id or a share token stays out of the logs and the traces. `otel.*` are the
/// names the OpenTelemetry layer reads; without it they are just fields.
pub fn request_span(request: &axum::http::Request<axum::body::Body>) -> tracing::Span {
    let route = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map_or("unmatched", |path| path.as_str());
    let method = request.method().as_str();
    let span = tracing::info_span!(
        "request",
        method,
        route,
        otel.name = tracing::field::Empty,
        otel.kind = tracing::field::Empty,
        http.response.status_code = tracing::field::Empty,
    );
    // Only when they go somewhere: in a log line they would just repeat the two above.
    if EXPORTING_TRACES.load(Ordering::Relaxed) {
        span.record("otel.name", format!("{method} {route}"));
        span.record("otel.kind", "server");
    }
    span
}

/// Whether spans are exported, which is when the request span names itself for them.
static EXPORTING_TRACES: AtomicBool = AtomicBool::new(false);

type MakeSpan = fn(&axum::http::Request<axum::body::Body>) -> tracing::Span;
type OnResponse = fn(&axum::http::Response<axum::body::Body>, std::time::Duration, &tracing::Span);
pub type TraceLayer = tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    MakeSpan,
    (),
    OnResponse,
    (),
    (),
    (),
>;

/// The request span around the router: [`request_span`], with the status recorded on it.
/// No events of its own: a failure is the handler's to log, with what it knows.
pub fn trace_layer() -> TraceLayer {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(request_span as fn(&_) -> _)
        .on_request(())
        .on_response(record_status as fn(&_, _, &_))
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}

fn record_status(
    response: &axum::http::Response<axum::body::Body>,
    _: std::time::Duration,
    span: &tracing::Span,
) {
    span.record("http.response.status_code", response.status().as_u16());
}

/// Every layer sits under the `RUST_LOG` filter, so it decides for the hooks too.
type BoxedLayer = Box<dyn Layer<Layered<EnvFilter, Registry>> + Send + Sync>;

fn log_layer(format: LogFormat) -> BoxedLayer {
    let layer = tracing_subscriber::fmt::layer().with_writer(std::io::stderr);
    match format {
        LogFormat::Json => layer.json().boxed(),
        // Colours only on a terminal: `docker logs` would otherwise show the escape codes.
        LogFormat::Pretty => layer.with_ansi(colours()).boxed(),
    }
}

fn colours() -> bool {
    std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_json_switches_the_format() {
        assert_eq!(LogFormat::parse(Some("json")), LogFormat::Json);
        assert_eq!(LogFormat::parse(Some(" JSON ")), LogFormat::Json);
        assert_eq!(LogFormat::parse(Some("pretty")), LogFormat::Pretty);
        assert_eq!(LogFormat::parse(Some("yaml")), LogFormat::Pretty);
        assert_eq!(LogFormat::parse(None), LogFormat::Pretty);
    }
}
