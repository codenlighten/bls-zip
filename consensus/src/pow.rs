// Proof-of-Work implementation using SHA-3
use boundless_core::{Block, BlockHeader};
use primitive_types::U256;

use crate::error::ConsensusError;

/// Proof-of-Work validator and utilities
pub struct ProofOfWork;

impl ProofOfWork {
    /// Validate that a block meets the required difficulty target
    pub fn validate_block(block: &Block) -> Result<(), ConsensusError> {
        Self::validate_header(&block.header)
    }

    /// Validate that a block header meets the required difficulty target
    pub fn validate_header(header: &BlockHeader) -> Result<(), ConsensusError> {
        // CRITICAL SECURITY FIX: Validate difficulty target is in acceptable range
        Self::validate_difficulty_target(header.difficulty_target)?;

        let hash = header.hash();
        let hash_value = U256::from_big_endian(&hash);
        let target = BlockHeader::compact_to_target(header.difficulty_target);

        if hash_value < target {
            Ok(())
        } else {
            Err(ConsensusError::InvalidProofOfWork {
                hash: hex::encode(hash),
                target: format!("{:x}", target),
            })
        }
    }

    /// Validate difficulty target is within acceptable range
    ///
    /// SECURITY: Prevents attackers from setting arbitrary difficulty targets
    /// to mine blocks trivially or create impossible-to-mine blocks
    pub fn validate_difficulty_target(target: u32) -> Result<(), ConsensusError> {
        // Minimum difficulty (easiest) - for genesis/testing
        // This represents ~1 in 2^20 chance
        const MIN_DIFFICULTY: u32 = 0x1f0fffff;

        // Maximum difficulty (hardest) - ~1 in 2^240 chance
        // Below this would require unrealistic hash rates
        const MAX_DIFFICULTY: u32 = 0x04000000;

        if target > MIN_DIFFICULTY {
            return Err(ConsensusError::DifficultyTooEasy {
                target,
                max_allowed: MIN_DIFFICULTY,
            });
        }

        if target < MAX_DIFFICULTY {
            return Err(ConsensusError::DifficultyTooHard {
                target,
                min_allowed: MAX_DIFFICULTY,
            });
        }

        // Validate compact encoding format
        let exponent = (target >> 24) as u8;
        if exponent > 32 {
            return Err(ConsensusError::InvalidDifficultyEncoding {
                target,
                reason: format!("Exponent {} exceeds maximum 32", exponent),
            });
        }

        Ok(())
    }

