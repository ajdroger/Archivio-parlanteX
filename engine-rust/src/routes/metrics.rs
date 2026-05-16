/// Prometheus metrics endpoint

use axum::{http::StatusCode, response::IntoResponse};
use lazy_static::lazy_static;
use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};
use std::sync::Arc;

lazy_static! {
    /// Global Prometheus registry
    pub static ref REGISTRY: Registry = Registry::new();

    /// HTTP requests counter
    pub static ref HTTP_REQUESTS_TOTAL: IntCounter = IntCounter::new(
        "http_requests_total",
        "Total number of HTTP requests"
    ).expect("metric can be created");

    /// HTTP errors counter
    pub static ref HTTP_ERRORS_TOTAL: IntCounter = IntCounter::new(
        "http_errors_total",
        "Total number of HTTP errors (4xx, 5xx)"
    ).expect("metric can be created");

    /// In-flight requests gauge
    pub static ref HTTP_REQUESTS_IN_FLIGHT: IntGauge = IntGauge::new(
        "http_requests_in_flight",
        "Current number of HTTP requests being processed"
    ).expect("metric can be created");

    /// LLM calls counter
    pub static ref LLM_CALLS_TOTAL: IntCounter = IntCounter::new(
        "llm_calls_total",
        "Total number of LLM API calls"
    ).expect("metric can be created");

    /// Qdrant queries counter
    pub static ref QDRANT_QUERIES_TOTAL: IntCounter = IntCounter::new(
        "qdrant_queries_total",
        "Total number of Qdrant vector searches"
    ).expect("metric can be created");

    /// Documents ingested counter
    pub static ref DOCUMENTS_INGESTED_TOTAL: IntCounter = IntCounter::new(
        "documents_ingested_total",
        "Total number of documents successfully ingested"
    ).expect("metric can be created");

    /// Chunks indexed counter
    pub static ref CHUNKS_INDEXED_TOTAL: IntCounter = IntCounter::new(
        "chunks_indexed_total",
        "Total number of chunks indexed in Qdrant"
    ).expect("metric can be created");
}

/// Initialize metrics registry (safe version returning Result)
fn init_metrics_safe() -> Result<(), prometheus::Error> {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(HTTP_ERRORS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(HTTP_REQUESTS_IN_FLIGHT.clone()))?;
    REGISTRY.register(Box::new(LLM_CALLS_TOTAL.clone()))?;
    REGISTRY.register(Box::new(QDRANT_QUERIES_TOTAL.clone()))?;
    REGISTRY.register(Box::new(DOCUMENTS_INGESTED_TOTAL.clone()))?;
    REGISTRY.register(Box::new(CHUNKS_INDEXED_TOTAL.clone()))?;
    Ok(())
}

/// Initialize metrics registry
///
/// Call this once at application startup.
/// Metrics failure is non-fatal - server will continue without metrics.
pub fn init_metrics() {
    match init_metrics_safe() {
        Ok(_) => {
            tracing::info!("Prometheus metrics initialized");
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to initialize Prometheus metrics - metrics endpoint will be unavailable"
            );
            tracing::warn!("Server will continue without metrics");
        }
    }
}

/// GET /metrics endpoint
///
/// Returns Prometheus-formatted metrics
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();

    let mut buffer = Vec::new();
    match encoder.encode(&metric_families, &mut buffer) {
        Ok(_) => {
            tracing::debug!("Metrics encoded successfully");
            (
                StatusCode::OK,
                [("content-type", "text/plain; version=0.0.4")],
                buffer,
            )
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [("content-type", "text/plain; version=0.0.4")],
                format!("Failed to encode metrics: {}", e).into_bytes(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Reset registry
        let test_registry = Registry::new();

        let counter = IntCounter::new("test_counter", "Test counter").unwrap();
        assert!(test_registry.register(Box::new(counter.clone())).is_ok());

        // Increment counter
        counter.inc();
        assert_eq!(counter.get(), 1);
    }

    #[tokio::test]
    async fn test_metrics_handler() {
        init_metrics();

        // Increment some metrics
        HTTP_REQUESTS_TOTAL.inc();
        LLM_CALLS_TOTAL.inc_by(5);

        // Call handler
        let response = metrics_handler().await.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify content-type header
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok());

        assert_eq!(content_type, Some("text/plain; version=0.0.4"));
    }
}
