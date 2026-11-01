use crate::types::{Address, Amount, Nonce};
use blackchain_crypto::{hash, Hash, Signature};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Invalid transaction signature")]
    InvalidSignature,
    #[error("Nonce mismatch (expected {expected}, got {got})")]
    NonceMismatch { expected: Nonce, got: Nonce },
    #[error("Insufficient balance for transaction")]
    InsufficientBalance,
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),
}

/// Standard peer-to-peer token transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferTx {
    pub from: Address,
    pub to: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub nonce: Nonce,
    pub signature: Signature,
}

impl TransferTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.from.as_bytes());
        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes.extend_from_slice(&self.fee.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes
    }

    pub fn tx_hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// Node staking deposit to become eligible for PoUW block production.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeTx {
    pub staker: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub nonce: Nonce,
    pub signature: Signature,
}

impl StakeTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.staker.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes.extend_from_slice(&self.fee.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes
    }

    pub fn tx_hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// Node unstaking request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnstakeTx {
    pub staker: Address,
    pub amount: Amount,
    pub fee: Amount,
    pub nonce: Nonce,
    pub signature: Signature,
}

impl UnstakeTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.staker.as_bytes());
        bytes.extend_from_slice(&self.amount.to_be_bytes());
        bytes.extend_from_slice(&self.fee.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes
    }

    pub fn tx_hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// AI Agent inference request submitted to the mempool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceRequestTx {
    pub client: Address,
    pub model_id: String,
    pub prompt_hash: Hash,
    pub max_tokens: u32,
    pub max_fee_per_cu: Amount,
    pub priority_tip_per_cu: Amount,
    pub deadline_block: u64,
    pub nonce: Nonce,
    pub signature: Signature,
}

impl InferenceRequestTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.client.as_bytes());
        bytes.extend_from_slice(self.model_id.as_bytes());
        bytes.extend_from_slice(self.prompt_hash.as_bytes());
        bytes.extend_from_slice(&self.max_tokens.to_be_bytes());
        bytes.extend_from_slice(&self.max_fee_per_cu.to_be_bytes());
        bytes.extend_from_slice(&self.priority_tip_per_cu.to_be_bytes());
        bytes.extend_from_slice(&self.deadline_block.to_be_bytes());
        bytes.extend_from_slice(&self.nonce.to_be_bytes());
        bytes
    }

    pub fn request_id(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// Compute receipt produced by a GPU worker upon completing an inference job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InferenceReceiptTx {
    pub request_id: Hash,
    pub miner: Address,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub output_hash: Hash,
    pub miner_signature: Signature,
}

impl InferenceReceiptTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.request_id.as_bytes());
        bytes.extend_from_slice(self.miner.as_bytes());
        bytes.extend_from_slice(&self.prompt_tokens.to_be_bytes());
        bytes.extend_from_slice(&self.completion_tokens.to_be_bytes());
        bytes.extend_from_slice(self.output_hash.as_bytes());
        bytes
    }

    /// The unique receipt hash used in the PoUW lottery calculation
    pub fn receipt_hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// Optimistic fraud proof submitted by a Fisherman to slash a dishonest miner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FraudProofTx {
    pub block_height: u64,
    pub receipt_index: u32,
    pub challenger: Address,
    pub expected_output_hash: Hash,
    pub signature: Signature,
}

impl FraudProofTx {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.block_height.to_be_bytes());
        bytes.extend_from_slice(&self.receipt_index.to_be_bytes());
        bytes.extend_from_slice(self.challenger.as_bytes());
        bytes.extend_from_slice(self.expected_output_hash.as_bytes());
        bytes
    }

    pub fn tx_hash(&self) -> Hash {
        hash(&self.signing_bytes())
    }
}

/// Unified transaction enum for the ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transaction {
    Transfer(TransferTx),
    Stake(StakeTx),
    Unstake(UnstakeTx),
    InferenceRequest(InferenceRequestTx),
    InferenceReceipt(InferenceReceiptTx),
    FraudProof(FraudProofTx),
}

impl Transaction {
    pub fn hash(&self) -> Hash {
        match self {
            Transaction::Transfer(tx) => tx.tx_hash(),
            Transaction::Stake(tx) => tx.tx_hash(),
            Transaction::Unstake(tx) => tx.tx_hash(),
            Transaction::InferenceRequest(tx) => tx.request_id(),
            Transaction::InferenceReceipt(tx) => tx.receipt_hash(),
            Transaction::FraudProof(tx) => tx.tx_hash(),
        }
    }
}
