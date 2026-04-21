mod chunker;
mod clients;
mod config;
mod errors;
mod models;
mod providers;
mod rag;
mod routes;
mod utils;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
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
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,archivio_parlante_rust_engine=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Archivio Parlante Rust Engine starting...");

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

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ingest", post(routes::ingest::handle_ingest))
        .route("/query", post(routes::query::handle_query))
        .route(
            "/compare_contracts",
            post(routes::compare::handle_compare_contracts),
        )
        .layer(CorsLayer::permissive()) // Dev only, configure properly in production
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Bind address
    let addr = config
        .listen_addr
        .parse::<SocketAddr>()
        .expect("Invalid listen address");
    tracing::info!("🚀 Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");
    axum::serve(listener, app)
        .await
        .expect("Server error");
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
