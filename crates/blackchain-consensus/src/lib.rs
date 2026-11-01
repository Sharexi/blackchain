pub mod difficulty;
pub mod ticket;
pub mod validation;

pub use difficulty::{
    calculate_next_target, meets_difficulty, DIFFICULTY_ADJUSTMENT_INTERVAL,
    EXPECTED_TIMESPAN_SECS, TARGET_BLOCK_TIME_SECS,
};
pub use ticket::{compute_pouw_ticket, generate_synthetic_prompt};
pub use validation::{validate_block, ConsensusError};
