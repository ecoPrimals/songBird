#!/bin/bash
# Start Tower A with all protocols enabled

echo "🚀 Starting Tower A (Primary) with Multi-Protocol Support"
echo ""
echo "Protocols:"
echo "  • HTTP:     http://localhost:8080"
echo "  • HTTPS:    https://localhost:8443"
echo "  • JSON-RPC: https://localhost:8443/jsonrpc"
echo "  • tarpc:    tarpc://localhost:8081"
echo ""

export SONGBIRD_PORT=8080
export SONGBIRD_TLS_PORT=8443
export SONGBIRD_TARPC_PORT=8081
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TARPC_ENABLED=true
export SONGBIRD_JSONRPC_ENABLED=true
export SONGBIRD_NODE_NAME="tower-a"
export SONGBIRD_NODE_ID="tower-a-001"

# Optional: Enable BTSP for BearDog integration
export SONGBIRD_BTSP_ENABLED=true
export SONGBIRD_BTSP_LOCAL_FALLBACK=true

# Start Songbird
cd "$(dirname "$0")/../.."
cargo run --release --bin songbird-orchestrator

