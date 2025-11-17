// Wallet Service API Endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, post},
    Router,
};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::services::WalletService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for wallet endpoints
pub fn routes(service: Arc<RwLock<WalletService>>) -> Router {
    Router::new()
        .route("/create", post(create_wallet))
        .route("/:id", get(get_wallet))
        .route("/:id/balances", get(get_balances))
        .route("/:id/transactions", get(get_transactions))
        .route("/:id/transfer", post(transfer))
        .route("/:id/sync", post(sync_balances))
        .route("/identity/:identity_id", get(get_identity_wallets))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct CreateWalletRequest {
    identity_id: Uuid,
    labels: Vec<String>,
    /// FIX M-8: Optional derivation path (defaults to "m/0" if not provided)
    #[serde(default)]
    derivation_path: Option<String>,
}

#[derive(Deserialize)]
struct TransferRequest {
    to_address: String,
    asset_type: AssetType,
    amount: u64,
}

#[derive(Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    100
}

// Endpoint handlers

async fn create_wallet(
    State(service): State<Arc<RwLock<WalletService>>>,
    Json(req): Json<CreateWalletRequest>,
) -> Result<Json<WalletAccount>, EnterpriseError> {
    let service = service.read().await;
    // FIX M-8: Pass optional derivation_path to service
    let wallet = service.create_wallet(req.identity_id, req.labels, req.derivation_path).await?;
    Ok(Json(wallet))
}

async fn get_wallet(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WalletAccount>, EnterpriseError> {
    let service = service.read().await;
    let wallet = service.get_wallet(id).await?;
    Ok(Json(wallet))
}

async fn get_balances(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<WalletBalance>>, EnterpriseError> {
    let service = service.read().await;
    let balances = service.get_balances(id).await?;
    Ok(Json(balances))
}

async fn get_transactions(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<Vec<WalletTransaction>>, EnterpriseError> {
    let service = service.read().await;
    let transactions = service.get_transactions(id, pagination.limit, pagination.offset).await?;
    Ok(Json(transactions))
}

async fn transfer(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<WalletTransaction>, EnterpriseError> {
    let service = service.read().await;
    let transaction = service.transfer(id, req.to_address, req.asset_type, req.amount).await?;
    Ok(Json(transaction))
}

async fn sync_balances(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.sync_balances(id).await?;
    Ok(Json(()))
}

async fn get_identity_wallets(
    State(service): State<Arc<RwLock<WalletService>>>,
    Path(identity_id): Path<Uuid>,
) -> Result<Json<Vec<WalletAccount>>, EnterpriseError> {
    let service = service.read().await;
    let wallets = service.get_identity_wallets(identity_id).await?;
    Ok(Json(wallets))
}
