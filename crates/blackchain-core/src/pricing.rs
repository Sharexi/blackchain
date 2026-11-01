use crate::types::{Amount, ComputeUnits};
use serde::{Deserialize, Serialize};

/// Constants for Compute EIP-1559 pricing.
pub const TARGET_COMPUTE_UNITS_PER_BLOCK: ComputeUnits = 1_000_000;
pub const MAX_COMPUTE_UNITS_PER_BLOCK: ComputeUnits = 2_000_000;
pub const MIN_BASE_FEE_PER_CU: Amount = 1_000; // Smallest unit per CU
pub const BASE_FEE_MAX_CHANGE_DENOMINATOR: u128 = 8; // Max 12.5% adjustment per block

/// Dynamic fee market state for AI compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicFeeMarket {
    /// Current base fee per compute unit in minimal token units
    pub base_fee_per_cu: Amount,
}

impl Default for DynamicFeeMarket {
    fn default() -> Self {
        Self {
            base_fee_per_cu: MIN_BASE_FEE_PER_CU,
        }
    }
}

impl DynamicFeeMarket {
    pub fn new(initial_base_fee: Amount) -> Self {
        Self {
            base_fee_per_cu: initial_base_fee.max(MIN_BASE_FEE_PER_CU),
        }
    }

    /// Calculate the next block's base fee based on the compute utilization of the current block.
    pub fn calculate_next_base_fee(&self, gas_used: ComputeUnits) -> Amount {
        if gas_used == TARGET_COMPUTE_UNITS_PER_BLOCK {
            return self.base_fee_per_cu;
        }

        if gas_used > TARGET_COMPUTE_UNITS_PER_BLOCK {
            let excess = gas_used - TARGET_COMPUTE_UNITS_PER_BLOCK;
            let delta = (self.base_fee_per_cu * (excess as u128))
                / (TARGET_COMPUTE_UNITS_PER_BLOCK as u128)
                / BASE_FEE_MAX_CHANGE_DENOMINATOR;
            let delta = delta.max(1); // Ensure minimum progression if congested
            self.base_fee_per_cu + delta
        } else {
            let deficit = TARGET_COMPUTE_UNITS_PER_BLOCK - gas_used;
            let delta = (self.base_fee_per_cu * (deficit as u128))
                / (TARGET_COMPUTE_UNITS_PER_BLOCK as u128)
                / BASE_FEE_MAX_CHANGE_DENOMINATOR;
            if self.base_fee_per_cu > MIN_BASE_FEE_PER_CU + delta {
                self.base_fee_per_cu - delta
            } else {
                MIN_BASE_FEE_PER_CU
            }
        }
    }

    /// Calculate total cost for an inference request with priority tip.
    pub fn calculate_total_cost(
        &self,
        compute_units: ComputeUnits,
        priority_tip_per_cu: Amount,
    ) -> Amount {
        let total_per_cu = self.base_fee_per_cu + priority_tip_per_cu;
        total_per_cu * (compute_units as u128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_fee_increase_on_congestion() {
        let market = DynamicFeeMarket::new(10_000);
        // Full block (2,000,000 CU) -> 12.5% increase
        let next_fee = market.calculate_next_base_fee(MAX_COMPUTE_UNITS_PER_BLOCK);
        assert_eq!(next_fee, 10_000 + (10_000 / 8));
    }

    #[test]
    fn test_base_fee_decrease_on_empty_block() {
        let market = DynamicFeeMarket::new(10_000);
        // Empty block (0 CU) -> 12.5% decrease
        let next_fee = market.calculate_next_base_fee(0);
        assert_eq!(next_fee, 10_000 - (10_000 / 8));
    }

    #[test]
    fn test_minimum_base_fee_floor() {
        let market = DynamicFeeMarket::new(MIN_BASE_FEE_PER_CU);
        let next_fee = market.calculate_next_base_fee(0);
        assert_eq!(next_fee, MIN_BASE_FEE_PER_CU);
    }
}
