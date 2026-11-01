use blackchain_crypto::Hash;

pub const TARGET_BLOCK_TIME_SECS: u64 = 6;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 20; // Re-target every 20 blocks
pub const EXPECTED_TIMESPAN_SECS: u64 = TARGET_BLOCK_TIME_SECS * DIFFICULTY_ADJUSTMENT_INTERVAL;

/// Check if a PoUW ticket meets the difficulty target (ticket <= target in big-endian order).
pub fn meets_difficulty(ticket: &Hash, target: &Hash) -> bool {
    ticket.as_bytes() <= target.as_bytes()
}

/// Calculate the adjusted difficulty target based on elapsed time over the interval.
pub fn calculate_next_target(
    current_target: &Hash,
    actual_timespan_secs: u64,
) -> Hash {
    // Dampen adjustments to prevent wild swings (max 4x harder, min 0.25x easier)
    let clamped_timespan = actual_timespan_secs
        .max(EXPECTED_TIMESPAN_SECS / 4)
        .min(EXPECTED_TIMESPAN_SECS * 4);

    let target_bytes = current_target.as_bytes();

    // Multiply 256-bit target by (clamped_timespan / EXPECTED_TIMESPAN_SECS)
    let high_val = u128::from_be_bytes(target_bytes[0..16].try_into().unwrap());
    let low_val = u128::from_be_bytes(target_bytes[16..32].try_into().unwrap());

    // Scale high and low parts
    let scaled_high = (high_val.saturating_mul(clamped_timespan as u128)) / (EXPECTED_TIMESPAN_SECS as u128);
    let scaled_low = (low_val.saturating_mul(clamped_timespan as u128)) / (EXPECTED_TIMESPAN_SECS as u128);

    let mut next_bytes = [0u8; 32];
    next_bytes[0..16].copy_from_slice(&scaled_high.to_be_bytes());
    next_bytes[16..32].copy_from_slice(&scaled_low.to_be_bytes());

    Hash(next_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meets_difficulty() {
        let hard_target = Hash([0x00, 0x00, 0x0f, 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let winning_ticket = Hash([0x00, 0x00, 0x0a, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let losing_ticket = Hash([0x00, 0x00, 0xff, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        assert!(meets_difficulty(&winning_ticket, &hard_target));
        assert!(!meets_difficulty(&losing_ticket, &hard_target));
    }
}
