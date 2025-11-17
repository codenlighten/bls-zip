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

    #[error("Invalid block timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("Difficulty adjustment error: {0}")]
    DifficultyAdjustment(String),
}
