use blackchain_core::{Block, InferenceRequestTx, Transaction};
use blackchain_crypto::Hash;
use serde::{Deserialize, Serialize};

/// High-level messages gossiped across the Blackchain P2P network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2pMessage {
    /// Peer connection initialization
    Handshake {
        node_id: String,
        best_height: u64,
        best_hash: Hash,
    },
    /// Handshake acknowledgment
    HandshakeAck {
        node_id: String,
    },
    /// Newly mined block broadcast
    NewBlock(Block),
    /// Newly submitted AI agent inference request
    NewInferenceRequest(InferenceRequestTx),
    /// Standard financial transaction
    NewTransaction(Transaction),
    /// Keep-alive ping
    Ping,
    /// Keep-alive pong
    Pong,
}

impl P2pMessage {
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut data = serde_json::to_vec(self)?;
        data.push(b'\n'); // Newline delimiter for stream framing
        Ok(data)
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}
