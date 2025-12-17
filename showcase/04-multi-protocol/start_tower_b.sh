#!/bin/bash
# Start Tower B with all protocols enabled (different ports)

echo "🚀 Starting Tower B (Secondary) with Multi-Protocol Support"
echo ""
echo "Protocols:"
echo "  • HTTP:     http://localhost:9080"
echo "  • HTTPS:    https://localhost:9443"
echo "  • JSON-RPC: https://localhost:9443/jsonrpc"
echo "  • tarpc:    tarpc://localhost:9081"
echo ""

export SONGBIRD_PORT=9080
export SONGBIRD_TLS_PORT=9443
export SONGBIRD_TARPC_PORT=9081
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TARPC_ENABLED=true
export SONGBIRD_JSONRPC_ENABLED=true
export SONGBIRD_NODE_NAME="tower-b"
export SONGBIRD_NODE_ID="tower-b-001"

# Optional: Enable BTSP for BearDog integration
export SONGBIRD_BTSP_ENABLED=true
export SONGBIRD_BTSP_LOCAL_FALLBACK=true

# Start Songbird
cd "$(dirname "$0")/../.."
cargo run --release --bin songbird-orchestrator

