# Enterprise Multipass - Implementation Guide

This document provides detailed implementation guidance for completing the Enterprise Multipass system.

## Current Status

### ✅ Completed Components

1. **Data Models** (`src/models.rs`) - All 7 services fully modeled
2. **Database Schema** (`src/db.rs`) - Complete PostgreSQL schema with migrations
3. **Error Handling** (`src/error.rs`) - Comprehensive error types with Axum integration
4. **Identity Service** (`src/services/identity.rs`) - Fully implemented reference implementation
5. **Main Library** (`src/lib.rs`) - System entry point and service coordination
6. **Documentation** - Architecture and API documentation

### 🚧 In Progress / TODO

1. **Wallet Service** - Needs implementation following Identity pattern
2. **Auth/SSO Service** - Needs implementation with JWT and session management
3. **Application Service** - Stub implementation needed
4. **Asset Service** - Stub implementation needed
5. **Event Service** - Stub implementation needed
6. **Hardware Service** - Stub implementation needed
7. **API Layer** (`src/api/`) - REST endpoints for all services
8. **Integration Tests** - End-to-end testing

## Implementation Pattern

All services follow the same pattern established by the Identity Service. Here's the template:

### Service Structure

```rust
// Service struct with database connection
pub struct XxxService {
    db: PgPool,
}

impl XxxService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    // CRUD operations following this pattern:

    /// Create a new resource
    pub async fn create_xxx(&self, params) -> Result<Model> {
        // 1. Validate inputs
        // 2. Generate IDs/keys if needed
        // 3. Insert into database
        // 4. Optionally anchor to Boundless chain
        // 5. Return model
    }

    /// Get resource by ID
    pub async fn get_xxx(&self, id: Uuid) -> Result<Model> {
        // 1. Query database
        // 2. Map to model
        // 3. Return or error if not found
    }

    /// Update resource
    pub async fn update_xxx(&self, id: Uuid, updates) -> Result<()> {
        // 1. Validate updates
        // 2. Update database
        // 3. Optionally emit events
    }

    /// Delete/revoke resource
    pub async fn delete_xxx(&self, id: Uuid) -> Result<()> {
        // 1. Mark as deleted/revoked
        // 2. Optionally anchor revocation to chain
    }

    /// List resources (with pagination)
    pub async fn list_xxx(&self, limit: i64, offset: i64) -> Result<Vec<Model>> {
        // Query with LIMIT/OFFSET
    }

    // Chain integration helpers
    async fn anchor_to_chain(&self, data) -> Result<String> {
        // 1. Create proof data
        // 2. Hash with SHA3-256
        // 3. Submit to Boundless chain
        // 4. Return TX hash
    }
}
```

### API Endpoint Pattern

```rust
// In src/api/xxx.rs

use axum::{
    extract::{Path, State},
    Json,
    routing::{get, post, put, delete},
    Router,
};

pub fn routes(service: Arc<RwLock<XxxService>>) -> Router {
    Router::new()
        .route("/xxx", post(create))
        .route("/xxx/:id", get(get_by_id))
        .route("/xxx/:id", put(update))
        .route("/xxx/:id", delete(delete))
        .route("/xxx", get(list))
        .with_state(service)
}

async fn create(
    State(service): State<Arc<RwLock<XxxService>>>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<Response>, EnterpriseError> {
    let service = service.read().await;
    let result = service.create_xxx(req.into()).await?;
    Ok(Json(result.into()))
}

// ... similar for other endpoints
```

## Service-by-Service Implementation Guide

### 1. Wallet Service

**File**: `src/services/wallet.rs`

**Key Functions to Implement:**

```rust
// Create wallet with Boundless PQC addresses
pub async fn create_wallet(
    &self,
    identity_id: Uuid,
    labels: Vec<String>,
) -> Result<WalletAccount>

// Get wallet balances for all assets
pub async fn get_balances(&self, wallet_id: Uuid) -> Result<Vec<WalletBalance>>

// Transfer assets (creates Boundless transaction)
pub async fn transfer(
    &self,
    wallet_id: Uuid,
    to_address: String,
    asset_type: AssetType,
    amount: u64,
) -> Result<WalletTransaction>

// Get transaction history
pub async fn get_transactions(
    &self,
    wallet_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<WalletTransaction>>

// Get UTXOs from Boundless chain
pub async fn get_utxos(&self, wallet_id: Uuid) -> Result<Vec<UTXO>>
```

