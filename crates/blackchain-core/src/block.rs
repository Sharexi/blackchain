use crate::pricing::MIN_BASE_FEE_PER_CU;
use crate::transaction::{InferenceReceiptTx, Transaction};
use crate::types::{Address, Amount, ComputeUnits};
use blackchain_crypto::{calculate_merkle_root, hash, Hash};
use serde::{Deserialize, Serialize};

/// The header of a Blackchain block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub prev_block_hash: Hash,
    pub timestamp: u64,
    pub state_root: Hash,
    pub txs_root: Hash,
    pub receipts_root: Hash,
    pub miner: Address,
    pub difficulty_target: Hash,
    pub nonce: u64,
    /// Cryptographic lottery ticket derived from serving inference
    pub pouw_ticket: Hash,
    pub base_fee_per_cu: Amount,
    pub compute_units_used: ComputeUnits,
}

impl BlockHeader {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.height.to_be_bytes());
        bytes.extend_from_slice(self.prev_block_hash.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(self.state_root.as_bytes());
        bytes.extend_from_slice(self.txs_root.as_bytes());
        bytes.extend_from_slice(self.receipts_root.as_bytes());
        bytes.extend_from_slice(self.miner.as_bytes());
        bytes.extend_from_slice(self.difficulty_target.as_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes.extend_from_slice(self.pouw_ticket.as_bytes());
        bytes.extend_from_slice(&self.base_fee_per_cu.to_be_bytes());
        bytes.extend_from_slice(&self.compute_units_used.to_be_bytes());
        bytes
    }

    pub fn hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// The body of a block containing standard transactions and verified inference receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BlockBody {
    pub transactions: Vec<Transaction>,
    pub inference_receipts: Vec<InferenceReceiptTx>,
}

impl BlockBody {
    pub fn new(transactions: Vec<Transaction>, inference_receipts: Vec<InferenceReceiptTx>) -> Self {
        Self {
            transactions,
            inference_receipts,
        }
    }

    pub fn calculate_txs_root(&self) -> Hash {
        let hashes: Vec<Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        calculate_merkle_root(&hashes)
    }

    pub fn calculate_receipts_root(&self) -> Hash {
        let hashes: Vec<Hash> = self.inference_receipts.iter().map(|r| r.receipt_hash()).collect();
        calculate_merkle_root(&hashes)
    }
}

/// A full Blackchain block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl Block {
    pub fn new(header: BlockHeader, body: BlockBody) -> Self {
        Self { header, body }
    }

    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Construct the canonical Genesis Block for Blackchain.
    pub fn genesis() -> Self {
        let header = BlockHeader {
            height: 0,
            prev_block_hash: Hash::ZERO,
            timestamp: 1730000000,
            state_root: Hash::ZERO,
            txs_root: Hash::ZERO,
            receipts_root: Hash::ZERO,
            miner: Address::ZERO,
            // Initial generous difficulty target: 0x00ffff...
            difficulty_target: Hash([
                0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]),
            nonce: 0,
            pouw_ticket: Hash::ZERO,
            base_fee_per_cu: MIN_BASE_FEE_PER_CU,
            compute_units_used: 0,
        };

        Self {
            header,
            body: BlockBody::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.prev_block_hash, Hash::ZERO);
        assert_ne!(genesis.hash(), Hash::ZERO);
    }
}
