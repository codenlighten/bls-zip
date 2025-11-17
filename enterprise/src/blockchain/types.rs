// Types for blockchain RPC communication
use serde::{Deserialize, Serialize};

/// UTXO (Unspent Transaction Output) data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Utxo {
    /// Transaction hash containing this UTXO
    pub tx_hash: String,

    /// Output index in the transaction
    pub output_index: u32,

    /// Amount in base units
    pub amount: u64,

    /// Public key hash (recipient address)
    pub pubkey_hash: String,

    /// Block height where this UTXO was created
    pub block_height: u64,
}

/// Balance and nonce information for an address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    /// Available balance in base units
    pub balance: u64,

    /// Current nonce (transaction count)
    pub nonce: u64,
}

/// Transaction submission result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxSubmitResult {
    /// Transaction hash
    pub tx_hash: String,

    /// Whether transaction was accepted to mempool
    pub accepted: bool,

    /// Optional message (e.g., rejection reason)
    pub message: Option<String>,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxStatus {
    /// In mempool, not yet mined
    Pending,

    /// Included in a block
    Confirmed,

    /// Not found
    NotFound,

    /// Transaction failed validation
    Invalid,
}

/// Transaction record with status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    /// Transaction hash
    pub tx_hash: String,

    /// Current status
    pub status: TxStatus,

    /// Block height (if confirmed)
    pub block_height: Option<u64>,

    /// Gas used
    pub gas_used: Option<u64>,

    /// Timestamp
    pub timestamp: u64,
}

/// Blockchain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    /// Current block height
    pub height: u64,

    /// Best block hash
    pub best_block_hash: String,

    /// Total supply
    pub total_supply: u64,

    /// Current difficulty
    pub difficulty: u32,
}
