// Error types for consensus module
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid proof of work: hash {hash} does not meet target {target}")]
    InvalidProofOfWork { hash: String, target: String },

    #[error("Invalid difficulty target")]
    InvalidDifficultyTarget,

    #[error("Mining operation was stopped")]
    MiningStopped,

    #[error("Invalid block timestamp: {reason}")]
    InvalidTimestamp { reason: String },

    #[error("Difficulty adjustment error: {0}")]
    DifficultyAdjustment(String),

    #[error("Block difficulty {target} is too easy (max allowed: {max_allowed})")]
    DifficultyTooEasy { target: u32, max_allowed: u32 },

    #[error("Block difficulty {target} is too hard (min allowed: {min_allowed})")]
    DifficultyTooHard { target: u32, min_allowed: u32 },

    #[error("Invalid difficulty encoding for target {target}: {reason}")]
    InvalidDifficultyEncoding { target: u32, reason: String },

    #[error("Block timestamp {current} not increasing from previous {previous}")]
    TimestampNotIncreasing { current: u64, previous: u64 },

    #[error("Block timestamp {block_time} is too far in future (current: {current_time}, max_drift: {max_drift}s)")]
    TimestampTooFarInFuture {
        block_time: u64,
        current_time: u64,
        max_drift: u64,
    },
}
