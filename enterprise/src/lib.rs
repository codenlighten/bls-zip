// Boundless Enterprise Multipass
//
// A unified access and control layer for enterprise blockchain applications
// Built on top of the Boundless BLS blockchain with post-quantum cryptography
//
// Architecture:
// - Identity & Attestation: KYC/AML and verified identity management
// - Wallet Service: Multi-asset wallet with Boundless addresses
// - Auth/SSO: Single sign-on and session management
// - Application Registry: Pluggable business application modules
// - Asset & Market: Token management and internal trading
// - Events & Reporting: Analytics, notifications, and reports
// - Hardware Pass: NFC card and secure element integration

pub mod models;
pub mod error;
pub mod db;
pub mod services;
pub mod api;
pub mod blockchain;
pub mod validation;
pub mod audit;
pub mod rate_limit;
pub mod middleware;
pub mod crypto;  // Real PQC cryptography (Dilithium + Kyber)
pub mod keystore;  // Encrypted private key storage
pub mod transaction;  // Transaction building and signing

// Re-export commonly used items
pub use error::{EnterpriseError, Result};
pub use db::{Database, DatabaseConfig};
pub use blockchain::BlockchainClient;

// Re-export security utilities
pub use validation::{
    validate_email, validate_name, validate_password, validate_phone,
    validate_organization_name, validate_url, validate_amount,
};
pub use rate_limit::{RateLimiter, RateLimitConfig};
pub use middleware::{rate_limit_middleware, rate_limit_ip_only, set_user_id, UserId};
pub use audit::{AuditLogger, AuditEvent, AuditEventType, EventResult};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Enterprise Multipass system
///
/// This is the main entry point that coordinates all services
pub struct EnterpriseMultipass {
    pub database: Database,
    pub identity_service: Arc<RwLock<services::IdentityService>>,
    pub wallet_service: Arc<RwLock<services::WalletService>>,
    pub auth_service: Arc<RwLock<services::AuthService>>,
    pub application_service: Arc<RwLock<services::ApplicationService>>,
    pub asset_service: Arc<RwLock<services::AssetService>>,
    pub event_service: Arc<RwLock<services::EventService>>,
    pub hardware_service: Arc<RwLock<services::HardwareService>>,

    // Security services
    pub rate_limiter: Arc<RateLimiter>,
    pub audit_logger: Arc<AuditLogger>,
}

impl EnterpriseMultipass {
    /// Create a new Enterprise Multipass system
    pub async fn new(db_config: DatabaseConfig) -> Result<Self> {
        // Extract blockchain RPC URL before moving db_config
        let blockchain_rpc_url = db_config.blockchain_rpc_url.clone();

        let database = Database::new(db_config).await?;
        let pool = database.pool().clone();

        // Run migrations
        database.migrate().await?;

        // Initialize business services
        let identity_service = Arc::new(RwLock::new(services::IdentityService::new(
            pool.clone(),
            blockchain_rpc_url.clone(),
        )));
        let wallet_service = Arc::new(RwLock::new(services::WalletService::new(pool.clone())));
        let auth_service = Arc::new(RwLock::new(services::AuthService::new(pool.clone())));
        let application_service = Arc::new(RwLock::new(services::ApplicationService::new(pool.clone())));
        let asset_service = Arc::new(RwLock::new(services::AssetService::new(
            pool.clone(),
            blockchain_rpc_url,
        )));
        let event_service = Arc::new(RwLock::new(services::EventService::new(pool.clone())));
        let hardware_service = Arc::new(RwLock::new(services::HardwareService::new(pool.clone())));

        // Initialize security services
        let rate_limiter = Arc::new(RateLimiter::new());
        let audit_logger = Arc::new(AuditLogger::new(pool.clone()));

        // Start rate limiter cleanup task
        rate_limit::start_cleanup_task(rate_limiter.clone());

        Ok(Self {
            database,
            identity_service,
            wallet_service,
            auth_service,
            application_service,
            asset_service,
            event_service,
            hardware_service,
            rate_limiter,
            audit_logger,
        })
    }

    /// Start the REST API server
    /// FIX M-11: Pass rate limiter to API server for login rate limiting
    pub async fn start_api_server(&self, bind_addr: &str) -> Result<()> {
        api::serve(
            bind_addr,
            self.identity_service.clone(),
            self.wallet_service.clone(),
            self.auth_service.clone(),
            self.application_service.clone(),
            self.asset_service.clone(),
            self.event_service.clone(),
            self.hardware_service.clone(),
            self.rate_limiter.clone(),
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enterprise_multipass_creation() {
        // TODO: Test with in-memory database
    }
}
