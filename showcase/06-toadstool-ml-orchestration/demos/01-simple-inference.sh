#!/usr/bin/env bash
# Demo 1: Simple ML Inference via Songbird → ToadStool
# Time: ~5 minutes
# Demonstrates basic orchestration with real service discovery

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$SHOWCASE_DIR/results"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}🎵🍄 Demo 1: Simple ML Inference${NC}"
echo -e "${BLUE}   Songbird orchestrates ToadStool for MNIST classification${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"

if ! pgrep -f "songbird-orchestrator" > /dev/null; then
    echo -e "${RED}❌ Songbird not running${NC}"
    echo "   Start it with: ./scripts/01-start-songbird.sh"
    exit 1
fi
echo -e "${GREEN}✅ Songbird running${NC}"

SONGBIRD_URL="http://localhost:8080"

# Check Songbird health
if ! curl -sf "$SONGBIRD_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Songbird not responding at $SONGBIRD_URL${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Songbird API responding${NC}"

# Discover ToadStool instances
echo
echo -e "${YELLOW}🔍 Discovering ToadStool compute primals...${NC}"

DISCOVERY_RESPONSE=$(curl -sf "$SONGBIRD_URL/api/federation/services" || echo "{}")
TOADSTOOL_COUNT=$(echo "$DISCOVERY_RESPONSE" | jq -r '.services | length' 2>/dev/null || echo "0")

if [[ "$TOADSTOOL_COUNT" -eq 0 ]]; then
    echo -e "${RED}❌ No ToadStool instances discovered${NC}"
    echo "   Start ToadStool with: ./scripts/02-start-toadstool.sh"
    echo
    echo "   Alternative: Manually register a ToadStool instance:"
    echo "   curl -X POST $SONGBIRD_URL/api/federation/register \\"
    echo "     -H 'Content-Type: application/json' \\"
    echo "     -d @configs/toadstool-registration.json"
    exit 1
fi

echo -e "${GREEN}✅ Found $TOADSTOOL_COUNT ToadStool instance(s)${NC}"
echo

# Show discovered services
echo -e "${BLUE}📊 Discovered Services:${NC}"
echo "$DISCOVERY_RESPONSE" | jq -r '.services[] | "   \(.service_name) @ \(.endpoint)\n   Capabilities: \(.capabilities | join(", "))\n   Tower: \(.tower_id)\n"' 2>/dev/null || true

# Find ML-capable service
echo -e "${YELLOW}🎯 Finding ML-capable compute...${NC}"

ML_SERVICES=$(curl -sf "$SONGBIRD_URL/api/capabilities/query?capability=ml-inference" 2>/dev/null || echo "[]")
ML_COUNT=$(echo "$ML_SERVICES" | jq -r 'length' 2>/dev/null || echo "0")

if [[ "$ML_COUNT" -eq 0 ]]; then
    echo -e "${RED}❌ No ML-inference capable services found${NC}"
    exit 1
fi

TOADSTOOL_ENDPOINT=$(echo "$ML_SERVICES" | jq -r '.[0].endpoint' 2>/dev/null)
echo -e "${GREEN}✅ Found ML compute at: $TOADSTOOL_ENDPOINT${NC}"
echo

# Submit inference task
echo -e "${YELLOW}📤 Submitting MNIST inference task...${NC}"

TASK_PAYLOAD=$(cat <<EOF
{
  "task_type": "ml_inference",
  "user_id": "demo-user",
  "spec": {
    "model": "mnist_cnn",
    "input_data": "demo/mnist_samples.npy",
    "batch_size": 10
  },
  "requirements": {
    "capabilities": ["ml-inference"],
    "min_memory_mb": 512,
    "prefer_gpu": true
  },
  "priority": "standard"
}
EOF
)

mkdir -p "$RESULTS_DIR"
RESULT_FILE="$RESULTS_DIR/inference_$(date +%s).json"

