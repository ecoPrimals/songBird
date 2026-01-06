#!/bin/bash
# 🎭 Songbird Localhost Fractal Demo
# Demonstrates 1 Albatross + 3 Songbirds + 10 Sparrows on a single machine

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY_PATH="/home/eastgate/Development/ecoPrimals/primalBins/songbird-orchestrator-v3.7.3-multiinstance"
FAMILY_ID="demo-fractal"
MULTICAST="239.255.42.99:4242"
LOG_DIR="/tmp/songbird-demo-logs"
SOCKET_DIR="/tmp"

# Colors for pretty output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                              ║"
echo "║            🎭 SONGBIRD LOCALHOST FRACTAL DEMO 🎭                             ║"
echo "║                                                                              ║"
echo "║   Deploying: 1 Albatross + 3 Songbirds + 10 Sparrows                         ║"
echo "║   Total: 14 nodes on your laptop!                                            ║"
echo "║                                                                              ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}❌ Binary not found: $BINARY_PATH${NC}"
    echo "Please build Songbird first:"
    echo "  cd /home/eastgate/Development/ecoPrimals/phase1/songbird"
    echo "  cargo build --release"
    exit 1
fi

# Cleanup previous run
echo -e "${YELLOW}🧹 Cleaning up previous demo...${NC}"
pkill -f "songbird-orchestrator" 2>/dev/null || true
sleep 2
rm -rf "$LOG_DIR"
mkdir -p "$LOG_DIR"
rm -f /tmp/songbird-${FAMILY_ID}-*.sock 2>/dev/null || true

echo -e "${GREEN}✅ Cleanup complete${NC}"
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ALBATROSS (High-capacity hub)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "${BLUE}🦅 Starting Albatross (core hub)...${NC}"

export SONGBIRD_FAMILY_ID="$FAMILY_ID"
export SONGBIRD_NODE_ID="albatross-main"
export SONGBIRD_MULTICAST_ADDR="$MULTICAST"
export SONGBIRD_CAPABILITIES="coordinator,multiplexer,load-balancer"
export SONGBIRD_MAX_CONNECTIONS=100
export SONGBIRD_WORKER_THREADS=4
export SONGBIRD_PORT=8080

"$BINARY_PATH" > "$LOG_DIR/albatross-main.log" 2>&1 &
ALBATROSS_PID=$!
echo -e "  ✅ Albatross started (PID: $ALBATROSS_PID, Port: 8080)"

sleep 2

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SONGBIRDS (Regional coordinators)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "${BLUE}🎵 Starting 3 Songbirds (regional towers)...${NC}"

for i in {1..3}; do
    export SONGBIRD_NODE_ID="songbird-tower-$i"
    export SONGBIRD_CAPABILITIES="orchestrator,federation-member,discovery"
    export SONGBIRD_MAX_CONNECTIONS=20
    export SONGBIRD_WORKER_THREADS=2
    export SONGBIRD_PORT=$((8080 + i))
    
    "$BINARY_PATH" > "$LOG_DIR/songbird-tower-$i.log" 2>&1 &
    echo -e "  ✅ Songbird $i started (PID: $!, Port: $((8080 + i)))"
done

sleep 2

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SPARROWS (Edge sensors)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo -e "${BLUE}🐦 Starting 10 Sparrows (edge sensors)...${NC}"

for i in {1..10}; do
    export SONGBIRD_NODE_ID="sparrow-sensor-$(printf "%03d" $i)"
    export SONGBIRD_CAPABILITIES="sensor,temperature,humidity,edge-node"
    export SONGBIRD_MAX_CONNECTIONS=5
    export SONGBIRD_WORKER_THREADS=1
    export SONGBIRD_PORT=$((8090 + i))
    
    "$BINARY_PATH" > "$LOG_DIR/sparrow-sensor-$(printf "%03d" $i).log" 2>&1 &
    
    # Print progress every 2 sparrows
    if [ $((i % 2)) -eq 0 ]; then
        echo -e "  ✅ Sparrows 1-$i started..."
    fi
done

echo -e "  ✅ All 10 Sparrows started (Ports: 8091-8100)"

echo ""
echo -e "${YELLOW}⏳ Waiting for mesh formation (15 seconds)...${NC}"
echo "   Nodes are discovering each other via encrypted multicast..."
sleep 15

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                              ║"
echo "║                     ✅ FRACTAL MESH DEPLOYED! ✅                              ║"
echo "║                                                                              ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}📊 Deployment Summary:${NC}"
echo "   • 1 Albatross   (hub)        - PID: $ALBATROSS_PID"
echo "   • 3 Songbirds   (towers)     - PIDs: $(pgrep -f 'songbird-tower' | tr '\n' ' ')"
echo "   • 10 Sparrows   (sensors)    - PIDs: $(pgrep -f 'sparrow-sensor' | head -5 | tr '\n' ' ')..."
echo "   • Total: 14 nodes"
echo ""
echo -e "${GREEN}🔍 Verification:${NC}"

# Query Albatross for discovered peers
ALBATROSS_SOCKET="/tmp/songbird-${FAMILY_ID}-albatross-main.sock"

if [ -S "$ALBATROSS_SOCKET" ]; then
    echo "   Querying Albatross for discovered peers..."
    PEER_COUNT=$(echo '{"jsonrpc":"2.0","method":"primal.list_all","id":1}' | \
        nc -U "$ALBATROSS_SOCKET" 2>/dev/null | \
        jq -r '.result.total_primals // 0' 2>/dev/null || echo "0")
    
    echo -e "   ${GREEN}✅ Albatross discovered: $PEER_COUNT peers${NC}"
    
    if [ "$PEER_COUNT" -ge 10 ]; then
        echo -e "   ${GREEN}✅ Mesh formation successful!${NC}"
    else
        echo -e "   ${YELLOW}⚠️  Still forming... (expected 14, got $PEER_COUNT)${NC}"
        echo "      Give it a few more seconds and run: ./query-mesh.sh"
    fi
else
    echo -e "   ${YELLOW}⚠️  Albatross socket not ready yet, try: ./query-mesh.sh${NC}"
fi

echo ""
echo -e "${BLUE}📂 Logs:${NC}"
echo "   $LOG_DIR/"
echo ""
echo -e "${BLUE}🎯 Next Steps:${NC}"
echo "   1. Query mesh status:"
echo "      ./query-mesh.sh"
echo ""
echo "   2. View topology:"
echo "      ./visualize-topology.sh"
echo ""
echo "   3. Watch live logs:"
echo "      tail -f $LOG_DIR/*.log"
echo ""
echo "   4. Stop demo:"
echo "      ./stop-demo.sh"
echo ""
echo -e "${GREEN}🎊 Demo is running! Explore the fractal mesh! 🦅🎵🐦${NC}"
echo ""

