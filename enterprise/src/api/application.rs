// Application Module Registry API Endpoints

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

use crate::services::ApplicationService;
use crate::models::*;
use crate::error::EnterpriseError;

/// Create router for application endpoints
pub fn routes(service: Arc<RwLock<ApplicationService>>) -> Router {
    Router::new()
        .route("/register", post(register_application))
        .route("/list", get(list_applications))
        .route("/:app_id", get(get_application))
        .route("/:app_id/enable", put(set_enabled))
        .route("/:app_id/update", put(update_application))
        .route("/:app_id/delete", delete(delete_application))
        .route("/:app_id/events", post(log_event))
        .route("/:app_id/events/list", get(get_events))
        .route("/identity/:identity_id/events", get(get_identity_events))
        .with_state(service)
}

// Request/Response DTOs

#[derive(Deserialize)]
struct RegisterApplicationRequest {
    name: String,
    description: String,
    category: AppCategory,
    api_base_url: String,
    required_scopes: Vec<String>,
    on_chain_contract_ref: Option<String>,
}

#[derive(Serialize)]
struct RegisterApplicationResponse {
    application: ApplicationModule,
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
struct SetEnabledRequest {
    enabled: bool,
}

#[derive(Deserialize)]
struct UpdateApplicationRequest {
    name: Option<String>,
    description: Option<String>,
    api_base_url: Option<String>,
    required_scopes: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct LogEventRequest {
    identity_id: Uuid,
    event_type: String,
    metadata: serde_json::Value,
}

#[derive(Serialize)]
struct LogEventResponse {
    event: ApplicationEvent,
}

// Endpoint handlers

async fn register_application(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Json(req): Json<RegisterApplicationRequest>,
) -> Result<Json<RegisterApplicationResponse>, EnterpriseError> {
    let service = service.read().await;
    let application = service
        .register_application(
            req.name,
            req.description,
            req.category,
            req.api_base_url,
            req.required_scopes,
            req.on_chain_contract_ref,
        )
        .await?;

    Ok(Json(RegisterApplicationResponse { application }))
}

async fn list_applications(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ApplicationModule>>, EnterpriseError> {
    let service = service.read().await;
    let applications = service
        .list_applications(query.limit, query.offset)
        .await?;

    Ok(Json(applications))
}

async fn get_application(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApplicationModule>, EnterpriseError> {
    let service = service.read().await;
    let application = service.get_application(app_id).await?;

    Ok(Json(application))
}

async fn set_enabled(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<SetEnabledRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.set_enabled(app_id, req.enabled).await?;

    Ok(Json(()))
}

async fn update_application(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<UpdateApplicationRequest>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service
        .update_application(
            app_id,
            req.name,
            req.description,
            req.api_base_url,
            req.required_scopes,
        )
        .await?;

    Ok(Json(()))
}

async fn delete_application(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<()>, EnterpriseError> {
    let service = service.read().await;
    service.delete_application(app_id).await?;

    Ok(Json(()))
}

async fn log_event(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<LogEventRequest>,
) -> Result<Json<LogEventResponse>, EnterpriseError> {
    let service = service.read().await;
    let event = service
        .log_event(app_id, req.identity_id, req.event_type, req.metadata)
        .await?;

    Ok(Json(LogEventResponse { event }))
}

async fn get_events(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(app_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ApplicationEvent>>, EnterpriseError> {
    let service = service.read().await;
    let events = service
        .get_events(app_id, query.limit, query.offset)
        .await?;

    Ok(Json(events))
}

async fn get_identity_events(
    State(service): State<Arc<RwLock<ApplicationService>>>,
    Path(identity_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<ApplicationEvent>>, EnterpriseError> {
    let service = service.read().await;
    let events = service
        .get_identity_events(identity_id, query.limit, query.offset)
        .await?;

    Ok(Json(events))
}
