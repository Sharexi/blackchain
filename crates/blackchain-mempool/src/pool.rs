use std::collections::HashMap;
use blackchain_core::{InferenceRequestTx, ModelRegistry, Transaction};
use blackchain_crypto::Hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MempoolError {
    #[error("Transaction already present in mempool")]
    DuplicateTransaction,
    #[error("Inference request already present in mempool")]
    DuplicateInferenceRequest,
    #[error("Mempool capacity exceeded")]
    PoolFull,
}

pub const MAX_MEMPOOL_CAPACITY: usize = 10_000;

/// Dual-queue mempool managing both standard financial transactions and high-priority AI inference requests.
#[derive(Debug, Default)]
pub struct Mempool {
    financial_txs: HashMap<Hash, Transaction>,
    inference_requests: HashMap<Hash, InferenceRequestTx>,
}

impl Mempool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a standard transaction (Transfer, Stake, Unstake, FraudProof)
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), MempoolError> {
        if self.financial_txs.len() >= MAX_MEMPOOL_CAPACITY {
            return Err(MempoolError::PoolFull);
        }
        let h = tx.hash();
        if self.financial_txs.contains_key(&h) {
            return Err(MempoolError::DuplicateTransaction);
        }
        self.financial_txs.insert(h, tx);
        Ok(())
    }

    /// Add an AI agent inference request
    pub fn add_inference_request(&mut self, req: InferenceRequestTx) -> Result<(), MempoolError> {
        if self.inference_requests.len() >= MAX_MEMPOOL_CAPACITY {
            return Err(MempoolError::PoolFull);
        }
        let req_id = req.request_id();
        if self.inference_requests.contains_key(&req_id) {
            return Err(MempoolError::DuplicateInferenceRequest);
        }
        self.inference_requests.insert(req_id, req);
        Ok(())
    }

    /// Pop a batch of inference requests prioritized by `priority_tip_per_cu` (descending)
    pub fn pop_inference_batch(
        &mut self,
        max_compute_units: u64,
        registry: &ModelRegistry,
    ) -> Vec<InferenceRequestTx> {
        let mut candidates: Vec<InferenceRequestTx> = self.inference_requests.values().cloned().collect();

        // Sort descending by priority tip
        candidates.sort_by(|a, b| b.priority_tip_per_cu.cmp(&a.priority_tip_per_cu));

        let mut selected = Vec::new();
        let mut total_cu = 0u64;

        for req in candidates {
            if let Some(model) = registry.get(&req.model_id) {
                let estimated_cu = model.calculate_compute_units(0, req.max_tokens);
                if total_cu + estimated_cu <= max_compute_units {
                    total_cu += estimated_cu;
                    self.inference_requests.remove(&req.request_id());
                    selected.push(req);
                }
            }
        }

        selected
    }

    /// Pop standard transactions up to a given limit
    pub fn pop_transactions(&mut self, limit: usize) -> Vec<Transaction> {
        let keys: Vec<Hash> = self.financial_txs.keys().take(limit).copied().collect();
        let mut txs = Vec::with_capacity(keys.len());
        for k in keys {
            if let Some(tx) = self.financial_txs.remove(&k) {
                txs.push(tx);
            }
        }
        txs
    }

    pub fn len_transactions(&self) -> usize {
        self.financial_txs.len()
    }

    pub fn len_inference(&self) -> usize {
        self.inference_requests.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blackchain_core::Address;
    use blackchain_crypto::Signature;

    #[test]
    fn test_inference_request_priority_sorting() {
        let mut pool = Mempool::new();
        let registry = ModelRegistry::default();

        let req_low = InferenceRequestTx {
            client: Address::ZERO,
            model_id: "llama-3-8b-instruct".to_string(),
            prompt_hash: Hash::ZERO,
            max_tokens: 100,
            max_fee_per_cu: 5000,
            priority_tip_per_cu: 10,
            deadline_block: 100,
            nonce: 0,
            signature: Signature::ZERO,
        };

        let req_high = InferenceRequestTx {
            client: Address::ZERO,
            model_id: "llama-3-8b-instruct".to_string(),
            prompt_hash: Hash::ZERO,
            max_tokens: 100,
            max_fee_per_cu: 5000,
            priority_tip_per_cu: 100,
            deadline_block: 100,
            nonce: 1,
            signature: Signature::ZERO,
        };

        pool.add_inference_request(req_low).unwrap();
        pool.add_inference_request(req_high.clone()).unwrap();

        let batch = pool.pop_inference_batch(100_000, &registry);
        assert_eq!(batch.len(), 2);
        // High priority must come first
        assert_eq!(batch[0].priority_tip_per_cu, 100);
        assert_eq!(batch[1].priority_tip_per_cu, 10);
    }
}
