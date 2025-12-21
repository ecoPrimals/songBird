#!/bin/bash
# Songbird Local Compute - Basic Task Spawning Demo
# Shows: Songbird handling simple compute tasks locally

set -e

SONGBIRD_URL="https://localhost:8080"
OUTPUT_DIR="./outputs/01-simple-task-$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  Demo 01: Songbird Local Compute - Simple Task                   ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Check Songbird is running
echo "📡 Checking Songbird status..."
if ! curl -sk "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo "❌ Songbird not running at ${SONGBIRD_URL}"
    echo "   Start with: ./target/release/songbird-orchestrator"
    exit 1
fi
echo "✅ Songbird is operational"
echo ""

# Submit simple task
echo "📝 Submitting simple compute task..."
TASK_ID=$(curl -sk -X POST "${SONGBIRD_URL}/api/v1/compute/task" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "echo \"Hello from Songbird Compute!\" && echo \"Tower: $(hostname)\" && echo \"Time: $(date)\"",
    "runtime": "shell",
    "description": "Simple test task"
  }' | jq -r '.task_id // .id // "unknown"')

echo "✅ Task submitted: ${TASK_ID}"
echo ""

# Wait for completion
echo "⏳ Waiting for task completion..."
sleep 3

# Get task status
echo "📊 Task Status:"
curl -sk "${SONGBIRD_URL}/api/v1/compute/task/${TASK_ID}" | jq '.'

# Save output
echo ""
echo "💾 Saving output to: ${OUTPUT_DIR}/task-${TASK_ID}.json"
curl -sk "${SONGBIRD_URL}/api/v1/compute/task/${TASK_ID}" > "${OUTPUT_DIR}/task-${TASK_ID}.json"

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ Demo Complete                                                 ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Key Observations:"
echo "  1. Task submitted to Songbird (not directly to compute)"
echo "  2. Songbird managed execution lifecycle"
echo "  3. Results returned through Songbird API"
echo "  4. No direct access to underlying compute"
echo ""
echo "This pattern extends to:"
echo "  - Toadstool (GPU compute)"
echo "  - Nestgate (data storage)"
echo "  - BearDog (security verification)"
echo "  - Squirrel (AI routing)"
echo ""
echo "Songbird is the universal orchestrator! 🎵"

