use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
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

pub struct RpcServer;

impl RpcServer {
    pub fn router(state: AppState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .route("/health", get(Self::health))
            .route("/status", get(Self::status))
            .route("/models", get(Self::models))
            .route("/account/:address", get(Self::account))
            .route("/agent/inference", post(Self::submit_inference))
            .route("/blocks/latest", get(Self::latest_block))
            .route("/blocks/:height", get(Self::block_by_height))
            .layer(cors)
            .with_state(state)
    }

    pub async fn start(state: AppState, addr: SocketAddr) -> Result<(), std::io::Error> {
        let app = Self::router(state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("[RPC] Agent HTTP Server listening on http://{}", addr);

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("[RPC] Server error: {:?}", e);
            }
        });

        Ok(())
    }

    async fn health() -> impl IntoResponse {
        Json(serde_json::json!({ "status": "ok", "service": "blackchain-node" }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_rpc_health() {
        let genesis = Block::genesis();
        let store = Arc::new(MemoryStore::new(genesis, blackchain_core::State::new()));
        let mempool = Arc::new(Mutex::new(Mempool::new()));
        let registry = Arc::new(ModelRegistry::default());

        let state = AppState {
            store,
            mempool,
            registry,
            network: None,
        };

        let app = RpcServer::router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
