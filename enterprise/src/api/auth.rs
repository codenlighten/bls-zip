// Auth/SSO Service API Endpoints

use axum::{
    extract::{Path, State},
    Json,
    routing::{get, post, put},
    Router,
    middleware,
};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::services::AuthService;
use crate::models::*;
use crate::error::EnterpriseError;
use crate::rate_limit::RateLimiter;
use crate::middleware::rate_limit::rate_limit_ip_only;

/// Create router for auth endpoints
/// FIX M-11: Added rate limiting to login endpoint
pub fn routes(service: Arc<RwLock<AuthService>>, limiter: Arc<RateLimiter>) -> Router {
    // Login route with rate limiting to prevent brute force attacks
    let login_route = Router::new()
        .route("/login", post(login))
        .layer(middleware::from_fn_with_state(
            limiter.clone(),
            rate_limit_ip_only
        ))
        .with_state(service.clone());

    // Other routes without rate limiting (or with different limits)
    Router::new()
        .route("/register", post(register))
        .route("/logout", post(logout))
        .route("/refresh/:session_id", post(refresh))
        .route("/verify", post(verify_token))
        .route("/session/:session_id", get(get_session))
        .route("/sessions/:identity_id", get(get_identity_sessions))
        .merge(login_route)
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct RegisterRequest {
    identity_id: Uuid,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    credential_id: Uuid,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    session: MultipassSession,
    token: String,
}

#[derive(Deserialize)]
struct LogoutRequest {
    session_id: Uuid,
}

#[derive(Deserialize)]
struct VerifyTokenRequest {
    token: String,
}

#[derive(Serialize)]
struct VerifyTokenResponse {
    identity_id: Uuid,
    valid: bool,
}

// Endpoint handlers

async fn register(
    State(service): State<Arc<RwLock<AuthService>>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, EnterpriseError> {
    let service = service.read().await;
    let credential = service.register(req.identity_id, req.password).await?;
    Ok(Json(RegisterResponse {
        credential_id: credential.credential_id,
    }))
}

async fn login(
    State(service): State<Arc<RwLock<AuthService>>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, EnterpriseError> {
    let service = service.read().await;
    let (session, token) = service.login(req.email, req.password).await?;
    Ok(Json(LoginResponse { session, token }))
}

async fn logout(
    State(service): State<Arc<RwLock<AuthService>>>,
    Json(req): Json<LogoutRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.logout(req.session_id).await?;
    Ok(Json(()))
}

async fn refresh(
    State(service): State<Arc<RwLock<AuthService>>>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<String>, EnterpriseError> {
    let service = service.read().await;
    let token = service.refresh_session(session_id).await?;
    Ok(Json(token))
}

async fn verify_token(
    State(service): State<Arc<RwLock<AuthService>>>,
    Json(req): Json<VerifyTokenRequest>,
) -> Result<Json<VerifyTokenResponse>, EnterpriseError> {
    let service = service.read().await;
    match service.verify_token(&req.token).await {
        Ok(identity_id) => Ok(Json(VerifyTokenResponse {
            identity_id,
            valid: true,
        })),
        Err(_) => Ok(Json(VerifyTokenResponse {
            identity_id: Uuid::nil(),
            valid: false,
        })),
    }
}

async fn get_session(
    State(service): State<Arc<RwLock<AuthService>>>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<MultipassSession>, EnterpriseError> {
    let service = service.read().await;
    let session = service.get_session(session_id).await?;
    Ok(Json(session))
}

async fn get_identity_sessions(
    State(service): State<Arc<RwLock<AuthService>>>,
    Path(identity_id): Path<Uuid>,
) -> Result<Json<Vec<MultipassSession>>, EnterpriseError> {
    let service = service.read().await;
    let sessions = service.get_identity_sessions(identity_id).await?;
    Ok(Json(sessions))
}
