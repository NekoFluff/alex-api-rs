#![allow(clippy::let_with_type_underscore)]
#![allow(clippy::default_constructed_unit_structs)] // warning since 1.71

use axum::extract::Path;
use axum::{response::IntoResponse, routing::get, BoxError, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use serde_json::json;
use std::net::SocketAddr;
use tracing::span::Id;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

mod init_otel;
mod trace_id_format;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
    // very opinionated init of tracing, look as is source to make your own
    // init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    init_otel::init_subscribers()?;

    let app = app();
    // run it
    let addr = &"0.0.0.0:3003".parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/` (with trace)"); //Devskim: ignore DS137138
    tracing::info!("try to call `curl -i http://127.0.0.1:3003/health` (with NO trace)"); //Devskim: ignore DS137138
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn app() -> Router {
    // build our application with a route
    Router::new()
        .route(
            "/proxy/:service/*path",
            get(proxy_handler).post(proxy_handler),
        )
        .route("/", get(index)) // request processed inside span
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .route("/health", get(health)) // request processed without span / trace
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({ "status" : "UP" }))
}

#[tracing::instrument]
async fn index() -> impl IntoResponse {
    let trace_id = find_current_trace_id();
    let span = tracing::Span::current();
    span.record("result", "winner");
    span.record("trace_id", &trace_id);
    span.record("trace.id2", &trace_id);
    span.record("trace.id", &trace_id);
    span.record("WTF", "WTF");
    dbg!(&trace_id);

    span.set_attribute("WWWEEEEEE", "a");
    span.set_attribute("trace.id", trace_id.clone().unwrap_or_default());

    //std::thread::sleep(std::time::Duration::from_secs(1));
    tracing::info!(something = "a", "shaving yaks again info");
    tracing::warn!(something2 = "hi", "shaving yaks again");
    index_info().await;
    index_warning().await;
    index_error("AAA").await;

    axum::Json(json!({ "my_trace_id": trace_id }))
}

#[tracing::instrument]
async fn index_error(e: &str) {
    let trace_id2 = find_current_trace_id();
    let span = tracing::Span::current();
    span.record("trace.id2", &trace_id2);
    span.record("WTF", "WTF");
    // let mut span_id: u64 = 0;
    // if let Some(id) = tracing::Span::current().id() {
    //     span_id = id.into_u64();
    // }

    // tracing::warn!(span.id = span_id, trace.id = trace_id, "test error");
    tracing::warn!(trace.id = trace_id2, "test error");
    tracing::error!("test error2");
}

#[tracing::instrument]
async fn index_info() {
    tracing::info!("test info");
}

#[tracing::instrument]
async fn index_warning() {
    tracing::warn!("test warning");
}

async fn proxy_handler(Path((service, path)): Path<(String, String)>) -> impl IntoResponse {
    // Overwrite the otel.name of the span
    tracing::Span::current().record("otel.name", format!("proxy {service}"));
    let trace_id = find_current_trace_id();
    axum::Json(
        json!({ "my_trace_id": trace_id, "fake_proxy": { "service": service, "path": path } }),
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("signal received, starting graceful shutdown");
    opentelemetry::global::shutdown_tracer_provider();
}
