-- Enterprise Multipass Database Schema Migration
-- This migration creates all tables for the Enterprise Multipass system
-- Tables are ordered by dependencies (parent tables first)

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =============================================================================
-- 1. IDENTITY & ATTESTATION SERVICE TABLES
-- =============================================================================

-- Identity Profiles - Core identity information
CREATE TABLE IF NOT EXISTS identity_profiles (
    identity_id UUID PRIMARY KEY,
    full_name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(50),
    country_code VARCHAR(3),
    date_of_birth DATE,
    verification_status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, verified, rejected
    kyc_level INTEGER NOT NULL DEFAULT 0, -- 0, 1, 2, 3
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_identity_email ON identity_profiles(email);
CREATE INDEX idx_identity_verification_status ON identity_profiles(verification_status);
CREATE INDEX idx_identity_kyc_level ON identity_profiles(kyc_level);

-- KYC Verifications - Know Your Customer verification records
CREATE TABLE IF NOT EXISTS kyc_verifications (
    verification_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    verification_type VARCHAR(50) NOT NULL, -- government_id, proof_of_address, selfie, biometric
    document_hash VARCHAR(255) NOT NULL, -- SHA3-256 hash of verification document
    verification_provider VARCHAR(100), -- Name of third-party verification service
    verified_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, approved, rejected, expired
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_kyc_identity ON kyc_verifications(identity_id);
CREATE INDEX idx_kyc_status ON kyc_verifications(status);
CREATE INDEX idx_kyc_type ON kyc_verifications(verification_type);

-- Attestations - Blockchain-anchored attestation proofs
CREATE TABLE IF NOT EXISTS attestations (
    attestation_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    attestation_type VARCHAR(50) NOT NULL, -- kyc_verified, accredited_investor, employment, education
    issuer VARCHAR(255) NOT NULL, -- Organization that issued the attestation
    proof_hash VARCHAR(255) NOT NULL, -- Hash of the attestation proof
    chain_anchor_tx VARCHAR(255), -- Transaction hash on Boundless blockchain
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attestation_identity ON attestations(identity_id);
CREATE INDEX idx_attestation_type ON attestations(attestation_type);
CREATE INDEX idx_attestation_chain ON attestations(chain_anchor_tx);
CREATE INDEX idx_attestation_revoked ON attestations(revoked);

-- =============================================================================
-- 2. WALLET SERVICE TABLES
-- =============================================================================

-- Wallet Accounts - Multi-asset wallets with Boundless addresses
CREATE TABLE IF NOT EXISTS wallet_accounts (
    wallet_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    boundless_addresses JSONB NOT NULL, -- Array of BoundlessAddress objects
    labels TEXT[] DEFAULT '{}', -- User-defined labels for organization
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallet_identity ON wallet_accounts(identity_id);

-- Wallet Balances - Asset balances for each wallet
CREATE TABLE IF NOT EXISTS wallet_balances (
    balance_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    asset_type VARCHAR(50) NOT NULL, -- Native, UtilityToken, EquityToken, CarbonCredit, NFT, SubscriptionPass
    total_amount BIGINT NOT NULL DEFAULT 0,
    locked_amount BIGINT NOT NULL DEFAULT 0,
    unlocked_amount BIGINT NOT NULL DEFAULT 0,
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(wallet_id, asset_type)
);

CREATE INDEX idx_balance_wallet ON wallet_balances(wallet_id);
CREATE INDEX idx_balance_asset_type ON wallet_balances(asset_type);

-- Wallet Transactions - Transaction history
CREATE TABLE IF NOT EXISTS wallet_transactions (
    tx_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    chain_tx_hash VARCHAR(255) NOT NULL, -- Blockchain transaction hash
    asset_type VARCHAR(50) NOT NULL,
    amount BIGINT NOT NULL,
    direction VARCHAR(20) NOT NULL, -- incoming, outgoing
    to_address VARCHAR(255),
    from_address VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, confirmed, failed
    block_height BIGINT,
    confirmations INTEGER NOT NULL DEFAULT 0,
    fee BIGINT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tx_wallet ON wallet_transactions(wallet_id);
CREATE INDEX idx_tx_chain_hash ON wallet_transactions(chain_tx_hash);
CREATE INDEX idx_tx_status ON wallet_transactions(status);
CREATE INDEX idx_tx_created ON wallet_transactions(created_at DESC);

-- =============================================================================
-- 3. AUTH & SSO SERVICE TABLES
-- =============================================================================

-- Multipass Credentials - User authentication credentials
CREATE TABLE IF NOT EXISTS multipass_credentials (
    credential_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL, -- Argon2id hash
    totp_secret VARCHAR(255), -- For 2FA
    backup_codes TEXT[], -- Emergency backup codes
    require_2fa BOOLEAN NOT NULL DEFAULT FALSE,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_credential_identity ON multipass_credentials(identity_id);
CREATE INDEX idx_credential_username ON multipass_credentials(username);

-- Multipass Sessions - Active user sessions with JWT tokens
CREATE TABLE IF NOT EXISTS multipass_sessions (
    session_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL, -- SHA3-256 hash of JWT token
    scopes TEXT[] DEFAULT '{}', -- Permissions granted to this session
    device_info JSONB DEFAULT '{}', -- Browser, OS, IP address
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_session_identity ON multipass_sessions(identity_id);
CREATE INDEX idx_session_token ON multipass_sessions(token_hash);
CREATE INDEX idx_session_expires ON multipass_sessions(expires_at);
CREATE INDEX idx_session_revoked ON multipass_sessions(revoked);

-- =============================================================================
-- 4. APPLICATION MODULE REGISTRY TABLES
-- =============================================================================

-- Application Modules - Pluggable business applications
CREATE TABLE IF NOT EXISTS application_modules (
    app_id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL, -- Finance, Supply Chain, Carbon, Healthcare, Education, Gaming
    api_base_url VARCHAR(500) NOT NULL,
    required_scopes TEXT[] DEFAULT '{}', -- Permissions required to use this app
    on_chain_contract_ref VARCHAR(255), -- Reference to smart contract (if applicable)
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_app_category ON application_modules(category);
CREATE INDEX idx_app_enabled ON application_modules(enabled);

-- Application Events - Event/action tracking for applications
CREATE TABLE IF NOT EXISTS application_events (
    event_id UUID PRIMARY KEY,
    app_id UUID NOT NULL REFERENCES application_modules(app_id) ON DELETE CASCADE,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL, -- app.installed, app.uninstalled, action.performed
    event_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_app_event_app ON application_events(app_id);
CREATE INDEX idx_app_event_identity ON application_events(identity_id);
CREATE INDEX idx_app_event_type ON application_events(event_type);
CREATE INDEX idx_app_event_created ON application_events(created_at DESC);

-- =============================================================================
-- 5. ASSET & MARKET SERVICE TABLES
-- =============================================================================

-- Asset Definitions - Token/asset definitions
CREATE TABLE IF NOT EXISTS asset_definitions (
    asset_id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(20) UNIQUE NOT NULL,
    asset_type VARCHAR(50) NOT NULL, -- Native, UtilityToken, EquityToken, CarbonCredit, NFT, SubscriptionPass
    total_supply BIGINT NOT NULL,
    circulating_supply BIGINT NOT NULL DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_asset_symbol ON asset_definitions(symbol);
CREATE INDEX idx_asset_type ON asset_definitions(asset_type);

-- Asset Balances - Internal asset holdings (for enterprise-issued assets)
CREATE TABLE IF NOT EXISTS asset_balances (
    balance_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id) ON DELETE CASCADE,
    total_quantity BIGINT NOT NULL DEFAULT 0,
    locked_quantity BIGINT NOT NULL DEFAULT 0,
    UNIQUE(wallet_id, asset_id)
);

CREATE INDEX idx_asset_balance_wallet ON asset_balances(wallet_id);
CREATE INDEX idx_asset_balance_asset ON asset_balances(asset_id);

-- Market Orders - Internal trading orders
CREATE TABLE IF NOT EXISTS market_orders (
    order_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id) ON DELETE CASCADE,
    order_type VARCHAR(20) NOT NULL, -- Buy, Sell
    quantity BIGINT NOT NULL,
    price BIGINT NOT NULL,
    filled_quantity BIGINT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'Open', -- Open, PartiallyFilled, Filled, Cancelled
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_order_wallet ON market_orders(wallet_id);
CREATE INDEX idx_order_asset ON market_orders(asset_id);
CREATE INDEX idx_order_status ON market_orders(status);
CREATE INDEX idx_order_type ON market_orders(order_type);
CREATE INDEX idx_order_created ON market_orders(created_at DESC);

-- Positions - Aggregated holdings with average cost
CREATE TABLE IF NOT EXISTS positions (
    position_id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL,
    average_cost BIGINT NOT NULL,
    UNIQUE(wallet_id, asset_id)
);

CREATE INDEX idx_position_wallet ON positions(wallet_id);
CREATE INDEX idx_position_asset ON positions(asset_id);

-- Trades - Executed trade history
CREATE TABLE IF NOT EXISTS trades (
    trade_id UUID PRIMARY KEY,
    asset_id UUID NOT NULL REFERENCES asset_definitions(asset_id) ON DELETE CASCADE,
    buy_order_id UUID NOT NULL REFERENCES market_orders(order_id) ON DELETE CASCADE,
    sell_order_id UUID NOT NULL REFERENCES market_orders(order_id) ON DELETE CASCADE,
    quantity BIGINT NOT NULL,
    price BIGINT NOT NULL,
    buyer_wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    seller_wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trade_asset ON trades(asset_id);
CREATE INDEX idx_trade_buyer ON trades(buyer_wallet_id);
CREATE INDEX idx_trade_seller ON trades(seller_wallet_id);
CREATE INDEX idx_trade_created ON trades(created_at DESC);

-- =============================================================================
-- 6. EVENT & REPORTING SERVICE TABLES
-- =============================================================================

-- Notifications - User notifications
CREATE TABLE IF NOT EXISTS notifications (
    notification_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL, -- Info, Warning, Error, Success
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_identity ON notifications(identity_id);
CREATE INDEX idx_notification_read ON notifications(read);
CREATE INDEX idx_notification_created ON notifications(created_at DESC);

-- Report Definitions - SQL-based report templates
CREATE TABLE IF NOT EXISTS report_definitions (
    report_id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL, -- TransactionSummary, BalanceSheet, AuditLog, Custom
    sql_template TEXT NOT NULL, -- Parameterized SQL query
    parameters TEXT[] DEFAULT '{}', -- List of parameter names
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_report_type ON report_definitions(report_type);

-- Generated Reports - Stored report results
CREATE TABLE IF NOT EXISTS generated_reports (
    generated_report_id UUID PRIMARY KEY,
    report_id UUID NOT NULL REFERENCES report_definitions(report_id) ON DELETE CASCADE,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    parameters JSONB DEFAULT '{}', -- Actual parameter values used
    format VARCHAR(20) NOT NULL, -- JSON, CSV, PDF
    result_data TEXT NOT NULL, -- Serialized report data
    chain_anchor_tx VARCHAR(255), -- Optional blockchain anchor for immutability
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_generated_report_def ON generated_reports(report_id);
CREATE INDEX idx_generated_report_identity ON generated_reports(identity_id);
CREATE INDEX idx_generated_report_created ON generated_reports(created_at DESC);

-- =============================================================================
-- 7. HARDWARE PASS SERVICE TABLES
-- =============================================================================

-- Hardware Passes - NFC cards and secure elements
CREATE TABLE IF NOT EXISTS hardware_passes (
    device_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    device_type VARCHAR(50) NOT NULL DEFAULT 'NFC_Card', -- NFC_Card, Secure_Element, Yubikey
    public_key TEXT NOT NULL, -- PQC public key (hex-encoded)
    capabilities TEXT[] DEFAULT '{}', -- Payment, Authentication, Signing, Access_Control
    status VARCHAR(20) NOT NULL DEFAULT 'Active', -- Active, Revoked, Lost, Expired
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_hardware_identity ON hardware_passes(identity_id);
CREATE INDEX idx_hardware_status ON hardware_passes(status);

-- Hardware Challenges - Challenge-response authentication
CREATE TABLE IF NOT EXISTS hardware_challenges (
    challenge_id UUID PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES hardware_passes(device_id) ON DELETE CASCADE,
    challenge BYTEA NOT NULL, -- Random challenge bytes
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_challenge_device ON hardware_challenges(device_id);
CREATE INDEX idx_challenge_expires ON hardware_challenges(expires_at);
CREATE INDEX idx_challenge_used ON hardware_challenges(used);

-- =============================================================================
-- TRIGGERS FOR AUTOMATIC TIMESTAMP UPDATES
-- =============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables with updated_at column
CREATE TRIGGER update_identity_profiles_updated_at BEFORE UPDATE ON identity_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_wallet_accounts_updated_at BEFORE UPDATE ON wallet_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_multipass_credentials_updated_at BEFORE UPDATE ON multipass_credentials
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_application_modules_updated_at BEFORE UPDATE ON application_modules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_asset_definitions_updated_at BEFORE UPDATE ON asset_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_market_orders_updated_at BEFORE UPDATE ON market_orders
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_report_definitions_updated_at BEFORE UPDATE ON report_definitions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_hardware_passes_updated_at BEFORE UPDATE ON hardware_passes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
