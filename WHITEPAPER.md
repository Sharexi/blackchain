# Blackchain Whitepaper

**A Post-Quantum, Verifiable Proof-of-Useful-Work (PoUW) Blockchain for Autonomous AI Compute**

*Author: Rehli (Blackchain Protocol Architect)*  
*Version: 1.0.0 — November 2026*  
*Standard: NIST FIPS 204 (ML-DSA-44)*  

---

## 1. Abstract

Modern blockchains expend millions of megawatt-hours calculating useless cryptographic hash collisions, while the artificial intelligence ecosystem faces an acute, centralized compute deficit. Furthermore, emerging quantum computers running Shor’s algorithm threaten to dismantle traditional asymmetric cryptography (ECDSA, Ed25519) within the coming decade.

**Blackchain** is a decentralized Layer-1 blockchain engineered to resolve both crises simultaneously:
1. **Inference-Proof-of-Useful-Work (PoUW)**: Network security and block production rights are proportional to useful, verifiable AI inference served to autonomous agents rather than arbitrary SHA-256 hashing.
2. **Post-Quantum Security**: Native integration of **NIST FIPS 204 ML-DSA-44** (Module-Lattice-Based Digital Signature Algorithm), providing mathematical immunity against both classical and quantum cryptanalysis.
3. **Compute-EIP-1559 Dynamic Pricing**: An algorithmic fee market pricing physical GPU resource demands (memory bandwidth, prefill prompt tokens, and autoregressive generation).

---

## 2. Post-Quantum Cryptography Architecture

Classical elliptic curve cryptography relies on the discrete logarithm problem over finite fields or elliptic curves:
$$Q = [k]P$$
Shor’s quantum algorithm solves this problem in polynomial time $\mathcal{O}((\log N)^3)$, rendering existing Bitcoin, Ethereum, and Solana accounts insecure.

### 2.1 Module Learning With Errors (M-LWE)
Blackchain replaces elliptic curve primitives with **ML-DSA-44** (standardized under NIST FIPS 204, derived from CRYSTALS-Dilithium). Security rests on the shortest vector problem across high-dimensional module lattices over the polynomial ring $\mathcal{R}_q = \mathbb{Z}_q[X]/(X^n + 1)$:

$$\mathbf{A} \mathbf{s}_1 + \mathbf{s}_2 \equiv \mathbf{t} \pmod q$$

Where $\mathbf{A} \in \mathcal{R}_q^{k \times \ell}$ is a public matrix, and $\mathbf{s}_1, \mathbf{s}_2$ are small secret vectors.

### 2.2 Compact On-Chain Account Derivation
While post-quantum public keys are 1,312 bytes and signatures are 2,420 bytes, Blackchain retains ultra-compact 32-byte addresses via Blake3 cryptographic reduction:
$$\text{Address} = \text{Blake3}(\text{PQ\_PublicKey})$$

---

## 3. Inference-Proof-of-Useful-Work (PoUW) Consensus

### 3.1 The Inference Lottery
Instead of guessing random nonces in an idle loop, miners earn consensus proposal tickets by executing real forward passes on registered foundation models (e.g., Llama-3, Mistral):

1. **Receipt Generation**: Serving request $i$ produces an execution digest:
   $$\text{Receipt}_i = \text{Blake3}(\text{RequestID}_i \parallel \text{OutputHash}_i \parallel \text{MinerAddress})$$
2. **Ticket Evaluation**: The miner evaluates whether the executed work unlocks a candidate block:
   $$\text{Ticket} = \text{Blake3}(\text{Receipt}_i \parallel \text{PrevBlockHash} \parallel \text{Nonce})$$
3. **Winning Condition**: A block proposal is valid if and only if:
   $$\text{Ticket} \le \text{TargetDifficulty}$$

**Economic Consequence**: A GPU worker producing $10,000\text{ tokens/sec}$ evaluates $10\times$ more tickets than a worker producing $1,000\text{ tokens/sec}$. Compute throughput directly maps to network consensus weight.

