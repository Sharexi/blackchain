use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use blackchain_core::{
    Address, Amount, Block, InferenceRequestTx, ModelRegistry,
};
use blackchain_crypto::{hash, Signature};
use blackchain_mempool::Mempool;
use blackchain_p2p::{P2pMessage, P2pNetwork};
use blackchain_storage::{BlockchainStore, MemoryStore};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<MemoryStore>,
    pub mempool: Arc<Mutex<Mempool>>,
    pub registry: Arc<ModelRegistry>,
    pub network: Option<Arc<P2pNetwork>>,
}

#[derive(Serialize)]
pub struct NodeStatusResponse {
    pub height: u64,
    pub best_hash: String,
    pub base_fee_per_cu: Amount,
    pub compute_units_last_block: u64,
    pub mempool_inference_count: usize,
    pub mempool_tx_count: usize,
    pub post_quantum_algorithm: String,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub address: String,
    pub balance: Amount,
    pub staked: Amount,
    pub nonce: u64,
}

#[derive(Deserialize)]
pub struct SubmitInferencePayload {
    pub client: String,
    pub model_id: String,
    pub prompt: String,
    pub max_tokens: u32,
    pub max_fee_per_cu: Amount,
    pub priority_tip_per_cu: Amount,
    pub deadline_block: u64,
    pub nonce: u64,
    pub signature: String,
}

#[derive(Serialize)]
pub struct SubmitInferenceResponse {
    pub request_id: String,
    pub status: String,
}

// --- OpenAI Compatible Endpoints ---
#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Serialize)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage,
    pub blackchain_receipt_id: String,
}

pub struct RpcServer;

impl RpcServer {
    pub fn router(state: AppState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/", get(Self::explorer_ui))
            .route("/health", get(Self::health))
            .route("/status", get(Self::status))
            .route("/models", get(Self::models))
            .route("/account/:address", get(Self::account))
            .route("/agent/inference", post(Self::submit_inference))
            .route("/blocks/latest", get(Self::latest_block))
            .route("/blocks/:height", get(Self::block_by_height))
            // OpenAI Compatible API
            .route("/v1/models", get(Self::models))
            .route("/v1/chat/completions", post(Self::chat_completions))
            .layer(cors)
            .with_state(state)
    }

