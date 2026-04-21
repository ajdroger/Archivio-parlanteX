/// Rate limiting middleware using tower_governor
///
/// Limits requests per IP address to prevent abuse

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::IpAddr;

/// Rate limit configuration
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
        }
    }
}

/// Simple in-memory rate limiter (placeholder)
///
/// TODO: Replace with tower_governor for production
/// This is a simplified implementation for now
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement actual rate limiting with tower_governor
    // For now, just pass through

    tracing::debug!("Rate limit check (placeholder - not enforced)");

    Ok(next.run(request).await)
}

/// Extract client IP from request
///
/// Checks X-Forwarded-For header first (if behind proxy),
/// then falls back to connection peer address
fn extract_client_ip(request: &Request) -> Option<IpAddr> {
    // Check X-Forwarded-For header
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take first IP from comma-separated list
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // TODO: Extract from connection peer address
    // This requires access to ConnectInfo which isn't available in middleware
    // For now, return None
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 100);
    }

    #[tokio::test]
    async fn test_rate_limit_middleware_placeholder() {
        use axum::{
            body::Body,
            http::{Request, StatusCode},
            middleware,
            routing::get,
            Router,
        };
        use tower::ServiceExt;

        async fn handler() -> &'static str {
            "OK"
        }

        let app = Router::new()
            .route("/test", get(handler))
            .layer(middleware::from_fn(rate_limit_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should pass through (placeholder implementation)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
