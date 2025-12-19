#!/bin/bash
# Deploy standalone tarpc servers to Strandgate for distributed benchmarking

set -e

STRANDGATE_URL="https://192.168.1.134:8081"
TARPC_BIN="../../../target/release/tarpc-server"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║     🚀 DEPLOYING tarpc SERVERS TO STRANDGATE 🚀                 ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
if [ ! -f "$TARPC_BIN" ]; then
    echo -e "${RED}❌ tarpc-server binary not found at $TARPC_BIN${NC}"
    echo "Building..."
    (cd ../tarpc-servers && cargo build --release)
fi

echo -e "${BLUE}Binary:${NC} $TARPC_BIN"
echo -e "${BLUE}Size:${NC}   $(du -h "$TARPC_BIN" | cut -f1)"
echo ""

# Deploy 3 tarpc servers
for port in 8091 8092 8093; do
    echo -e "${BLUE}[$(($port - 8090))/3]${NC} Deploying tarpc-server on port $port..."
    
    ../scripts/deploy_binary.sh \
        "$STRANDGATE_URL" \
        "$TARPC_BIN" \
        "tarpc-server-$port" \
        true
    
    # Give it a moment to start
    sleep 2
    
    echo ""
done

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║          ✨ DEPLOYMENT COMPLETE ✨                               ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Strandgate tarpc servers:"
echo "  • 192.168.1.134:8091"
echo "  • 192.168.1.134:8092"
echo "  • 192.168.1.134:8093"
echo ""
echo "To test:"
echo "  ./benchmark_distributed.sh"
echo ""

