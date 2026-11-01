use crate::difficulty::meets_difficulty;
use blackchain_core::{Block, BlockHeader, State, MINIMUM_MINING_STAKE};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Block height mismatch: expected {expected}, got {got}")]
    InvalidHeight { expected: u64, got: u64 },
    #[error("Previous block hash mismatch")]
    PrevHashMismatch,
    #[error("Block timestamp is earlier than previous block")]
    TimestampDrift,
    #[error("PoUW ticket does not satisfy target difficulty")]
    DifficultyNotMet,
    #[error("PoUW ticket was not derived from any valid receipt in the block")]
    InvalidPoUWTicket,
    #[error("Transactions Merkle root mismatch")]
    InvalidTxsRoot,
    #[error("Inference receipts Merkle root mismatch")]
    InvalidReceiptsRoot,
    #[error("Miner {0} does not hold the minimum required staking bond")]
    InsufficientMinerStake(blackchain_core::Address),
}

/// Validate a candidate block against the previous block header and the current ledger state.
pub fn validate_block(
    block: &Block,
    prev_header: &BlockHeader,
    state: &State,
) -> Result<(), ConsensusError> {
    let header = &block.header;

    // 1. Verify block height
    if header.height != prev_header.height + 1 {
        return Err(ConsensusError::InvalidHeight {
            expected: prev_header.height + 1,
            got: header.height,
        });
    }

    // 2. Verify previous block hash linkage
    if header.prev_block_hash != prev_header.hash() {
        return Err(ConsensusError::PrevHashMismatch);
    }

    // 3. Verify monotonic timestamp progression
    if header.timestamp < prev_header.timestamp {
        return Err(ConsensusError::TimestampDrift);
    }

    // 4. Verify miner staking requirement
    let miner_account = state.get_account(&header.miner);
    if miner_account.staked < MINIMUM_MINING_STAKE {
        return Err(ConsensusError::InsufficientMinerStake(header.miner));
    }

    // 5. Verify PoUW Ticket satisfies difficulty
    if !meets_difficulty(&header.pouw_ticket, &header.difficulty_target) {
        return Err(ConsensusError::DifficultyNotMet);
    }

    // 6. Verify Merkle roots match the block body
    if header.txs_root != block.body.calculate_txs_root() {
        return Err(ConsensusError::InvalidTxsRoot);
    }

    if header.receipts_root != block.body.calculate_receipts_root() {
        return Err(ConsensusError::InvalidReceiptsRoot);
    }

    Ok(())
}
