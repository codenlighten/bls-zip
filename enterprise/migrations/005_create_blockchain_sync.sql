-- Migration 005: Create blockchain synchronization tables
--
-- Stores blockchain transaction history and sync state
-- Enables local caching of on-chain data for faster queries
--
-- Tables:
--  - blockchain_transactions: Local cache of blockchain transactions
--  - sync_state: Tracks synchronization progress per wallet

-- Blockchain transaction cache
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    -- Primary key
    tx_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Transaction identification
    tx_hash VARCHAR(66) NOT NULL UNIQUE,  -- 0x + 64 hex chars
    block_hash VARCHAR(66),
    block_height BIGINT,

    -- Transaction details
    from_address VARCHAR(255) NOT NULL,
    to_address VARCHAR(255) NOT NULL,
    amount BIGINT NOT NULL,
    fee BIGINT NOT NULL DEFAULT 0,

    -- Timestamps
    block_timestamp TIMESTAMP WITH TIME ZONE,
    confirmed_at TIMESTAMP WITH TIME ZONE,
    indexed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Status tracking
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, confirmed, failed
    confirmations INTEGER NOT NULL DEFAULT 0,

    -- Transaction metadata
    transaction_type VARCHAR(50) DEFAULT 'transfer',  -- transfer, asset_transfer, proof_anchor, etc.
    asset_type VARCHAR(100),                          -- For multi-asset transfers
    data JSONB,                                       -- Additional transaction data

    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('pending', 'confirmed', 'failed', 'orphaned')),
    CONSTRAINT valid_amount CHECK (amount >= 0),
    CONSTRAINT valid_fee CHECK (fee >= 0),
    CONSTRAINT valid_confirmations CHECK (confirmations >= 0)
);

-- Blockchain synchronization state
CREATE TABLE IF NOT EXISTS sync_state (
    -- Primary key
    sync_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Wallet being synced
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    blockchain_address VARCHAR(255) NOT NULL,

    -- Sync progress
    last_synced_block BIGINT NOT NULL DEFAULT 0,
    last_synced_at TIMESTAMP WITH TIME ZONE,

    -- Sync metadata
    total_transactions INTEGER NOT NULL DEFAULT 0,
    sync_status VARCHAR(50) NOT NULL DEFAULT 'syncing',  -- syncing, synced, error
    sync_error TEXT,

    -- Performance tracking
    sync_started_at TIMESTAMP WITH TIME ZONE,
    sync_completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT unique_wallet_sync UNIQUE(wallet_id, blockchain_address),
    CONSTRAINT valid_sync_status CHECK (sync_status IN ('syncing', 'synced', 'error', 'paused'))
);

-- Indexes for performance
CREATE INDEX idx_blockchain_tx_hash ON blockchain_transactions(tx_hash);
CREATE INDEX idx_blockchain_block_height ON blockchain_transactions(block_height);
CREATE INDEX idx_blockchain_from_address ON blockchain_transactions(from_address);
CREATE INDEX idx_blockchain_to_address ON blockchain_transactions(to_address);
CREATE INDEX idx_blockchain_status ON blockchain_transactions(status);
CREATE INDEX idx_blockchain_timestamp ON blockchain_transactions(block_timestamp DESC);
CREATE INDEX idx_blockchain_asset_type ON blockchain_transactions(asset_type) WHERE asset_type IS NOT NULL;

CREATE INDEX idx_sync_state_wallet_id ON sync_state(wallet_id);
CREATE INDEX idx_sync_state_address ON sync_state(blockchain_address);
CREATE INDEX idx_sync_state_status ON sync_state(sync_status);

-- Comments
COMMENT ON TABLE blockchain_transactions IS 'Local cache of blockchain transactions for fast queries';
COMMENT ON TABLE sync_state IS 'Blockchain synchronization state per wallet address';

COMMENT ON COLUMN blockchain_transactions.tx_hash IS 'Unique transaction hash from blockchain';
COMMENT ON COLUMN blockchain_transactions.confirmations IS 'Number of block confirmations';
COMMENT ON COLUMN blockchain_transactions.transaction_type IS 'Type of transaction (transfer, proof_anchor, etc.)';
COMMENT ON COLUMN blockchain_transactions.asset_type IS 'For multi-asset support (Native, Equity, Utility, etc.)';

COMMENT ON COLUMN sync_state.last_synced_block IS 'Last blockchain block height synced';
COMMENT ON COLUMN sync_state.total_transactions IS 'Total transactions indexed for this address';
COMMENT ON COLUMN sync_state.sync_status IS 'Current synchronization status';

-- Sample queries

-- Get recent transactions for an address (outgoing)
-- SELECT * FROM blockchain_transactions
-- WHERE from_address = 'bls1...'
-- ORDER BY block_timestamp DESC
-- LIMIT 50;

-- Get recent transactions for an address (incoming)
-- SELECT * FROM blockchain_transactions
-- WHERE to_address = 'bls1...'
-- ORDER BY block_timestamp DESC
-- LIMIT 50;

-- Get transaction history for a wallet (both incoming and outgoing)
-- SELECT bt.*
-- FROM blockchain_transactions bt
-- WHERE bt.from_address IN (
--     SELECT blockchain_address FROM wallet_keys WHERE wallet_id = $1
-- ) OR bt.to_address IN (
--     SELECT blockchain_address FROM wallet_keys WHERE wallet_id = $1
-- )
-- ORDER BY bt.block_timestamp DESC;

-- Get sync status for a wallet
-- SELECT *
-- FROM sync_state
-- WHERE wallet_id = $1
-- ORDER BY last_synced_at DESC;

-- Update sync state after syncing
-- UPDATE sync_state
-- SET last_synced_block = $1,
--     last_synced_at = NOW(),
--     total_transactions = total_transactions + $2,
--     sync_status = 'synced',
--     sync_completed_at = NOW(),
--     updated_at = NOW()
-- WHERE wallet_id = $3 AND blockchain_address = $4;
