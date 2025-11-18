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
    pub state_root: String,
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
            state_root: hex::encode(block.header.state_root),
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

/// Proof anchor information returned by RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofAnchorInfo {
    /// Unique proof identifier (hex-encoded)
    pub proof_id: String,

    /// Identity that owns this proof (hex-encoded)
    pub identity_id: String,

    /// Type of proof
    pub proof_type: String,

    /// Hash of the actual proof data (hex-encoded)
    pub proof_hash: String,

    /// Block height where proof was anchored
    pub block_height: u64,

    /// Timestamp of anchoring
    pub timestamp: u64,

    /// Optional metadata (hex-encoded)
    pub metadata: String,
}

impl ProofAnchorInfo {
    pub fn from_proof(proof: &boundless_core::ProofAnchor) -> Self {
        Self {
            proof_id: hex::encode(proof.proof_id),
            identity_id: hex::encode(proof.identity_id),
            proof_type: proof.proof_type.as_str().to_string(),
            proof_hash: hex::encode(proof.proof_hash),
            block_height: proof.block_height,
            timestamp: proof.timestamp,
            metadata: hex::encode(&proof.metadata),
        }
    }
}

/// Full transaction details returned by RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetailInfo {
    pub tx_hash: String,
    pub version: u32,
    pub timestamp: u64,
    pub block_height: u64,
    pub block_hash: String,
    pub inputs: Vec<TxInputInfo>,
    pub outputs: Vec<TxOutputInfo>,
    pub fee: u64,
    pub status: String,
}

/// Transaction input information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInputInfo {
    pub prev_output_hash: String,
    pub output_index: u32,
    pub signature_type: String,
    pub signature_size_bytes: usize,
    pub public_key: String,
}

/// Transaction output information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutputInfo {
    pub amount: u64,
    pub recipient_hash: String,
    pub script_type: String,
    pub is_spent: Option<bool>,
}

impl TransactionDetailInfo {
    pub fn from_transaction_record(record: &boundless_core::TransactionRecord) -> Self {
        Self {
            tx_hash: hex::encode(record.tx_hash),
            version: 1, // Default version
            timestamp: record.timestamp,
            block_height: record.block_height,
            block_hash: hex::encode(record.block_hash),
            inputs: record.inputs.iter().map(TxInputInfo::from_tx_input).collect(),
            outputs: record.outputs.iter().map(TxOutputInfo::from_tx_output).collect(),
            fee: record.fee,
            status: format!("{:?}", record.status),
        }
    }
}

impl TxInputInfo {
    pub fn from_tx_input(input: &boundless_core::TxInput) -> Self {
        Self {
            prev_output_hash: hex::encode(input.previous_output_hash),
            output_index: input.output_index,
            signature_type: format!("{:?}", &input.signature),
            signature_size_bytes: input.signature.size_bytes(),
            public_key: hex::encode(&input.public_key),
        }
    }
}

impl TxOutputInfo {
    pub fn from_tx_output(output: &boundless_core::TxOutput) -> Self {
        Self {
            amount: output.amount,
            recipient_hash: hex::encode(&output.recipient_pubkey_hash),
            script_type: if output.script.is_some() { "Custom".to_string() } else { "Standard".to_string() },
            is_spent: None, // Would need UTXO tracking to determine
        }
    }
}
