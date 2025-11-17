// Database management for Enterprise Multipass
//
// Uses PostgreSQL (via sqlx) for relational data with strong consistency
// All critical proofs and transactions are anchored to Boundless chain

use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::error::{EnterpriseError, Result};

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub blockchain_rpc_url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://boundless:boundless@localhost/boundless_enterprise".to_string()),
            max_connections: 10,
            blockchain_rpc_url: std::env::var("BLOCKCHAIN_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:9933".to_string()),
        }
    }
}

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.database_url)
            .await
            .map_err(|e| EnterpriseError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

        Ok(Self { pool })
    }

    /// Get database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| EnterpriseError::DatabaseError(format!("Migration failed: {}", e)))?;

        Ok(())
    }
}

/// SQL schema for database tables
/// This should be run as migrations, but included here for reference
pub const SCHEMA: &str = r#"
-- Identity & Attestation tables

CREATE TYPE kyc_status AS ENUM ('pending', 'verified', 'rejected', 'revoked');
CREATE TYPE attestation_type AS ENUM ('kyc', 'address_proof', 'income_proof', 'asset_ownership', 'social_graph', 'professional_credential');
CREATE TYPE attestation_status AS ENUM ('valid', 'expired', 'revoked');

CREATE TABLE identity_profiles (
    identity_id UUID PRIMARY KEY,
    root_pki_key_id TEXT NOT NULL,
    legal_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    org_name TEXT,
    kyc_status kyc_status NOT NULL DEFAULT 'pending',
    aml_risk_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE identity_attestations (
    attestation_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    attestation_type attestation_type NOT NULL,
    evidence_refs TEXT[] NOT NULL,
    issuer TEXT NOT NULL,
    status attestation_status NOT NULL DEFAULT 'valid',
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    chain_anchor_tx TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Wallet tables

CREATE TYPE tx_direction AS ENUM ('in', 'out');
CREATE TYPE tx_status AS ENUM ('pending', 'confirmed', 'failed');
CREATE TYPE asset_type AS ENUM ('native', 'utility_token', 'equity_token', 'carbon_credit', 'nft', 'subscription_pass');

CREATE TABLE wallet_accounts (
    wallet_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    boundless_addresses JSONB NOT NULL,
    labels TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE wallet_transactions (
    tx_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id),
    chain_tx_hash TEXT NOT NULL,
    asset_type asset_type NOT NULL,
    amount BIGINT NOT NULL,
    direction tx_direction NOT NULL,
    counterparty TEXT,
    application_context UUID,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status tx_status NOT NULL DEFAULT 'pending'
);

CREATE TABLE wallet_balances (
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id),
    asset_type asset_type NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0,
    locked_amount BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (wallet_id, asset_type)
);

-- Auth/SSO tables

CREATE TYPE credential_status AS ENUM ('active', 'suspended', 'compromised');

CREATE TABLE multipass_credentials (
    credential_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) UNIQUE,
    password_hash TEXT NOT NULL,
    webauthn_credentials TEXT[] NOT NULL DEFAULT '{}',
    nfc_card_id TEXT,
    pki_key_ids TEXT[] NOT NULL,
    status credential_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE multipass_sessions (
    session_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    device_fingerprint TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    scopes TEXT[] NOT NULL,
    token TEXT NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

-- Application Module tables

CREATE TYPE app_category AS ENUM ('security', 'invoicing', 'ticketing', 'healthcare', 'supply_chain', 'compliance', 'finance', 'marketplace');

CREATE TABLE application_modules (
    app_id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    category app_category NOT NULL,
    api_base_url TEXT NOT NULL,
    required_scopes TEXT[] NOT NULL,
    on_chain_contract_ref TEXT,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE application_events (
    event_id UUID PRIMARY KEY,
    app_id UUID NOT NULL REFERENCES application_modules(app_id),
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    event_type TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL
);

-- Asset & Market tables

CREATE TYPE order_type AS ENUM ('buy', 'sell');
CREATE TYPE order_status AS ENUM ('open', 'partially_filled', 'filled', 'cancelled');

CREATE TABLE asset_definitions (
    asset_id UUID PRIMARY KEY,
    asset_type asset_type NOT NULL,
    issuer_identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    chain_contract_ref TEXT NOT NULL,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    metadata JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE asset_positions (
    position_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id),
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id),
    quantity BIGINT NOT NULL DEFAULT 0,
    locked_quantity BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (wallet_id, asset_id)
);

CREATE TABLE market_orders (
    order_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id),
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id),
    order_type order_type NOT NULL,
    quantity BIGINT NOT NULL,
    price BIGINT NOT NULL,
    status order_status NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    filled_at TIMESTAMPTZ,
    settlement_tx TEXT
);

-- Event & Reporting tables

CREATE TYPE notification_type AS ENUM ('security_alert', 'payment_received', 'payment_sent', 'permission_change', 'report_ready', 'system_update', 'application_event');
CREATE TYPE report_type AS ENUM ('financial', 'asset', 'application', 'security', 'compliance');

CREATE TABLE notifications (
    notification_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    notification_type notification_type NOT NULL,
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE report_definitions (
    report_def_id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    report_type report_type NOT NULL,
    parameters_schema JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE report_instances (
    report_id UUID PRIMARY KEY,
    report_def_id UUID NOT NULL REFERENCES report_definitions(report_def_id),
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    parameters JSONB NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    data JSONB NOT NULL
);

-- HardwarePass tables

CREATE TYPE hardware_status AS ENUM ('active', 'lost', 'revoked');

CREATE TABLE hardware_passes (
    device_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id),
    public_key TEXT NOT NULL UNIQUE,
    capabilities TEXT[] NOT NULL,
    status hardware_status NOT NULL DEFAULT 'active',
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ
);

-- Indexes for performance

CREATE INDEX idx_identity_email ON identity_profiles(email);
CREATE INDEX idx_identity_kyc_status ON identity_profiles(kyc_status);

CREATE INDEX idx_attestation_identity ON identity_attestations(identity_id);
CREATE INDEX idx_attestation_status ON identity_attestations(status);

CREATE INDEX idx_wallet_identity ON wallet_accounts(identity_id);
CREATE INDEX idx_wallet_tx_wallet ON wallet_transactions(wallet_id);
CREATE INDEX idx_wallet_tx_status ON wallet_transactions(status);

CREATE INDEX idx_session_identity ON multipass_sessions(identity_id);
CREATE INDEX idx_session_expires ON multipass_sessions(expires_at);

CREATE INDEX idx_app_event_app ON application_events(app_id);
CREATE INDEX idx_app_event_identity ON application_events(identity_id);

CREATE INDEX idx_asset_pos_wallet ON asset_positions(wallet_id);
CREATE INDEX idx_market_order_status ON market_orders(status);

CREATE INDEX idx_notification_identity ON notifications(identity_id);
CREATE INDEX idx_notification_read ON notifications(read);
"#;
