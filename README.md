# Blackchain

A **Post-Quantum, Inference-Proof-of-Useful-Work (PoUW)** blockchain designed for autonomous AI agents and verifiable GPU compute.

[![GitHub license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![PQC Standard](https://img.shields.io/badge/PQC-NIST%20FIPS%20204%20(ML--DSA--44)-brightgreen)](https://csrc.nist.gov/pubs/fips/204/final)
[![Consensus](https://img.shields.io/badge/Consensus-Inference--PoUW-orange)](WHITEPAPER.md)

Read the official [**Technical Whitepaper (WHITEPAPER.md)**](WHITEPAPER.md) for detailed mathematical specifications.

---

## Key Features

1. **NIST FIPS 204 Post-Quantum Cryptography (ML-DSA-44)**:
   * Uses lattice-based cryptography (Module Learning With Errors / M-LWE) mathematically immune to Shor's quantum algorithm.
   * Public keys and transaction signatures are quantum-secure.
   * Addresses remain compact 32-byte hashes: $\text{Address} = \text{Blake3}(\text{PQ\_PublicKey})$.
2. **Inference-Proof-of-Useful-Work (PoUW)**:
   * Replaces arbitrary hashing with useful deep learning forward passes.
   * Block production probabilities scale directly with token generation throughput (tokens/second).
3. **Built-in Web Explorer & Dashboard**:
   * Open `http://localhost:8545/` (or your public tunnel) in any web browser to view live block production, compute metrics, and dispatch prompts directly from the UI.
4. **Drop-in OpenAI Compatibility (`/v1/chat/completions`)**:
   * Seamlessly route prompts from LangChain, AutoGen, CrewAI, or cursor using standard OpenAI client libraries.
5. **Compute EIP-1559 Dynamic Pricing**:
   * Algorithmic fee adjustment accounting for memory bandwidth and prompt prefill complexity.
6. **Tokenomics (1 Billion Total Supply)**:
   * **10% Founder Share (100,000,000 $BLACK)** credited at Genesis Block #0.
   * **90% Community Emission (900,000,000 $BLACK)** mined via PoUW GPU inference.

---

## Quickstart

### 1. Run the Node
```bash
# Windows
.\start_node.bat

# Linux / macOS
./start_node.sh
```

### 2. View the Web Explorer
Open your browser to:
```text
http://127.0.0.1:8545/
```

### 3. Check Founder Wallet Balance
```bash
.\wallet.bat show
.\wallet.bat balance
```

### 4. Use with OpenAI SDK
```python
from openai import OpenAI

client = OpenAI(base_url="http://127.0.0.1:8545/v1", api_key="blackchain")

response = client.chat.completions.create(
    model="llama-3-8b-instruct",
    messages=[{"role": "user", "content": "Explain PoUW in 20 words."}]
)

print(response.choices[0].message.content)
```

---

## Workspace Architecture

| Crate | Purpose |
| :--- | :--- |
| [`blackchain-crypto`](crates/blackchain-crypto) | NIST FIPS 204 ML-DSA-44 post-quantum signatures, Blake3 hashing, and Merkle proofs. |
| [`blackchain-core`](crates/blackchain-core) | Domain primitives (`Address`, `Amount`, `ComputeUnits`), EIP-1559 pricing, model registry, and state. |
| [`blackchain-consensus`](crates/blackchain-consensus) | PoUW lottery evaluation, synthetic task generation, and difficulty retargeting. |
| [`blackchain-compute`](crates/blackchain-compute) | Deterministic AI inference engine and token accounting. |
| [`blackchain-mempool`](crates/blackchain-mempool) | Dual-queue mempool with priority tip ordering. |
| [`blackchain-storage`](crates/blackchain-storage) | Block persistence and thread-safe in-memory store. |
| [`blackchain-p2p`](crates/blackchain-p2p) | Async TCP peer-to-peer gossip network. |
| [`blackchain-rpc`](crates/blackchain-rpc) | HTTP REST, Web Explorer dashboard, and OpenAI-compatible API server. |
| [`blackchain-node`](crates/blackchain-node) | Complete node daemon coordinating mining, consensus, P2P, and RPC. |
