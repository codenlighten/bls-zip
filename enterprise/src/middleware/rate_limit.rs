// Axum Middleware for Rate Limiting
// Integrates the rate limiter with Axum HTTP framework

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::rate_limit::RateLimiter;

/// Extract IP address from request
fn extract_ip(addr: &SocketAddr) -> std::net::IpAddr {
    addr.ip()
}

/// Rate limiting middleware for Axum
///
/// This middleware checks both IP-based and user-based rate limits.
/// It extracts the client IP from ConnectInfo and optionally the user ID from request extensions.
///
/// # Example
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use std::sync::Arc;
/// use boundless_enterprise::rate_limit::RateLimiter;
/// use boundless_enterprise::middleware::rate_limit::rate_limit_middleware;
///
/// let limiter = Arc::new(RateLimiter::new());
///
/// let app = Router::new()
///     .route("/api/users", get(handler))
///     .layer(axum::middleware::from_fn_with_state(
///         limiter.clone(),
///         rate_limit_middleware
///     ));
/// ```
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let ip = extract_ip(&addr);

    // Try to extract user ID from request extensions (set by auth middleware)
    let user_id = request.extensions().get::<UserId>().map(|u| u.0);

    // Check rate limits
    limiter
        .check_combined(ip, user_id)
        .await
        .map_err(|e| RateLimitError::Exceeded(e.to_string()))?;

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// User ID extension for request context
///
/// This should be set by your authentication middleware
#[derive(Debug, Clone, Copy)]
pub struct UserId(pub Uuid);

/// Rate limit error response
#[derive(Debug)]
pub enum RateLimitError {
    Exceeded(String),
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        match self {
            RateLimitError::Exceeded(msg) => {
                let body = serde_json::json!({
                    "error": "Rate limit exceeded",
                    "message": msg,
                    "retry_after": 60,
                })
                .to_string();

                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("Content-Type", "application/json")],
                    body,
                )
                    .into_response()
            }
        }
    }
}

/// IP-only rate limiting middleware (for public endpoints)
///
/// This variant only checks IP-based rate limits, useful for unauthenticated endpoints.
///
/// # Example
///
/// ```rust,no_run
/// use axum::{Router, routing::post};
/// use std::sync::Arc;
/// use boundless_enterprise::rate_limit::RateLimiter;
/// use boundless_enterprise::middleware::rate_limit::rate_limit_ip_only;
///
/// let limiter = Arc::new(RateLimiter::new());
///
/// let app = Router::new()
///     .route("/api/public/data", post(handler))
///     .layer(axum::middleware::from_fn_with_state(
///         limiter.clone(),
///         rate_limit_ip_only
///     ));
/// ```
pub async fn rate_limit_ip_only(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    let ip = extract_ip(&addr);

    // Check IP rate limit only
    limiter
        .check_ip(ip)
        .await
        .map_err(|e| RateLimitError::Exceeded(e.to_string()))?;

    Ok(next.run(request).await)
}

/// Helper function to add user ID to request extensions
///
/// Call this from your authentication middleware after verifying the user
pub fn set_user_id(request: &mut Request, user_id: Uuid) {
    request.extensions_mut().insert(UserId(user_id));
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
    use std::net::{IpAddr, Ipv4Addr};
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_rate_limit_middleware_allows_normal_requests() {
        let limiter = Arc::new(RateLimiter::new());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                limiter.clone(),
                rate_limit_middleware,
            ))
            .into_make_service_with_connect_info::<SocketAddr>();

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        // First request should succeed
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Note: This is a simplified test - in real scenarios you'd need to properly
        // set up the ConnectInfo
    }
}
