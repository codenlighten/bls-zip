// Error types for Boundless BLS Core
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid proof of work")]
    InvalidProofOfWork,

    #[error("Invalid Merkle root")]
    InvalidMerkleRoot,

    #[error("Transaction has no inputs")]
    NoInputs,

    #[error("Transaction has no outputs")]
    NoOutputs,

    #[error("Transaction output amount is zero")]
    ZeroAmount,

    #[error("Transaction amount overflow")]
    AmountOverflow,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid block timestamp")]
    InvalidTimestamp,

    #[error("Block too large")]
    BlockTooLarge,

    #[error("Transaction too large")]
    TransactionTooLarge,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Missing signature")]
    MissingSignature,

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Unsupported signature type: {0}")]
    UnsupportedSignature(String),

    #[error("Transaction fee too low: {actual} < {minimum} (minimum: {min_fee_per_byte} per byte × {size_bytes} bytes)")]
    FeeTooLow {
        actual: u64,
        minimum: u64,
        min_fee_per_byte: u64,
        size_bytes: usize,
    },

    #[error("Too many inputs: {count} exceeds maximum {max}")]
    TooManyInputs { count: usize, max: usize },

    #[error("Too many outputs: {count} exceeds maximum {max}")]
    TooManyOutputs { count: usize, max: usize },

    #[error("Transaction size {size} exceeds maximum {max}")]
    TransactionSizeExceeded { size: usize, max: usize },
}
