-- Create smart contract management tables
-- This migration adds support for E2 Multipass smart contract deployment and interaction

-- Create contract status enum
CREATE TYPE contract_status AS ENUM (
    'pending',
    'deploying',
    'deployed',
    'failed',
    'paused',
    'terminated'
);

-- Create contract template type enum
CREATE TYPE contract_template_type AS ENUM (
    'identity_access_control',
    'multisig_wallet',
    'asset_escrow',
    'app_authorization',
    'custom'
);

-- Create contracts table
CREATE TABLE IF NOT EXISTS contracts (
    contract_id UUID PRIMARY KEY,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    template_type contract_template_type NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    wasm_hash VARCHAR(64) NOT NULL,
    contract_address VARCHAR(42),  -- Ethereum-style address
    abi_json JSONB,  -- Contract ABI for interaction
    constructor_args JSONB,  -- Arguments used during deployment
    status contract_status NOT NULL DEFAULT 'pending',
    gas_used BIGINT,
    deployment_tx_hash VARCHAR(66),  -- Transaction hash
    metadata JSONB,  -- Additional contract metadata
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deployed_at TIMESTAMP WITH TIME ZONE,

    -- Indexes
    CONSTRAINT unique_contract_address UNIQUE (contract_address)
);

-- Create index for fast identity lookup
CREATE INDEX IF NOT EXISTS idx_contracts_identity ON contracts(identity_id);

-- Create index for status queries
CREATE INDEX IF NOT EXISTS idx_contracts_status ON contracts(status) WHERE status = 'deployed';

-- Create index for template type
CREATE INDEX IF NOT EXISTS idx_contracts_template ON contracts(template_type);

-- Create contract interactions table
CREATE TABLE IF NOT EXISTS contract_interactions (
    interaction_id UUID PRIMARY KEY,
    contract_id UUID NOT NULL REFERENCES contracts(contract_id) ON DELETE CASCADE,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,
    method_name VARCHAR(255) NOT NULL,
    method_args JSONB,  -- Method arguments
    tx_hash VARCHAR(66),  -- Transaction hash (for state-changing calls)
    status VARCHAR(50) NOT NULL,  -- success, failed, pending
    gas_used BIGINT,
    result JSONB,  -- Call result
    error_message TEXT,  -- Error message if failed
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create index for fast contract lookup
CREATE INDEX IF NOT EXISTS idx_interactions_contract ON contract_interactions(contract_id, created_at DESC);

-- Create index for identity lookup
CREATE INDEX IF NOT EXISTS idx_interactions_identity ON contract_interactions(identity_id);

-- Create index for transaction hash lookup
CREATE INDEX IF NOT EXISTS idx_interactions_tx_hash ON contract_interactions(tx_hash) WHERE tx_hash IS NOT NULL;

-- Add comments
COMMENT ON TABLE contracts IS 'Smart contracts deployed via E2 Multipass';
COMMENT ON TABLE contract_interactions IS 'Record of all contract method calls and transactions';
COMMENT ON COLUMN contracts.wasm_hash IS 'SHA3-256 hash of the WASM bytecode';
COMMENT ON COLUMN contracts.contract_address IS 'On-chain contract address';
COMMENT ON COLUMN contracts.abi_json IS 'Contract ABI for frontend integration';
COMMENT ON COLUMN contract_interactions.method_name IS 'Contract method being called';
COMMENT ON COLUMN contract_interactions.tx_hash IS 'Transaction hash for state-changing calls';
