use std::sync::Arc;
use tokio::sync::Mutex;
use blackchain_compute::{InferenceBackend, InferenceTask};
use blackchain_consensus::{compute_pouw_ticket, generate_synthetic_prompt, meets_difficulty};
use blackchain_core::{
    Address, Block, BlockBody, BlockHeader, DynamicFeeMarket, InferenceReceiptTx,
    ModelRegistry,
};
use blackchain_crypto::{hash, Hash, Keypair};
use blackchain_mempool::Mempool;
use blackchain_storage::BlockchainStore;

pub struct Miner {
    pub keypair: Keypair,
    pub address: Address,
    pub registry: ModelRegistry,
    pub compute_engine: Arc<dyn InferenceBackend>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub store: Arc<dyn BlockchainStore>,
}

impl Miner {
    pub fn new(
        keypair: Keypair,
        registry: ModelRegistry,
        compute_engine: Arc<dyn InferenceBackend>,
        mempool: Arc<Mutex<Mempool>>,
        store: Arc<dyn BlockchainStore>,
    ) -> Self {
        let address = Address::from_public_key(&keypair.public_key());
        Self {
            keypair,
            address,
            registry,
            compute_engine,
            mempool,
            store,
        }
    }

    /// Execute one mining attempt either from mempool inference requests or synthetic benchmarks.
    pub async fn mine_next_block(&self) -> Option<Block> {
        let latest_block = self.store.get_latest_block();
        let prev_hash = latest_block.hash();
        let next_height = latest_block.header.height + 1;
        let target = latest_block.header.difficulty_target;

        // 1. Fetch pending inference tasks from mempool
        let mut mempool_guard = self.mempool.lock().await;
        let pending_inference = mempool_guard.pop_inference_batch(1_000_000, &self.registry);
        let pending_txs = mempool_guard.pop_transactions(50);
        drop(mempool_guard);

        let mut receipts = Vec::new();
        let mut total_cu_used = 0u64;

        if !pending_inference.is_empty() {
            // Process real AI agent inference requests through the compute engine
            for req in &pending_inference {
                let task = InferenceTask {
                    model_id: req.model_id.clone(),
                    prompt: format!("Prompt hash: {}", req.prompt_hash),
                    max_tokens: req.max_tokens,
                };

                if let Ok(output) = self.compute_engine.execute(&task, &self.registry) {
                    total_cu_used += output.compute_units;

                    let mut receipt = InferenceReceiptTx {
                        request_id: req.request_id(),
                        miner: self.address,
                        prompt_tokens: output.prompt_tokens,
                        completion_tokens: output.completion_tokens,
                        output_hash: output.output_hash,
                        miner_signature: blackchain_crypto::Signature::ZERO,
                    };
                    let sig = self.keypair.sign(&receipt.signing_bytes()).unwrap_or(blackchain_crypto::Signature::ZERO);
                    receipt.miner_signature = sig;
                    receipts.push(receipt);
                }
            }
        } else {
            // Idle queue: run deterministic synthetic benchmark task
            let synthetic_prompt = generate_synthetic_prompt(&prev_hash, next_height);
            let task = InferenceTask {
                model_id: "llama-3-8b-instruct".to_string(),
                prompt: synthetic_prompt,
                max_tokens: 64,
            };

            if let Ok(output) = self.compute_engine.execute(&task, &self.registry) {
                total_cu_used += output.compute_units;

                let mut receipt = InferenceReceiptTx {
                    request_id: hash(format!("synthetic-task-{}", next_height).as_bytes()),
                    miner: self.address,
                    prompt_tokens: output.prompt_tokens,
                    completion_tokens: output.completion_tokens,
                    output_hash: output.output_hash,
                    miner_signature: blackchain_crypto::Signature::ZERO,
                };
                let sig = self.keypair.sign(&receipt.signing_bytes()).unwrap_or(blackchain_crypto::Signature::ZERO);
                receipt.miner_signature = sig;
                receipts.push(receipt);
            }
        }

        if receipts.is_empty() {
            return None;
        }

        // 2. PoUW Lottery ticket evaluation
        let winning_receipt = &receipts[0];
        let receipt_hash = winning_receipt.receipt_hash();

        // Check nonces for lottery ticket
        for nonce in 0..50_000u64 {
            let ticket = compute_pouw_ticket(&receipt_hash, &prev_hash, nonce);

            if meets_difficulty(&ticket, &target) {
                // Winning ticket found! Construct candidate block
                let body = BlockBody::new(pending_txs, receipts);
                let current_market = DynamicFeeMarket::new(latest_block.header.base_fee_per_cu);
                let next_base_fee = current_market.calculate_next_base_fee(total_cu_used);

                let header = BlockHeader {
                    height: next_height,
                    prev_block_hash: prev_hash,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    state_root: Hash::ZERO, // State transition updates in engine
                    txs_root: body.calculate_txs_root(),
                    receipts_root: body.calculate_receipts_root(),
                    miner: self.address,
                    difficulty_target: target,
                    nonce,
                    pouw_ticket: ticket,
                    base_fee_per_cu: next_base_fee,
                    compute_units_used: total_cu_used,
                };

                return Some(Block::new(header, body));
            }
        }

        None
    }
}
