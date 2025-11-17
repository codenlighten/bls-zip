// Boundless BLS network protocol messages
use boundless_core::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// Network protocol version
pub const PROTOCOL_VERSION: u32 = 1;

/// Network protocol identifier
pub const PROTOCOL_ID: &str = "/boundless/1.0.0";

/// Message types exchanged between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Request blocks starting from a specific height
    GetBlocks { start_height: u64, count: u32 },

    /// Response with requested blocks
    Blocks { blocks: Vec<Block> },

    /// Announce a new block
    NewBlock { block: Block },

    /// Announce a new transaction
    NewTransaction { transaction: Transaction },

    /// Request blockchain status
    GetStatus,

    /// Response with blockchain status
    Status {
        height: u64,
        best_block_hash: [u8; 32],
        total_supply: u64,
    },

    /// Ping message
    Ping { nonce: u64 },

    /// Pong response
    Pong { nonce: u64 },
}

impl Message {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Get message type as string
    pub fn message_type(&self) -> &'static str {
        match self {
            Message::GetBlocks { .. } => "GetBlocks",
            Message::Blocks { .. } => "Blocks",
            Message::NewBlock { .. } => "NewBlock",
            Message::NewTransaction { .. } => "NewTransaction",
            Message::GetStatus => "GetStatus",
            Message::Status { .. } => "Status",
            Message::Ping { .. } => "Ping",
            Message::Pong { .. } => "Pong",
        }
    }
}

/// Message type enum for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    GetBlocks,
    Blocks,
    NewBlock,
    NewTransaction,
    GetStatus,
    Status,
    Ping,
    Pong,
}

/// Boundless protocol handler
pub struct BoundlessProtocol;

impl BoundlessProtocol {
    /// Create protocol identifier
    pub fn protocol_id() -> &'static str {
        PROTOCOL_ID
    }

    /// Get protocol version
    pub fn version() -> u32 {
        PROTOCOL_VERSION
    }
}