### 3.2 Idle Baseline: Deterministic Synthetic Benchmarks
To guarantee consistent block cadence (target: **6 seconds**) during low user traffic, the protocol algorithmically injects synthetic calibration prompts derived pseudo-randomly from the previous block hash:
$$\text{Prompt}_{\text{synthetic}} = \text{PRNG}(\text{PrevBlockHash}, \text{Height})$$

---

## 4. Compute EIP-1559 Dynamic Fee Market

Traditional blockchain gas assumes all operations consume uniform CPU instructions. Deep learning inference, however, exhibits two distinct bottlenecks:
1. **Prefill Phase (Prompt)**: Compute-bound matrix-matrix multiplication.
2. **Decoding Phase (Completion)**: Memory-bandwidth-bound autoregressive token generation.

### 4.1 Compute Units (CU) Formulation
Workload is normalized into **Compute Units (CU)**:
$$\text{CU} = \left(\text{PromptTokens} \times 1 + \text{CompletionTokens} \times 4\right) \times \text{Multiplier}_{\text{model}}$$

* `Llama-3-8B-Instruct`: Multiplier = $1.0\times$ (100 bps)
* `Mistral-7B-Instruct`: Multiplier = $0.9\times$ (90 bps)
* `Llama-3-70B-Instruct`: Multiplier = $8.5\times$ (850 bps)

### 4.2 Base Fee Elasticity
Each block targets $1,000,000\text{ CU}$. If utilization in block $n$ exceeds target capacity:
$$\text{BaseFee}_{n+1} = \text{BaseFee}_n \times \left(1 + \frac{1}{8} \cdot \frac{\text{CU}_{\text{used}} - \text{CU}_{\text{target}}}{\text{CU}_{\text{target}}}\right)$$

---

## 5. Tokenomics & Distribution

* **Token Ticker**: `$BLACK`
* **Total Initial Supply**: `1,000,000,000 $BLACK` ($1\text{ Billion tokens}$)
* **Decimal Precision**: $10^{18}$ minimal indivisible units

### 5.1 Distribution Breakdown
```
┌────────────────────────────────────────────────────────┐
│  [90%] Mined via PoUW GPU Inference (900,000,000 $BLACK)│
├────────────────────────────────────────────────────────┤
│  [10%] Founder & Core Architecture (100,000,000 $BLACK)│
└────────────────────────────────────────────────────────┘
```
1. **10% Founder Share (100,000,000 $BLACK)**: Credited at Genesis Block #0 to the founder post-quantum address `0x914e8ffbed01e47d6e98d0f5af72b3d4499de1bd14c4f3044a6bbd61f3bc16b2`.
2. **90% Community PoUW Mining (900,000,000 $BLACK)**: Emitted dynamically per block (+5 $BLACK base block reward + user priority tips) to nodes serving verified inference.

---

## 6. Developer Integration & OpenAI Compatibility

To eliminate developer adoption friction, Blackchain nodes expose native OpenAI-compatible REST endpoints directly on their HTTP RPC servers:

```python
from openai import OpenAI

# Zero-configuration drop-in replacement for OpenAI SDK
client = OpenAI(
    base_url="https://era-symbol-foot-church.trycloudflare.com/v1",
    api_key="blackchain-public"
)

response = client.chat.completions.create(
    model="llama-3-8b-instruct",
    messages=[{"role": "user", "content": "Explain quantum entanglement in 20 words."}]
)

print(response.choices[0].message.content)
```

---

## 7. Conclusion

Blackchain creates a self-sustaining decentralized economic loop: autonomous AI agents deposit micro-fees to acquire compute, and GPU miners earn post-quantum secure $BLACK tokens by serving real inference workloads. By coupling real-world AI utility with quantum-immune lattice cryptography, Blackchain provides the computational foundation for the next generation of sovereign artificial intelligence.
