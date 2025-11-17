// Audit Logging Module
// Provides comprehensive audit trail for security-critical events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{EnterpriseError, Result};

/// Types of auditable events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Authentication events (login, logout, etc.)
    Authentication,

    /// Authorization events (permission checks, role changes)
    Authorization,

    /// Data access (read operations on sensitive data)
    DataAccess,

    /// Data modification (create, update, delete)
    DataModification,

    /// Administrative actions
    AdminAction,

    /// Cryptographic operations (signing, encryption)
    CryptographicOperation,

    /// Security-relevant events (failed auth, violations)
    SecurityEvent,

    /// System configuration changes
    ConfigurationChange,

    /// Financial transactions
    FinancialTransaction,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::Authentication => "authentication",
            AuditEventType::Authorization => "authorization",
            AuditEventType::DataAccess => "data_access",
            AuditEventType::DataModification => "data_modification",
            AuditEventType::AdminAction => "admin_action",
            AuditEventType::CryptographicOperation => "cryptographic_operation",
            AuditEventType::SecurityEvent => "security_event",
            AuditEventType::ConfigurationChange => "configuration_change",
            AuditEventType::FinancialTransaction => "financial_transaction",
        }
    }
}

/// Result of an audited event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventResult {
    /// Operation completed successfully
    Success,

    /// Operation failed
    Failure,

    /// Operation encountered an error
    Error,
}

impl EventResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventResult::Success => "success",
            EventResult::Failure => "failure",
            EventResult::Error => "error",
        }
    }
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique identifier for this audit event
    pub event_id: Uuid,

    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,

    /// Type of event
    pub event_type: AuditEventType,

    /// User/identity that performed the action (if applicable)
    pub user_id: Option<Uuid>,

    /// IP address of the requester (if applicable)
    pub ip_address: Option<String>,

    /// User agent string (if applicable)
    pub user_agent: Option<String>,

    /// Action that was performed
    pub action: String,

    /// Resource that was affected
    pub resource: String,

    /// Result of the operation
    pub result: EventResult,

    /// Additional metadata (JSON)
    pub metadata: JsonValue,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: AuditEventType,
        action: impl Into<String>,
        resource: impl Into<String>,
        result: EventResult,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            user_id: None,
            ip_address: None,
            user_agent: None,
            action: action.into(),
            resource: resource.into(),
            result,
            metadata: serde_json::json!({}),
        }
    }

    /// Set the user ID
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the IP address
    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set the user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit logger that persists events to database
pub struct AuditLogger {
    db: PgPool,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log
            (event_id, timestamp, event_type, user_id, ip_address, user_agent, action, resource, result, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            event.event_id,
            event.timestamp,
            event.event_type.as_str(),
            event.user_id,
            event.ip_address,
            event.user_agent,
            event.action,
            event.resource,
            event.result.as_str(),
            event.metadata
        )
        .execute(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        Ok(())
    }

