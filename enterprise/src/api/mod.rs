// Enterprise Multipass API Layer
// REST API endpoints for all services

use axum::{
    Router,
    middleware,
    http::{Request, StatusCode, Method, HeaderValue},
    response::{IntoResponse, Response},
    body::Body,
    extract::State,
};
use tower_http::cors::{CorsLayer, AllowOrigin};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::*;
use crate::error::{EnterpriseError, Result};
use crate::rate_limit::RateLimiter;
use uuid::Uuid;

pub mod identity;
pub mod wallet;
pub mod auth;
pub mod application;
pub mod asset;
pub mod events;
pub mod hardware;
pub mod contract;

/// Start the Enterprise Multipass API server
/// FIX M-11: Added RateLimiter parameter for login rate limiting
pub async fn serve(
    bind_addr: &str,
    identity_service: Arc<RwLock<IdentityService>>,
    wallet_service: Arc<RwLock<WalletService>>,
    auth_service: Arc<RwLock<AuthService>>,
    application_service: Arc<RwLock<ApplicationService>>,
    asset_service: Arc<RwLock<AssetService>>,
    event_service: Arc<RwLock<EventService>>,
    hardware_service: Arc<RwLock<HardwareService>>,
    contract_service: Arc<RwLock<ContractService>>,
    rate_limiter: Arc<RateLimiter>,
) -> Result<()> {
    // Build the main router with auth middleware
    // Note: /api/auth routes are public (for login/register)
    // All other routes require authentication
    let protected_routes = Router::new()
        .nest("/api/identity", identity::routes(identity_service.clone()))
        .nest("/api/wallet", wallet::routes(wallet_service))
        .nest("/api/applications", application::routes(application_service))
        .nest("/api/assets", asset::routes(asset_service.clone()))
        .nest("/api/market", asset::market_routes(asset_service))
        .nest("/api/notifications", events::notification_routes(event_service.clone()))
        .nest("/api/reports", events::report_routes(event_service))
        .nest("/api/hardware", hardware::routes(hardware_service))
        .nest("/api/contracts", contract::routes(contract_service.clone()))
        .layer(middleware::from_fn_with_state(
            auth_service.clone(),
            auth_middleware
        ));

    // SECURITY FIX: Configure CORS with whitelist instead of permissive
    let cors = configure_cors();

    let app = Router::new()
        .merge(protected_routes)
        // FIX M-11: Pass rate limiter to auth routes for login rate limiting
        .nest("/api/auth", auth::routes(auth_service, rate_limiter))
        // Public contract routes (templates browsing)
        .nest("/api/contracts", contract::public_routes(contract_service))
        .layer(cors);

    // Start the server
    let listener = tokio::net::TcpListener::bind(bind_addr).await
        .map_err(|e| EnterpriseError::Internal(format!("Failed to bind to {}: {}", bind_addr, e)))?;

    tracing::info!("Enterprise Multipass API listening on {}", bind_addr);

    // Add ConnectInfo layer for rate limiting middleware to access client IP
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>()
    ).await
        .map_err(|e| EnterpriseError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

/// Configure CORS with environment-based origin whitelist
///
/// SECURITY: Only allows specific origins from ENTERPRISE_CORS_ORIGINS environment variable
/// Format: comma-separated list of origins (e.g., "http://localhost:3000,https://app.example.com")
/// Default: No origins allowed (secure by default)
fn configure_cors() -> CorsLayer {
    // Read allowed origins from environment variable
    let allowed_origins_str = std::env::var("ENTERPRISE_CORS_ORIGINS")
        .unwrap_or_else(|_| String::new());

    if allowed_origins_str.is_empty() {
        tracing::warn!(
            "ENTERPRISE_CORS_ORIGINS not set - CORS will reject all origins. \
            Set ENTERPRISE_CORS_ORIGINS environment variable for production use."
        );
    }

    // Parse origins from comma-separated string
    let allowed_origins: Vec<HeaderValue> = allowed_origins_str
        .split(',')
        .filter_map(|origin| {
            let trimmed = origin.trim();
            if trimmed.is_empty() {
                None
            } else {
                match HeaderValue::from_str(trimmed) {
                    Ok(header) => {
                        tracing::info!("CORS: Allowing origin: {}", trimmed);
                        Some(header)
                    }
                    Err(e) => {
                        tracing::error!("CORS: Invalid origin '{}': {}", trimmed, e);
                        None
                    }
                }
            }
        })
        .collect();

    // Build CORS layer with strict configuration
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true)
}

/// Authentication middleware - verifies JWT tokens
async fn auth_middleware(
    State(auth_service): State<Arc<RwLock<AuthService>>>,
    mut req: Request<Body>,
    next: middleware::Next,
) -> std::result::Result<Response, StatusCode> {
    // 1. Extract Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. Extract token (format: "Bearer <token>")
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 3. Verify JWT token using AuthService
    let auth = auth_service.read().await;
    let identity_id = auth
        .verify_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 4. Add identity_id to request extensions for use in handlers
    req.extensions_mut().insert(identity_id);

    // 5. Call next middleware/handler
    Ok(next.run(req).await)
}
