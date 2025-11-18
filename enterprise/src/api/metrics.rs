// Metrics & Analytics API Endpoints

use axum::{
    extract::{Path, Query, State},
    Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::services::MetricsService;
use crate::services::metrics::*;
use crate::error::EnterpriseError;

/// Create router for metrics endpoints
pub fn routes(service: Arc<RwLock<MetricsService>>) -> Router {
    Router::new()
        .route("/dashboard", get(get_dashboard_summary))
        .route("/blockchain", get(get_blockchain_metrics))
        .route("/users", get(get_user_metrics))
        .route("/system", get(get_system_metrics))
        .route("/sustainability", get(get_sustainability_metrics))
        .route("/timeseries/:metric_name", get(get_metric_timeseries))
        .route("/collect/blockchain", post(collect_blockchain_metrics))
        .route("/collect/users", post(collect_user_metrics))
        .route("/collect/system", post(collect_system_metrics))
        .route("/collect/sustainability", post(collect_sustainability_metrics))
        .route("/collect/all", post(collect_all_metrics))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Serialize)]
struct DashboardResponse {
    dashboard: DashboardSummary,
}

#[derive(Serialize)]
struct BlockchainMetricsResponse {
    metrics: BlockchainMetricsSnapshot,
}

#[derive(Serialize)]
struct UserMetricsResponse {
    metrics: UserActivityMetrics,
}

#[derive(Serialize)]
struct SystemMetricsResponse {
    metrics: SystemPerformanceMetrics,
}

#[derive(Serialize)]
struct SustainabilityMetricsResponse {
    metrics: SustainabilityMetrics,
}

#[derive(Serialize)]
struct TimeseriesResponse {
    metric_name: String,
    data_points: Vec<PlatformMetric>,
    count: usize,
}

#[derive(Deserialize)]
struct TimeseriesQuery {
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: Option<i64>,
}

#[derive(Serialize)]
struct CollectResponse {
    success: bool,
    message: String,
}

// Endpoint handlers

/// Get comprehensive dashboard summary
async fn get_dashboard_summary(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<DashboardResponse>, EnterpriseError> {
    let service = service.read().await;
    let dashboard = service.get_dashboard_summary().await?;

    Ok(Json(DashboardResponse { dashboard }))
}

/// Get latest blockchain metrics
async fn get_blockchain_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<BlockchainMetricsResponse>, EnterpriseError> {
    let service = service.read().await;
    let metrics = service.get_latest_blockchain_metrics().await?
        .ok_or_else(|| EnterpriseError::NotFound("Blockchain metrics not available".to_string()))?;

    Ok(Json(BlockchainMetricsResponse { metrics }))
}

/// Get latest user activity metrics
async fn get_user_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<UserMetricsResponse>, EnterpriseError> {
    let service = service.read().await;
    let metrics = service.get_latest_user_metrics().await?
        .ok_or_else(|| EnterpriseError::NotFound("User metrics not available".to_string()))?;

    Ok(Json(UserMetricsResponse { metrics }))
}

/// Get latest system performance metrics
async fn get_system_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<SystemMetricsResponse>, EnterpriseError> {
    let service = service.read().await;
    let metrics = service.get_latest_system_metrics().await?
        .ok_or_else(|| EnterpriseError::NotFound("System metrics not available".to_string()))?;

    Ok(Json(SystemMetricsResponse { metrics }))
}

/// Get latest sustainability metrics
async fn get_sustainability_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<SustainabilityMetricsResponse>, EnterpriseError> {
    let service = service.read().await;
    let metrics = service.get_latest_sustainability_metrics().await?
        .ok_or_else(|| EnterpriseError::NotFound("Sustainability metrics not available".to_string()))?;

    Ok(Json(SustainabilityMetricsResponse { metrics }))
}

/// Get time-series data for a specific metric
async fn get_metric_timeseries(
    State(service): State<Arc<RwLock<MetricsService>>>,
    Path(metric_name): Path<String>,
    Query(query): Query<TimeseriesQuery>,
) -> Result<Json<TimeseriesResponse>, EnterpriseError> {
    let service = service.read().await;

    let now = Utc::now();
    let from = query.from.unwrap_or_else(|| now - chrono::Duration::hours(24));
    let to = query.to.unwrap_or(now);
    let limit = query.limit.unwrap_or(1000);

    let data_points = service.get_metric_timeseries(&metric_name, from, to, limit).await?;
    let count = data_points.len();

    Ok(Json(TimeseriesResponse {
        metric_name,
        data_points,
        count,
    }))
}

/// Trigger blockchain metrics collection
async fn collect_blockchain_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<CollectResponse>, EnterpriseError> {
    let service = service.read().await;
    service.collect_blockchain_metrics().await?;

    Ok(Json(CollectResponse {
        success: true,
        message: "Blockchain metrics collected successfully".to_string(),
    }))
}

/// Trigger user metrics collection
async fn collect_user_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<CollectResponse>, EnterpriseError> {
    let service = service.read().await;
    service.collect_user_metrics().await?;

    Ok(Json(CollectResponse {
        success: true,
        message: "User metrics collected successfully".to_string(),
    }))
}

/// Trigger system metrics collection
async fn collect_system_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<CollectResponse>, EnterpriseError> {
    let service = service.read().await;
    service.collect_system_metrics().await?;

    Ok(Json(CollectResponse {
        success: true,
        message: "System metrics collected successfully".to_string(),
    }))
}

/// Trigger sustainability metrics collection
async fn collect_sustainability_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<CollectResponse>, EnterpriseError> {
    let service = service.read().await;
    service.calculate_sustainability_metrics().await?;

    Ok(Json(CollectResponse {
        success: true,
        message: "Sustainability metrics collected successfully".to_string(),
    }))
}

/// Trigger collection of all metrics
async fn collect_all_metrics(
    State(service): State<Arc<RwLock<MetricsService>>>,
) -> Result<Json<CollectResponse>, EnterpriseError> {
    let service = service.read().await;

    // Collect all metrics in parallel
    let (blockchain_result, user_result, system_result, sustainability_result) = tokio::join!(
        service.collect_blockchain_metrics(),
        service.collect_user_metrics(),
        service.collect_system_metrics(),
        service.calculate_sustainability_metrics(),
    );

    // Check for errors
    blockchain_result?;
    user_result?;
    system_result?;
    sustainability_result?;

    Ok(Json(CollectResponse {
        success: true,
        message: "All metrics collected successfully".to_string(),
    }))
}
