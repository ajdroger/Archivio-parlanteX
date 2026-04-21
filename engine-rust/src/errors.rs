/// Error types for Archivio Parlante Rust Engine
///
/// Defines application-level errors with proper HTTP status mapping for Axum.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application-level errors
#[derive(Error, Debug)]
pub enum AppError {
    /// Document parsing error
    #[error("Failed to parse document: {0}")]
    Parse(String),

    /// Qdrant vector database error
    #[error("Qdrant error: {0}")]
    Qdrant(String),

    /// Ollama LLM error
    #[error("Ollama error: {0}")]
    Ollama(String),

    /// Python worker communication error
    #[error("Python worker error: {0}")]
    PythonWorker(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request from client
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error (catch-all)
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),

    /// Authentication/authorization error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Budget exceeded (cost guard)
    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::Parse(msg) => (StatusCode::BAD_REQUEST, "PARSE_ERROR", msg),
            AppError::Qdrant(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "QDRANT_ERROR", msg),
            AppError::Ollama(msg) => (StatusCode::BAD_GATEWAY, "OLLAMA_ERROR", msg),
            AppError::PythonWorker(msg) => (
                StatusCode::BAD_GATEWAY,
                "PYTHON_WORKER_ERROR",
                msg,
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                err.to_string(),
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            AppError::RateLimitExceeded(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                msg,
            ),
            AppError::BudgetExceeded(msg) => (
                StatusCode::PAYMENT_REQUIRED,
                "BUDGET_EXCEEDED",
                msg,
            ),
        };

        // Log error (will be picked up by tracing middleware)
        tracing::error!(
            status = status.as_u16(),
            error_code = error_code,
            message = %message,
            "Request error"
        );

        // Return JSON error response
        let body = Json(json!({
            "error": message,
            "code": error_code,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Convenience result type
pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::Parse("Invalid PDF format".to_string());
        assert_eq!(err.to_string(), "Failed to parse document: Invalid PDF format");

        let err = AppError::NotFound("Document kb_123/doc_456 not found".to_string());
        assert_eq!(err.to_string(), "Not found: Document kb_123/doc_456 not found");
    }

    #[test]
    fn test_error_into_response() {
        let err = AppError::BadRequest("Missing required field: kb_id".to_string());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
