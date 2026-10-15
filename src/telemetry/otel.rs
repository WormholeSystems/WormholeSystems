//! OpenTelemetry traces over OTLP/HTTP, on only when `OTEL_EXPORTER_OTLP_ENDPOINT` (or the
//! traces-only `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`) is set. The rest is the SDK's standard
//! environment: `OTEL_SERVICE_NAME` (here `wormholesystems-api` when unset),
//! `OTEL_TRACES_SAMPLER` and `OTEL_TRACES_SAMPLER_ARG`, `OTEL_EXPORTER_OTLP_HEADERS`.
//!
//! Every `tracing` span the `RUST_LOG` filter lets through becomes an OpenTelemetry span:
//! the request span from the router, and whatever runs inside it.

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::registry::LookupSpan;

/// The service name when `OTEL_SERVICE_NAME` does not give one.
const SERVICE_NAME: &str = "wormholesystems-api";

/// Flushes the spans still queued when it is dropped.
pub struct Tracing(SdkTracerProvider);

impl Drop for Tracing {
    fn drop(&mut self) {
        let _ = self.0.shutdown();
    }
}

fn configured() -> bool {
    [
        "OTEL_EXPORTER_OTLP_ENDPOINT",
        "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
    ]
    .iter()
    .any(|key| std::env::var(key).is_ok_and(|v| !v.trim().is_empty()))
}

/// The exporter and the `tracing` layer that feeds it, when an endpoint is configured.
pub fn init<S>() -> Option<(Tracing, impl tracing_subscriber::Layer<S> + Send + Sync)>
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    if !configured() {
        return None;
    }
    let exporter = match SpanExporter::builder().with_http().build() {
        Ok(exporter) => exporter,
        Err(err) => {
            // Logging is not up yet.
            eprintln!("OTLP exporter could not be built ({err}), tracing export is off");
            return None;
        }
    };
    let mut resource = Resource::builder().with_attribute(opentelemetry::KeyValue::new(
        "service.version",
        env!("CARGO_PKG_VERSION"),
    ));
    if !std::env::var("OTEL_SERVICE_NAME").is_ok_and(|v| !v.trim().is_empty()) {
        resource = resource.with_service_name(SERVICE_NAME);
    }
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource.build())
        .build();
    let tracer = provider.tracer("wormholesystems");
    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    Some((Tracing(provider), layer))
}
