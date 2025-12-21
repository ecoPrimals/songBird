#!/bin/bash
# showcase/09-local-compute/02-spawn-python-task.sh
#
# This script demonstrates Songbird's capability to spawn and manage a Python task locally.
# It simulates a client requesting Songbird to execute a Python script with imports and data processing.
#
# Prerequisites:
# - Songbird Orchestrator running locally on its default port (8080).
# - Python 3 installed on the system
# - `curl` and `jq` installed.
#
# Usage:
# ./02-spawn-python-task.sh

set -euo pipefail

# --- Configuration ---
SONGBIRD_URL="https://localhost:8080"
TASK_ENDPOINT="/api/v1/compute/local/task"
HEALTH_ENDPOINT="/health"
# -------------------

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║       🎵 Songbird Local Compute: Python Task Demo                ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Helper functions
info() { echo -e "\033[0;36m[INFO]\033[0m $*"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $*"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $*"; }
step() { echo -e "\033[1;35m==> $*\033[0m"; }

# 1. Check if Songbird is running
step "[1/3] Checking Songbird Orchestrator health..."
HEALTH_STATUS=$(curl -sk "${SONGBIRD_URL}${HEALTH_ENDPOINT}" 2>/dev/null)

if [[ "${HEALTH_STATUS}" == "OK" ]]; then
    success "Songbird Orchestrator is running."
else
    error "Songbird Orchestrator is not running or not reachable at ${SONGBIRD_URL}."
    echo "   Please start Songbird Orchestrator (e.g., 'cargo run --release') and try again."
    exit 1
fi

echo ""

# 2. Submit a Python compute task to Songbird
step "[2/3] Submitting Python compute task to Songbird..."

PYTHON_CODE='
import sys
import json
import math
from datetime import datetime

print("=== Songbird Local Compute: Python Task ===")
print(f"Python version: {sys.version}")
print(f"Timestamp: {datetime.now().isoformat()}")
print()

# Perform some computation
data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
results = {
    "sum": sum(data),
    "average": sum(data) / len(data),
    "max": max(data),
    "min": min(data),
    "sqrt_sum": math.sqrt(sum(data)),
}

print("Data:", data)
print("Results:", json.dumps(results, indent=2))
print()
print("✅ Python task completed successfully!")
'

TASK_PAYLOAD=$(jq -n \
  --arg cmd "python3" \
  --arg code "$PYTHON_CODE" \
  '{
    "command": $cmd,
    "args": ["-c", $code],
    "timeout_seconds": 10,
    "working_directory": "/tmp"
  }')

echo "   Payload:"
echo "${TASK_PAYLOAD}" | jq -C '.'
echo ""

TASK_RESPONSE=$(curl -sk -X POST \
    -H "Content-Type: application/json" \
    -d "${TASK_PAYLOAD}" \
    "${SONGBIRD_URL}${TASK_ENDPOINT}" 2>/dev/null)

if [ $? -ne 0 ]; then
    error "Failed to submit task to Songbird."
    echo "   Response: ${TASK_RESPONSE}"
    exit 1
fi

# 3. Display the task execution result
step "[3/3] Task execution result from Songbird:"
echo "${TASK_RESPONSE}" | jq -C '.'

# Basic validation of the response
TASK_STATUS=$(echo "${TASK_RESPONSE}" | jq -r '.status')
TASK_OUTPUT=$(echo "${TASK_RESPONSE}" | jq -r '.output')

if [[ "${TASK_STATUS}" == "completed" && "${TASK_OUTPUT}" == *"Python task completed successfully"* ]]; then
    echo ""
    success "Demo successful! Songbird executed the Python task and returned the output."
else
    echo ""
    error "Demo failed: Task did not complete successfully or output was unexpected."
    echo "   Status: ${TASK_STATUS}"
    echo "   Output: ${TASK_OUTPUT}"
    exit 1
fi

echo ""
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Key Takeaways:"
echo "  ✅ Songbird can orchestrate Python tasks locally"
echo "  ✅ Full Python environment available (imports, stdlib)"
echo "  ✅ JSON output parsing supported"
echo "  ✅ Timeout management (10s limit)"
echo ""
echo "Next: Try 03-spawn-concurrent-tasks.sh for parallel execution"
echo ""

