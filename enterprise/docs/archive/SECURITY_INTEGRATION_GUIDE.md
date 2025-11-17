# Security Integration Guide

Complete guide for integrating all security features into your Boundless Enterprise API.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Rate Limiting](#rate-limiting)
3. [Input Validation](#input-validation)
4. [Audit Logging](#audit-logging)
5. [Complete Example](#complete-example)
6. [Testing](#testing)

---

## Quick Start

### 1. Add Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
boundless-enterprise = { path = "../enterprise" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

### 2. Set Environment Variables

```bash
# CRITICAL: JWT secret (required)
export JWT_SECRET=$(openssl rand -hex 32)

# Database connection
export DATABASE_URL="postgres://user:pass@localhost/boundless"
```

### 3. Run Migrations

```bash
cd enterprise
sqlx migrate run
```

---

## Rate Limiting

### Basic Setup

```rust
use boundless_enterprise::rate_limit::{RateLimiter, RateLimitConfig};
use std::sync::Arc;

// Create with default config (100 req/min per IP, 200 req/min per user)
let limiter = Arc::new(RateLimiter::new());

// Or with custom config
let config = RateLimitConfig {
    max_requests_per_ip: 50,
    max_requests_per_user: 100,
    window_secs: 60,
    enable_burst_protection: true,
    burst_size: 5,
};
let limiter = Arc::new(RateLimiter::with_config(config));
```

### Axum Integration

#### Option 1: IP-Only Rate Limiting (Public Endpoints)

```rust
use axum::{Router, routing::post};
use boundless_enterprise::middleware::rate_limit_ip_only;

let app = Router::new()
    .route("/api/login", post(login_handler))
    .layer(axum::middleware::from_fn_with_state(
        limiter.clone(),
        rate_limit_ip_only
    ));
```

#### Option 2: Combined IP + User Rate Limiting (Protected Endpoints)

```rust
use axum::{Router, routing::get};
use boundless_enterprise::middleware::rate_limit_middleware;

let app = Router::new()
    .route("/api/profile", get(profile_handler))
    .layer(axum::middleware::from_fn_with_state(
        limiter.clone(),
        rate_limit_middleware
    ));
```

### Manual Rate Limit Checks

```rust
use std::net::IpAddr;
use uuid::Uuid;

// Check IP only
limiter.check_ip(ip_addr).await?;

// Check user only
limiter.check_user(user_id).await?;

// Check both
limiter.check_combined(ip_addr, Some(user_id)).await?;
```

### Automatic Cleanup

Start a background task to clean up expired entries:

```rust
use boundless_enterprise::rate_limit::start_cleanup_task;

start_cleanup_task(limiter.clone());
```

This runs every 5 minutes automatically.

### Statistics

```rust
let stats = limiter.get_stats().await;
println!("Tracked IPs: {}", stats.tracked_ips);
println!("Tracked users: {}", stats.tracked_users);
```

---

## Input Validation

### Available Validators

```rust
use boundless_enterprise::validation;

// Email validation (RFC 5322 compliant, max 254 chars)
validation::validate_email(&email)?;

// Name validation (Unicode support, XSS prevention)
validation::validate_name(&name, "Full name")?;

// Password strength (12+ chars, complexity requirements)
validation::validate_password(&password)?;

// Phone number (international format, min 10 digits)
validation::validate_phone(&phone)?;

// Organization name
validation::validate_organization_name(&org_name)?;

// URL (http/https only, max 2048 chars)
validation::validate_url(&url)?;

// Amount (non-zero, overflow protection)
validation::validate_amount(amount)?;

// Description/text (control character prevention, max 2000 chars)
validation::validate_description(&description, "Description")?;

// UUID
let uuid = validation::validate_uuid(&uuid_str, "User ID")?;
```

### Validation Limits

```rust
use boundless_enterprise::validation::limits;

limits::MAX_EMAIL_LENGTH;        // 254
limits::MAX_NAME_LENGTH;         // 100
limits::MIN_PASSWORD_LENGTH;     // 12
limits::MAX_PASSWORD_LENGTH;     // 128
limits::MAX_DESCRIPTION_LENGTH;  // 2000
limits::MAX_URL_LENGTH;          // 2048
```

### Example: Validate User Input

```rust
#[derive(Deserialize)]
struct CreateUserRequest {
    full_name: String,
    email: String,
    phone: Option<String>,
    password: String,
}

async fn create_user(
    Json(payload): Json<CreateUserRequest>
) -> Result<Json<UserResponse>, ApiError> {
    // Validate all inputs
    validation::validate_name(&payload.full_name, "Full name")?;
    validation::validate_email(&payload.email)?;
    validation::validate_password(&payload.password)?;

    if let Some(ref phone) = payload.phone {
        validation::validate_phone(phone)?;
    }

    // Inputs are now safe to use
    // ...
}
```

### Password Requirements

- Minimum 12 characters
- Must contain uppercase letter
- Must contain lowercase letter
- Must contain digit
- Must contain special character

Example valid passwords:
- `SecurePass123!`
- `Tr0ng#P@ssw0rd`
- `MyP@ssw0rd2024`

---

## Audit Logging

**Note:** Audit logging requires the database migration to be run first.

### Setup

```rust
use boundless_enterprise::audit::AuditLogger;

let audit_logger = AuditLogger::new(db_pool.clone());
```

### Pre-built Logging Functions

```rust
// Successful login
audit_logger.log_auth_success(user_id, "192.168.1.1").await?;

// Failed login
audit_logger.log_auth_failure("user@example.com", "192.168.1.1", "invalid password").await?;

// Logout
audit_logger.log_logout(user_id, "192.168.1.1").await?;

// Data access
audit_logger.log_data_access(user_id, "wallet:123", "read").await?;

// Data modification
audit_logger.log_data_modification(
    user_id,
    "user:456",
    "update",
    serde_json::json!({ "field": "email" })
).await?;

// Admin action
audit_logger.log_admin_action(
    user_id,
    "delete_user",
    "user:789",
    serde_json::json!({ "reason": "policy violation" })
).await?;

// Financial transaction
audit_logger.log_financial_transaction(
    user_id,
    "tx_abc123",
    1000000,
    serde_json::json!({ "type": "transfer" })
).await?;

// Security event
audit_logger.log_security_event(
    "rate_limit_exceeded",
    "api:/users",
    Some("192.168.1.1"),
    serde_json::json!({ "attempts": 150 })
).await?;
```

### Custom Audit Events

```rust
use boundless_enterprise::audit::{AuditEvent, AuditEventType, EventResult};

let event = AuditEvent::new(
    AuditEventType::DataModification,
    "update_profile",
    "user:123",
    EventResult::Success,
)
.with_user(user_id)
.with_ip("192.168.1.1")
.with_user_agent("Mozilla/5.0...")
.with_metadata(serde_json::json!({
    "fields_changed": ["email", "phone"]
}));

audit_logger.log(event).await?;
```

### Querying Audit Logs

```rust
// Get user's audit history
let events = audit_logger.get_user_audit_log(user_id, 50, 0).await?;

for event in events {
    println!("{}: {} on {}",
        event.timestamp, event.action, event.resource);
}

// Get security events in time range
use chrono::{Utc, Duration};

let start = Utc::now() - Duration::hours(24);
let end = Utc::now();

let security_events = audit_logger
    .get_security_events(start, end, 100)
    .await?;
```

---

## Complete Example

### Full Integration with Authentication

```rust
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use boundless_enterprise::{
    middleware::{rate_limit_ip_only, rate_limit_middleware, set_user_id},
    rate_limit::{RateLimiter, start_cleanup_task},
    validation,
    audit::AuditLogger,
};

#[derive(Clone)]
struct AppState {
    rate_limiter: Arc<RateLimiter>,
    audit_logger: Arc<AuditLogger>,
    db_pool: sqlx::PgPool,
}

// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract token from Authorization header
    let token = extract_token(&request)?;

    // Verify token and get user ID
    let user_id = verify_jwt_token(&token)?;

    // Add user ID to request extensions
    set_user_id(&mut request, user_id);

    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    // Setup
    let db_pool = connect_to_database().await;
    let rate_limiter = Arc::new(RateLimiter::new());
    let audit_logger = Arc::new(AuditLogger::new(db_pool.clone()));

    // Start cleanup task
    start_cleanup_task(rate_limiter.clone());

    let state = AppState {
        rate_limiter: rate_limiter.clone(),
        audit_logger,
        db_pool,
    };

    // Public routes (IP rate limited)
    let public = Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_ip_only
        ))
        .with_state(state.clone());

    // Protected routes (authenticated + rate limited)
    let protected = Router::new()
        .route("/profile", get(get_profile))
        .route("/profile", post(update_profile))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware
        ))
        .with_state(state.clone());

    // Combine
    let app = Router::new()
        .nest("/api", public)
        .nest("/api", protected)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Handler Example with All Security Features

```rust
#[derive(Deserialize)]
struct UpdateProfileRequest {
    full_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

async fn update_profile(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    user_id: UserId,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // 1. Input validation
    if let Some(ref name) = payload.full_name {
        validation::validate_name(name, "Full name")?;
    }

    if let Some(ref email) = payload.email {
        validation::validate_email(email)?;
    }

    if let Some(ref phone) = payload.phone {
        validation::validate_phone(phone)?;
    }

    // 2. Business logic (update database)
    let updated_user = update_user_in_db(&state.db_pool, user_id.0, payload).await?;

    // 3. Audit logging
    state.audit_logger.log_data_modification(
        user_id.0,
        &format!("user:{}", user_id.0),
        "update_profile",
        serde_json::json!({
            "ip": addr.ip().to_string(),
            "fields": ["name", "email"]
        })
    ).await?;

    // 4. Return response
    Ok(Json(updated_user))
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        let ip = "127.0.0.1".parse().unwrap();

        // Should allow normal requests
        assert!(limiter.check_ip(ip).await.is_ok());
    }

    #[test]
    fn test_email_validation() {
        assert!(validation::validate_email("test@example.com").is_ok());
        assert!(validation::validate_email("invalid").is_err());
    }

    #[test]
    fn test_password_strength() {
        assert!(validation::validate_password("SecurePass123!").is_ok());
        assert!(validation::validate_password("weak").is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_rate_limit_middleware() {
    let limiter = Arc::new(RateLimiter::new());

    let app = Router::new()
        .route("/test", get(|| async { "OK" }))
        .layer(middleware::from_fn_with_state(
            limiter,
            rate_limit_ip_only
        ))
        .into_make_service_with_connect_info::<SocketAddr>();

    // Test with axum-test or similar
}
```

### Load Testing

```bash
# Install bombardier
go install github.com/codesenberg/bombardier@latest

# Test rate limiting
bombardier -c 10 -n 200 http://localhost:3000/api/health

# Should see 429 responses after limit is reached
```

---

## Troubleshooting

### Rate Limiting Not Working

1. Ensure `ConnectInfo<SocketAddr>` is available:
   ```rust
   let app = app.into_make_service_with_connect_info::<SocketAddr>();
   ```

2. Check middleware order (rate limit should be before auth):
   ```rust
   Router::new()
       .layer(auth_middleware)
       .layer(rate_limit_middleware)  // Applied first (bottom-up)
   ```

### User ID Not Found in Protected Routes

Make sure auth middleware sets the user ID:
```rust
use boundless_enterprise::middleware::set_user_id;

set_user_id(&mut request, user_id);
```

### Audit Logging Fails

1. Ensure migration is run:
   ```bash
   sqlx migrate run
   ```

2. Check database connection:
   ```bash
   psql $DATABASE_URL -c "SELECT * FROM audit_log LIMIT 1;"
   ```

---

## Best Practices

1. **Always validate input** before using it
2. **Log security events** (failed auth, rate limits, etc.)
3. **Use rate limiting** on all public endpoints
4. **Combine IP + user limits** for authenticated endpoints
5. **Set JWT_SECRET** before deployment
6. **Monitor audit logs** regularly
7. **Test rate limits** under load
8. **Keep cleanup task running** for rate limiter

---

## Production Checklist

- [ ] JWT_SECRET set (32+ characters)
- [ ] Database migrations run
- [ ] Rate limiter cleanup task started
- [ ] Audit logging enabled
- [ ] Input validation on all endpoints
- [ ] Rate limiting on public endpoints
- [ ] Combined rate limiting on protected endpoints
- [ ] Load testing completed
- [ ] Monitoring configured
- [ ] Logs being collected

---

For more examples, see:
- `enterprise/examples/secure_api_server.rs`
- `enterprise/src/middleware/rate_limit.rs`
- `enterprise/src/validation.rs`
- `enterprise/src/audit.rs`
