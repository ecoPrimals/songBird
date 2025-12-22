#!/usr/bin/env bash
set -euo pipefail

# Test script for rendezvous server

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🌍 Testing Songbird Rendezvous Server"
echo ""

# Build
echo "Building..."
cargo build --release --quiet

# Start server in background
echo "Starting server..."
cargo run --release &
SERVER_PID=$!
sleep 2

# Cleanup on exit
cleanup() {
    echo ""
    echo "Stopping server..."
    kill $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test health
echo ""
echo "Test 1: Health Check"
RESPONSE=$(curl -s http://localhost:8888/health)
if [ "$RESPONSE" = "OK" ]; then
    echo "✅ Health check passed"
else
    echo "❌ Health check failed"
    exit 1
fi

# Test registration
echo ""
echo "Test 2: Node Registration"
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "register_presence",
    "version": "1.0",
    "timestamp": "2025-12-21T23:00:00Z",
    "node_identity": {
      "node_id": "550e8400-e29b-41d4-a716-446655440000",
      "ephemeral_session_id": "",
      "public_key_fingerprint": "sha256:abc123",
      "capabilities": ["orchestration", "federation"],
      "protocols": ["https", "btsp"]
    },
    "network_context": {
      "nat_type": "cone",
      "reachability": "direct",
      "connection_quality": "excellent"
    },
    "security": {
      "signature": null
    }
  }')

SESSION_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.session_id')
if [ -n "$SESSION_ID" ] && [ "$SESSION_ID" != "null" ]; then
    echo "✅ Registration successful: $SESSION_ID"
else
    echo "❌ Registration failed"
    echo "$REGISTER_RESPONSE"
    exit 1
fi

# Test heartbeat
echo ""
echo "Test 3: Heartbeat"
HEARTBEAT_STATUS=$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:8888/api/v1/heartbeat \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"timestamp\": \"2025-12-21T23:01:00Z\",
    \"signature\": null
  }")

if [ "$HEARTBEAT_STATUS" = "200" ]; then
    echo "✅ Heartbeat successful"
else
    echo "❌ Heartbeat failed: $HEARTBEAT_STATUS"
    exit 1
fi

# Test query
echo ""
echo "Test 4: Peer Query"
QUERY_RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "message_type": "query_peers",
    "version": "1.0",
    "timestamp": "2025-12-21T23:02:00Z",
    "requester": {
      "session_id": "'$SESSION_ID'",
      "signature": null
    },
    "query": {
      "capabilities_required": ["orchestration"],
      "capabilities_optional": [],
      "exclude_node_ids": [],
      "max_results": 10
    },
    "filters": null
  }')

PEER_COUNT=$(echo "$QUERY_RESPONSE" | jq '.peers | length')
if [ "$PEER_COUNT" -ge "1" ]; then
    echo "✅ Query successful: found $PEER_COUNT peer(s)"
else
    echo "❌ Query failed"
    exit 1
fi

# Test get peer info
echo ""
echo "Test 5: Get Peer Info"
PEER_INFO=$(curl -s http://localhost:8888/api/v1/peers/$SESSION_ID)
PEER_SESSION=$(echo "$PEER_INFO" | jq -r '.ephemeral_session_id')
if [ "$PEER_SESSION" = "$SESSION_ID" ]; then
    echo "✅ Get peer info successful"
else
    echo "❌ Get peer info failed"
    exit 1
fi

echo ""
echo "═══════════════════════════════════════"
echo "✅ All tests passed!"
echo "═══════════════════════════════════════"
echo ""
echo "Rendezvous server is working correctly!"

