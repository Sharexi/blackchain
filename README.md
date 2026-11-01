# Blackchain

A **Post-Quantum, Inference-Proof-of-Useful-Work (PoUW)** blockchain designed for autonomous AI agents and verifiable GPU compute.

---

## Key Architectural Principles

1. **NIST FIPS 204 Post-Quantum Cryptography (ML-DSA-44)**:
   * Uses lattice-based cryptography (Module Learning With Errors / M-LWE) resistant to Shor's quantum algorithm.
   * Public keys and transaction signatures are quantum-secure.
   * Wallet addresses remain compact 32-byte hashes: $\text{Address} = \text{Blake3}(\text{PQ\_PublicKey})$.
2. **Inference-PoUW Consensus**:
   * Instead of wasting electricity on meaningless hash puzzles, nodes earn block proposal rights by serving real AI inference.
   * A node producing $10,000\text{ tokens/sec}$ generates $10\times$ more lottery tickets than a node producing $1,000\text{ tokens/sec}$.
3. **Idle Baseline (Synthetic Benchmarks)**:
   * When user inference queues are empty, the chain generates deterministic synthetic calibration prompts derived from the previous block hash to maintain a constant 6-second block cadence.
4. **Optimistic Staking & Verification**:
   * Miners lock a stake bond. Inference responses are accepted optimistically with zero block latency; a challenge window allows "Fisherman" nodes to spot-check outputs and slash dishonest nodes.
5. **Compute EIP-1559 Dynamic Pricing**:
   * Base fees adjust dynamically based on target Compute Units (CU) per block:
     $$\text{CU} = (\text{Prompt Tokens} \times 1 + \text{Completion Tokens} \times 4) \times \text{Model Multiplier}$$
   * AI agents attach a priority tip to jump to the front of the compute queue.
6. **P2P Gossip Network**:
   * Async peer-to-peer TCP networking layer for broadcasting newly mined blocks, transactions, and inference requests across nodes.
7. **HTTP / JSON-RPC Agent Server**:
   * REST and JSON-RPC API server (`axum`) enabling external AI agents (LangChain, AutoGen, custom agents) to submit prompts, inspect accounts, and stream compute receipts.

---

## Workspace Crates

| Crate | Purpose |
| :--- | :--- |
| [`blackchain-crypto`](crates/blackchain-crypto) | NIST FIPS 204 ML-DSA post-quantum signatures, Blake3 hashing, and Merkle inclusion proofs. |
| [`blackchain-core`](crates/blackchain-core) | Domain types (`Address`, `Amount`, `ComputeUnits`), transactions, blocks, EIP-1559 math, and state ledger. |
| [`blackchain-consensus`](crates/blackchain-consensus) | PoUW lottery evaluation, synthetic task generation, dynamic difficulty retargeting, and block validation. |
| [`blackchain-compute`](crates/blackchain-compute) | Deterministic AI inference engine, token accounting, and execution receipt hashing. |
| [`blackchain-mempool`](crates/blackchain-mempool) | Dual-queue mempool prioritizing AI inference jobs by tip per Compute Unit. |
| [`blackchain-storage`](crates/blackchain-storage) | Block persistence and thread-safe in-memory store. |
| [`blackchain-p2p`](crates/blackchain-p2p) | Peer-to-peer gossip protocol for blocks and inference requests over TCP. |
| [`blackchain-rpc`](crates/blackchain-rpc) | HTTP REST API server for autonomous AI agents and developer tooling. |
| [`blackchain-node`](crates/blackchain-node) | Complete node daemon coordinating mining, consensus, P2P gossip, and the RPC server. |

---

## HTTP Agent API Endpoints

When running `blackchain-node` (listening on `http://127.0.0.1:8545`):

* `GET /health`: Node health status.
* `GET /status`: Current block height, best hash, base fee per CU, and mempool queue size.
* `GET /models`: List of registered AI models and their CU multipliers.
* `GET /account/:address`: Balance, staked amount, and nonce for an address.
* `POST /agent/inference`: Submit a signed inference request with a priority tip.
* `GET /blocks/latest`: Fetch the latest mined block.
* `GET /blocks/:height`: Fetch a historical block by height.

---

## Running the Node

```bash
# Run all 21 unit tests across the workspace
cargo test

# Launch the post-quantum node with P2P and the Agent RPC server
cargo run --bin blackchain-node
```
