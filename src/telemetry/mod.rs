//! What the process says about itself: its logs, and the opt-in hooks that send errors,
//! metrics and traces somewhere. Every hook is off until its variable is set, so a default
//! install keeps everything on the machine.

pub mod sentry;
pub mod tunnel;

use std::io::IsTerminal;

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
    tracing_subscriber::registry()
        .with(filter)
        .with(layers)
        .init();
    if sentry.is_some() {
        tracing::info!("error reporting on, release {}", sentry::release());
    }
    Telemetry { _sentry: sentry }
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
