// Example: Secure API Server with Rate Limiting
//
// This example demonstrates how to set up a secure Axum API server with:
// - Rate limiting (IP and user-based)
// - Input validation
// - Audit logging
// - JWT authentication
//
// Run with: cargo run --example secure_api_server

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use boundless_enterprise::{
    middleware::{rate_limit_ip_only, rate_limit_middleware, set_user_id, UserId},
    rate_limit::{RateLimiter, start_cleanup_task},
    validation,
    // audit::AuditLogger, // Uncomment when audit module is enabled
};

/// Application state
#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
    // audit_logger: Arc<AuditLogger>, // Uncomment when audit module is enabled
}

/// Login request
#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

/// Login response
#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user_id: Uuid,
}

/// Create user request
#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    full_name: String,
    email: String,
    phone: Option<String>,
    password: String,
}

/// User response
#[derive(Debug, Serialize)]
struct UserResponse {
    user_id: Uuid,
    full_name: String,
    email: String,
}

/// Health check handler (public, IP rate limited only)
async fn health_check() -> &'static str {
    "OK"
}

/// Login handler (public, IP rate limited only)
async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // SECURITY: Validate email format
    validation::validate_email(&payload.email)?;

    // TODO: Verify credentials against database
    // For demo purposes, create a mock user ID
    let user_id = Uuid::new_v4();

    // TODO: Log successful authentication
    // state.audit_logger.log_auth_success(user_id, &addr.ip().to_string()).await?;

    Ok(Json(LoginResponse {
        token: "mock_jwt_token".to_string(),
        user_id,
    }))
}

/// Create user handler (public, IP rate limited only)
async fn create_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // SECURITY: Validate all inputs
    validation::validate_name(&payload.full_name, "Full name")?;
    validation::validate_email(&payload.email)?;
    validation::validate_password(&payload.password)?;

    if let Some(ref phone) = payload.phone {
        validation::validate_phone(phone)?;
    }

    // TODO: Create user in database
    let user_id = Uuid::new_v4();

    // TODO: Log user creation
    // state.audit_logger.log_data_modification(
    //     user_id,
    //     &format!("user:{}", user_id),
    //     "create_user",
    //     serde_json::json!({ "email": payload.email })
    // ).await?;

    Ok(Json(UserResponse {
        user_id,
        full_name: payload.full_name,
        email: payload.email,
    }))
}

/// Get user profile (authenticated, user + IP rate limited)
async fn get_profile(
    user_id: UserId,
) -> Result<Json<UserResponse>, ApiError> {
    // TODO: Fetch user from database
    Ok(Json(UserResponse {
        user_id: user_id.0,
        full_name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    }))
}

/// Mock authentication middleware
///
/// In production, this would verify JWT tokens and extract the user ID
async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // TODO: Extract and verify JWT token from Authorization header
    // For demo purposes, inject a mock user ID
    let mock_user_id = Uuid::new_v4();
    set_user_id(&mut request, mock_user_id);

    Ok(next.run(request).await)
}

/// API error type
#[derive(Debug)]
enum ApiError {
    Validation(String),
    Internal(String),
}

impl From<boundless_enterprise::error::EnterpriseError> for ApiError {
    fn from(err: boundless_enterprise::error::EnterpriseError) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": message
        });

        (status, Json(body)).into_response()
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔐 Starting Secure API Server Example...\n");

    // Initialize rate limiter
    let rate_limiter = Arc::new(RateLimiter::new());
    println!("✅ Rate limiter initialized");
    println!("   - IP limit: 100 requests/minute");
    println!("   - User limit: 200 requests/minute");
    println!("   - Burst protection: 10 extra requests\n");

    // Start automatic cleanup task
    start_cleanup_task(rate_limiter.clone());
    println!("✅ Rate limiter cleanup task started\n");

    // Initialize application state
    let app_state = AppState {
        rate_limiter: rate_limiter.clone(),
    };

    // Build public routes (IP rate limited only)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/auth/login", post(login))
        .route("/users", post(create_user))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_ip_only,
        ))
        .with_state(app_state.clone());

    // Build protected routes (authenticated, user + IP rate limited)
    let protected_routes = Router::new()
        .route("/profile", get(get_profile))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
        .with_state(app_state.clone());

    // Combine routes
    let app = Router::new()
        .nest("/api/v1", public_routes)
        .nest("/api/v1", protected_routes)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server starting on {}\n", addr);

    println!("📋 Available endpoints:");
    println!("   GET  /api/v1/health       - Health check");
    println!("   POST /api/v1/auth/login   - Login");
    println!("   POST /api/v1/users        - Create user");
    println!("   GET  /api/v1/profile      - Get profile (authenticated)\n");

    println!("🔒 Security features enabled:");
    println!("   ✅ Rate limiting (IP and user-based)");
    println!("   ✅ Input validation");
    println!("   ✅ JWT authentication (mock)");
    println!("   ✅ Password strength enforcement");
    println!("   ⏳ Audit logging (awaiting database setup)\n");

    println!("💡 Try it out:");
    println!("   curl http://localhost:3000/api/v1/health");
    println!("   curl -X POST http://localhost:3000/api/v1/users \\");
    println!("     -H 'Content-Type: application/json' \\");
    println!("     -d '{{\"full_name\":\"John Doe\",\"email\":\"john@example.com\",\"password\":\"SecurePass123!\"}}'");
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
