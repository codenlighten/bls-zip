// Enterprise Multipass Server Binary
//
// Starts the Enterprise Multipass REST API server with all services

use boundless_enterprise::{EnterpriseMultipass, DatabaseConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    tracing::info!("Starting Boundless Enterprise Multipass Server");

    // Load environment variables from .env file if present
    dotenv::dotenv().ok();

    // Validate keystore initialization (FIX C-2)
    // This ensures MASTER_ENCRYPTION_KEY is set before the server starts
    tracing::info!("Validating keystore configuration...");
    use boundless_enterprise::keystore::Keystore;
    let _ = Keystore::new()
        .expect("FATAL: MASTER_ENCRYPTION_KEY must be set in .env file. Generate with: openssl rand -hex 32");
    tracing::info!("Keystore validation successful");

    // Database configuration
    let db_config = DatabaseConfig {
        database_url: env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/boundless_enterprise".to_string()),
        max_connections: env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        blockchain_rpc_url: env::var("BLOCKCHAIN_RPC_URL")
            .unwrap_or_else(|_| "http://localhost:9933".to_string()),
    };

    tracing::info!("Database URL: {}", mask_password(&db_config.database_url));
    tracing::info!("Max database connections: {}", db_config.max_connections);

    // Initialize Enterprise Multipass system
    tracing::info!("Initializing Enterprise Multipass system...");
    let multipass = EnterpriseMultipass::new(db_config).await?;
    tracing::info!("Enterprise Multipass initialized successfully");

    // API server configuration
    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    tracing::info!("Starting API server on {}", bind_addr);

    // Start the API server
    multipass.start_api_server(&bind_addr).await?;

    Ok(())
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            if let Some(protocol_end) = url.find("://") {
                let protocol = &url[..=protocol_end + 2];
                let user_start = protocol_end + 3;
                let user = &url[user_start..colon_pos];
                let after_at = &url[at_pos..];
                return format!("{}{}:****{}", protocol, user, after_at);
            }
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        assert_eq!(
            mask_password("postgresql://user:password@localhost:5432/db"),
            "postgresql://user:****@localhost:5432/db"
        );
        assert_eq!(
            mask_password("postgresql://localhost:5432/db"),
            "postgresql://localhost:5432/db"
        );
    }
}
