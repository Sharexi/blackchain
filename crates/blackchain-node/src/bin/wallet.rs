use std::fs;
use std::path::Path;
use blackchain_core::Address;
use blackchain_crypto::Keypair;
use serde::{Deserialize, Serialize};

const WALLET_FILE: &str = "wallet.json";
const RPC_URL: &str = "http://127.0.0.1:8545";

#[derive(Serialize, Deserialize, Debug)]
struct StoredWallet {
    address: String,
    public_key_hex: String,
    secret_key_hex: String,
    algorithm: String,
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "new" => {
            if Path::new(WALLET_FILE).exists() {
                eprintln!("[!] Warning: '{}' already exists! Rename or delete it first to avoid overwriting your keys.", WALLET_FILE);
                return;
            }

            let keypair = Keypair::generate().expect("Failed to generate ML-DSA-44 keypair");
            let pub_key = keypair.public_key();
            let address = Address::from_public_key(&pub_key);
            let secret_bytes = keypair.secret_bytes();

            let stored = StoredWallet {
                address: address.to_string(),
                public_key_hex: hex::encode(pub_key.as_bytes()),
                secret_key_hex: hex::encode(secret_bytes),
                algorithm: "NIST-FIPS-204-ML-DSA-44".to_string(),
            };

            let json = serde_json::to_string_pretty(&stored).expect("JSON serialization failed");
            fs::write(WALLET_FILE, json).expect("Failed to write wallet.json");

            println!("============================================================");
            println!("   NEW POST-QUANTUM FOUNDER WALLET GENERATED                ");
            println!("============================================================");
            println!("Algorithm  : {}", stored.algorithm);
            println!("Address    : {}", stored.address);
            println!("Saved to   : {}", WALLET_FILE);
            println!("============================================================");
            println!("Keep '{}' secure! It contains your private key.", WALLET_FILE);
        }

        "show" | "address" => {
            if !Path::new(WALLET_FILE).exists() {
                eprintln!("No wallet found at '{}'. Run 'cargo run --bin blackchain-wallet new' first.", WALLET_FILE);
                return;
            }

            let content = fs::read_to_string(WALLET_FILE).expect("Failed to read wallet file");
            let stored: StoredWallet = serde_json::from_str(&content).expect("Invalid wallet JSON");

            println!("Post-Quantum Address: {}", stored.address);
            println!("Algorithm           : {}", stored.algorithm);
        }

        "balance" => {
            if !Path::new(WALLET_FILE).exists() {
                eprintln!("No wallet found at '{}'. Run 'cargo run --bin blackchain-wallet new' first.", WALLET_FILE);
                return;
            }

            let content = fs::read_to_string(WALLET_FILE).expect("Failed to read wallet file");
            let stored: StoredWallet = serde_json::from_str(&content).expect("Invalid wallet JSON");

            let url = format!("{}/account/{}", RPC_URL, stored.address);
            println!("Querying node at {}...", url);

            match reqwest_get(&url).await {
                Ok(resp_json) => {
                    println!("\n============================================================");
                    println!("   WALLET BALANCE REPORT                                    ");
                    println!("============================================================");
                    println!("Address : {}", stored.address);
                    if let Some(balance_val) = resp_json.get("balance").and_then(|v| v.as_u64()) {
                        let tokens = (balance_val as f64) / 1_000_000_000_000_000_000.0;
                        println!("Balance : {} $BLACK (raw: {})", tokens, balance_val);
                    } else if let Some(balance_str) = resp_json.get("balance").and_then(|v| v.as_str()) {
                        println!("Balance : {} minimal units", balance_str);
                    }
                    if let Some(staked) = resp_json.get("staked") {
                        println!("Staked  : {}", staked);
                    }
                    println!("============================================================");
                }
                Err(e) => {
                    eprintln!("Could not reach node at {}. Make sure 'cargo run --bin blackchain-node' is running! Error: {:?}", RPC_URL, e);
                }
            }
        }

        _ => {
            println!("Blackchain Post-Quantum Wallet CLI");
            println!("\nUsage:");
            println!("  cargo run --bin blackchain-wallet new       # Generate a new ML-DSA-44 wallet");
            println!("  cargo run --bin blackchain-wallet show      # Display your wallet address");
            println!("  cargo run --bin blackchain-wallet balance   # Query your on-chain balance");
        }
    }
}

async fn reqwest_get(url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let parsed_url = url.strip_prefix("http://").unwrap_or(url);
    let mut parts = parsed_url.splitn(2, '/');
    let host_port = parts.next().unwrap_or("127.0.0.1:8545");
    let path = format!("/{}", parts.next().unwrap_or(""));

    let mut stream = TcpStream::connect(host_port).await?;
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\n\r\n",
        path, host_port
    );

    stream.write_all(request.as_bytes()).await?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;

    let response_str = String::from_utf8_lossy(&response);
    if let Some(body_start) = response_str.find("\r\n\r\n") {
        let body = &response_str[body_start + 4..];
        let json: serde_json::Value = serde_json::from_str(body)?;
        Ok(json)
    } else {
        Err("Invalid HTTP response".into())
    }
}
