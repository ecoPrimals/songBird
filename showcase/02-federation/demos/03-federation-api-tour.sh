#!/usr/bin/env bash
# 🎵 Demo 3: Federation API Tour
# Comprehensive demonstration of all federation capabilities

set -e

TOWER_A="${TOWER_A:-localhost:8080}"
TOWER_B="${TOWER_B:-192.168.1.134:8081}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 Federation API Tour"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Testing with:"
echo "  Tower A: $TOWER_A"
echo "  Tower B: $TOWER_B"
echo
read -p "Press Enter to begin the tour..."
echo

# Test 1: Health Checks
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  Basic Health Checks"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Tower A:"
HEALTH_A=$(curl -s http://$TOWER_A/health)
echo "  Status: $HEALTH_A"
echo
echo "Tower B:"
HEALTH_B=$(curl -s http://$TOWER_B/health 2>/dev/null || echo "⚠️ Unreachable")
echo "  Status: $HEALTH_B"
echo
read -p "Press Enter to continue..."
echo

# Test 2: Federation Status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  Federation Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Checking /api/federation/status..."
echo
STATUS_RESULT=$(curl -s http://$TOWER_A/api/federation/status 2>&1)
if echo "$STATUS_RESULT" | grep -q "html\|error\|404"; then
    echo "  ℹ️  Endpoint may not be fully implemented yet"
    echo "  Raw response:"
    echo "$STATUS_RESULT" | head -10
else
    echo "$STATUS_RESULT" | jq '.' 2>/dev/null || echo "$STATUS_RESULT"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 3: Federation Nodes
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  Federation Nodes"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Checking /api/federation/nodes..."
echo
NODES_RESULT=$(curl -s http://$TOWER_A/api/federation/nodes 2>&1)
if echo "$NODES_RESULT" | grep -q "html\|error\|404"; then
    echo "  ℹ️  Endpoint may not be fully implemented yet"
else
    echo "$NODES_RESULT" | jq '.' 2>/dev/null || echo "$NODES_RESULT"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 4: Services Registry
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  Services Registry"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Checking /api/federation/services..."
echo
SERVICES_RESULT=$(curl -s http://$TOWER_A/api/federation/services 2>&1)
if echo "$SERVICES_RESULT" | grep -q "html\|error\|404"; then
    echo "  ℹ️  No services registered yet"
else
    echo "$SERVICES_RESULT" | jq '.' 2>/dev/null || echo "$SERVICES_RESULT"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 5: JSON-RPC API
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  JSON-RPC 2.0 API"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Testing JSON-RPC health check..."
echo

JSONRPC_RESULT=$(curl -s -X POST http://$TOWER_A/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.health",
    "params": {},
    "id": 1
  }')

echo "$JSONRPC_RESULT" | jq '.' 2>/dev/null || echo "$JSONRPC_RESULT"
echo
read -p "Press Enter to continue..."
echo

# Test 6: Cross-Tower Communication
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  Cross-Tower Communication"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Testing Tower A → Tower B:"
echo -n "  "
if curl -s --max-time 3 http://$TOWER_B/health > /dev/null 2>&1; then
    CROSS_HEALTH=$(curl -s http://$TOWER_B/health)
    echo "✅ Success: $CROSS_HEALTH"
else
    echo "⚠️ Cannot reach Tower B from this location"
fi
echo
echo "Network latency:"
if ping -c 1 $(echo $TOWER_B | cut -d: -f1) > /dev/null 2>&1; then
    ping -c 3 $(echo $TOWER_B | cut -d: -f1) | tail -1
else
    echo "  ℹ️  Ping not available"
fi
echo
read -p "Press Enter to continue..."
echo

# Test 7: Deployment Capabilities
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7️⃣  Deployment API Capabilities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Checking /api/deployment/capabilities..."
echo
DEPLOY_CAPS=$(curl -s http://$TOWER_A/api/deployment/capabilities 2>&1)
echo "$DEPLOY_CAPS" | jq '.' 2>/dev/null || echo "$DEPLOY_CAPS" | head -20
echo
read -p "Press Enter to continue..."
echo

# Test 8: tarpc High-Performance Port
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8️⃣  tarpc High-Performance Port"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "Checking if tarpc port (8091) is listening..."
TARPC_A_PORT=$(echo $TOWER_A | cut -d: -f2)
TARPC_A_PORT=$((TARPC_A_PORT + 11))  # Default: 8080 + 11 = 8091
if lsof -i :$TARPC_A_PORT > /dev/null 2>&1; then
    echo "  ✅ tarpc server listening on port $TARPC_A_PORT"
    echo "  Protocol: Binary RPC (tarpc + bincode)"
    echo "  Use case: High-performance primal-to-primal communication"
else
    echo "  ℹ️  tarpc port not detected (may be on different port)"
fi
echo
read -p "Press Enter to continue..."
echo

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 FEDERATION API SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo
echo "✅ Available Endpoints:"
echo "  • /health - Basic health check"
echo "  • /api/federation/status - Federation status"
echo "  • /api/federation/nodes - List federated nodes"
echo "  • /api/federation/services - Service registry"
echo "  • /jsonrpc - JSON-RPC 2.0 universal gateway"
echo "  • /api/deployment/* - HTTP-based deployment"
echo "  • Port 8091 - tarpc binary RPC"
echo
echo "🔍 What You Can Do:"
echo "  1. Register services on one tower"
echo "  2. Discover services from another tower"
echo "  3. Deploy services via HTTP (no SSH needed)"
echo "  4. Monitor federation health"
echo "  5. Use JSON-RPC for language-agnostic access"
echo "  6. High-performance RPC via tarpc"
echo
echo "📝 Next Steps:"
echo "  • Try service registration demo"
echo "  • Test deployment API with actual service"
echo "  • Explore JSON-RPC methods"
echo "  • Add more towers to the mesh"
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎵 Federation API Tour Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

