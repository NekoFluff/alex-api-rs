#![allow(clippy::let_with_type_underscore)]
#![allow(clippy::default_constructed_unit_structs)] // warning since 1.71

use axum::extract::Path;
use axum::{response::IntoResponse, routing::get, routing::post, BoxError, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use dsp::ComputedRecipeRequest;
use optimizer::OptimizerConfig;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tracing::span::Id;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::find_current_trace_id;

mod data;
mod dsp;
mod init_otel;
mod layer;
mod optimizer;
mod scrape;
mod timekeeper;
mod trace_id_format;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3003".to_string());

    // std::env::set_var("RUST_LOG", "warn,otel::tracing=info,otel=debug");
    // very opinionated init of tracing, look as is source to make your own
    // init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    init_otel::init_subscribers()?;

    let app = app();
    // run it
    let addr = &format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;
    tracing::warn!("listening on {}", addr);
    tracing::info!(
        "try to call `curl -i http://127.0.0.1:{}/` (with trace)",
        port
    ); //Devskim: ignore DS137138
    tracing::info!(
        "try to call `curl -i http://127.0.0.1:{}/health` (with NO trace)",
        port
    ); //Devskim: ignore DS137138
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
        .route("/dsp/recipes", get(dsp_recipes))
        .route("/dsp/computedRecipes", post(dsp_computed_recipes))
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

    index_info().await;
    index_warning().await;
    index_error("AAA").await;

    axum::Json(json!({ "my_trace_id": trace_id }))
}

#[tracing::instrument]
async fn dsp_recipes() -> impl IntoResponse {
    let recipes = dsp::load_recipes().await;
    axum::Json(json!(recipes))
}

#[tracing::instrument]
#[axum::debug_handler]
async fn dsp_computed_recipes(
    axum::Json(payload): axum::Json<ComputedRecipeRequest>,
) -> impl IntoResponse {
    let mut optimizer = optimizer::Optimizer::new(OptimizerConfig {});
    let recipes = dsp::load_recipes().await;
    optimizer.set_recipes(recipes);
    let mut seen = HashMap::new();
    let computed_recipes = optimizer.get_optimal_recipe(
        payload.name,
        payload.rate,
        "".to_string(),
        &mut seen,
        0,
        payload.requirements,
    );
    axum::Json(json!(computed_recipes))
}

#[tracing::instrument]
async fn index_error(e: &str) {
    tracing::warn!("test warn");
    tracing::error!("test error");
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
