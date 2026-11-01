mod engine;
mod miner;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use blackchain_core::{Address, State, MINIMUM_MINING_STAKE};
use blackchain_crypto::Keypair;
use blackchain_rpc::{AppState, RpcServer};
use engine::NodeEngine;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

struct NodeConfig {
    p2p_addr: String,
    rpc_addr: SocketAddr,
    bootnodes: Vec<String>,
}

fn parse_args() -> NodeConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut p2p_addr = "0.0.0.0:9100".to_string();
    let mut rpc_addr: SocketAddr = "127.0.0.1:8545".parse().unwrap();
    let mut bootnodes = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--p2p" => {
                if i + 1 < args.len() {
                    p2p_addr = args[i + 1].clone();
                    i += 1;
                }
            }
            "--rpc" => {
                if i + 1 < args.len() {
                    if let Ok(addr) = args[i + 1].parse() {
                        rpc_addr = addr;
                    }
                    i += 1;
                }
            }
            "--bootnode" => {
                if i + 1 < args.len() {
                    bootnodes.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Blackchain Node Daemon");
                println!("\nOptions:");
                println!("  --p2p <ADDR>       P2P gossip bind address (default: 0.0.0.0:9100)");
                println!("  --rpc <ADDR>       RPC HTTP bind address   (default: 127.0.0.1:8545)");
                println!("  --bootnode <ADDR>  Connect to remote seed peer (e.g., 198.51.100.1:9100)");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    NodeConfig {
        p2p_addr,
        rpc_addr,
        bootnodes,
    }
}

#[tokio::main]
async fn main() {
    let config = parse_args();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    println!("============================================================");
    println!("     BLACKCHAIN: Inference-PoUW Decentralized AI Node       ");
    println!("============================================================");

    // 1. Miner Identity
    let miner_keypair = Keypair::generate().expect("PQC keygen");
    let miner_addr = Address::from_public_key(&miner_keypair.public_key());
    info!("Miner Identity Initialized (ML-DSA-44): {}", miner_addr);

    // 2. Genesis State Setup
    let mut initial_state = State::new();

    // Check for Founder Wallet allocation (10% Genesis Premine)
    if let Ok(content) = std::fs::read_to_string("wallet.json") {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(addr_str) = val.get("address").and_then(|a| a.as_str()) {
                if let Ok(founder_addr) = addr_str.parse::<Address>() {
                    let founder_allocation = blackchain_core::TOTAL_GENESIS_SUPPLY / 10; // 100 Million Tokens
                    initial_state.credit(&founder_addr, founder_allocation);
                    info!("FOUNDER ALLOCATION ACTIVATED: Credited 10% (100,000,000 $BLACK) to {}", founder_addr);
                }
            }
        }
    }

    let miner_account = initial_state.accounts.entry(miner_addr).or_default();
    miner_account.balance = 100_000_000_000_000_000_000; // 100 Tokens
    miner_account.staked = MINIMUM_MINING_STAKE; // 10 Tokens staked

    // 3. Initialize Node Engine
    let node_id = format!("node-{}", hex::encode(&miner_addr.as_bytes()[0..4]));
    let node = Arc::new(NodeEngine::new(node_id, miner_keypair, initial_state));
    info!("Genesis block active. Base fee: {} units/CU", node.get_latest_block().header.base_fee_per_cu);

    // 4. Start P2P Listener
    if let Err(e) = node.network.listen(&config.p2p_addr).await {
        tracing::warn!("Could not bind P2P port {}: {:?}", config.p2p_addr, e);
    }

    // 5. Connect to Outbound Bootnodes if specified
    for peer in config.bootnodes {
        info!("[P2P] Connecting to bootnode peer: {}", peer);
        let net = node.network.clone();
        tokio::spawn(async move {
            if let Err(e) = net.connect(&peer).await {
                tracing::warn!("[P2P] Could not connect to bootnode {}: {:?}", peer, e);
            }
        });
    }

    // 6. Start HTTP Agent API Server
    let rpc_state = AppState {
        store: node.store.clone(),
        mempool: node.mempool.clone(),
        registry: node.registry.clone(),
        network: Some(node.network.clone()),
    };

    if let Err(e) = RpcServer::start(rpc_state, config.rpc_addr).await {
        tracing::warn!("Could not start RPC server on {}: {:?}", config.rpc_addr, e);
    }

    // 7. Background Autonomous PoUW Mining Loop
    println!("\n[Consensus] Autonomous PoUW Mining Daemon Active (Target: 6s blocks)...");
    let node_worker = node.clone();
    tokio::spawn(async move {
        loop {
            if let Some(block) = node_worker.process_cycle().await {
                println!(
                    "[Mined] Block #{} | Hash: {}... | CU: {} | Reward: +5 $BLACK",
                    block.header.height,
                    &block.hash().to_string()[..12],
                    block.header.compute_units_used
                );
            }
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
    });

    // Keep the main process alive
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
    info!("Shutting down Blackchain node gracefully...");
}