**Chain Integration:**
- Use `boundless_core::Transaction` to create transfers
- Query Boundless RPC for balance/UTXO data
- Store transaction records in `wallet_transactions` table

**Database Tables:**
- `wallet_accounts`
- `wallet_transactions`
- `wallet_balances`

### 2. Auth/SSO Service

**File**: `src/services/auth.rs`

**Key Functions to Implement:**

```rust
// Register new credentials for an identity
pub async fn register(
    &self,
    identity_id: Uuid,
    password: String,
) -> Result<MultipassCredential>

// Login with email/password
pub async fn login(
    &self,
    email: String,
    password: String,
) -> Result<(MultipassSession, String)>  // Returns session + JWT token

// Verify JWT token
pub async fn verify_token(&self, token: &str) -> Result<Uuid>  // Returns identity_id

// Refresh session
pub async fn refresh_session(&self, session_id: Uuid) -> Result<String>

// Logout (revoke session)
pub async fn logout(&self, session_id: Uuid) -> Result<()>

// Check permissions
pub async fn has_scope(&self, session_id: Uuid, scope: &str) -> Result<bool>
```

**Implementation Notes:**
- Use `argon2` for password hashing
- Use `jsonwebtoken` crate for JWT creation/verification
- Sign JWT with Boundless PQC key from identity profile
- Session expiry: 24 hours default
- Implement refresh token mechanism

**Database Tables:**
- `multipass_credentials`
- `multipass_sessions`

### 3. Application Service

**File**: `src/services/application.rs`

**Key Functions to Implement:**

```rust
// Register new application module
pub async fn register_application(
    &self,
    name: String,
    description: String,
    category: AppCategory,
    api_base_url: String,
    required_scopes: Vec<String>,
    on_chain_contract_ref: Option<String>,
) -> Result<ApplicationModule>

// Log application event
pub async fn log_event(
    &self,
    app_id: Uuid,
    identity_id: Uuid,
    event_type: String,
    metadata: serde_json::Value,
) -> Result<ApplicationEvent>

// Get events for application
pub async fn get_events(
    &self,
    app_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<ApplicationEvent>>

// Enable/disable application
pub async fn set_enabled(&self, app_id: Uuid, enabled: bool) -> Result<()>
```

**Database Tables:**
- `application_modules`
- `application_events`

### 4. Asset Service

**File**: `src/services/asset.rs`

**Key Functions to Implement:**

```rust
// Define new asset
pub async fn define_asset(
    &self,
    issuer_identity_id: Uuid,
    asset_type: AssetType,
    symbol: String,
    name: String,
    chain_contract_ref: String,
    metadata: serde_json::Value,
) -> Result<AssetDefinition>

// Get asset positions for wallet
pub async fn get_positions(&self, wallet_id: Uuid) -> Result<Vec<AssetPosition>>

// Create market order
pub async fn create_order(
    &self,
    wallet_id: Uuid,
    asset_id: Uuid,
    order_type: OrderType,
    quantity: u64,
    price: u64,
) -> Result<MarketOrder>

// Match and execute orders
pub async fn match_orders(&self, asset_id: Uuid) -> Result<Vec<MarketOrder>>

// Get order book
pub async fn get_orderbook(&self, asset_id: Uuid) -> Result<OrderBook>
```

**Chain Integration:**
- Deploy/interact with Boundless smart contracts for token standards
- Settle trades with on-chain transactions
- Track asset ownership via blockchain state

**Database Tables:**
- `asset_definitions`
- `asset_positions`
- `market_orders`

### 5. Event Service

**File**: `src/services/events.rs`

**Key Functions to Implement:**

```rust
// Send notification to user
pub async fn notify(
    &self,
    identity_id: Uuid,
    notification_type: NotificationType,
    source: String,
    title: String,
    message: String,
) -> Result<Notification>

// Get notifications for user
pub async fn get_notifications(
    &self,
    identity_id: Uuid,
    unread_only: bool,
) -> Result<Vec<Notification>>

// Mark notification as read
pub async fn mark_read(&self, notification_id: Uuid) -> Result<()>

// Generate report
pub async fn generate_report(
    &self,
    report_def_id: Uuid,
    identity_id: Uuid,
    parameters: serde_json::Value,
) -> Result<ReportInstance>

// Get reports for user
pub async fn get_reports(
    &self,
    identity_id: Uuid,
) -> Result<Vec<ReportInstance>>
```

