#!/usr/bin/env bash
# 🎵 Demo 2: Connect to Remote Tower
# Connect a local Songbird instance to a remote tower

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FEDERATION_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 Demo 2: Connect to Remote Tower"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "📝 This demo connects your local Songbird to a remote tower"
echo
echo "Prerequisites:"
echo "  • Another tower running Songbird"
echo "  • Network connectivity to that tower"
echo "  • Know the remote tower's IP and port"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Get remote tower details
if [ -z "$REMOTE_TOWER" ]; then
    echo "Enter remote tower address (e.g., 192.168.1.134:8000):"
    read -p "Remote tower: " REMOTE_TOWER
fi

if [ -z "$REMOTE_TOWER" ]; then
    echo "❌ No remote tower specified"
    echo
    echo "Usage:"
    echo "  REMOTE_TOWER=192.168.1.134:8000 $0"
    echo "  OR run interactively and enter when prompted"
    exit 1
fi

echo
echo "Testing connectivity to remote tower: $REMOTE_TOWER"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Extract host and port
REMOTE_HOST=$(echo $REMOTE_TOWER | cut -d: -f1)
REMOTE_PORT=$(echo $REMOTE_TOWER | cut -d: -f2)

# Test connectivity
echo "1️⃣  Testing TCP connectivity..."
if timeout 5 bash -c "cat < /dev/null > /dev/tcp/$REMOTE_HOST/$REMOTE_PORT" 2>/dev/null; then
    echo "   ✅ TCP connection successful"
else
    echo "   ❌ Cannot connect to $REMOTE_HOST:$REMOTE_PORT"
    echo
    echo "Troubleshooting:"
    echo "  • Is the remote tower running?"
    echo "  • Check firewall rules"
    echo "  • Verify the IP address and port"
    exit 1
fi

echo
echo "2️⃣  Testing HTTP health endpoint..."
if curl -s --max-time 5 "http://$REMOTE_TOWER/health" > /dev/null 2>&1; then
    HEALTH=$(curl -s "http://$REMOTE_TOWER/health")
    echo "   ✅ Remote tower is healthy: $HEALTH"
else
    echo "   ⚠️  Could not reach health endpoint"
    echo "   (Tower may be running but endpoint not available)"
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting local tower connected to remote"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Clean up any existing local instance
killall -q songbird-orchestrator 2>/dev/null || true
sleep 1

# Start local tower with remote peer
LOCAL_PORT="${LOCAL_PORT:-8001}"

echo "Starting local tower on port $LOCAL_PORT..."
echo "Connecting to remote peer: $REMOTE_TOWER"
echo

cd "$FEDERATION_DIR/scripts"
SONGBIRD_PORT=$LOCAL_PORT \
SONGBIRD_NODE_ID="local-tower" \
SONGBIRD_PEERS="$REMOTE_TOWER" \
./start-tower.sh

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Connected to Remote Tower!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "📊 Federation Status:"
echo "  Local Tower:  localhost:$LOCAL_PORT"
echo "  Remote Tower: $REMOTE_TOWER"
echo
echo "🔍 Test the connection:"
echo "  curl http://localhost:$LOCAL_PORT/health"
echo "  curl http://$REMOTE_TOWER/health"
echo
echo "📝 View logs:"
echo "  tail -f $FEDERATION_DIR/logs/local-tower.log"
echo

