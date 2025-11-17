// Asset & Market Service API Endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, post, put, delete},
    Router,
};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::services::AssetService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for asset endpoints
pub fn routes(service: Arc<RwLock<AssetService>>) -> Router {
    Router::new()
        .route("/define", post(define_asset))
        .route("/list", get(list_assets))
        .route("/:asset_id", get(get_asset))
        .route("/:asset_id/issue", post(issue_asset))
        .route("/:asset_id/transfer", post(transfer_asset))
        .route("/:asset_id/balance/:wallet_id", get(get_balance))
        .with_state(service)
}

/// Create router for market endpoints
pub fn market_routes(service: Arc<RwLock<AssetService>>) -> Router {
    Router::new()
        .route("/orders", post(create_order))
        .route("/orders/:order_id", get(get_order))
        .route("/orders/:order_id/cancel", put(cancel_order))
        .route("/wallet/:wallet_id/orders", get(list_wallet_orders))
        .route("/orderbook/:asset_id", get(get_orderbook))
        .route("/positions/:wallet_id", get(get_positions))
        .route("/trades/:asset_id", get(get_trades))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct DefineAssetRequest {
    name: String,
    symbol: String,
    asset_type: AssetType,
    total_supply: u64,
    metadata: serde_json::Value,
}

#[derive(Serialize)]
struct DefineAssetResponse {
    asset: AssetDefinition,
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Deserialize)]
struct IssueAssetRequest {
    to_wallet: Uuid,
    amount: u64,
}

#[derive(Deserialize)]
struct TransferAssetRequest {
    from_wallet: Uuid,
    to_wallet: Uuid,
    amount: u64,
}

#[derive(Deserialize)]
struct CreateOrderRequest {
    wallet_id: Uuid,
    asset_id: Uuid,
    order_type: OrderType,
    quantity: u64,
    price: u64,
}

#[derive(Serialize)]
struct CreateOrderResponse {
    order: MarketOrder,
}

// Asset endpoint handlers

async fn define_asset(
    State(service): State<Arc<RwLock<AssetService>>>,
    Json(req): Json<DefineAssetRequest>,
) -> Result<Json<DefineAssetResponse>, EnterpriseError> {
    let service = service.read().await;

    // Convert u64 to i64 for total_supply
    let total_supply_i64: i64 = req.total_supply.try_into()
        .map_err(|_| EnterpriseError::InvalidInput("Total supply too large".to_string()))?;

    let asset = service
        .define_asset(
            req.asset_type,
            req.symbol,
            req.name,
            total_supply_i64,
            req.metadata,
        )
        .await?;

    Ok(Json(DefineAssetResponse { asset }))
}

async fn list_assets(
    State(service): State<Arc<RwLock<AssetService>>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<AssetDefinition>>, EnterpriseError> {
    let service = service.read().await;
    let assets = service
        .list_assets(query.limit, query.offset)
        .await?;

    Ok(Json(assets))
}

async fn get_asset(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<AssetDefinition>, EnterpriseError> {
    let service = service.read().await;
    let asset = service.get_asset(asset_id).await?;

    Ok(Json(asset))
}

async fn issue_asset(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(asset_id): Path<Uuid>,
    Json(req): Json<IssueAssetRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service
        .issue_asset(asset_id, req.to_wallet, req.amount)
        .await?;

    Ok(Json(()))
}

async fn transfer_asset(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(asset_id): Path<Uuid>,
    Json(req): Json<TransferAssetRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service
        .transfer_asset(asset_id, req.from_wallet, req.to_wallet, req.amount)
        .await?;

    Ok(Json(()))
}

async fn get_balance(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path((asset_id, wallet_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<AssetBalance>, EnterpriseError> {
    let service = service.read().await;
    let balance = service.get_balance(wallet_id, asset_id).await?;

    Ok(Json(balance))
}

// Market endpoint handlers

async fn create_order(
    State(service): State<Arc<RwLock<AssetService>>>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<CreateOrderResponse>, EnterpriseError> {
    let service = service.read().await;

    // Convert u64 to i64 for quantity and price
    let quantity_i64: i64 = req.quantity.try_into()
        .map_err(|_| EnterpriseError::InvalidInput("Quantity too large".to_string()))?;
    let price_i64: i64 = req.price.try_into()
        .map_err(|_| EnterpriseError::InvalidInput("Price too large".to_string()))?;

    let order = service
        .create_order(
            req.wallet_id,
            req.asset_id,
            req.order_type,
            quantity_i64,
            price_i64,
        )
        .await?;

    Ok(Json(CreateOrderResponse { order }))
}

async fn get_order(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<MarketOrder>, EnterpriseError> {
    let service = service.read().await;
    let order = service.get_order(order_id).await?;

    Ok(Json(order))
}

async fn cancel_order(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.cancel_order(order_id).await?;

    Ok(Json(()))
}

async fn list_wallet_orders(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(wallet_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<MarketOrder>>, EnterpriseError> {
    let service = service.read().await;
    let orders = service
        .get_wallet_orders(wallet_id, query.limit, query.offset)
        .await?;

    Ok(Json(orders))
}

async fn get_orderbook(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(asset_id): Path<Uuid>,
) -> Result<Json<OrderBook>, EnterpriseError> {
    let service = service.read().await;
    let orderbook = service.get_orderbook(asset_id).await?;

    Ok(Json(orderbook))
}

async fn get_positions(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(wallet_id): Path<Uuid>,
) -> Result<Json<Vec<AssetPosition>>, EnterpriseError> {
    let service = service.read().await;
    let positions = service.get_positions(wallet_id).await?;

    Ok(Json(positions))
}

async fn get_trades(
    State(service): State<Arc<RwLock<AssetService>>>,
    Path(asset_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Trade>>, EnterpriseError> {
    let service = service.read().await;
    let trades = service
        .get_trades(asset_id, query.limit, query.offset)
        .await?;

    Ok(Json(trades))
}
