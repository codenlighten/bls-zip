-- Migration 004: Create wallet_keys table for encrypted private key storage
--
-- Stores PQC private keys encrypted with AES-256-GCM
-- Master encryption key must be set in MASTER_ENCRYPTION_KEY environment variable
--
-- Security: Private keys are never stored in plain text
--           - Encrypted with AES-256-GCM
--           - Unique nonce per key
--           - Master key rotation supported via keystore.reencrypt_key()

CREATE TABLE IF NOT EXISTS wallet_keys (
    -- Primary key
    key_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Ownership
    wallet_id UUID NOT NULL REFERENCES wallet_accounts(wallet_id) ON DELETE CASCADE,
    identity_id UUID NOT NULL REFERENCES identity_profiles(identity_id) ON DELETE CASCADE,

    -- Blockchain address (derived from public key)
    blockchain_address VARCHAR(255) NOT NULL UNIQUE,

    -- Public key (not encrypted, needed for verification)
    public_key BYTEA NOT NULL,

    -- Encrypted private key (AES-256-GCM ciphertext)
    encrypted_private_key TEXT NOT NULL,

    -- Encryption nonce (base64-encoded, 96 bits for GCM)
    encryption_nonce TEXT NOT NULL,

    -- Key metadata
    key_algorithm VARCHAR(50) NOT NULL DEFAULT 'Dilithium5', -- ML-DSA, Falcon512, etc.
    key_purpose VARCHAR(100) DEFAULT 'signing',              -- signing, encryption, hybrid
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,

    -- Key management
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_backup BOOLEAN NOT NULL DEFAULT false,
    backed_up_at TIMESTAMP WITH TIME ZONE,

    -- Constraints
    CONSTRAINT unique_wallet_address UNIQUE(wallet_id, blockchain_address),
    CONSTRAINT valid_algorithm CHECK (key_algorithm IN ('Dilithium5', 'Dilithium3', 'Falcon512', 'Falcon1024', 'Hybrid'))
);

-- Indexes for performance
CREATE INDEX idx_wallet_keys_wallet_id ON wallet_keys(wallet_id);
CREATE INDEX idx_wallet_keys_identity_id ON wallet_keys(identity_id);
CREATE INDEX idx_wallet_keys_blockchain_address ON wallet_keys(blockchain_address);
CREATE INDEX idx_wallet_keys_active ON wallet_keys(is_active) WHERE is_active = true;

-- Comments
COMMENT ON TABLE wallet_keys IS 'Encrypted PQC private keys for blockchain wallets';
COMMENT ON COLUMN wallet_keys.encrypted_private_key IS 'AES-256-GCM encrypted private key (base64-encoded ciphertext)';
COMMENT ON COLUMN wallet_keys.encryption_nonce IS 'AES-GCM nonce (base64-encoded, 96 bits)';
COMMENT ON COLUMN wallet_keys.public_key IS 'PQC public key (Dilithium/Falcon, not encrypted)';
COMMENT ON COLUMN wallet_keys.blockchain_address IS 'Boundless address derived from public key (bls1...)';

-- Sample query to retrieve and decrypt a key (pseudo-code):
--
-- SELECT encrypted_private_key, encryption_nonce, public_key
-- FROM wallet_keys
-- WHERE blockchain_address = 'bls1...'
-- AND is_active = true;
--
-- Then in Rust:
-- let keystore = Keystore::new()?;  // Loads MASTER_ENCRYPTION_KEY
-- let encrypted_key = EncryptedKey { ciphertext, nonce };
-- let private_key = keystore.decrypt_key(&encrypted_key)?;
-- let keypair = PqcKeyPair::from_bytes(public_key, private_key)?;
