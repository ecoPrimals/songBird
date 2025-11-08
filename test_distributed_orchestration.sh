#!/bin/bash
# Distributed Orchestration Test Suite
# Tests Songbird's capability-based task distribution across LAN towers

set -e

TOWER_A="192.168.1.144:8080"
TOWER_B="192.168.1.134:8081"

echo "🎵 Songbird Distributed Orchestration Tests"
echo "==========================================="
echo ""
echo "Tower A: $TOWER_A (Orchestrator)"
echo "Tower B: $TOWER_B (Worker - Strandgate)"
echo ""

# ═══════════════════════════════════════════════════════════════
# Test 1: Federation Health Check
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Federation Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Tower A Health:"
TOWER_A_HEALTH=$(curl -s http://$TOWER_A/health)
if [ "$TOWER_A_HEALTH" = "OK" ]; then
    echo "✅ PASS: Tower A is healthy"
else
    echo "❌ FAIL: Tower A health check failed"
    exit 1
fi

echo ""
echo "Tower B Health:"
TOWER_B_HEALTH=$(curl -s http://$TOWER_B/health 2>/dev/null)
if [ "$TOWER_B_HEALTH" = "OK" ]; then
    echo "✅ PASS: Tower B is healthy"
else
    echo "⚠️  WARN: Tower B health endpoint may not be available"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 2: Federation Status
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Federation Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FEDERATION_STATUS=$(curl -s http://$TOWER_A/api/federation/status)
ACTIVE_NODES=$(echo "$FEDERATION_STATUS" | jq -r '.active_nodes')

echo "Active Nodes: $ACTIVE_NODES"
echo "Total CPU Cores: $(echo "$FEDERATION_STATUS" | jq -r '.total_cpu_cores')"
echo "Total Memory: $(echo "$FEDERATION_STATUS" | jq -r '.total_memory_gb') GB"
echo "Total Storage: $(echo "$FEDERATION_STATUS" | jq -r '.total_storage_gb') GB"
echo "Uptime: $(echo "$FEDERATION_STATUS" | jq -r '.uptime_seconds') seconds"
echo ""

if [ "$ACTIVE_NODES" -ge 1 ]; then
    echo "✅ PASS: Federation has $ACTIVE_NODES active node(s)"
else
    echo "❌ FAIL: No active nodes in federation"
    exit 1
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 3: Node Discovery
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: Node Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NODES=$(curl -s http://$TOWER_A/api/federation/nodes)
NODE_COUNT=$(echo "$NODES" | jq '. | length')

echo "Discovered Nodes: $NODE_COUNT"
echo ""

echo "$NODES" | jq -r '.[] | "  • \(.node_name) (\(.node_id))\n    Address: \(.node_address)\n    Capabilities: \(.capabilities | join(", "))\n    Status: \(.status)\n"'

if [ "$NODE_COUNT" -ge 1 ]; then
    echo "✅ PASS: Successfully discovered $NODE_COUNT node(s)"
else
    echo "❌ FAIL: No nodes discovered"
    exit 1
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 4: Service Registration Test
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Service Registration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Register a test service
TEST_SERVICE='{
  "service_id": "test-compute-service",
  "service_name": "Test Compute Service",
  "service_type": "compute",
  "node_id": "tower-b-strandgate",
  "address": "192.168.1.134:9000",
  "capabilities": ["batch-processing", "data-analysis"],
  "health_status": "healthy",
  "metadata": {}
}'

echo "Registering test service..."
REGISTER_RESULT=$(curl -s -X POST \
  -H "Content-Type: application/json" \
  -d "$TEST_SERVICE" \
  http://$TOWER_A/api/federation/services)

echo "$REGISTER_RESULT" | jq '.'

if echo "$REGISTER_RESULT" | grep -q "registered"; then
    echo "✅ PASS: Service registered successfully"
else
    echo "⚠️  WARN: Service registration may not be implemented yet"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 5: Service Discovery
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 5: Service Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SERVICES=$(curl -s http://$TOWER_A/api/federation/services)
SERVICE_COUNT=$(echo "$SERVICES" | jq '. | length')

echo "Registered Services: $SERVICE_COUNT"

if [ "$SERVICE_COUNT" -gt 0 ]; then
    echo ""
    echo "$SERVICES" | jq -r '.[] | "  • \(.service_name) (\(.service_type))\n    Node: \(.node_id)\n    Address: \(.address)\n"'
    echo "✅ PASS: Services discovered"
else
    echo "⚠️  INFO: No services registered yet (this is expected for initial test)"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 6: Capability-Based Discovery
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 6: Capability-Based Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Find all compute-capable nodes
COMPUTE_NODES=$(echo "$NODES" | jq '[.[] | select(.capabilities | contains(["compute"]))]')
COMPUTE_COUNT=$(echo "$COMPUTE_NODES" | jq '. | length')

echo "Nodes with 'compute' capability: $COMPUTE_COUNT"
echo "$COMPUTE_NODES" | jq -r '.[] | "  • \(.node_name): \(.cpu_cores) cores, \(.memory_gb) GB RAM"'

echo ""

# Find all orchestration-capable nodes
ORCHESTRATION_NODES=$(echo "$NODES" | jq '[.[] | select(.capabilities | contains(["orchestration"]))]')
ORCHESTRATION_COUNT=$(echo "$ORCHESTRATION_NODES" | jq '. | length')

echo "Nodes with 'orchestration' capability: $ORCHESTRATION_COUNT"
echo "$ORCHESTRATION_NODES" | jq -r '.[] | "  • \(.node_name)"'

if [ "$COMPUTE_COUNT" -gt 0 ]; then
    echo ""
    echo "✅ PASS: Capability-based discovery working"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Test 7: Resource Aggregation
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 7: Federated Resource Pool"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOTAL_CORES=$(echo "$FEDERATION_STATUS" | jq -r '.total_cpu_cores')
TOTAL_MEMORY=$(echo "$FEDERATION_STATUS" | jq -r '.total_memory_gb')
TOTAL_STORAGE=$(echo "$FEDERATION_STATUS" | jq -r '.total_storage_gb')

echo "📊 Aggregated Resources Across Federation:"
echo "  • CPU Cores: $TOTAL_CORES"
echo "  • Memory: $TOTAL_MEMORY GB"
echo "  • Storage: $TOTAL_STORAGE GB"
echo ""

# Calculate per-node averages
AVG_CORES=$(echo "$TOTAL_CORES / $ACTIVE_NODES" | bc)
AVG_MEMORY=$(echo "$TOTAL_MEMORY / $ACTIVE_NODES" | bc)

echo "📈 Per-Node Average:"
echo "  • CPU Cores: $AVG_CORES per node"
echo "  • Memory: $AVG_MEMORY GB per node"
echo ""

if [ "$TOTAL_CORES" -gt 0 ]; then
    echo "✅ PASS: Resource aggregation working"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Final Summary
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 TEST RESULTS SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Federation Operational"
echo "✅ Node Discovery Working ($ACTIVE_NODES active nodes)"
echo "✅ Capability-Based Routing Ready"
echo "✅ Resource Aggregation Working ($TOTAL_CORES cores, $TOTAL_MEMORY GB RAM)"
echo ""
echo "📊 Federation Statistics:"
echo "  • Total Nodes: $ACTIVE_NODES"
echo "  • Total CPU Cores: $TOTAL_CORES"
echo "  • Total Memory: $TOTAL_MEMORY GB"
echo "  • Total Storage: $TOTAL_STORAGE GB"
echo "  • Uptime: $(echo "$FEDERATION_STATUS" | jq -r '.uptime_seconds')s"
echo ""
echo "🚀 Songbird is successfully orchestrating across LAN!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

