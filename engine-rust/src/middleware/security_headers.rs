/// Security headers middleware
///
/// Adds HTTP security headers to all responses to mitigate common attacks

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Security headers middleware
///
/// Adds the following headers to all responses:
/// - X-Content-Type-Options: nosniff (prevent MIME sniffing)
/// - X-Frame-Options: DENY (prevent clickjacking)
/// - X-XSS-Protection: 1; mode=block (legacy XSS protection)
/// - Strict-Transport-Security: enforce HTTPS (production only)
/// - Content-Security-Policy: restrict content sources
/// - Referrer-Policy: no-referrer (privacy)
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Prevent MIME sniffing attacks
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // Legacy XSS protection (for older browsers)
    headers.insert(
        header::HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Content Security Policy (restrict resource loading)
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; script-src 'none'; object-src 'none'"),
    );

    // Referrer policy (privacy)
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );

    // HSTS (HTTPS enforcement) - only in production
    if std::env::var("APP_ENV").unwrap_or_default() == "production" {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    // Permissions Policy (disable unused browser features)
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_security_headers_added() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();

        // Verify security headers are present
        assert_eq!(
            headers.get("x-content-type-options").unwrap(),
            "nosniff"
        );
        assert_eq!(
            headers.get("x-frame-options").unwrap(),
            "DENY"
        );
        assert_eq!(
            headers.get("x-xss-protection").unwrap(),
            "1; mode=block"
        );
        assert!(headers.contains_key("content-security-policy"));
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "no-referrer"
        );
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_hsts_only_in_production() {
        std::env::set_var("APP_ENV", "production");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // HSTS should be present in production
        assert!(response.headers().contains_key("strict-transport-security"));

        std::env::remove_var("APP_ENV");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_no_hsts_in_dev() {
        std::env::set_var("APP_ENV", "dev");

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // HSTS should NOT be present in dev
        assert!(!response.headers().contains_key("strict-transport-security"));

        std::env::remove_var("APP_ENV");
    }
}