**Database Tables:**
- `notifications`
- `report_definitions`
- `report_instances`

### 6. Hardware Service

**File**: `src/services/hardware.rs`

**Key Functions to Implement:**

```rust
// Register new hardware device
pub async fn register_device(
    &self,
    identity_id: Uuid,
    public_key: String,
    capabilities: Vec<HardwareCapability>,
) -> Result<HardwarePass>

// Authenticate with hardware device
pub async fn authenticate(
    &self,
    device_id: Uuid,
    challenge: Vec<u8>,
    signature: Vec<u8>,
) -> Result<bool>

// Revoke device
pub async fn revoke(&self, device_id: Uuid) -> Result<()>

// Update last used timestamp
pub async fn update_last_used(&self, device_id: Uuid) -> Result<()>
```

**Implementation Notes:**
- Integrate with WebAuthn for browser-based hardware keys
- Support NFC card protocols (ISO 14443)
- Use Boundless PQC for device key generation

**Database Tables:**
- `hardware_passes`

## API Layer Implementation

Create `src/api/` module with the following structure:

```
src/api/
├── mod.rs          # Main API router
├── identity.rs     # Identity endpoints
├── wallet.rs       # Wallet endpoints
├── auth.rs         # Auth endpoints
├── application.rs  # Application endpoints
├── asset.rs        # Asset endpoints
├── events.rs       # Events endpoints
└── hardware.rs     # Hardware endpoints
```

### Main API Router (src/api/mod.rs)

```rust
use axum::{Router, middleware};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::RwLock;

mod identity;
mod wallet;
mod auth;
mod application;
mod asset;
mod events;
mod hardware;

pub async fn serve(
    bind_addr: &str,
    identity_service: Arc<RwLock<IdentityService>>,
    wallet_service: Arc<RwLock<WalletService>>,
    auth_service: Arc<RwLock<AuthService>>,
    application_service: Arc<RwLock<ApplicationService>>,
    asset_service: Arc<RwLock<AssetService>>,
    event_service: Arc<RwLock<EventService>>,
    hardware_service: Arc<RwLock<HardwareService>>,
) -> Result<()> {
    let app = Router::new()
        .nest("/api/identity", identity::routes(identity_service))
        .nest("/api/wallet", wallet::routes(wallet_service))
        .nest("/api/auth", auth::routes(auth_service))
        .nest("/api/applications", application::routes(application_service))
        .nest("/api/assets", asset::routes(asset_service))
        .nest("/api/market", asset::market_routes(asset_service))
        .nest("/api/notifications", events::notification_routes(event_service))
        .nest("/api/reports", events::report_routes(event_service))
        .nest("/api/hardware", hardware::routes(hardware_service))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(auth_middleware));

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    tracing::info!("Enterprise Multipass API listening on {}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// Auth middleware to verify JWT tokens
async fn auth_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, EnterpriseError> {
    // Extract Authorization header
    // Verify JWT token
    // Add identity_id to request extensions
    // Call next
    todo!("Implement auth middleware")
}
```

## Testing Strategy

### Unit Tests

Each service should have unit tests with mock database:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_create_identity() {
        // Setup test database
        // Create service
        // Call create_identity
        // Assert results
    }
}
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
#[tokio::test]
async fn test_full_user_flow() {
    // 1. Create identity
    // 2. Register credentials
    // 3. Login and get token
    // 4. Create wallet
    // 5. Transfer assets
    // 6. Generate report
}
```

## Deployment Checklist

- [ ] Set up PostgreSQL database
- [ ] Run database migrations
- [ ] Configure environment variables
- [ ] Set up Boundless blockchain node connection
- [ ] Configure CORS for frontend
- [ ] Set up SSL/TLS certificates
- [ ] Configure rate limiting
- [ ] Set up monitoring and logging
- [ ] Create backup procedures
- [ ] Security audit

## Next Steps

1. **Implement Wallet Service** - Follow Identity service pattern
2. **Implement Auth Service** - Add JWT and session management
3. **Create API Layer** - REST endpoints for all services
4. **Write Tests** - Unit and integration tests
5. **Add Monitoring** - Metrics and health checks
6. **Deploy** - Production deployment guide

## Resources

- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Boundless Core API](../core/README.md)
- [JWT Best Practices](https://datatracker.ietf.org/doc/html/rfc8725)
