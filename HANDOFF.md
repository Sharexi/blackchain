# Blackchain — Project Handoff & Continuity Brief

**Last Updated:** November 1, 2026  
**Status:** All background processes stopped. Clean state verified. Ready to resume anytime.

---

## 1. Executive Summary & Architecture

Blackchain is a **Post-Quantum, Proof-of-Useful-Work (PoUW) Layer-1 blockchain** written in Rust, built specifically for autonomous AI agents and verifiable GPU compute.

### Core Architectural Pillars
* **Post-Quantum Security**: Implements **NIST FIPS 204 ML-DSA-44** (Module-Lattice-Based Digital Signature Algorithm). Immune to Shor’s quantum algorithm.
* **Inference-PoUW Consensus**: Eliminates arbitrary hashing puzzles; block production rights scale directly with token generation throughput (tokens/second) on registered models (`Llama-3-8B`, `Mistral-7B`, `Llama-3-70B`).
* **Idle Baseline**: When user traffic is zero, the chain generates deterministic synthetic calibration benchmarks to maintain a steady **6-second block time**.
* **Compute EIP-1559 Pricing**: Algorithmic fee adjustment distinguishing between prompt prefill matrix operations and memory-bandwidth-bound token decoding.
* **OpenAI Drop-In Compatibility**: Exposes native `/v1/chat/completions` and `/v1/models` endpoints directly on RPC.
* **Built-in Web Explorer**: Interactive dark-mode dashboard served directly at `GET /`.

---

## 2. Key Accounts & Tokenomics

* **Total Initial Supply**: `1,000,000,000 $BLACK` (1 Billion tokens with $10^{18}$ precision).
* **Founder Allocation (10%)**: `100,000,000 $BLACK` credited at Genesis Block #0.
* **Community Mining (90%)**: `900,000,000 $BLACK` emitted via PoUW mining (+5 $BLACK/block + tips).

### Founder Wallet Credentials
* **Address**: `0x914e8ffbed01e47d6e98d0f5af72b3d4499de1bd14c4f3044a6bbd61f3bc16b2`
* **Keystore File**: `wallet.json` *(excluded from Git via `.gitignore` for security)*
* **Algorithm**: NIST FIPS 204 ML-DSA-44

---

## 3. Repository & Public Links

* **GitHub Repository**: [https://github.com/Sharexi/blackchain](https://github.com/Sharexi/blackchain)
* **Technical Litepaper**: [`WHITEPAPER.md`](WHITEPAPER.md)
* **Open-Source License**: Dual MIT / Apache-2.0

---

## 4. How to Resume the Network (1-Click)

Whenever you return, start the network in two simple steps:

### A. Start the Node & Web Explorer
Double-click `start_node.bat` (or in terminal):
```cmd
.\start_node.bat
```
* **Local Web Explorer**: [http://localhost:8545](http://localhost:8545)
* **P2P Gossip Port**: `0.0.0.0:9100`

### B. Expose to the Internet for $0 (Optional)
In a second terminal:
```cmd
cloudflared tunnel --url http://127.0.0.1:8545
```
This gives you a public HTTPS URL (e.g. `https://your-tunnel.trycloudflare.com`).

### C. Wallet Management
```cmd
.\wallet.bat show      # View your post-quantum address
.\wallet.bat balance   # View your 100M $BLACK balance
```

### D. Test AI Inference
```cmd
python agent_client.py http://127.0.0.1:8545
```

---

## 5. Next Horizon / Future Roadmap

When resuming work, here are the logical next milestones:
1. **Real GGUF Weight Execution**: Plug in Hugging Face's `candle` or `llama.cpp` to run actual quantized model weights on consumer NVIDIA/AMD GPUs.
2. **Permanent 24/7 Seed Node**: Deploying a bootnode to Oracle Cloud Always-Free Tier ($0/mo, 24GB RAM, static IP) so the network stays permanently online without relying on your home PC.
3. **Public Community Launch**: Announcing the GitHub repo on `r/LocalLLaMA`, `r/gpumining`, and X to attract independent GPU miners to mine the 90% unmined tokens.
