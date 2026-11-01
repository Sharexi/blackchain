use std::sync::Arc;
use tokio::sync::Mutex;
use blackchain_compute::{DeterministicInferenceEngine, InferenceBackend};
use blackchain_consensus::validate_block;
use blackchain_core::{
    Address, Amount, Block, InferenceRequestTx, ModelRegistry, State, Transaction,
};
use blackchain_crypto::Hash;
use blackchain_mempool::{Mempool, MempoolError};
use blackchain_p2p::{P2pMessage, P2pNetwork};
use blackchain_storage::{BlockchainStore, MemoryStore};
use crate::miner::Miner;
use tracing::warn;

pub struct NodeEngine {
    pub store: Arc<MemoryStore>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub registry: Arc<ModelRegistry>,
    pub miner: Miner,
    pub network: Arc<P2pNetwork>,
}

impl NodeEngine {
    pub fn new(
        node_id: String,
        miner_keypair: blackchain_crypto::Keypair,
        initial_state: State,
    ) -> Self {
        let genesis = Block::genesis();
        let store = Arc::new(MemoryStore::new(genesis, initial_state));
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        let registry = Arc::new(ModelRegistry::default());
        let compute_engine: Arc<dyn InferenceBackend> = Arc::new(DeterministicInferenceEngine::new());
        let (network, mut inbound_rx) = P2pNetwork::new(node_id);

        let miner = Miner::new(
            miner_keypair,
            (*registry).clone(),
            compute_engine,
            mempool.clone(),
            store.clone(),
        );

        // Spawn P2P message handler loop
        let mempool_clone = mempool.clone();
        let store_clone = store.clone();
        tokio::spawn(async move {
            while let Ok(msg) = inbound_rx.recv().await {
                match msg {
                    P2pMessage::NewInferenceRequest(req) => {
                        let mut guard = mempool_clone.lock().await;
                        let _ = guard.add_inference_request(req);
                    }
                    P2pMessage::NewTransaction(tx) => {
                        let mut guard = mempool_clone.lock().await;
                        let _ = guard.add_transaction(tx);
                    }
                    P2pMessage::NewBlock(block) => {
                        let prev = store_clone.get_latest_block();
                        let state = store_clone.get_state();
                        if validate_block(&block, &prev.header, &state).is_ok() {
                            let _ = store_clone.put_block(block);
                        }
                    }
                    _ => {}
                }
            }
        });

        Self {
            store,
            mempool,
            registry,
            miner,
            network,
        }
    }

    /// Submit an AI Agent inference request to the network
    #[allow(dead_code)]
    pub async fn submit_inference(&self, req: InferenceRequestTx) -> Result<Hash, MempoolError> {
        let req_id = req.request_id();
        {
            let mut guard = self.mempool.lock().await;
            guard.add_inference_request(req.clone())?;
        }
        // Broadcast over P2P network
        self.network.broadcast(P2pMessage::NewInferenceRequest(req)).await;
        Ok(req_id)
    }

    /// Submit a standard financial transaction
    #[allow(dead_code)]
    pub async fn submit_transaction(&self, tx: Transaction) -> Result<Hash, MempoolError> {
        let tx_hash = tx.hash();
        {
            let mut guard = self.mempool.lock().await;
            guard.add_transaction(tx.clone())?;
        }
        self.network.broadcast(P2pMessage::NewTransaction(tx)).await;
        Ok(tx_hash)
    }

    /// Run a consensus mining and validation tick
    pub async fn process_cycle(&self) -> Option<Block> {
        if let Some(mut candidate_block) = self.miner.mine_next_block().await {
            let prev_block = self.store.get_latest_block();
            let mut state = self.store.get_state();

            // Calculate state root
            candidate_block.header.state_root = state.calculate_state_root();

            // Validate candidate block against consensus rules
            if let Err(e) = validate_block(&candidate_block, &prev_block.header, &state) {
                warn!("Mined block failed consensus validation: {:?}", e);
                return None;
            }

            // Reward miner (simulate block reward: 5 native tokens)
            let block_reward = 5_000_000_000_000_000_000u128; // 5 Tokens
            state.credit(&candidate_block.header.miner, block_reward);

            // Store new state and block
            self.store.save_state(state).expect("Failed to persist state");
            self.store.put_block(candidate_block.clone()).expect("Failed to persist block");

            // Broadcast newly mined block to peer nodes
            self.network.broadcast(P2pMessage::NewBlock(candidate_block.clone())).await;

            return Some(candidate_block);
        }
        None
    }

    #[allow(dead_code)]
    pub fn get_account_balance(&self, address: &Address) -> Amount {
        let state = self.store.get_state();
        state.get_account(address).balance
    }

    pub fn get_latest_block(&self) -> Block {
        self.store.get_latest_block()
    }
}
