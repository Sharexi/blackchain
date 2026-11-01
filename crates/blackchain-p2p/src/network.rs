use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{error, info, warn};
use crate::message::P2pMessage;

pub struct P2pNetwork {
    pub node_id: String,
    peers: Arc<RwLock<HashMap<SocketAddr, mpsc::Sender<P2pMessage>>>>,
    inbound_tx: broadcast::Sender<P2pMessage>,
}

impl P2pNetwork {
    pub fn new(node_id: String) -> (Arc<Self>, broadcast::Receiver<P2pMessage>) {
        let (inbound_tx, inbound_rx) = broadcast::channel(1000);
        let network = Arc::new(Self {
            node_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            inbound_tx,
        });
        (network, inbound_rx)
    }

    /// Start listening for inbound peer connections on the given address.
    pub async fn listen(self: &Arc<Self>, bind_addr: &str) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(bind_addr).await?;
        info!("[P2P] Node listening on {}", bind_addr);

        let this = self.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("[P2P] Accepted peer connection from {}", addr);
                        this.handle_connection(stream, addr).await;
                    }
                    Err(e) => {
                        warn!("[P2P] Listener error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Dial an outbound peer address.
    pub async fn connect(self: &Arc<Self>, peer_addr: &str) -> Result<(), std::io::Error> {
        let stream = TcpStream::connect(peer_addr).await?;
        let addr: SocketAddr = peer_addr.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        info!("[P2P] Connected to outbound peer {}", addr);
        self.handle_connection(stream, addr).await;
        Ok(())
    }

    /// Broadcast a message to all connected peers.
    pub async fn broadcast(&self, msg: P2pMessage) {
        let peers_guard = self.peers.read().await;
        for (addr, sender) in peers_guard.iter() {
            if let Err(e) = sender.send(msg.clone()).await {
                warn!("[P2P] Failed to send message to {}: {:?}", addr, e);
            }
        }
    }

    /// Subscribe to receive inbound network messages.
    pub fn subscribe(&self) -> broadcast::Receiver<P2pMessage> {
        self.inbound_tx.subscribe()
    }

    async fn handle_connection(self: &Arc<Self>, stream: TcpStream, addr: SocketAddr) {
        let (reader, mut writer) = stream.into_split();
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<P2pMessage>(100);

        // Register peer
        self.peers.write().await.insert(addr, outbound_tx);

        let inbound_sender = self.inbound_tx.clone();
        let peers_ref = self.peers.clone();

        // Spawn read loop
        tokio::spawn(async move {
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                line.clear();
                match buf_reader.read_line(&mut line).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            if let Ok(msg) = P2pMessage::from_slice(trimmed.as_bytes()) {
                                let _ = inbound_sender.send(msg);
                            }
                        }
                    }
                    Err(e) => {
                        error!("[P2P] Read error from {}: {:?}", addr, e);
                        break;
                    }
                }
            }
            peers_ref.write().await.remove(&addr);
            info!("[P2P] Peer disconnected: {}", addr);
        });

        // Spawn write loop
        tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                if let Ok(bytes) = msg.to_bytes() {
                    if let Err(e) = writer.write_all(&bytes).await {
                        warn!("[P2P] Write error to {}: {:?}", addr, e);
                        break;
                    }
                }
            }
        });
    }

    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_handshake_and_broadcast() {
        let (node_a, _rx_a) = P2pNetwork::new("node_a".to_string());
        let (node_b, mut rx_b) = P2pNetwork::new("node_b".to_string());

        // Listen on loopback ports
        node_a.listen("127.0.0.1:19870").await.unwrap();
        node_b.listen("127.0.0.1:19871").await.unwrap();

        // Connect node_b to node_a
        node_b.connect("127.0.0.1:19870").await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Broadcast from node_a
        node_a.broadcast(P2pMessage::Ping).await;

        // Node B should receive Ping
        tokio::select! {
            msg = rx_b.recv() => {
                assert_eq!(msg.unwrap(), P2pMessage::Ping);
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                panic!("Timeout waiting for P2P message");
            }
        }
    }
}
