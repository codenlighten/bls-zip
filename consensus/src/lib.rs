// Boundless BLS Consensus - SHA-3 Proof-of-Work Implementation
//
// This module implements the Proof-of-Work consensus mechanism using SHA-3/SHAKE256
// for cryptographic diversity and ASIC resistance.

pub mod difficulty;
pub mod error;
pub mod miner;
pub mod pow;

pub use difficulty::DifficultyAdjustment;
pub use error::ConsensusError;
pub use miner::{Miner, MiningResult};
pub use pow::ProofOfWork;

/// Target block time in seconds (5 minutes)
pub const TARGET_BLOCK_TIME: u64 = 300;

/// Difficulty adjustment interval (every ~3.5 days)
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 1008;

/// Maximum adjustment factor (4x increase or 0.25x decrease)
pub const MAX_ADJUSTMENT_FACTOR: u64 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TARGET_BLOCK_TIME, 300);
        assert_eq!(DIFFICULTY_ADJUSTMENT_INTERVAL, 1008);
        assert_eq!(MAX_ADJUSTMENT_FACTOR, 4);
    }
}
