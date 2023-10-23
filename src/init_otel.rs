use init_tracing_opentelemetry::Error;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::trace::TraceError;
// use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::Config;
use tracing::{info, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::layer::OpenTelemetryTracingBridge;
use crate::trace_id_format::TraceIdFormat;

#[must_use]
pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::FULL)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
            // .event_format(TraceIdFormat),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                //.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_current_span(true),
        )
    }
}

#[must_use]
pub fn build_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
    // filter what is output on log (fmt)
    // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
    std::env::set_var(
        "RUST_LOG",
        format!(
            // `otel::tracing` should be a level info to emit opentelemetry trace & span
            // `otel::setup` set to debug to log detected resources, configuration read and infered
            "{},otel::tracing=trace,otel=debug",
            std::env::var("RUST_LOG")
                .or_else(|_| std::env::var("OTEL_LOG_LEVEL"))
                .unwrap_or_else(|_| "info".to_string())
        ),
    );
    EnvFilter::from_default_env()
}

pub fn build_otel_tracing_layer<S>() -> Result<OpenTelemetryLayer<S, Tracer>, TraceError>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // use crate::{
    //     init_propagator, //stdio,
    //     otlp,
    //     resource::DetectResource,
    // };
    // let otel_rsrc = DetectResource::default()
    //     //.with_fallback_service_name(env!("CARGO_PKG_NAME"))
    //     //.with_fallback_service_version(env!("CARGO_PKG_VERSION"))
    //     .build();
    // let otel_tracer = otlp::init_tracer(otel_rsrc, otlp::identity)?;
    // // to not send trace somewhere, but continue to create and propagate,...
    // // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
    // // or to `std::io::stdout()` to print
    // //
    // // let otel_tracer = stdio::init_tracer(
    // //     otel_rsrc,
    // //     stdio::identity::<stdio::WriteNoWhere>,
    // //     stdio::WriteNoWhere::default(),
    // // )?;
    // // init_propagator()?;
    // Ok(tracing_opentelemetry::layer()
    //     .with_exception_field_propagation(true)
    //     .with_tracer(otel_tracer))
    let service_name: &str = env!("CARGO_BIN_NAME");
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name,
            )]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("pipeline install failure");

    Ok(tracing_opentelemetry::layer().with_tracer(tracer))
}

pub fn build_otel_logging_layer() {
    let service_name: &str = env!("CARGO_BIN_NAME");
    let _logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            Config::default().with_resource(opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name,
                ),
            ])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("pipeline install failure");
}

pub fn init_subscribers() -> Result<(), Error> {
    //setup a temporary subscriber to log output during setup
    let subscriber = tracing_subscriber::registry()
        .with(build_loglevel_filter_layer())
        .with(build_logger_text());
    let _guard = tracing::subscriber::set_default(subscriber);
    info!("init logging & tracing");

    build_otel_logging_layer();
    let logger_provider = opentelemetry::global::logger_provider();
    let otel_logging_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    let subscriber = tracing_subscriber::registry()
        .with(build_otel_tracing_layer()?)
        .with(build_loglevel_filter_layer())
        .with(build_logger_text())
        .with(otel_logging_layer);

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
