// Block and BlockHeader structures for Boundless BLS
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::error::CoreError;
use crate::transaction::Transaction;

/// Block header containing metadata for Proof-of-Work
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    /// Protocol version
    pub version: u32,

    /// Hash of the previous block header (SHA3-256)
    pub previous_hash: [u8; 32],

    /// Merkle root of all transactions in the block
    pub merkle_root: [u8; 32],

    /// Block timestamp (Unix timestamp in seconds)
    pub timestamp: u64,

    /// Difficulty target (compact representation)
    pub difficulty_target: u32,

    /// Nonce for Proof-of-Work mining
    pub nonce: u64,

    /// Block height in the chain
    pub height: u64,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        version: u32,
        previous_hash: [u8; 32],
        merkle_root: [u8; 32],
        timestamp: u64,
        difficulty_target: u32,
        nonce: u64,
        height: u64,
    ) -> Self {
        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty_target,
            nonce,
            height,
        }
    }

    /// Calculate the SHA3-256 hash of this block header
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();

        // Hash all fields in deterministic order
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.previous_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.difficulty_target.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.height.to_le_bytes());

        hasher.finalize().into()
    }

    /// Check if the block header meets the difficulty target
    pub fn meets_difficulty_target(&self) -> bool {
        let hash = self.hash();
        let hash_value = U256::from_big_endian(&hash);
        let target = Self::compact_to_target(self.difficulty_target);

        hash_value < target
    }

    /// Convert compact difficulty representation to full U256 target
    pub fn compact_to_target(compact: u32) -> U256 {
        let exponent = (compact >> 24) as usize;
        let mantissa = compact & 0x00ffffff;

        if exponent <= 3 {
            U256::from(mantissa) >> (8 * (3 - exponent))
        } else {
            U256::from(mantissa) << (8 * (exponent - 3))
        }
    }

    /// Convert U256 target to compact difficulty representation
    pub fn target_to_compact(target: U256) -> u32 {
        let mut size = 32;
        let mut compact = 0u32;

        // Find the most significant byte
        let bytes = {
            let mut b = [0u8; 32];
            target.to_big_endian(&mut b);
            b
        };

        // Find first non-zero byte
        let mut start = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte != 0 {
                start = i;
                break;
            }
        }

        size = 32 - start;

        if size <= 3 {
            compact = (bytes[start] as u32) << (8 * (3 - size));
            compact |= (size as u32) << 24;
        } else {
            compact = (bytes[start] as u32) << 16;
            compact |= (bytes[start + 1] as u32) << 8;
            compact |= bytes[start + 2] as u32;
            compact |= (size as u32) << 24;
        }

        compact
    }
}

