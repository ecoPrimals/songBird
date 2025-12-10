#!/usr/bin/env bash
# 🎵 Start a Songbird Tower for Federation
# Designed for multi-machine mesh formation

set -e

# Configuration via environment variables
SONGBIRD_PORT="${SONGBIRD_PORT:-8000}"
SONGBIRD_NODE_ID="${SONGBIRD_NODE_ID:-songbird-$(hostname)}"
SONGBIRD_FEDERATION="${SONGBIRD_FEDERATION:-true}"
SONGBIRD_PEERS="${SONGBIRD_PEERS:-}"  # Comma-separated list of seed peers
SONGBIRD_BIND="${SONGBIRD_BIND:-0.0.0.0}"  # Bind to all interfaces for multi-machine

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEDERATION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$FEDERATION_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/songbird-orchestrator"

echo "🎵 Starting Songbird Tower"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found: $BINARY"
    echo
    echo "Please build first:"
    echo "  cd $PROJECT_ROOT"
    echo "  cargo build --release --bin songbird-orchestrator"
    exit 1
fi

# Display configuration
echo "📝 Configuration:"
echo "  Node ID:     $SONGBIRD_NODE_ID"
echo "  Port:        $SONGBIRD_PORT"
echo "  Bind Address: $SONGBIRD_BIND"
echo "  Federation:  $SONGBIRD_FEDERATION"
if [ -n "$SONGBIRD_PEERS" ]; then
    echo "  Seed Peers:  $SONGBIRD_PEERS"
else
    echo "  Seed Peers:  (none - will be discoverable seed node)"
fi
echo

# Get local IP addresses for reference
echo "📡 Local IP Addresses:"
if command -v ip &> /dev/null; then
    ip -4 addr show | grep inet | grep -v 127.0.0.1 | awk '{print "  " $2}' || echo "  (unable to detect)"
elif command -v ifconfig &> /dev/null; then
    ifconfig | grep "inet " | grep -v 127.0.0.1 | awk '{print "  " $2}' || echo "  (unable to detect)"
else
    echo "  (ip/ifconfig not available)"
fi
echo

# Create logs directory
mkdir -p "$FEDERATION_DIR/logs"
LOG_FILE="$FEDERATION_DIR/logs/$SONGBIRD_NODE_ID.log"

echo "📊 Log file: $LOG_FILE"
echo

# Check if port is already in use
if lsof -i :$SONGBIRD_PORT -sTCP:LISTEN > /dev/null 2>&1; then
    echo "⚠️  WARNING: Port $SONGBIRD_PORT is already in use!"
    echo
    lsof -i :$SONGBIRD_PORT -sTCP:LISTEN
    echo
    read -p "Kill existing process and continue? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        PID=$(lsof -i :$SONGBIRD_PORT -sTCP:LISTEN | tail -1 | awk '{print $2}')
        echo "Killing PID $PID..."
        kill $PID
        sleep 2
    else
        echo "Exiting..."
        exit 1
    fi
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Songbird..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Export environment variables
export RUST_LOG="${RUST_LOG:-info}"
export SONGBIRD_PORT
export SONGBIRD_NODE_ID
export SONGBIRD_FEDERATION
export SONGBIRD_BIND
if [ -n "$SONGBIRD_PEERS" ]; then
    export SONGBIRD_PEERS
fi

# Start Songbird in background
"$BINARY" > "$LOG_FILE" 2>&1 &
PID=$!

echo "📝 Process ID: $PID"
echo "⏳ Waiting for startup..."
echo

# Wait for health check
STARTED=false
for i in {1..30}; do
    sleep 1
    if curl -s http://localhost:$SONGBIRD_PORT/health > /dev/null 2>&1; then
        STARTED=true
        break
    fi
    echo -n "."
done
echo

if [ "$STARTED" = true ]; then
    echo "✅ Songbird is ready!"
    echo
    
    HEALTH=$(curl -s http://localhost:$SONGBIRD_PORT/health)
    echo "📊 Health: $HEALTH"
    echo
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ Tower Started Successfully!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo
    echo "🔗 Endpoints:"
    echo "  Health:   http://localhost:$SONGBIRD_PORT/health"
    echo "  Services: http://localhost:$SONGBIRD_PORT/api/v1/services"
    echo
    echo "📊 Logs:"
    echo "  tail -f $LOG_FILE"
    echo
    echo "🛑 To stop:"
    echo "  kill $PID"
    echo "  OR: killall songbird-orchestrator"
    echo
    
    # Show initial log output
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📝 Recent Log Output:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    tail -20 "$LOG_FILE"
    echo
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "💡 For other machines to connect to this tower:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  SONGBIRD_PEERS=\"$(hostname -I | awk '{print $1}'):$SONGBIRD_PORT\" ./start-tower.sh"
    echo
else
    echo "❌ Failed to start Songbird"
    echo
    echo "Check logs:"
    echo "  tail -100 $LOG_FILE"
    echo
    exit 1
fi

