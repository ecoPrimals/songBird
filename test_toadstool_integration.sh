#!/bin/bash
# Toadstool + Songbird Integration Test
# Tests distributed compute across towers

set -e

TOWER_A_SONGBIRD="192.168.1.144:8080"
TOWER_B_SONGBIRD="192.168.1.134:8081"

echo "🍄🎵 Toadstool + Songbird Integration Test"
echo "=========================================="
echo ""

# ═══════════════════════════════════════════════════════════════
# Phase 1: Check Prerequisites
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 1: Prerequisites Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check Songbird
echo "1️⃣ Checking Songbird status..."
SONGBIRD_HEALTH=$(curl -s http://$TOWER_A_SONGBIRD/health)
if [ "$SONGBIRD_HEALTH" = "OK" ]; then
    echo "✅ Songbird (Tower A) is running"
else
    echo "❌ Songbird (Tower A) is not responding"
    exit 1
fi

# Check Federation
FEDERATION_STATUS=$(curl -s http://$TOWER_A_SONGBIRD/api/federation/status)
ACTIVE_NODES=$(echo "$FEDERATION_STATUS" | jq -r '.active_nodes')
echo "✅ Federation active with $ACTIVE_NODES node(s)"
echo ""

# Check Toadstool (Tower A)
echo "2️⃣ Checking Toadstool services..."
TOADSTOOL_A_RUNNING=$(curl -s http://192.168.1.144:9000/health 2>/dev/null || echo "NOT_RUNNING")
if [ "$TOADSTOOL_A_RUNNING" = "NOT_RUNNING" ]; then
    echo "⚠️  Toadstool (Tower A) is not running on port 9000"
    echo "   Start with: cd ../toadstool && ./target/release/toadstool-server"
else
    echo "✅ Toadstool (Tower A) is running"
fi

# Check Toadstool (Tower B)
TOADSTOOL_B_RUNNING=$(curl -s http://192.168.1.134:9000/health 2>/dev/null || echo "NOT_RUNNING")
if [ "$TOADSTOOL_B_RUNNING" = "NOT_RUNNING" ]; then
    echo "⚠️  Toadstool (Tower B) is not running on port 9000"
    echo "   Start on Tower B first"
else
    echo "✅ Toadstool (Tower B) is running"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Phase 2: GPU Detection
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 2: GPU Detection"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Tower A GPU Detection:"
if command -v nvidia-smi &> /dev/null; then
    GPU_COUNT=$(nvidia-smi --query-gpu=count --format=csv,noheader | head -1 2>/dev/null || echo "0")
    if [ "$GPU_COUNT" -gt 0 ]; then
        GPU_MODEL=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -1)
        GPU_MEMORY=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -1)
        echo "  ✅ GPU: $GPU_MODEL"
        echo "  ✅ Memory: ${GPU_MEMORY}MB"
        echo "  ✅ Count: $GPU_COUNT"
    else
        echo "  ⚠️  No NVIDIA GPUs detected"
    fi
else
    echo "  ⚠️  nvidia-smi not found (no NVIDIA GPUs)"
fi

# Check for AMD GPUs
if command -v rocm-smi &> /dev/null; then
    echo "  ✅ AMD ROCm detected"
else
    echo "  ℹ️  No AMD ROCm GPUs detected"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Phase 3: Service Registration
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 3: Service Registration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$TOADSTOOL_A_RUNNING" != "NOT_RUNNING" ]; then
    echo "Registering Toadstool (Tower A)..."
    
    REGISTER_RESULT=$(curl -s -X POST http://$TOWER_A_SONGBIRD/api/federation/services \
      -H "Content-Type: application/json" \
      -d "{
        \"service_id\": \"toadstool-compute-tower-a\",
        \"service_name\": \"Toadstool Compute Service (Tower A)\",
        \"service_type\": \"compute\",
        \"tower_id\": \"tower-a-orchestrator\",
        \"tower_name\": \"Tower A\",
        \"endpoint\": \"http://192.168.1.144:9000\",
        \"capabilities\": [\"compute\", \"cpu\", \"batch-processing\"],
        \"metadata\": {
          \"cpu_cores\": \"16\",
          \"memory_gb\": \"64\",
          \"platform\": \"linux-x86_64\"
        },
        \"health_status\": \"healthy\",
        \"registered_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"last_seen\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
      }")
    
    if echo "$REGISTER_RESULT" | grep -q "registered"; then
        echo "✅ Toadstool (Tower A) registered successfully"
    else
        echo "⚠️  Registration response: $REGISTER_RESULT"
    fi
fi

if [ "$TOADSTOOL_B_RUNNING" != "NOT_RUNNING" ]; then
    echo "Registering Toadstool (Tower B)..."
    
    REGISTER_RESULT=$(curl -s -X POST http://$TOWER_A_SONGBIRD/api/federation/services \
      -H "Content-Type: application/json" \
      -d "{
        \"service_id\": \"toadstool-compute-tower-b\",
        \"service_name\": \"Toadstool Compute Service (Tower B - Strandgate)\",
        \"service_type\": \"compute\",
        \"tower_id\": \"tower-b-strandgate\",
        \"tower_name\": \"Strandgate\",
        \"endpoint\": \"http://192.168.1.134:9000\",
        \"capabilities\": [\"compute\", \"cpu\", \"batch-processing\", \"parallel-computing\"],
        \"metadata\": {
          \"cpu_cores\": \"128\",
          \"memory_gb\": \"251\",
          \"storage_gb\": \"1000\",
          \"platform\": \"linux-x86_64\",
          \"specialty\": \"high-core-count-cpu\"
        },
        \"health_status\": \"healthy\",
        \"registered_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"last_seen\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
      }")
    
    if echo "$REGISTER_RESULT" | grep -q "registered"; then
        echo "✅ Toadstool (Tower B) registered successfully"
    else
        echo "⚠️  Registration response: $REGISTER_RESULT"
    fi
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Phase 4: Verify Service Discovery
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 4: Service Discovery Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

