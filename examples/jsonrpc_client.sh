#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# JSON-RPC 2.0 Client Examples for Songbird
# Uses current wire method names from songbird-types JsonRpcMethod

BASE_URL="${SONGBIRD_URL:-http://localhost:8080}"

echo "Songbird JSON-RPC 2.0 Client Examples"
echo "======================================"
echo ""

call_jsonrpc() {
    local method=$1
    local params=${2:-"{}"}
    
    curl -s -X POST "$BASE_URL/jsonrpc" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"$method\",
            \"params\": $params,
            \"id\": 1
        }" | jq '.'
}

echo "1. Health Liveness:"
call_jsonrpc "health.liveness"
echo ""

echo "2. Health Readiness:"
call_jsonrpc "health.readiness"
echo ""

echo "3. Health Check:"
call_jsonrpc "health.check"
echo ""

echo "4. Identity:"
call_jsonrpc "identity.get"
echo ""

echo "5. Capabilities List (Wire Standard L3):"
call_jsonrpc "capabilities.list"
echo ""

echo "6. Capabilities Methods:"
call_jsonrpc "capabilities.methods"
echo ""

echo "7. Songbird Version:"
call_jsonrpc "songbird.version"
echo ""

echo "8. Discover Services by Capability:"
call_jsonrpc "ipc.find_capability" '{"capability": "compute"}'
echo ""

echo "9. List Registered Services:"
call_jsonrpc "songbird.services.list"
echo ""

echo "10. Protocol Negotiation:"
call_jsonrpc "protocol.negotiate" '{"desired_protocol": "tarpc", "peer_id": "tower-2"}'
echo ""

echo "All examples complete!"

