// Transaction Indexing for History Queries
//
// Enables efficient lookup of transaction history by address
// Required for E2 integration transaction history endpoints

use crate::{Transaction, TxInput, TxOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transaction index entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionRecord {
    /// Transaction hash
    pub tx_hash: [u8; 32],

    /// Block height where tx was included
    pub block_height: u64,

    /// Block hash
    pub block_hash: [u8; 32],

    /// Timestamp when tx was included
    pub timestamp: u64,

    /// Transaction inputs
    pub inputs: Vec<TxInput>,

    /// Transaction outputs
    pub outputs: Vec<TxOutput>,

    /// Transaction fee
    pub fee: u64,

    /// Transaction status
    pub status: TransactionStatus,
}

impl TransactionRecord {
    /// Create a new transaction record
    pub fn new(
        tx: &Transaction,
        block_height: u64,
        block_hash: [u8; 32],
        timestamp: u64,
        fee: u64,
    ) -> Self {
        Self {
            tx_hash: tx.hash(),
            block_height,
            block_hash,
            timestamp,
            inputs: tx.inputs.clone(),
            outputs: tx.outputs.clone(),
            fee,
            status: TransactionStatus::Confirmed,
        }
    }

    /// Get transaction hash as hex string
    pub fn tx_hash_hex(&self) -> String {
        hex::encode(self.tx_hash)
    }

    /// Get block hash as hex string
    pub fn block_hash_hex(&self) -> String {
        hex::encode(self.block_hash)
    }

    /// Get total input amount
    pub fn total_input(&self) -> u64 {
        // Note: Would need UTXO lookup to get actual amounts
        // For now, return 0 (requires state integration)
        0
    }

    /// Get total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|out| out.amount).sum()
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Pending in mempool
    Pending,
    /// Confirmed in a block
    Confirmed,
    /// Failed validation
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Failed => "failed",
        }
    }
}

/// Transaction index for efficient history queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransactionIndex {
    /// Map of tx_hash -> TransactionRecord
    transactions: HashMap<[u8; 32], TransactionRecord>,

    /// Map of address -> list of tx_hashes (sent from or received to)
    address_txs: HashMap<[u8; 32], Vec<[u8; 32]>>,

    /// Map of block_height -> list of tx_hashes
    block_txs: HashMap<u64, Vec<[u8; 32]>>,
}

impl TransactionIndex {
    /// Create a new transaction index
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a transaction
    pub fn index_transaction(
        &mut self,
        tx: &Transaction,
        block_height: u64,
        block_hash: [u8; 32],
        timestamp: u64,
        fee: u64,
    ) {
        let tx_hash = tx.hash();

        // Create transaction record
        let record = TransactionRecord::new(tx, block_height, block_hash, timestamp, fee);

        // Store transaction
        self.transactions.insert(tx_hash, record);

        // Index by addresses involved
        let mut addresses: Vec<[u8; 32]> = Vec::new();

        // Add input addresses (senders)
        for input in &tx.inputs {
            // Extract address from public key using SHA3-256 hash
            use sha3::{Digest, Sha3_256};
            let mut hasher = Sha3_256::new();
            hasher.update(&input.public_key);
            let result = hasher.finalize();
            let mut address = [0u8; 32];
            address.copy_from_slice(&result);
            addresses.push(address);
        }

        // Add output addresses (recipients)
        for output in &tx.outputs {
            addresses.push(output.recipient_pubkey_hash);
        }

        // Update address index
        for address in addresses {
            self.address_txs
                .entry(address)
                .or_insert_with(Vec::new)
                .push(tx_hash);
        }

        // Update block index
        self.block_txs
            .entry(block_height)
            .or_insert_with(Vec::new)
            .push(tx_hash);
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &[u8; 32]) -> Option<&TransactionRecord> {
        self.transactions.get(tx_hash)
    }