    /// Log a successful authentication
    pub async fn log_auth_success(&self, user_id: Uuid, ip: &str) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "login",
            format!("identity:{}", user_id),
            EventResult::Success,
        )
        .with_user(user_id)
        .with_ip(ip);

        self.log(event).await
    }

    /// Log a failed authentication
    pub async fn log_auth_failure(&self, email: &str, ip: &str, reason: &str) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::SecurityEvent,
            "login_failed",
            format!("email:{}", email),
            EventResult::Failure,
        )
        .with_ip(ip)
        .with_metadata(serde_json::json!({
            "reason": reason
        }));

        self.log(event).await
    }

    /// Log a logout event
    pub async fn log_logout(&self, user_id: Uuid, ip: &str) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "logout",
            format!("identity:{}", user_id),
            EventResult::Success,
        )
        .with_user(user_id)
        .with_ip(ip);

        self.log(event).await
    }

    /// Log data access
    pub async fn log_data_access(
        &self,
        user_id: Uuid,
        resource: &str,
        operation: &str,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::DataAccess,
            operation,
            resource,
            EventResult::Success,
        )
        .with_user(user_id);

        self.log(event).await
    }

    /// Log data modification
    pub async fn log_data_modification(
        &self,
        user_id: Uuid,
        resource: &str,
        operation: &str,
        metadata: JsonValue,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::DataModification,
            operation,
            resource,
            EventResult::Success,
        )
        .with_user(user_id)
        .with_metadata(metadata);

        self.log(event).await
    }

    /// Log an administrative action
    pub async fn log_admin_action(
        &self,
        user_id: Uuid,
        action: &str,
        resource: &str,
        metadata: JsonValue,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::AdminAction,
            action,
            resource,
            EventResult::Success,
        )
        .with_user(user_id)
        .with_metadata(metadata);

        self.log(event).await
    }

    /// Log a financial transaction
    pub async fn log_financial_transaction(
        &self,
        user_id: Uuid,
        transaction_id: &str,
        amount: u64,
        metadata: JsonValue,
    ) -> Result<()> {
        let event = AuditEvent::new(
            AuditEventType::FinancialTransaction,
            "transaction",
            format!("tx:{}", transaction_id),
            EventResult::Success,
        )
        .with_user(user_id)
        .with_metadata(serde_json::json!({
            "amount": amount,
            "details": metadata
        }));

        self.log(event).await
    }

    /// Log a security event (suspicious activity, violations, etc.)
    pub async fn log_security_event(
        &self,
        event_name: &str,
        resource: &str,
        ip: Option<&str>,
        metadata: JsonValue,
    ) -> Result<()> {
        let mut event = AuditEvent::new(
            AuditEventType::SecurityEvent,
            event_name,
            resource,
            EventResult::Failure,
        )
        .with_metadata(metadata);

        if let Some(ip_addr) = ip {
            event = event.with_ip(ip_addr);
        }

        self.log(event).await
    }

    /// Query audit logs by user
    pub async fn get_user_audit_log(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditEvent>> {
        let rows = sqlx::query!(
            r#"
            SELECT event_id, timestamp, event_type, user_id, ip_address, user_agent,
                   action, resource, result, metadata
            FROM audit_log
            WHERE user_id = $1
            ORDER BY timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let events = rows
            .into_iter()
            .map(|row| {
                let event_type = match row.event_type.as_str() {
                    "authentication" => AuditEventType::Authentication,
                    "authorization" => AuditEventType::Authorization,
                    "data_access" => AuditEventType::DataAccess,
                    "data_modification" => AuditEventType::DataModification,
                    "admin_action" => AuditEventType::AdminAction,
                    "cryptographic_operation" => AuditEventType::CryptographicOperation,
                    "security_event" => AuditEventType::SecurityEvent,
                    "configuration_change" => AuditEventType::ConfigurationChange,
                    "financial_transaction" => AuditEventType::FinancialTransaction,
                    _ => AuditEventType::SecurityEvent,
                };

                let result = match row.result.as_str() {
                    "success" => EventResult::Success,
                    "failure" => EventResult::Failure,
                    "error" => EventResult::Error,
                    _ => EventResult::Error,
                };

                AuditEvent {
                    event_id: row.event_id,
                    timestamp: row.timestamp,
                    event_type,
                    user_id: row.user_id,
                    ip_address: row.ip_address,
                    user_agent: row.user_agent,
                    action: row.action,
                    resource: row.resource,
                    result,
                    metadata: row.metadata.unwrap_or(serde_json::json!({})),
                }
            })
            .collect();

        Ok(events)
    }

    /// Query security events within a time range
    pub async fn get_security_events(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<AuditEvent>> {
        let rows = sqlx::query!(
            r#"
            SELECT event_id, timestamp, event_type, user_id, ip_address, user_agent,
                   action, resource, result, metadata
            FROM audit_log
            WHERE event_type = 'security_event'
              AND timestamp >= $1
              AND timestamp <= $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
            start_time,
            end_time,
            limit
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| EnterpriseError::from_db_error(e))?;

        let events = rows
            .into_iter()
            .map(|row| {
                AuditEvent {
                    event_id: row.event_id,
                    timestamp: row.timestamp,
                    event_type: AuditEventType::SecurityEvent,
                    user_id: row.user_id,
                    ip_address: row.ip_address,
                    user_agent: row.user_agent,
                    action: row.action,
                    resource: row.resource,
                    result: match row.result.as_str() {
                        "success" => EventResult::Success,
                        "failure" => EventResult::Failure,
                        _ => EventResult::Error,
                    },
                    metadata: row.metadata.unwrap_or(serde_json::json!({})),
                }
            })
            .collect();

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::Authentication,
            "login",
            "user:123",
            EventResult::Success,
        );

        assert_eq!(event.action, "login");
        assert_eq!(event.resource, "user:123");
        assert!(matches!(event.result, EventResult::Success));
    }

    #[test]
    fn test_audit_event_with_context() {
        let user_id = Uuid::new_v4();
        let event = AuditEvent::new(
            AuditEventType::DataAccess,
            "read",
            "wallet:456",
            EventResult::Success,
        )
        .with_user(user_id)
        .with_ip("192.168.1.1")
        .with_metadata(serde_json::json!({
            "amount": 1000
        }));

        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(event.metadata["amount"], 1000);
    }
}
