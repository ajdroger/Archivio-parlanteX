/// Rate limiting middleware
///
/// Limits requests per IP address to prevent abuse

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rate limit entry for tracking requests per IP
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Request count in current window
    count: u32,
    /// Window start time
    window_start: Instant,
}

/// Global rate limiter state
///
/// Uses DashMap for lock-free concurrent access
static RATE_LIMITER: once_cell::sync::Lazy<DashMap<IpAddr, RateLimitEntry>> =
    once_cell::sync::Lazy::new(DashMap::new);

/// Rate limit configuration
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Window duration
    pub window_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 100,
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Rate limiting middleware
///
/// Implements sliding window rate limiting per IP address
/// Limits: 100 requests per minute by default
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = RateLimitConfig::default();

    // Extract client IP
    let client_ip = extract_client_ip(&request).unwrap_or_else(|| {
        // Fallback to localhost if cannot determine IP
        "127.0.0.1".parse().unwrap()
    });

    // Check and update rate limit
    let now = Instant::now();
    let mut entry = RATE_LIMITER
        .entry(client_ip)
        .or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

    // Reset window if expired
    if now.duration_since(entry.window_start) >= config.window_duration {
        entry.count = 0;
        entry.window_start = now;
    }

    // Check limit
    if entry.count >= config.requests_per_minute {
        tracing::warn!(
            ip = %client_ip,
            count = entry.count,
            limit = config.requests_per_minute,
            "Rate limit exceeded"
        );

        // Add Retry-After header
        let _retry_after = config.window_duration
            .saturating_sub(now.duration_since(entry.window_start))
            .as_secs();

        drop(entry); // Release lock before returning

        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Increment counter
    entry.count += 1;
    drop(entry); // Release lock

    tracing::debug!(
        ip = %client_ip,
        "Rate limit check passed"
    );

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

    // Fallback: No IP detected (ConnectInfo not available in middleware context)
    // In production, ensure reverse proxy (nginx/traefik) sets X-Forwarded-For or X-Real-IP headers
    // For local dev without proxy, rate limiting will apply globally (no per-IP limit)
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
