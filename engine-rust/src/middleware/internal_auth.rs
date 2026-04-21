/// Internal authentication middleware
///
/// Validates X-Internal-Token header against configured token

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

const INTERNAL_TOKEN_HEADER: &str = "x-internal-token";

/// Internal authentication middleware
///
/// Checks that requests have valid X-Internal-Token header
/// matching the configured RUST_ENGINE_INTERNAL_TOKEN
pub async fn internal_auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get expected token from environment
    let expected_token = std::env::var("RUST_ENGINE_INTERNAL_TOKEN")
        .unwrap_or_else(|_| String::new());

    // If no token configured, allow (dev mode)
    if expected_token.is_empty() {
        tracing::warn!("RUST_ENGINE_INTERNAL_TOKEN not set - authentication bypassed");
        return Ok(next.run(request).await);
    }

    // Extract token from header
    let provided_token = headers
        .get(INTERNAL_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok());

    match provided_token {
        Some(token) if token == expected_token => {
            // Valid token
            tracing::debug!("Internal auth: token valid");
            Ok(next.run(request).await)
        }
        Some(_) => {
            // Invalid token
            tracing::warn!("Internal auth: invalid token provided");
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            // Missing token
            tracing::warn!("Internal auth: missing X-Internal-Token header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_internal_auth_with_valid_token() {
        std::env::set_var("RUST_ENGINE_INTERNAL_TOKEN", "test_token_123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(internal_auth_middleware));

        let request = Request::builder()
            .uri("/test")
            .header("x-internal-token", "test_token_123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        std::env::remove_var("RUST_ENGINE_INTERNAL_TOKEN");
    }

    #[tokio::test]
    async fn test_internal_auth_with_invalid_token() {
        std::env::set_var("RUST_ENGINE_INTERNAL_TOKEN", "test_token_123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(internal_auth_middleware));

        let request = Request::builder()
            .uri("/test")
            .header("x-internal-token", "wrong_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::env::remove_var("RUST_ENGINE_INTERNAL_TOKEN");
    }

    #[tokio::test]
    async fn test_internal_auth_missing_token() {
        std::env::set_var("RUST_ENGINE_INTERNAL_TOKEN", "test_token_123");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(internal_auth_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::env::remove_var("RUST_ENGINE_INTERNAL_TOKEN");
    }

    #[tokio::test]
    async fn test_internal_auth_no_token_configured() {
        std::env::remove_var("RUST_ENGINE_INTERNAL_TOKEN");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(internal_auth_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should allow when no token configured (dev mode)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
