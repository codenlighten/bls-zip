// Contract API endpoints
//
// Provides REST API for smart contract deployment and management

use crate::error::{EnterpriseError, Result};
use crate::services::contract::*;
use crate::services::ContractService;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Contract API routes (protected)
pub fn routes(contract_service: Arc<RwLock<ContractService>>) -> Router {
    Router::new()
        .route("/deploy", post(deploy_contract))
        .route("/list", get(list_contracts))
        .route("/:contract_id", get(get_contract))
        .route("/:contract_id/call", post(call_contract))
        .route("/:contract_id/send", post(send_transaction))
        .route("/:contract_id/interactions", get(get_interactions))
        .with_state(contract_service)
}

/// Public contract routes (no authentication required)
pub fn public_routes(contract_service: Arc<RwLock<ContractService>>) -> Router {
    Router::new()
        .route("/templates", get(get_templates))
        .with_state(contract_service)
}

/// Get available contract templates
///
/// GET /api/contracts/templates
/// Public endpoint - no authentication required
async fn get_templates(
    State(service): State<Arc<RwLock<ContractService>>>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;

    // For now, we don't filter by category - the frontend handles filtering
    let templates = service.get_templates(None)?;

    Ok(Json(templates))
}

/// Deploy a new smart contract
///
/// POST /api/contracts/deploy
/// Requires authentication
async fn deploy_contract(
    Extension(identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
    Json(request): Json<DeployContractRequest>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let contract = service.deploy_contract(identity_id, request).await?;

    Ok((StatusCode::CREATED, Json(json!({
        "success": true,
        "contract": contract
    }))))
}

/// List contracts for the authenticated user
///
/// GET /api/contracts/list
/// Requires authentication
async fn list_contracts(
    Extension(identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let contracts = service.list_contracts(identity_id).await?;

    Ok(Json(json!({
        "success": true,
        "contracts": contracts,
        "count": contracts.len()
    })))
}

/// Get a specific contract
///
/// GET /api/contracts/:contract_id
/// Requires authentication
async fn get_contract(
    Extension(_identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
    Path(contract_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let contract = service.get_contract(contract_id).await?;

    Ok(Json(json!({
        "success": true,
        "contract": contract
    })))
}

/// Call a contract method (read-only)
///
/// POST /api/contracts/:contract_id/call
/// Requires authentication
async fn call_contract(
    Extension(identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
    Path(contract_id): Path<Uuid>,
    Json(request): Json<ContractCallRequest>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let result = service.call_contract(identity_id, contract_id, request).await?;

    Ok(Json(json!({
        "success": true,
        "result": result
    })))
}

/// Send a transaction to a contract (state-changing)
///
/// POST /api/contracts/:contract_id/send
/// Requires authentication
async fn send_transaction(
    Extension(identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
    Path(contract_id): Path<Uuid>,
    Json(request): Json<ContractCallRequest>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let result = service.send_transaction(identity_id, contract_id, request).await?;

    Ok(Json(json!({
        "success": true,
        "result": result
    })))
}

/// Get contract interactions history
///
/// GET /api/contracts/:contract_id/interactions
/// Requires authentication
async fn get_interactions(
    Extension(_identity_id): Extension<Uuid>,
    State(service): State<Arc<RwLock<ContractService>>>,
    Path(contract_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let service = service.read().await;
    let interactions = service.get_interactions(contract_id).await?;

    Ok(Json(json!({
        "success": true,
        "interactions": interactions,
        "count": interactions.len()
    })))
}