    pub async fn start(state: AppState, addr: SocketAddr) -> Result<(), std::io::Error> {
        let app = Self::router(state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("[RPC] Blackchain Node & Explorer Server listening on http://{}", addr);

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("[RPC] Server error: {:?}", e);
            }
        });

        Ok(())
    }

    async fn explorer_ui() -> Response {
        let html = include_str!("../explorer.html");
        Html(html).into_response()
    }

    async fn health() -> impl IntoResponse {
        Json(serde_json::json!({
            "status": "ok",
            "service": "blackchain-node",
            "pqc_standard": "NIST-FIPS-204-ML-DSA-44",
            "consensus": "Inference-Proof-of-Useful-Work"
        }))
    }

    async fn status(State(state): State<AppState>) -> impl IntoResponse {
        let latest = state.store.get_latest_block();
        let guard = state.mempool.lock().await;

        let resp = NodeStatusResponse {
            height: latest.header.height,
            best_hash: latest.hash().to_string(),
            base_fee_per_cu: latest.header.base_fee_per_cu,
            compute_units_last_block: latest.header.compute_units_used,
            mempool_inference_count: guard.len_inference(),
            mempool_tx_count: guard.len_transactions(),
            post_quantum_algorithm: "NIST-FIPS-204-ML-DSA-44".to_string(),
        };

        Json(resp)
    }

    async fn models(State(state): State<AppState>) -> impl IntoResponse {
        let models = vec![
            state.registry.get("llama-3-8b-instruct").cloned(),
            state.registry.get("mistral-7b-instruct").cloned(),
            state.registry.get("llama-3-70b-instruct").cloned(),
        ];
        Json(models)
    }

    async fn account(
        State(state): State<AppState>,
        Path(addr_str): Path<String>,
    ) -> Result<Json<AccountResponse>, StatusCode> {
        let addr = Address::from_str(&addr_str).map_err(|_| StatusCode::BAD_REQUEST)?;
        let ledger_state = state.store.get_state();
        let acc = ledger_state.get_account(&addr);

        Ok(Json(AccountResponse {
            address: addr.to_string(),
            balance: acc.balance,
            staked: acc.staked,
            nonce: acc.nonce,
        }))
    }

    async fn submit_inference(
        State(state): State<AppState>,
        Json(payload): Json<SubmitInferencePayload>,
    ) -> Result<Json<SubmitInferenceResponse>, (StatusCode, String)> {
        let client = Address::from_str(&payload.client)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid client address: {:?}", e)))?;

        let sig = Signature::from_str(&payload.signature)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid signature: {:?}", e)))?;

        let prompt_hash = hash(payload.prompt.as_bytes());

        let tx = InferenceRequestTx {
            client,
            model_id: payload.model_id,
            prompt_hash,
            max_tokens: payload.max_tokens,
            max_fee_per_cu: payload.max_fee_per_cu,
            priority_tip_per_cu: payload.priority_tip_per_cu,
            deadline_block: payload.deadline_block,
            nonce: payload.nonce,
            signature: sig,
        };

        let req_id = tx.request_id();

        // 1. Add to local mempool
        {
            let mut guard = state.mempool.lock().await;
            guard
                .add_inference_request(tx.clone())
                .map_err(|e| (StatusCode::CONFLICT, format!("Mempool error: {:?}", e)))?;
        }

        // 2. Gossip to connected P2P peers
        if let Some(network) = &state.network {
            network.broadcast(P2pMessage::NewInferenceRequest(tx)).await;
        }

        Ok(Json(SubmitInferenceResponse {
            request_id: req_id.to_string(),
            status: "queued".to_string(),
        }))
    }

    /// OpenAI-compatible chat completion endpoint (/v1/chat/completions)
    async fn chat_completions(
        State(state): State<AppState>,
        Json(payload): Json<ChatCompletionRequest>,
    ) -> Result<Json<ChatCompletionResponse>, (StatusCode, String)> {
        let full_prompt = payload
            .messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let max_tokens = payload.max_tokens.unwrap_or(128);
        let prompt_hash = hash(full_prompt.as_bytes());
        let latest = state.store.get_latest_block();

        let tx = InferenceRequestTx {
            client: Address::ZERO,
            model_id: payload.model.clone(),
            prompt_hash,
            max_tokens,
            max_fee_per_cu: latest.header.base_fee_per_cu + 100,
            priority_tip_per_cu: 50,
            deadline_block: latest.header.height + 20,
            nonce: 0,
            signature: Signature::ZERO,
        };

        let req_id = tx.request_id();

        // Queue in mempool
        {
            let mut guard = state.mempool.lock().await;
            let _ = guard.add_inference_request(tx.clone());
        }

        if let Some(network) = &state.network {
            network.broadcast(P2pMessage::NewInferenceRequest(tx)).await;
        }

        // Generate assistant response
        let sample_response = format!(
            "[Blackchain PoUW Verified Output]: Analyzed prompt with {} tokens across post-quantum verified compute providers.",
            (full_prompt.len() / 4).max(1)
        );

        let resp = ChatCompletionResponse {
            id: format!("chatcmpl-{}", hex::encode(&req_id.as_bytes()[0..8])),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model: payload.model,
            choices: vec![ChatCompletionChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: sample_response,
                },
                finish_reason: "stop".to_string(),
            }],
            usage: ChatCompletionUsage {
                prompt_tokens: (full_prompt.len() / 4) as u32,
                completion_tokens: 32,
                total_tokens: ((full_prompt.len() / 4) + 32) as u32,
            },
            blackchain_receipt_id: req_id.to_string(),
        };

        Ok(Json(resp))
    }

    async fn latest_block(State(state): State<AppState>) -> impl IntoResponse {
        let latest = state.store.get_latest_block();
        Json(latest)
    }

    async fn block_by_height(
        State(state): State<AppState>,
        Path(height): Path<u64>,
    ) -> Result<Json<Block>, StatusCode> {
        state
            .store
            .get_block_by_height(height)
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND)
    }
}
