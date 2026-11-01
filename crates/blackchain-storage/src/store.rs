use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use blackchain_core::{Block, State};
use blackchain_crypto::Hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Block not found")]
    BlockNotFound,
    #[error("Storage lock error")]
    LockError,
}

/// Trait defining the storage operations required by the node.
pub trait BlockchainStore: Send + Sync {
    fn put_block(&self, block: Block) -> Result<(), StorageError>;
    fn get_block_by_height(&self, height: u64) -> Option<Block>;
    fn get_block_by_hash(&self, hash: &Hash) -> Option<Block>;
    fn get_latest_block(&self) -> Block;
    fn get_state(&self) -> State;
    fn save_state(&self, state: State) -> Result<(), StorageError>;
}

/// Thread-safe in-memory store for development, integration testing, and fast node sync.
#[derive(Clone)]
pub struct MemoryStore {
    inner: Arc<RwLock<MemoryStoreInner>>,
}

struct MemoryStoreInner {
    blocks_by_height: HashMap<u64, Block>,
    blocks_by_hash: HashMap<Hash, Block>,
    latest_height: u64,
    state: State,
}

impl MemoryStore {
    pub fn new(genesis: Block, initial_state: State) -> Self {
        let genesis_hash = genesis.hash();
        let mut blocks_by_height = HashMap::new();
        let mut blocks_by_hash = HashMap::new();

        blocks_by_height.insert(0, genesis.clone());
        blocks_by_hash.insert(genesis_hash, genesis);

        let inner = MemoryStoreInner {
            blocks_by_height,
            blocks_by_hash,
            latest_height: 0,
            state: initial_state,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

impl BlockchainStore for MemoryStore {
    fn put_block(&self, block: Block) -> Result<(), StorageError> {
        let mut guard = self.inner.write().map_err(|_| StorageError::LockError)?;
        let h = block.hash();
        let height = block.header.height;

        guard.blocks_by_hash.insert(h, block.clone());
        guard.blocks_by_height.insert(height, block);

        if height > guard.latest_height {
            guard.latest_height = height;
        }
        Ok(())
    }

    fn get_block_by_height(&self, height: u64) -> Option<Block> {
        let guard = self.inner.read().ok()?;
        guard.blocks_by_height.get(&height).cloned()
    }

    fn get_block_by_hash(&self, hash: &Hash) -> Option<Block> {
        let guard = self.inner.read().ok()?;
        guard.blocks_by_hash.get(hash).cloned()
    }

    fn get_latest_block(&self) -> Block {
        let guard = self.inner.read().expect("Lock poisoned");
        guard.blocks_by_height.get(&guard.latest_height).cloned().expect("Latest block missing")
    }

    fn get_state(&self) -> State {
        let guard = self.inner.read().expect("Lock poisoned");
        guard.state.clone()
    }

    fn save_state(&self, state: State) -> Result<(), StorageError> {
        let mut guard = self.inner.write().map_err(|_| StorageError::LockError)?;
        guard.state = state;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_put_and_get() {
        let genesis = Block::genesis();
        let store = MemoryStore::new(genesis.clone(), State::new());

        let latest = store.get_latest_block();
        assert_eq!(latest.header.height, 0);
        assert_eq!(latest.hash(), genesis.hash());
    }
}
