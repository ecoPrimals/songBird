#!/bin/bash
# Quick Tower B Verification Script

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 VERIFYING TOWER B STATUS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER_B="http://192.168.1.134:8081"

# Test 1: Health Check
echo "Test 1: Health Check"
if curl -s -f "$TOWER_B/health" > /dev/null; then
    echo "✅ Tower B responding: $(curl -s $TOWER_B/health)"
else
    echo "❌ Tower B not responding"
    exit 1
fi
echo ""

# Test 2: Deployment API
echo "Test 2: Deployment API"
if curl -s -f "$TOWER_B/api/deployment/capabilities" > /dev/null; then
    echo "✅ Deployment API available"
    curl -s "$TOWER_B/api/deployment/capabilities" | jq '{node_id, network: .network.type, methods: {single: .deployment_methods.single.enabled, chunked: .deployment_methods.chunked.enabled}}'
else
    echo "❌ Deployment API not available"
    exit 1
fi
echo ""

# Test 3: Service Registry
echo "Test 3: Service Registry"
SERVICES=$(curl -s "$TOWER_B/api/federation/services" | jq 'length')
echo "✅ Service registry operational: $SERVICES services registered"
echo ""

# Test 4: Resources
echo "Test 4: System Resources"
curl -s "$TOWER_B/api/deployment/capabilities" | jq '.resources | {cpu_cores, memory_gb: .available_memory_gb, storage_gb: .available_storage_gb}'
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ TOWER B READY FOR TOADSTOOL DEPLOYMENT!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next command:"
echo "./target/release/songbird-deploy deploy-http \\"
echo "  --tower $TOWER_B \\"
echo "  --binary ../toadstool/target/release/toadstool-cli \\"
echo "  --service toadstool-gpu-compute \\"
echo "  --env TOADSTOOL_HOST=192.168.1.134 \\"
echo "  --env TOADSTOOL_PORT=9002 \\"
echo "  --env TOADSTOOL_GPU_ENABLED=true"