TASK_RESPONSE=$(curl -sf -X POST "$SONGBIRD_URL/api/tasks/submit" \
  -H "Content-Type: application/json" \
  -d "$TASK_PAYLOAD" 2>/dev/null || echo '{"error": "Task submission failed"}')

echo "$TASK_RESPONSE" | jq . > "$RESULT_FILE" 2>/dev/null || echo "$TASK_RESPONSE" > "$RESULT_FILE"

TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.task_id' 2>/dev/null || echo "")

if [[ -z "$TASK_ID" || "$TASK_ID" == "null" ]]; then
    echo -e "${RED}❌ Task submission failed${NC}"
    echo "$TASK_RESPONSE" | jq . 2>/dev/null || echo "$TASK_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✅ Task submitted! ID: $TASK_ID${NC}"
echo

# Monitor task progress
echo -e "${YELLOW}⏳ Monitoring task execution...${NC}"

MAX_WAIT=30
WAIT_COUNT=0

while [[ $WAIT_COUNT -lt $MAX_WAIT ]]; do
    TASK_STATUS=$(curl -sf "$SONGBIRD_URL/api/tasks/$TASK_ID/status" 2>/dev/null || echo '{"status": "unknown"}')
    STATUS=$(echo "$TASK_STATUS" | jq -r '.status' 2>/dev/null || echo "unknown")
    
    case "$STATUS" in
        "completed")
            echo -e "${GREEN}✅ Task completed successfully!${NC}"
            break
            ;;
        "failed")
            echo -e "${RED}❌ Task failed${NC}"
            echo "$TASK_STATUS" | jq . 2>/dev/null || echo "$TASK_STATUS"
            exit 1
            ;;
        "running")
            PROGRESS=$(echo "$TASK_STATUS" | jq -r '.progress // 0' 2>/dev/null)
            echo -e "   Status: ${YELLOW}running${NC} | Progress: ${PROGRESS}%"
            ;;
        *)
            echo -e "   Status: ${BLUE}$STATUS${NC}"
            ;;
    esac
    
    sleep 1
    ((WAIT_COUNT++))
done

if [[ $WAIT_COUNT -ge $MAX_WAIT ]]; then
    echo -e "${YELLOW}⚠️  Task still running after ${MAX_WAIT}s (may be expected for large workloads)${NC}"
fi

echo

# Fetch results
echo -e "${YELLOW}📊 Retrieving results...${NC}"

RESULTS=$(curl -sf "$SONGBIRD_URL/api/tasks/$TASK_ID/results" 2>/dev/null || echo '{"error": "Results not available"}')
echo "$RESULTS" | jq . > "$RESULT_FILE" 2>/dev/null || true

echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}📈 Results:${NC}"
echo "$RESULTS" | jq . 2>/dev/null || echo "$RESULTS"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"

# Parse and display metrics
ACCURACY=$(echo "$RESULTS" | jq -r '.accuracy // "N/A"' 2>/dev/null)
INFERENCE_TIME=$(echo "$RESULTS" | jq -r '.inference_time_ms // "N/A"' 2>/dev/null)
GPU_UTIL=$(echo "$RESULTS" | jq -r '.gpu_utilization // "N/A"' 2>/dev/null)

echo
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Demo Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════${NC}"
echo
echo -e "${BLUE}📊 Metrics:${NC}"
echo "   Accuracy: $ACCURACY"
echo "   Inference Time: ${INFERENCE_TIME}ms"
echo "   GPU Utilization: ${GPU_UTIL}%"
echo "   ToadStool Endpoint: $TOADSTOOL_ENDPOINT"
echo
echo -e "${BLUE}💾 Results saved to: $RESULT_FILE${NC}"
echo
echo -e "${YELLOW}🚀 Next Steps:${NC}"
echo "   1. Try distributed training: ./demos/02-distributed-training.sh"
echo "   2. Test GPU routing: ./demos/03-gpu-routing.sh"
echo "   3. Set up multi-tower mesh: ./demos/04-multi-tower-mesh.sh"
echo