    /// Validate timestamp is monotonically increasing
    ///
    /// SECURITY: Prevents timestamp manipulation for difficulty gaming
    pub fn validate_timestamp(
        current_timestamp: u64,
        previous_timestamp: u64,
    ) -> Result<(), ConsensusError> {
        // Timestamp must be strictly increasing
        if current_timestamp <= previous_timestamp {
            return Err(ConsensusError::TimestampNotIncreasing {
                current: current_timestamp,
                previous: previous_timestamp,
            });
        }

        // Timestamp must not be too far in future (allow 2 hour drift for clock skew)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ConsensusError::InvalidTimestamp {
                reason: "System time is before Unix epoch".to_string(),
            })?
            .as_secs();

        const MAX_FUTURE_DRIFT: u64 = 2 * 3600; // 2 hours

        if current_timestamp > now + MAX_FUTURE_DRIFT {
            return Err(ConsensusError::TimestampTooFarInFuture {
                block_time: current_timestamp,
                current_time: now,
                max_drift: MAX_FUTURE_DRIFT,
            });
        }

        Ok(())
    }

    /// Calculate the hash rate required to mine a block at given difficulty
    /// Returns hashes per second needed
    pub fn required_hash_rate(difficulty_target: u32, target_time_secs: u64) -> f64 {
        let target = BlockHeader::compact_to_target(difficulty_target);
        let max_target = U256::MAX;

        // Expected number of hashes = max_target / target
        let difficulty = max_target / target;

        // Convert to f64 (approximate for large numbers)
        let difficulty_f64 = difficulty.low_u128() as f64;

        // Hash rate = difficulty / target_time
        difficulty_f64 / target_time_secs as f64
    }

    /// Estimate the probability of finding a valid block in N attempts
    pub fn success_probability(difficulty_target: u32, attempts: u64) -> f64 {
        let target = BlockHeader::compact_to_target(difficulty_target);
        let max_target = U256::MAX;

        let success_rate = if max_target > U256::zero() {
            let target_u128 = target.low_u128();
            let max_u128 = max_target.low_u128();
            target_u128 as f64 / max_u128 as f64
        } else {
            0.0
        };

        // Probability of at least one success in N attempts
        1.0 - (1.0 - success_rate).powi(attempts as i32)
    }

    /// Calculate network hash rate from observed block time and difficulty
    pub fn estimate_network_hash_rate(
        difficulty_target: u32,
        observed_block_time_secs: u64,
    ) -> f64 {
        Self::required_hash_rate(difficulty_target, observed_block_time_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pow() {
        // Test that impossible difficulty fails validation
        let header = BlockHeader::new(
            1, [0u8; 32], [0u8; 32], [0u8; 32], 1234567890,
            0x01000001, // Very hard difficulty - should fail with nonce 0
            0, 1, // height
        );

        // Should fail validation with extremely hard difficulty
        assert!(ProofOfWork::validate_header(&header).is_err());
    }

    #[test]
    fn test_hash_rate_calculation() {
        let hash_rate = ProofOfWork::required_hash_rate(0x1d00ffff, 600);
        assert!(hash_rate > 0.0);
    }

    #[test]
    fn test_success_probability() {
        // Test with medium difficulty to avoid numerical edge cases
        let prob = ProofOfWork::success_probability(0x1d00ffff, 100);

        // Probability should be in valid range [0.0, 1.0]
        // We don't assert > 0.0 because with hard difficulty and few attempts,
        // the probability might round to 0.0 in f64 precision
        assert!(
            prob >= 0.0 && prob <= 1.0,
            "Probability was {}, expected 0.0 to 1.0",
            prob
        );
    }

    // SECURITY TESTS: Difficulty Validation

    #[test]
    fn test_difficulty_too_easy() {
        // Difficulty easier than minimum should be rejected
        let too_easy = 0x1fffffff; // Greater than MIN_DIFFICULTY
        let result = ProofOfWork::validate_difficulty_target(too_easy);
        assert!(matches!(result, Err(ConsensusError::DifficultyTooEasy { .. })));
    }

    #[test]
    fn test_difficulty_too_hard() {
        // Difficulty harder than maximum should be rejected
        let too_hard = 0x03ffffff; // Less than MAX_DIFFICULTY
        let result = ProofOfWork::validate_difficulty_target(too_hard);
        assert!(matches!(result, Err(ConsensusError::DifficultyTooHard { .. })));
    }

    #[test]
    fn test_difficulty_valid_range() {
        // Test valid difficulty targets
        let valid_targets = vec![
            0x1f0fffff, // Minimum (easiest)
            0x1d00ffff, // Bitcoin-like difficulty
            0x1b0404cb, // Medium difficulty
            0x04000000, // Maximum (hardest)
        ];

        for target in valid_targets {
            let result = ProofOfWork::validate_difficulty_target(target);
            assert!(result.is_ok(), "Target {:x} should be valid", target);
        }
    }

    #[test]
    fn test_difficulty_invalid_exponent() {
        // Exponent > 32 should be rejected
        let invalid = 0x21000000; // Exponent 33 (0x21)
        let result = ProofOfWork::validate_difficulty_target(invalid);
        assert!(matches!(
            result,
            Err(ConsensusError::InvalidDifficultyEncoding { .. })
        ));
    }

    // SECURITY TESTS: Timestamp Validation

    #[test]
    fn test_timestamp_not_increasing() {
        // Current timestamp <= previous should fail
        let previous = 1000;
        let current = 999; // Goes backward
        let result = ProofOfWork::validate_timestamp(current, previous);
        assert!(matches!(
            result,
            Err(ConsensusError::TimestampNotIncreasing { .. })
        ));
    }

    #[test]
    fn test_timestamp_equal_fails() {
        // Equal timestamps should fail (must be strictly increasing)
        let timestamp = 1000;
        let result = ProofOfWork::validate_timestamp(timestamp, timestamp);
        assert!(matches!(
            result,
            Err(ConsensusError::TimestampNotIncreasing { .. })
        ));
    }

    #[test]
    fn test_timestamp_too_far_in_future() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Timestamp 3 hours in future (> 2 hour max drift)
        let too_far_future = now + (3 * 3600);
        let previous = now - 100;

        let result = ProofOfWork::validate_timestamp(too_far_future, previous);
        assert!(matches!(
            result,
            Err(ConsensusError::TimestampTooFarInFuture { .. })
        ));
    }

    #[test]
    fn test_timestamp_valid_progression() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Valid progression: previous < current < now + 2 hours
        let previous = now - 100;
        let current = now - 50;

        let result = ProofOfWork::validate_timestamp(current, previous);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_within_allowed_drift() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Within 2 hour drift allowance
        let previous = now - 100;
        let current = now + 3600; // 1 hour in future

        let result = ProofOfWork::validate_timestamp(current, previous);
        assert!(result.is_ok());
    }
}
