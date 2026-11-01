"""
Blackchain Autonomous AI Agent Client
-------------------------------------
Demonstrates how external AI agents (LangChain, AutoGen, ElizaOS) 
query the Blackchain network, check dynamic compute pricing, and
submit verifiable inference requests.
"""

import sys
import json
import urllib.request
import urllib.error

DEFAULT_RPC_URL = "http://127.0.0.1:8545"

def get_json(url):
    req = urllib.request.Request(url, headers={"Accept": "application/json"})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())

def post_json(url, data):
    body = json.dumps(data).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        headers={"Content-Type": "application/json", "Accept": "application/json"}
    )
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())

def main():
    rpc_url = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_RPC_URL
    print(f"Connecting to Blackchain RPC at: {rpc_url}\n")

    try:
        # 1. Check network health
        health = get_json(f"{rpc_url}/health")
        print(f"[OK] Network Status: {health.get('status')} ({health.get('service')})")

        # 2. Query dynamic compute fee market
        status = get_json(f"{rpc_url}/status")
        base_fee = status.get("base_fee_per_cu", 1000)
        height = status.get("height", 0)
        print(f"[OK] Block Height: #{height} | Base Fee: {base_fee} units/CU | Mempool: {status.get('mempool_inference_count', 0)} jobs")

        # 3. Check registered AI models
        models = get_json(f"{rpc_url}/models")
        print(f"[OK] Registered Models available:")
        for m in models:
            if m:
                print(f"    - {m['id']} (Multiplier: {m['cu_multiplier_bps'] / 100}x, Min VRAM: {m['min_vram_mb'] // 1000}GB)")

        # 4. Submit an AI Agent Inference Request
        prompt = "Explain why post-quantum cryptography is essential for decentralized AI."
        print(f"\n[AI Agent] Submitting prompt: \"{prompt}\"")

        payload = {
            "client": "0x914e8ffbed01e47d6e98d0f5af72b3d4499de1bd14c4f3044a6bbd61f3bc16b2", # Founder/Agent address
            "model_id": "llama-3-8b-instruct",
            "prompt": prompt,
            "max_tokens": 128,
            "max_fee_per_cu": base_fee + 200,
            "priority_tip_per_cu": 50, # Tip to jump queue
            "deadline_block": height + 50,
            "nonce": 0,
            "signature": "0x" # Empty signature placeholder for demo (or real ML-DSA-44 sig)
        }

        resp = post_json(f"{rpc_url}/agent/inference", payload)
        print(f"\n============================================================")
        print(f"  [SUCCESS] INFERENCE REQUEST SUBMITTED TO BLACKCHAIN")
        print(f"============================================================")
        print(f"  Request ID : {resp.get('request_id')}")
        print(f"  Status     : {resp.get('status')}")
        print(f"============================================================")
        print("Compute providers on the P2P network will now execute this prompt,")
        print("produce cryptographic execution receipts, and mine the next block.")

    except urllib.error.URLError as e:
        print(f"[!] Error connecting to node at {rpc_url}: {e}")
        print("    Make sure 'cargo run --bin blackchain-node' is running first!")

if __name__ == "__main__":
    main()
