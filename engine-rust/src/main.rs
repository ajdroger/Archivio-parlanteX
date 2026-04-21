mod chunker;
mod clients;
mod config;
mod errors;
mod middleware;
mod models;
mod providers;
mod rag;
mod routes;
mod utils;

use axum::{
    extract::State,
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use utoipa_swagger_ui::SwaggerUi;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use clients::python_worker::PythonWorkerClient;
use config::Config;
use providers::registry::LlmRegistry;
use routes::ingest::AppState;

#[tokio::main]
async fn main() {
    // Initialize structured JSON logging
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,archivio_parlante_rust_engine=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,archivio_parlante_rust_engine=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    tracing::info!("🦀 Archivio Parlante Rust Engine starting...");

    // Initialize Prometheus metrics
    routes::metrics::init_metrics();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    tracing::info!(
        listen_addr = %config.listen_addr,
        ollama_url = %config.ollama_url,
        "Configuration loaded"
    );

    // Initialize LLM registry
    let llm_registry = LlmRegistry::from_config(&config);

    // Initialize Python worker client
    let python_worker = PythonWorkerClient::new(config.python_worker_url.clone());

    // Build application state
    let state = AppState {
        config: Arc::new(config.clone()),
        llm_registry: Arc::new(llm_registry),
        python_worker: Arc::new(python_worker),
    };

    // Build API documentation
    let openapi_spec = routes::docs::get_openapi_spec();

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(routes::metrics::metrics_handler))
        .route("/openapi.json", get(move || async { openapi_spec }))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", routes::docs::ApiDoc::openapi()))
        .route("/ingest", post(routes::ingest::handle_ingest))
        .route("/query", post(routes::query::handle_query))
        .route(
            "/compare_contracts",
            post(routes::compare::handle_compare_contracts),
        )
        // KB management endpoints
        .route("/kb/:kb_id/documents", get(routes::kb::list_documents))
        .route(
            "/kb/:kb_id/documents/:doc_id",
            delete(routes::kb::delete_document),
        )
        .route("/kb/:kb_id/graph", get(routes::kb::get_graph))
        .route("/kb/:kb_id/stats", get(routes::kb::get_stats))
        .route("/admin/reindex/:kb_id", post(routes::kb::reindex_kb))
        // Security middleware (applied to all routes except /health, /metrics, /docs)
        .layer(axum_middleware::from_fn(middleware::rate_limit::rate_limit_middleware))
        .layer(axum_middleware::from_fn(middleware::internal_auth::internal_auth_middleware))
        // Cross-cutting middleware
        .layer(CorsLayer::permissive()) // Dev only, configure properly in production
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind address
    let addr = config
        .listen_addr
        .parse::<SocketAddr>()
        .expect("Invalid listen address");
    tracing::info!("🚀 Listening on {}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server ready, press Ctrl+C to shutdown gracefully");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");

    tracing::info!("Server shutdown complete");
}

/// Graceful shutdown signal handler
///
/// Waits for SIGTERM or Ctrl+C, then allows in-flight requests to complete
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        },
    }

    tracing::info!("Shutdown signal received, waiting for in-flight requests to complete...");
}

/// Health check endpoint
async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    tracing::debug!("Health check requested");

    let providers = state.llm_registry.list_providers();

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "rust-engine",
            "version": env!("CARGO_PKG_VERSION"),
            "providers": providers,
            "cloud_enabled": state.config.has_cloud_providers(),
        })),
    )
}

// Route handlers now in routes:: modules:
// - routes::ingest::handle_ingest
// - routes::query::handle_query
// - routes::compare::handle_compare_contracts
