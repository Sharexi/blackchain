#!/usr/bin/env bash
set -e

echo "============================================================"
echo "     Starting Blackchain Inference-PoUW Node Daemon         "
echo "============================================================"

cargo run --release --bin blackchain-node -- "$@"
