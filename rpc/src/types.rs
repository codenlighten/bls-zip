// RPC type definitions
use boundless_core::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// Block information returned by RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub difficulty_target: u32,
    pub nonce: u64,
    pub merkle_root: String,
    pub transaction_count: usize,
    pub transactions: Vec<TransactionInfo>,
}

impl BlockInfo {
    pub fn from_block(block: &Block) -> Self {
        Self {
            height: block.header.height,
            hash: hex::encode(block.header.hash()),
            previous_hash: hex::encode(block.header.previous_hash),
            timestamp: block.header.timestamp,
            difficulty_target: block.header.difficulty_target,
            nonce: block.header.nonce,
            merkle_root: hex::encode(block.header.merkle_root),
            transaction_count: block.transactions.len(),
            transactions: block
                .transactions
                .iter()
                .map(TransactionInfo::from_transaction)
                .collect(),
        }
    }
}

/// Transaction information returned by RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub version: u32,
    pub input_count: usize,
    pub output_count: usize,
    pub timestamp: u64,
    pub size_bytes: usize,
}

impl TransactionInfo {
    pub fn from_transaction(tx: &Transaction) -> Self {
        Self {
            hash: hex::encode(tx.hash()),
            version: tx.version,
            input_count: tx.inputs.len(),
            output_count: tx.outputs.len(),
            timestamp: tx.timestamp,
            size_bytes: tx.size_bytes(),
        }
    }
}

/// Blockchain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub height: u64,
    pub best_block_hash: String,
    pub total_supply: u64,
    pub difficulty: u32,
}

/// Account balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

/// Submit transaction response
/// HIGH PRIORITY FIX: Removed success field to comply with JSON-RPC 2.0 semantics
/// On success: returns tx_hash via Result::Ok
/// On failure: returns error via Result::Err (not success: false)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitTxResponse {
    pub tx_hash: String,
}

/// HIGH PRIORITY FIX: Pagination limits to prevent DoS
/// Maximum number of items that can be requested in a single query
pub const MAX_PAGINATION_LIMIT: u32 = 100;

/// Helper to enforce pagination limits
pub fn enforce_pagination_limit(requested: u32) -> u32 {
    if requested == 0 || requested > MAX_PAGINATION_LIMIT {
        MAX_PAGINATION_LIMIT
    } else {
        requested
    }
}

/// Network info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub node_version: String,
    pub protocol_version: u32,
    pub peer_count: usize,
    pub is_mining: bool,
}

/// UTXO (Unspent Transaction Output) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoData {
    /// Transaction hash containing this UTXO
    pub tx_hash: String,

    /// Output index in the transaction
    pub output_index: u32,

    /// Amount in satoshis
    pub amount: u64,

    /// Block height where this UTXO was created
    pub block_height: u64,

    /// Optional locking script (hex-encoded)
    pub script: Option<String>,
}
