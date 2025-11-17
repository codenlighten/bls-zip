// Hardware Pass Service API Endpoints

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

use crate::services::HardwareService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for hardware endpoints
pub fn routes(service: Arc<RwLock<HardwareService>>) -> Router {
    Router::new()
        .route("/register", post(register_device))
        .route("/:device_id", get(get_device))
        .route("/:device_id", delete(delete_device))
        .route("/identity/:identity_id", get(get_identity_devices))
        .route("/:device_id/authenticate", post(authenticate))
        .route("/:device_id/challenge", post(generate_challenge))
        .route("/:device_id/revoke", put(revoke_device))
        .route("/:device_id/lost", put(mark_device_lost))
        .route("/:device_id/capability", post(check_capability))
        .route("/:device_id/stats", get(get_device_stats))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct RegisterDeviceRequest {
    identity_id: Uuid,
    device_type: String,
    public_key: String,
    capabilities: Vec<String>,
}

#[derive(Serialize)]
struct RegisterDeviceResponse {
    device: HardwarePass,
}

#[derive(Deserialize)]
struct AuthenticateRequest {
    challenge: Vec<u8>,
    signature: Vec<u8>,
}

#[derive(Serialize)]
struct AuthenticateResponse {
    valid: bool,
}

#[derive(Serialize)]
struct ChallengeResponse {
    challenge: Vec<u8>,
}

#[derive(Deserialize)]
struct CheckCapabilityRequest {
    capability: String,
}

#[derive(Serialize)]
struct CheckCapabilityResponse {
    has_capability: bool,
}

// Endpoint handlers

async fn register_device(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, EnterpriseError> {
    let service = service.read().await;
    let device = service
        .register_device(
            req.identity_id,
            req.device_type,
            req.public_key,
            req.capabilities,
        )
        .await?;

    Ok(Json(RegisterDeviceResponse { device }))
}

async fn get_device(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<HardwarePass>, EnterpriseError> {
    let service = service.read().await;
    let device = service.get_device(device_id).await?;

    Ok(Json(device))
}

async fn delete_device(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.delete_device(device_id).await?;

    Ok(Json(()))
}

async fn get_identity_devices(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(identity_id): Path<Uuid>,
) -> Result<Json<Vec<HardwarePass>>, EnterpriseError> {
    let service = service.read().await;
    let devices = service.get_identity_devices(identity_id).await?;

    Ok(Json(devices))
}

async fn authenticate(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<AuthenticateRequest>,
) -> Result<Json<AuthenticateResponse>, EnterpriseError> {
    let service = service.read().await;
    let valid = service
        .authenticate(device_id, req.challenge, req.signature)
        .await?;

    Ok(Json(AuthenticateResponse { valid }))
}

async fn generate_challenge(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<ChallengeResponse>, EnterpriseError> {
    let service = service.read().await;
    let challenge = service.generate_challenge(device_id).await?;

    Ok(Json(ChallengeResponse { challenge }))
}

async fn revoke_device(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.revoke(device_id).await?;

    Ok(Json(()))
}

async fn mark_device_lost(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.mark_lost(device_id).await?;

    Ok(Json(()))
}

async fn check_capability(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<CheckCapabilityRequest>,
) -> Result<Json<CheckCapabilityResponse>, EnterpriseError> {
    let service = service.read().await;
    let has_capability = service
        .has_capability(device_id, &req.capability)
        .await?;

    Ok(Json(CheckCapabilityResponse { has_capability }))
}

async fn get_device_stats(
    State(service): State<Arc<RwLock<HardwareService>>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<crate::services::hardware::DeviceStats>, EnterpriseError> {
    let service = service.read().await;
    let stats = service.get_device_stats(device_id).await?;

    Ok(Json(stats))
}
