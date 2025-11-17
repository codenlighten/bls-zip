// Identity & Attestation API Endpoints

use axum::{
    extract::{Path, State},
    Json,
    routing::{get, post, put, delete},
    Router,
};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::services::IdentityService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for identity endpoints
pub fn routes(service: Arc<RwLock<IdentityService>>) -> Router {
    Router::new()
        .route("/create", post(create_identity))
        .route("/:id", get(get_identity))
        .route("/email/:email", get(get_identity_by_email))
        .route("/:id/kyc-status", put(update_kyc_status))
        .route("/:id/attestations", post(create_attestation))
        .route("/:id/attestations", get(get_attestations))
        .route("/attestations/:attestation_id", delete(revoke_attestation))
        .route("/list", get(list_identities))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct CreateIdentityRequest {
    full_name: String,
    email: String,
    phone: Option<String>,
    country_code: Option<String>,
}

#[derive(Serialize)]
struct CreateIdentityResponse {
    identity: IdentityProfile,
}

#[derive(Deserialize)]
struct UpdateKycStatusRequest {
    verification_status: String,
    kyc_level: i32,
}

#[derive(Deserialize)]
struct CreateAttestationRequest {
    attestation_type: AttestationType,
    evidence_refs: Vec<String>,
    issuer: String,
    valid_to: Option<chrono::DateTime<chrono::Utc>>,
    anchor_to_chain: bool,
}

// Endpoint handlers

async fn create_identity(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Json(req): Json<CreateIdentityRequest>,
) -> Result<Json<CreateIdentityResponse>, EnterpriseError> {
    let service = service.read().await;
    let identity = service.create_identity(
        req.full_name,
        req.email,
        req.phone,
        req.country_code,
    ).await?;

    Ok(Json(CreateIdentityResponse {
        identity,
    }))
}

async fn get_identity(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<IdentityProfile>, EnterpriseError> {
    let service = service.read().await;
    let identity = service.get_identity(id).await?;
    Ok(Json(identity))
}

async fn get_identity_by_email(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(email): Path<String>,
) -> Result<Json<IdentityProfile>, EnterpriseError> {
    let service = service.read().await;
    let identity = service.get_identity_by_email(&email).await?;
    Ok(Json(identity))
}

async fn update_kyc_status(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateKycStatusRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.update_kyc_status(id, req.verification_status, req.kyc_level).await?;
    Ok(Json(()))
}

async fn create_attestation(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateAttestationRequest>,
) -> Result<Json<IdentityAttestation>, EnterpriseError> {
    let service = service.read().await;
    let attestation = service.create_attestation(
        id,
        req.attestation_type,
        req.evidence_refs,
        req.issuer,
        req.valid_to,
        req.anchor_to_chain,
    ).await?;
    Ok(Json(attestation))
}

async fn get_attestations(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<IdentityAttestation>>, EnterpriseError> {
    let service = service.read().await;
    let attestations = service.get_attestations(id).await?;
    Ok(Json(attestations))
}

async fn revoke_attestation(
    State(service): State<Arc<RwLock<IdentityService>>>,
    Path(attestation_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.revoke_attestation(attestation_id).await?;
    Ok(Json(()))
}

async fn list_identities(
    State(service): State<Arc<RwLock<IdentityService>>>,
) -> Result<Json<Vec<IdentityProfile>>, EnterpriseError> {
    let service = service.read().await;
    let identities = service.list_identities(100, 0).await?;
    Ok(Json(identities))
}
