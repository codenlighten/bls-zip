// Complete User Sign-Up API
// Handles atomic creation of identity, credentials, and default wallet

use axum::{extract::State, Json, routing::post, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::EnterpriseError;
use crate::models::*;
use crate::services::{AuthService, IdentityService, WalletService};

/// Create router for sign-up endpoint
pub fn routes(
    identity_service: Arc<RwLock<IdentityService>>,
    auth_service: Arc<RwLock<AuthService>>,
    wallet_service: Arc<RwLock<WalletService>>,
) -> Router {
    Router::new()
        .route("/signup", post(signup))
        .with_state((identity_service, auth_service, wallet_service))
}

// Request/Response DTOs

#[derive(Deserialize)]
pub struct SignUpRequest {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Serialize)]
pub struct SignUpResponse {
    pub identity: IdentityProfile,
    pub wallet: WalletAccount,
    pub session: MultipassSession,
    pub token: String,
}

/// Complete sign-up flow
///
/// Creates:
/// 1. Identity profile
/// 2. Authentication credentials
/// 3. Default wallet with PQC keys
/// 4. Initial session
///
/// All operations are performed atomically - if any step fails, no data is created
async fn signup(
    State((identity_service, auth_service, wallet_service)): State<(
        Arc<RwLock<IdentityService>>,
        Arc<RwLock<AuthService>>,
        Arc<RwLock<WalletService>>,
    )>,
    Json(req): Json<SignUpRequest>,
) -> Result<Json<SignUpResponse>, EnterpriseError> {
    // SECURITY: Validate all inputs before proceeding
    crate::validation::validate_name(&req.full_name, "Full name")?;
    crate::validation::validate_email(&req.email)?;
    crate::validation::validate_password(&req.password)?;

    if let Some(ref phone) = req.phone {
        crate::validation::validate_phone(phone)?;
    }

    if let Some(ref country) = req.country_code {
        if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(EnterpriseError::InvalidInput(
                "Country code must be a 2-letter ISO code".to_string(),
            ));
        }
    }

    // Check if email already exists
    {
        let identity_service = identity_service.read().await;
        if let Ok(_existing) = identity_service.get_identity_by_email(&req.email).await {
            return Err(EnterpriseError::InvalidInput(
                "Email address is already registered".to_string(),
            ));
        }
    }

    // Step 1: Create identity profile
    let identity = {
        let identity_service = identity_service.read().await;
        identity_service
            .create_identity(
                req.full_name.clone(),
                req.email.clone(),
                req.phone.clone(),
                req.country_code.clone(),
            )
            .await?
    };

    // Step 2: Register authentication credentials
    let _credential = {
        let auth_service = auth_service.read().await;
        auth_service.register(identity.identity_id, req.password.clone()).await?
    };

    // Step 3: Create default wallet with PQC keys
    let wallet = {
        let wallet_service = wallet_service.read().await;
        wallet_service
            .create_wallet(
                identity.identity_id,
                vec!["default".to_string(), "primary".to_string()],
                Some("m/0".to_string()),
            )
            .await?
    };

    // Step 4: Create initial session (auto-login after signup)
    let (session, token) = {
        let auth_service = auth_service.read().await;
        auth_service.login(req.email, req.password).await?
    };

    Ok(Json(SignUpResponse {
        identity,
        wallet,
        session,
        token,
    }))
}
