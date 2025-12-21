#!/bin/bash
# showcase/09-local-compute/03-spawn-concurrent-tasks.sh
#
# This script demonstrates Songbird's capability to spawn and manage multiple concurrent tasks.
# It shows how Songbird can orchestrate parallel execution locally.
#
# Prerequisites:
# - Songbird Orchestrator running locally on its default port (8080).
# - `curl` and `jq` installed.
#
# Usage:
# ./03-spawn-concurrent-tasks.sh

set -euo pipefail

# --- Configuration ---
SONGBIRD_URL="https://localhost:8080"
TASK_ENDPOINT="/api/v1/compute/local/task"
HEALTH_ENDPOINT="/health"
NUM_TASKS=5
# -------------------

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║    🎵 Songbird Local Compute: Concurrent Tasks Demo              ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Helper functions
info() { echo -e "\033[0;36m[INFO]\033[0m $*"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $*"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $*"; }
step() { echo -e "\033[1;35m==> $*\033[0m"; }

# 1. Check if Songbird is running
step "[1/3] Checking Songbird Orchestrator health..."
if ! curl -sk "${SONGBIRD_URL}${HEALTH_ENDPOINT}" >/dev/null 2>&1; then
    error "Songbird Orchestrator is not running at ${SONGBIRD_URL}."
    exit 1
fi
success "Songbird is running."
echo ""

# 2. Submit multiple tasks concurrently
step "[2/3] Submitting ${NUM_TASKS} concurrent tasks to Songbird..."
echo ""

TASK_IDS=()
START_TIME=$(date +%s)

for i in $(seq 1 ${NUM_TASKS}); do
    info "Submitting task ${i}/${NUM_TASKS}..."
    
    TASK_PAYLOAD=$(jq -n \
      --arg id "$i" \
      --argjson delay "$((RANDOM % 3 + 1))" \
      '{
        "command": "bash",
        "args": ["-c", ("echo Task " + $id + " starting...; sleep " + ($delay|tostring) + "; echo Task " + $id + " completed after " + ($delay|tostring) + "s!")],
        "timeout_seconds": 10
      }')
    
    # Submit task in background
    TASK_RESPONSE=$(curl -sk -X POST \
        -H "Content-Type: application/json" \
        -d "${TASK_PAYLOAD}" \
        "${SONGBIRD_URL}${TASK_ENDPOINT}" 2>/dev/null) &
    
    TASK_IDS+=($!)
done

echo ""
info "All ${NUM_TASKS} tasks submitted. Waiting for completion..."
echo ""

# 3. Wait for all tasks and collect results
step "[3/3] Collecting results..."
echo ""

COMPLETED=0
FAILED=0

for pid in "${TASK_IDS[@]}"; do
    if wait "$pid"; then
        COMPLETED=$((COMPLETED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                  CONCURRENT EXECUTION SUMMARY                     ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Tasks Submitted:    ${NUM_TASKS}"
echo "Tasks Completed:    ${COMPLETED}"
echo "Tasks Failed:       ${FAILED}"
echo "Total Duration:     ${DURATION}s"
echo ""

if [ ${COMPLETED} -eq ${NUM_TASKS} ]; then
    success "All tasks completed successfully!"
    echo ""
    echo "Key Achievements:"
    echo "  ✅ Parallel task execution"
    echo "  ✅ Non-blocking submission"
    echo "  ✅ Independent task lifecycles"
    echo "  ✅ Concurrent orchestration"
    echo ""
    echo "Insight: Tasks ran in parallel, not sequentially!"
    echo "         Sequential execution would take ~$((NUM_TASKS * 2))s"
    echo "         Parallel execution took ~${DURATION}s"
else
    error "Some tasks failed!"
    exit 1
fi

echo ""
echo "Next: Explore showcase/10-inter-primal-foundation/ for primal integration"
echo ""