    /// Get transaction history for an address
    pub fn get_address_transactions(
        &self,
        address: &[u8; 32],
        limit: usize,
        offset: usize,
    ) -> Vec<&TransactionRecord> {
        self.address_txs
            .get(address)
            .map(|tx_hashes| {
                tx_hashes
                    .iter()
                    .skip(offset)
                    .take(limit)
                    .filter_map(|hash| self.transactions.get(hash))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all transactions in a block
    pub fn get_block_transactions(&self, block_height: u64) -> Vec<&TransactionRecord> {
        self.block_txs
            .get(&block_height)
            .map(|tx_hashes| {
                tx_hashes
                    .iter()
                    .filter_map(|hash| self.transactions.get(hash))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get total number of transactions for an address
    pub fn get_address_tx_count(&self, address: &[u8; 32]) -> usize {
        self.address_txs
            .get(address)
            .map(|txs| txs.len())
            .unwrap_or(0)
    }

    /// Get total number of indexed transactions
    pub fn total_transactions(&self) -> usize {
        self.transactions.len()
    }

    /// Get total number of addresses with transactions
    pub fn total_addresses(&self) -> usize {
        self.address_txs.len()
    }

    /// Remove transactions from index (for chain reorganization)
    pub fn remove_block_transactions(&mut self, block_height: u64) {
        if let Some(tx_hashes) = self.block_txs.remove(&block_height) {
            for tx_hash in tx_hashes {
                // Remove from main index
                if let Some(record) = self.transactions.remove(&tx_hash) {
                    // Remove from address indices
                    for output in &record.outputs {
                        if let Some(addr_txs) =
                            self.address_txs.get_mut(&output.recipient_pubkey_hash)
                        {
                            addr_txs.retain(|h| h != &tx_hash);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Signature, TxInput, TxOutput};

    fn create_test_transaction(from: [u8; 32], to: [u8; 32], amount: u64) -> Transaction {
        Transaction {
            version: 1,
            inputs: vec![TxInput {
                previous_output_hash: [0u8; 32],
                output_index: 0,
                signature: Signature::Classical(from.to_vec()),
                public_key: from.to_vec(),
                nonce: None,
            }],
            outputs: vec![TxOutput {
                amount,
                recipient_pubkey_hash: to,
                script: None,
            }],
            timestamp: 0,
            data: None,
        }
    }

    #[test]
    fn test_transaction_indexing() {
        let mut index = TransactionIndex::new();

        let from = [1u8; 32];
        let to = [2u8; 32];
        let tx = create_test_transaction(from, to, 1000);
        let tx_hash = tx.hash();

        // Index transaction
        index.index_transaction(&tx, 100, [3u8; 32], 1234567890, 100);

        // Retrieve transaction
        let record = index.get_transaction(&tx_hash);
        assert!(record.is_some());
        assert_eq!(record.unwrap().block_height, 100);
        assert_eq!(record.unwrap().fee, 100);

        // Check address index
        let addr_txs = index.get_address_transactions(&to, 10, 0);
        assert_eq!(addr_txs.len(), 1);
        assert_eq!(addr_txs[0].tx_hash, tx_hash);
    }

    #[test]
    fn test_address_transaction_history() {
        let mut index = TransactionIndex::new();

        let from = [1u8; 32];
        let to = [2u8; 32];

        // Index multiple transactions
        for i in 0..5 {
            let tx = create_test_transaction(from, to, 1000 * (i + 1));
            index.index_transaction(&tx, 100 + i, [3u8; 32], 1234567890 + i, 100);
        }

        // Get transaction history with limit and offset
        let txs = index.get_address_transactions(&to, 2, 0);
        assert_eq!(txs.len(), 2);

        let txs_offset = index.get_address_transactions(&to, 2, 2);
        assert_eq!(txs_offset.len(), 2);

        // Check total count
        assert_eq!(index.get_address_tx_count(&to), 5);
    }

    #[test]
    fn test_block_transaction_index() {
        let mut index = TransactionIndex::new();

        let block_height = 100u64;

        // Index multiple transactions in same block
        for i in 0..3 {
            let from = [1u8; 32];
            let to = [(i + 2) as u8; 32];
            let tx = create_test_transaction(from, to, 1000);
            index.index_transaction(&tx, block_height, [3u8; 32], 1234567890, 100);
        }

        // Get all transactions in block
        let block_txs = index.get_block_transactions(block_height);
        assert_eq!(block_txs.len(), 3);
    }

    #[test]
    fn test_remove_block_transactions() {
        let mut index = TransactionIndex::new();

        let block_height = 100u64;
        let to = [2u8; 32];

        // Index transaction
        let tx = create_test_transaction([1u8; 32], to, 1000);
        let tx_hash = tx.hash();
        index.index_transaction(&tx, block_height, [3u8; 32], 1234567890, 100);

        assert!(index.get_transaction(&tx_hash).is_some());

        // Remove block transactions
        index.remove_block_transactions(block_height);

        // Transaction should be removed
        assert!(index.get_transaction(&tx_hash).is_none());
        assert_eq!(index.get_address_tx_count(&to), 0);
    }

    #[test]
    fn test_transaction_status() {
        assert_eq!(TransactionStatus::Pending.as_str(), "pending");
        assert_eq!(TransactionStatus::Confirmed.as_str(), "confirmed");
        assert_eq!(TransactionStatus::Failed.as_str(), "failed");
    }
}
