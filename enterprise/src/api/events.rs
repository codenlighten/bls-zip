// Event & Reporting Service API Endpoints

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

use crate::services::EventService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for notification endpoints
pub fn notification_routes(service: Arc<RwLock<EventService>>) -> Router {
    Router::new()
        .route("/", post(create_notification))
        .route("/:notification_id", get(get_notification))
        .route("/:notification_id/read", put(mark_as_read))
        .route("/:notification_id", delete(delete_notification))
        .route("/identity/:identity_id", get(get_identity_notifications))
        .route("/identity/:identity_id/unread", get(get_unread_count))
        .route("/identity/:identity_id/mark_all_read", put(mark_all_as_read))
        .with_state(service)
}

/// Create router for report endpoints
pub fn report_routes(service: Arc<RwLock<EventService>>) -> Router {
    Router::new()
        .route("/definitions", post(create_report_definition))
        .route("/definitions", get(list_report_definitions))
        .route("/definitions/:report_id", get(get_report_definition))
        .route("/definitions/:report_id", delete(delete_report_definition))
        .route("/generate", post(generate_report))
        .route("/:generated_report_id", get(get_generated_report))
        .route("/identity/:identity_id", get(list_generated_reports))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct CreateNotificationRequest {
    identity_id: Uuid,
    notification_type: NotificationType,
    title: String,
    message: String,
    metadata: serde_json::Value,
}

#[derive(Serialize)]
struct CreateNotificationResponse {
    notification: Notification,
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

#[derive(Serialize)]
struct UnreadCountResponse {
    count: i64,
}

#[derive(Serialize)]
struct MarkAllReadResponse {
    marked_count: u64,
}

#[derive(Deserialize)]
struct CreateReportDefRequest {
    name: String,
    description: String,
    report_type: ReportType,
    sql_template: String,
    parameters: Vec<String>,
}

#[derive(Serialize)]
struct CreateReportDefResponse {
    definition: ReportDefinition,
}

#[derive(Deserialize)]
struct GenerateReportRequest {
    report_id: Uuid,
    identity_id: Uuid,
    parameters: serde_json::Value,
    format: ExportFormat,
}

#[derive(Serialize)]
struct GenerateReportResponse {
    report: GeneratedReport,
}

// Notification endpoint handlers

async fn create_notification(
    State(service): State<Arc<RwLock<EventService>>>,
    Json(req): Json<CreateNotificationRequest>,
) -> Result<Json<CreateNotificationResponse>, EnterpriseError> {
    let service = service.read().await;
    let notification = service
        .create_notification(
            req.identity_id,
            req.notification_type,
            req.title,
            req.message,
            req.metadata,
        )
        .await?;

    Ok(Json(CreateNotificationResponse { notification }))
}

async fn get_notification(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<Notification>, EnterpriseError> {
    let service = service.read().await;
    let notification = service.get_notification(notification_id).await?;

    Ok(Json(notification))
}

async fn mark_as_read(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.mark_as_read(notification_id).await?;

    Ok(Json(()))
}

async fn delete_notification(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(notification_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.delete_notification(notification_id).await?;

    Ok(Json(()))
}

async fn get_identity_notifications(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(identity_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Notification>>, EnterpriseError> {
    let service = service.read().await;
    let notifications = service
        .get_identity_notifications(identity_id, query.limit, query.offset)
        .await?;

    Ok(Json(notifications))
}

async fn get_unread_count(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(identity_id): Path<Uuid>,
) -> Result<Json<UnreadCountResponse>, EnterpriseError> {
    let service = service.read().await;
    let count = service.get_unread_count(identity_id).await?;

    Ok(Json(UnreadCountResponse { count }))
}

async fn mark_all_as_read(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(identity_id): Path<Uuid>,
) -> Result<Json<MarkAllReadResponse>, EnterpriseError> {
    let service = service.read().await;
    let marked_count = service.mark_all_as_read(identity_id).await?;

    Ok(Json(MarkAllReadResponse { marked_count }))
}

// Report endpoint handlers

async fn create_report_definition(
    State(service): State<Arc<RwLock<EventService>>>,
    Json(req): Json<CreateReportDefRequest>,
) -> Result<Json<CreateReportDefResponse>, EnterpriseError> {
    let service = service.read().await;
    let definition = service
        .create_report_definition(
            req.name,
            req.description,
            req.report_type,
            req.sql_template,
            req.parameters,
        )
        .await?;

    Ok(Json(CreateReportDefResponse { definition }))
}

async fn get_report_definition(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<ReportDefinition>, EnterpriseError> {
    let service = service.read().await;
    let definition = service.get_report_definition(report_id).await?;

    Ok(Json(definition))
}

async fn list_report_definitions(
    State(service): State<Arc<RwLock<EventService>>>,
) -> Result<Json<Vec<ReportDefinition>>, EnterpriseError> {
    let service = service.read().await;
    let definitions = service.list_report_definitions().await?;

    Ok(Json(definitions))
}

async fn delete_report_definition(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.delete_report_definition(report_id).await?;

    Ok(Json(()))
}

async fn generate_report(
    State(service): State<Arc<RwLock<EventService>>>,
    Json(req): Json<GenerateReportRequest>,
) -> Result<Json<GenerateReportResponse>, EnterpriseError> {
    let service = service.read().await;
    let report = service
        .generate_report(
            req.report_id,
            req.identity_id,
            req.parameters,
            req.format,
        )
        .await?;

    Ok(Json(GenerateReportResponse { report }))
}

async fn get_generated_report(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(generated_report_id): Path<Uuid>,
) -> Result<Json<GeneratedReport>, EnterpriseError> {
    let service = service.read().await;
    let report = service.get_generated_report(generated_report_id).await?;

    Ok(Json(report))
}

async fn list_generated_reports(
    State(service): State<Arc<RwLock<EventService>>>,
    Path(identity_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<GeneratedReport>>, EnterpriseError> {
    let service = service.read().await;
    let reports = service
        .list_generated_reports(identity_id, query.limit, query.offset)
        .await?;

    Ok(Json(reports))
}
