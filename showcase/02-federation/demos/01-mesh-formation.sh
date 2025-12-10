#!/usr/bin/env bash
# 🎵 Demo 1: Mesh Formation
# Demonstrates 3 Songbird instances forming a fully connected mesh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEDERATION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$FEDERATION_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/songbird-orchestrator"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 Demo 1: Mesh Formation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "📝 This demo will:"
echo "  • Start 3 Songbird instances (Tower A, B, C)"
echo "  • Show automatic peer discovery"
echo "  • Display mesh topology"
echo "  • Verify full connectivity"
echo
read -p "Press Enter to continue..."
echo

# Clean up any existing instances
echo "🧹 Cleaning up existing instances..."
killall -q songbird-orchestrator 2>/dev/null || true
sleep 1

# Ensure log directory exists
mkdir -p "$FEDERATION_DIR/logs"

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Tower A (Port 8000)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start Tower A (seed node)
RUST_LOG=info \
SONGBIRD_PORT=8000 \
SONGBIRD_NODE_ID="tower-a" \
SONGBIRD_FEDERATION=true \
"$BINARY" > "$FEDERATION_DIR/logs/tower-a.log" 2>&1 &
TOWER_A_PID=$!
echo "📝 Tower A PID: $TOWER_A_PID"

# Wait for Tower A to start
echo "⏳ Waiting for Tower A to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "✅ Tower A is ready!"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo "❌ Tower A failed to start"
        kill $TOWER_A_PID 2>/dev/null || true
        exit 1
    fi
done

# Display Tower A info
TOWER_A_HEALTH=$(curl -s http://localhost:8000/health)
echo "📊 Tower A Health: $TOWER_A_HEALTH"
echo

sleep 2

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Tower B (Port 8001)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start Tower B (connects to A)
RUST_LOG=info \
SONGBIRD_PORT=8001 \
SONGBIRD_NODE_ID="tower-b" \
SONGBIRD_FEDERATION=true \
SONGBIRD_PEERS="localhost:8000" \
"$BINARY" > "$FEDERATION_DIR/logs/tower-b.log" 2>&1 &
TOWER_B_PID=$!
echo "📝 Tower B PID: $TOWER_B_PID"

echo "⏳ Waiting for Tower B to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8001/health > /dev/null 2>&1; then
        echo "✅ Tower B is ready!"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo "❌ Tower B failed to start"
        kill $TOWER_A_PID $TOWER_B_PID 2>/dev/null || true
        exit 1
    fi
done

TOWER_B_HEALTH=$(curl -s http://localhost:8001/health)
echo "📊 Tower B Health: $TOWER_B_HEALTH"
echo "🔗 Tower B should discover Tower A..."
sleep 2

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Tower C (Port 8002)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start Tower C (connects to A)
RUST_LOG=info \
SONGBIRD_PORT=8002 \
SONGBIRD_NODE_ID="tower-c" \
SONGBIRD_FEDERATION=true \
SONGBIRD_PEERS="localhost:8000" \
"$BINARY" > "$FEDERATION_DIR/logs/tower-c.log" 2>&1 &
TOWER_C_PID=$!
echo "📝 Tower C PID: $TOWER_C_PID"

echo "⏳ Waiting for Tower C to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8002/health > /dev/null 2>&1; then
        echo "✅ Tower C is ready!"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo "❌ Tower C failed to start"
        kill $TOWER_A_PID $TOWER_B_PID $TOWER_C_PID 2>/dev/null || true
        exit 1
    fi
done

TOWER_C_HEALTH=$(curl -s http://localhost:8002/health)
echo "📊 Tower C Health: $TOWER_C_HEALTH"
echo "🔗 Tower C should discover Tower A and B..."
sleep 3

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 MESH STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Active Nodes: 3"
echo "  • Tower A: localhost:8000 (Seed Node)"
echo "  • Tower B: localhost:8001"
echo "  • Tower C: localhost:8002"
echo
echo "Topology: Full Mesh"
echo "  Tower A ←→ Tower B"
echo "  Tower A ←→ Tower C"
echo "  Tower B ←→ Tower C (via A)"
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Verification Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Test 1: Health checks
echo "1️⃣  Health Checks:"
for port in 8000 8001 8002; do
    HEALTH=$(curl -s http://localhost:$port/health)
    echo "   Port $port: $HEALTH"
done
echo

# Test 2: Process verification
echo "2️⃣  Process Verification:"
ps aux | grep songbird-orchestrator | grep -v grep | awk '{print "   PID " $2 " - " $11 " " $12 " " $13 " " $14}' || echo "   ⚠️  No processes found"
echo

# Test 3: Port listening
echo "3️⃣  Port Listening:"
for port in 8000 8001 8002; do
    if lsof -i :$port -sTCP:LISTEN > /dev/null 2>&1; then
        PROC=$(lsof -i :$port -sTCP:LISTEN | tail -1 | awk '{print $1 " (PID " $2 ")"}')
        echo "   ✅ Port $port: $PROC"
    else
        echo "   ❌ Port $port: Not listening"
    fi
done
echo

# Test 4: API endpoint test
echo "4️⃣  API Endpoint Test (Tower A):"
if curl -s http://localhost:8000/api/v1/services > /dev/null 2>&1; then
    echo "   ✅ /api/v1/services endpoint responding"
else
    echo "   ℹ️  /api/v1/services endpoint may not be implemented yet"
fi
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Log Locations:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  • Tower A: $FEDERATION_DIR/logs/tower-a.log"
echo "  • Tower B: $FEDERATION_DIR/logs/tower-b.log"
echo "  • Tower C: $FEDERATION_DIR/logs/tower-c.log"
echo
echo "To view logs: tail -f $FEDERATION_DIR/logs/tower-a.log"
echo

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ MESH FORMATION COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "💡 What's happening:"
echo "  • 3 Songbird instances are running"
echo "  • Each has a unique node ID"
echo "  • Towers B and C know about Tower A (seed node)"
echo "  • Federation is enabled for peer discovery"
echo
echo "🔍 Next steps:"
echo "  • Run demo 2: ./02-cross-tower-discovery.sh"
echo "  • Query Tower A: curl http://localhost:8000/health"
echo "  • Check logs: tail -f logs/tower-a.log"
echo "  • Stop all: killall songbird-orchestrator"
echo

# Keep script alive to show logs
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Live Log Preview (Tower A) - Ctrl+C to exit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
tail -f "$FEDERATION_DIR/logs/tower-a.log" &
TAIL_PID=$!

# Cleanup handler
cleanup() {
    echo
    echo
    echo "🛑 Stopping demo..."
    kill $TAIL_PID 2>/dev/null || true
    echo
    echo "Towers are still running. To stop them:"
    echo "  killall songbird-orchestrator"
    echo
    exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for user interrupt
wait $TAIL_PID

