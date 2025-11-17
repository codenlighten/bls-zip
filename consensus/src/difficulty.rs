// Difficulty adjustment algorithm (Bitcoin-style epoch-based)
use boundless_core::BlockHeader;
use primitive_types::U256;

use crate::{DIFFICULTY_ADJUSTMENT_INTERVAL, MAX_ADJUSTMENT_FACTOR, TARGET_BLOCK_TIME};

/// Difficulty adjustment calculator
pub struct DifficultyAdjustment;

impl DifficultyAdjustment {
    /// Calculate the new difficulty target for the next epoch
    ///
    /// # Arguments
    /// * `current_target` - The current difficulty target (compact format)
    /// * `actual_time_secs` - Actual time taken for the last epoch (in seconds)
    /// * `expected_time_secs` - Expected time for the epoch (interval * target_block_time)
    ///
    /// # Returns
    /// New difficulty target (compact format)
    pub fn adjust_difficulty(
        current_target: u32,
        actual_time_secs: u64,
        expected_time_secs: u64,
    ) -> u32 {
        // Convert compact to full target
        let current_target_full = BlockHeader::compact_to_target(current_target);

        // Calculate adjustment factor, clamped to MAX_ADJUSTMENT_FACTOR
        let min_time = expected_time_secs / MAX_ADJUSTMENT_FACTOR;
        let max_time = expected_time_secs * MAX_ADJUSTMENT_FACTOR;

        let clamped_time = actual_time_secs.clamp(min_time, max_time);

        // new_target = current_target * (actual_time / expected_time)
        // Higher time = easier difficulty (higher target)
        // Lower time = harder difficulty (lower target)

        let new_target = if clamped_time > expected_time_secs {
            // Blocks too slow, make easier (increase target)
            let factor =
                U256::from(clamped_time) * U256::from(100) / U256::from(expected_time_secs);
            current_target_full * factor / U256::from(100)
        } else {
            // Blocks too fast, make harder (decrease target)
            let factor =
                U256::from(expected_time_secs) * U256::from(100) / U256::from(clamped_time);
            current_target_full * U256::from(100) / factor
        };

        // Ensure we don't exceed max target
        let max_target = U256::MAX >> 32;
        let final_target = if new_target > max_target {
            max_target
        } else {
            new_target
        };

        // Convert back to compact format
        BlockHeader::target_to_compact(final_target)
    }

    /// Calculate expected time for an epoch
    pub fn expected_epoch_time() -> u64 {
        DIFFICULTY_ADJUSTMENT_INTERVAL * TARGET_BLOCK_TIME
    }

    /// Check if difficulty adjustment should occur at this block height
    pub fn should_adjust(block_height: u64) -> bool {
        block_height % DIFFICULTY_ADJUSTMENT_INTERVAL == 0 && block_height > 0
    }

    /// Estimate the difficulty multiplier from compact representation
    /// (relative to maximum difficulty)
    pub fn difficulty_multiplier(compact_target: u32) -> f64 {
        let target = BlockHeader::compact_to_target(compact_target);
        let max_target = U256::MAX >> 32;

        if target == U256::zero() {
            return f64::INFINITY;
        }

        let max_u128 = max_target.low_u128();
        let target_u128 = target.low_u128();

        max_u128 as f64 / target_u128 as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_adjust() {
        assert!(!DifficultyAdjustment::should_adjust(0));
        assert!(!DifficultyAdjustment::should_adjust(100));
        assert!(DifficultyAdjustment::should_adjust(1008));
        assert!(DifficultyAdjustment::should_adjust(2016));
        assert!(!DifficultyAdjustment::should_adjust(2017));
    }

    #[test]
    fn test_difficulty_adjustment_increase() {
        let current = 0x1d00ffff;
        let expected = DifficultyAdjustment::expected_epoch_time();

        // Blocks came too fast (half the expected time) -> increase difficulty
        let actual = expected / 2;
        let new_target = DifficultyAdjustment::adjust_difficulty(current, actual, expected);

        // New target should be lower (harder difficulty)
        let current_full = BlockHeader::compact_to_target(current);
        let new_full = BlockHeader::compact_to_target(new_target);

        assert!(new_full < current_full);
    }

    #[test]
    fn test_difficulty_adjustment_decrease() {
        let current = 0x1d00ffff;
        let expected = DifficultyAdjustment::expected_epoch_time();

        // Blocks came too slow (double the expected time) -> decrease difficulty
        let actual = expected * 2;
        let new_target = DifficultyAdjustment::adjust_difficulty(current, actual, expected);

        // New target should be higher (easier difficulty)
        let current_full = BlockHeader::compact_to_target(current);
        let new_full = BlockHeader::compact_to_target(new_target);

        assert!(new_full > current_full);
    }

    #[test]
    fn test_difficulty_adjustment_clamping() {
        let current = 0x1d00ffff;
        let expected = DifficultyAdjustment::expected_epoch_time();

        // Try to adjust by 10x (should be clamped to 4x)
        let actual = expected * 10;
        let new_target = DifficultyAdjustment::adjust_difficulty(current, actual, expected);

        let current_full = BlockHeader::compact_to_target(current);
        let new_full = BlockHeader::compact_to_target(new_target);

        // Should be at most 4x easier
        assert!(new_full <= current_full * U256::from(MAX_ADJUSTMENT_FACTOR));
    }

    #[test]
    fn test_expected_epoch_time() {
        let expected = DifficultyAdjustment::expected_epoch_time();
        assert_eq!(expected, 1008 * 300); // 302,400 seconds (~3.5 days)
    }
}
