use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive()) // Dev only, configure properly in production
        .layer(TraceLayer::new_for_http());

    // Bind address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8090));
    tracing::info!("🚀 Listening on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    tracing::debug!("Health check requested");

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "rust-engine",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}