/// Complete block including header and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,

    /// List of transactions in this block
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create a new block
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    /// Get the hash of this block (hash of the header)
    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }

    /// Calculate the Merkle root of all transactions
    pub fn calculate_merkle_root(&self) -> [u8; 32] {
        if self.transactions.is_empty() {
            return [0u8; 32];
        }

        let mut hashes: Vec<[u8; 32]> = self.transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(*hashes.last().unwrap());
            }

            hashes = hashes
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Sha3_256::new();
                    hasher.update(pair[0]);
                    hasher.update(pair[1]);
                    hasher.finalize().into()
                })
                .collect();
        }

        hashes[0]
    }

    /// Verify that the block's Merkle root matches the calculated one
    pub fn verify_merkle_root(&self) -> bool {
        self.header.merkle_root == self.calculate_merkle_root()
    }

    /// Validate the entire block
    pub fn validate(&self) -> Result<(), CoreError> {
        // SECURITY FIX: Validate block size limits
        self.validate_size()?;

        // SECURITY FIX: Validate transaction count
        self.validate_transaction_count()?;

        // Check PoW
        if !self.header.meets_difficulty_target() {
            return Err(CoreError::InvalidProofOfWork);
        }

        // Check Merkle root
        if !self.verify_merkle_root() {
            return Err(CoreError::InvalidMerkleRoot);
        }

        // Validate all transactions
        for tx in &self.transactions {
            tx.validate()?;
        }

        Ok(())
    }

    /// Validate block size limits
    ///
    /// SECURITY: Prevents DoS attacks via oversized blocks
    /// Maximum block size: 4MB
    pub fn validate_size(&self) -> Result<(), CoreError> {
        const MAX_BLOCK_SIZE: usize = 4_000_000; // 4MB

        let size = self.size_bytes();
        if size > MAX_BLOCK_SIZE {
            return Err(CoreError::BlockTooLarge);
        }

        Ok(())
    }

    /// Validate transaction count
    ///
    /// SECURITY: Prevents DoS attacks via excessive transaction counts
    /// Maximum transactions per block: 10,000
    pub fn validate_transaction_count(&self) -> Result<(), CoreError> {
        const MAX_TRANSACTIONS: usize = 10_000;

        if self.transactions.len() > MAX_TRANSACTIONS {
            return Err(CoreError::InvalidTransaction(format!(
                "Block contains {} transactions, maximum allowed is {}",
                self.transactions.len(),
                MAX_TRANSACTIONS
            )));
        }

        Ok(())
    }

    /// Get the block height (must be determined from blockchain context)
    pub fn size_bytes(&self) -> usize {
        bincode::serialize(self).unwrap_or_default().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_hash() {
        let header = BlockHeader::new(1, [0u8; 32], [0u8; 32], 1234567890, 0x1d00ffff, 0, 1);

        let hash = header.hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_compact_target_conversion() {
        // Create U256 from byte array (256-bit value)
        let target = U256::from_big_endian(&[
            0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);
        let compact = BlockHeader::target_to_compact(target);
        let converted = BlockHeader::compact_to_target(compact);

        // Should be approximately equal (some precision loss in compact format)
        assert!(target >= converted);
    }

    #[test]
    fn test_merkle_root_empty() {
        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0, 0, 1),
            vec![],
        );

        assert_eq!(block.calculate_merkle_root(), [0u8; 32]);
    }

    // SECURITY TESTS: Block Size Validation

    #[test]
    fn test_block_size_within_limit() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a normal-sized block with a few transactions
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx; 10], // 10 transactions
        );

        // Should pass size validation
        assert!(block.validate_size().is_ok());
    }

    #[test]
    fn test_block_size_exceeded() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a block with a very large transaction that exceeds 4MB
        // Each transaction has ~4.2MB of data
        let large_data = vec![0u8; 4_200_000];
        let large_tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(large_data),
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![large_tx],
        );

        // Should fail size validation (>4MB)
        let result = block.validate_size();
        assert!(matches!(result, Err(CoreError::BlockTooLarge)));
    }

    #[test]
    fn test_block_at_max_size() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a block close to but under 4MB limit
        // Account for block header overhead (~100 bytes)
        let data_size = 4_000_000 - 500;
        let data = vec![0u8; data_size];
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(data),
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx],
        );

        // Should pass (just under limit)
        assert!(block.validate_size().is_ok());
    }

    // SECURITY TESTS: Transaction Count Validation

    #[test]
    fn test_block_transaction_count_within_limit() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a block with a reasonable number of transactions
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx; 100], // 100 transactions
        );

        // Should pass transaction count validation
        assert!(block.validate_transaction_count().is_ok());
    }

    #[test]
    fn test_block_too_many_transactions() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a block with more than 10,000 transactions
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx; 10_001], // 10,001 transactions
        );

        // Should fail transaction count validation
        let result = block.validate_transaction_count();
        assert!(matches!(result, Err(CoreError::InvalidTransaction(_))));
    }

    #[test]
    fn test_block_at_max_transaction_count() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Create a block with exactly 10,000 transactions (at limit)
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx; 10_000], // Exactly 10,000 transactions
        );

        // Should pass transaction count validation
        assert!(block.validate_transaction_count().is_ok());
    }

    #[test]
    fn test_block_validate_integrates_all_checks() {
        use crate::transaction::{Signature, Transaction, TxInput, TxOutput};

        // Test that validate() calls all security checks

        // 1. Test that oversized block fails via validate()
        let large_data = vec![0u8; 4_200_000];
        let large_tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            Some(large_data),
        );

        let oversized_block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![large_tx],
        );

        // Should fail validation due to size
        assert!(oversized_block.validate().is_err());

        // 2. Test that too many transactions fails via validate()
        let tx = Transaction::new(
            1,
            vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(vec![0u8; 64]),
                public_key: vec![0u8; 33],
                nonce: None,
            }],
            vec![TxOutput {
                amount: 1000,
                recipient_pubkey_hash: [0u8; 32],
                script: None,
            }],
            1234567890,
            None,
        );

        let too_many_tx_block = Block::new(
            BlockHeader::new(1, [0u8; 32], [0u8; 32], 0, 0x1d00ffff, 0, 1),
            vec![tx; 10_001],
        );

        // Should fail validation due to transaction count
        assert!(too_many_tx_block.validate().is_err());
    }
}
