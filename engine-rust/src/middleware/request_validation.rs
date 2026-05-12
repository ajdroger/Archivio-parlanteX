/// Request validation middleware
///
/// Validates incoming requests for size limits and content type

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Maximum request body size (50 MB for PDF uploads)
const MAX_BODY_SIZE: u64 = 50 * 1024 * 1024;

/// Request validation middleware
///
/// Checks:
/// 1. Content-Length header doesn't exceed MAX_BODY_SIZE
/// 2. Content-Type is valid (JSON or multipart for uploads)
pub async fn request_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check Content-Length
    if let Some(content_length) = request.headers().get(header::CONTENT_LENGTH) {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > MAX_BODY_SIZE {
                    tracing::warn!(
                        content_length = length,
                        max_allowed = MAX_BODY_SIZE,
                        "Request body too large"
                    );
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    // Check Content-Type for POST/PUT requests
    let method = request.method();
    if method == axum::http::Method::POST || method == axum::http::Method::PUT {
        if let Some(content_type) = request.headers().get(header::CONTENT_TYPE) {
            if let Ok(ct_str) = content_type.to_str() {
                let is_valid = ct_str.starts_with("application/json")
                    || ct_str.starts_with("multipart/form-data")
                    || ct_str.starts_with("application/x-www-form-urlencoded");

                if !is_valid {
                    tracing::warn!(
                        content_type = ct_str,
                        "Unsupported content type"
                    );
                    return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
                }
            }
        } else {
            // POST/PUT without Content-Type is suspicious
            tracing::warn!("POST/PUT request missing Content-Type header");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_valid_json_request() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, "100")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_too_large() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        let too_large = MAX_BODY_SIZE + 1;

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, too_large.to_string())
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_missing_content_type() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(header::CONTENT_LENGTH, "100")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_unsupported_content_type() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(header::CONTENT_TYPE, "text/xml")
            .header(header::CONTENT_LENGTH, "100")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_multipart_allowed() {
        let app = Router::new()
            .route("/test", post(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header(header::CONTENT_TYPE, "multipart/form-data; boundary=----")
            .header(header::CONTENT_LENGTH, "1000")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_request_no_validation() {
        let app = Router::new()
            .route("/test", axum::routing::get(test_handler))
            .layer(middleware::from_fn(request_validation_middleware));

        // GET requests don't need Content-Type
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
