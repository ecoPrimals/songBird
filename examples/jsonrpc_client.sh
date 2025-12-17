#!/bin/bash
# JSON-RPC 2.0 Client Examples for Songbird

BASE_URL="${SONGBIRD_URL:-http://localhost:8080}"

echo "🎼 Songbird JSON-RPC 2.0 Client Examples"
echo "========================================"
echo ""

# Helper function to call JSON-RPC
call_jsonrpc() {
    local method=$1
    local params=${2:-"[]"}
    
    curl -s -X POST "$BASE_URL/jsonrpc" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"method\": \"$method\",
            \"params\": $params,
            \"id\": 1
        }" | jq '.'
}

# 1. Get Songbird version
echo "1. Get Version:"
call_jsonrpc "songbird.version"
echo ""

# 2. Check health
echo "2. Health Check:"
call_jsonrpc "songbird.health"
echo ""

# 3. Get supported protocols
echo "3. Supported Protocols:"
call_jsonrpc "songbird.protocols"
echo ""

# 4. Discover services by capability
echo "4. Discover Services (compute capability):"
call_jsonrpc "songbird.discover" '["compute"]'
echo ""

# 5. Discover all services
echo "5. Discover All Services:"
call_jsonrpc "songbird.discoverAll"
echo ""

# 6. Register a service
echo "6. Register Service:"
call_jsonrpc "songbird.register" '[{
    "service_id": "test-service-1",
    "capability": "compute",
    "endpoint": "http://localhost:9001",
    "metadata": {"provider": "test"}
}]'
echo ""

# 7. Protocol negotiation
echo "7. Negotiate Protocol (upgrade to tarpc):"
call_jsonrpc "songbird.negotiateProtocol" '[{
    "desired_protocol": "tarpc",
    "peer_id": "tower-2"
}]'
echo ""

echo "✅ All examples complete!"