COMPUTE_SERVICES=$(curl -s http://$TOWER_A_SONGBIRD/api/federation/services/type/compute)
COMPUTE_COUNT=$(echo "$COMPUTE_SERVICES" | jq '. | length')

echo "Registered Compute Services: $COMPUTE_COUNT"
if [ "$COMPUTE_COUNT" -gt 0 ]; then
    echo ""
    echo "$COMPUTE_SERVICES" | jq -r '.[] | "  • \(.service_name)\n    Endpoint: \(.endpoint)\n    Capabilities: \(.capabilities | join(", "))\n    CPU Cores: \(.metadata.cpu_cores)\n    Memory: \(.metadata.memory_gb)GB\n"'
    echo "✅ PASS: Compute services discovered"
else
    echo "⚠️  No compute services registered yet"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Phase 5: Resource Aggregation
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Phase 5: HPC Resource Pool"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOTAL_CPU_CORES=0
TOTAL_MEMORY_GB=0
TOTAL_GPU_COUNT=0

if [ "$COMPUTE_COUNT" -gt 0 ]; then
    # Calculate totals
    TOTAL_CPU_CORES=$(echo "$COMPUTE_SERVICES" | jq -r '[.[] | .metadata.cpu_cores | tonumber] | add')
    TOTAL_MEMORY_GB=$(echo "$COMPUTE_SERVICES" | jq -r '[.[] | .metadata.memory_gb | tonumber] | add')
    TOTAL_GPU_COUNT=$(echo "$COMPUTE_SERVICES" | jq -r '[.[] | .metadata.gpu_count // "0" | tonumber] | add')
    
    echo "📊 Aggregated HPC Resources:"
    echo "  • CPU Cores: $TOTAL_CPU_CORES"
    echo "  • Memory: ${TOTAL_MEMORY_GB}GB"
    echo "  • GPUs: $TOTAL_GPU_COUNT"
    echo ""
    
    if [ "$TOTAL_CPU_CORES" -gt 0 ]; then
        echo "✅ PASS: HPC resource pool operational"
    fi
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# Final Summary
# ═══════════════════════════════════════════════════════════════

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 INTEGRATION TEST SUMMARY"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Songbird Orchestrator: Running"
echo "✅ Federation: $ACTIVE_NODES active node(s)"
echo "✅ Compute Services: $COMPUTE_COUNT registered"
echo "✅ HPC Resources: ${TOTAL_CPU_CORES} CPU cores, ${TOTAL_MEMORY_GB}GB RAM"
echo ""

if [ "$COMPUTE_COUNT" -ge 1 ]; then
    echo "🚀 Ready for distributed compute workloads!"
    echo ""
    echo "Next steps:"
    echo "  1. Run load tests: ./load_test_distributed_compute.sh"
    echo "  2. Submit test tasks via API"
    echo "  3. Monitor resource utilization"
else
    echo "⚠️  Start Toadstool services on both towers first"
    echo ""
    echo "Tower A:"
    echo "  cd ../toadstool"
    echo "  TOADSTOOL_PORT=9000 ./target/release/toadstool-server"
    echo ""
    echo "Tower B:"
    echo "  cd ../toadstool"
    echo "  TOADSTOOL_PORT=9000 ./target/release/toadstool-server"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

