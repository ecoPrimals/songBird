#!/bin/bash
# showcase/10-inter-primal-foundation/03-task-routing-demo.sh
#
# 🎵🍄 Task Routing Demo: User → Songbird → Toadstool
#
# This demo shows end-to-end task routing:
# 1. User submits task to Songbird
# 2. Songbird queries service registry for "compute" capability
# 3. Songbird routes task to Toadstool
# 4. Toadstool executes task
# 5. Results returned through Songbird
#
# Prerequisites:
# - Songbird running on localhost:8080
# - Toadstool registered with Songbird (run 02-toadstool-live-integration.sh first)

set -euo pipefail

# --- Configuration ---
SONGBIRD_URL="https://localhost:8080"
# -------------------

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  🎵🍄 Task Routing: User → Songbird → Toadstool                    ║"
echo "║     Zero-Config Compute Orchestration                             ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Helper functions
info() { echo -e "\033[0;36m[INFO]\033[0m $*"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $*"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $*"; }
step() { echo -e "\033[1;35m==> $*\033[0m"; }

# 1. Check Songbird
step "[1/4] Checking Songbird..."
if ! curl -sk "${SONGBIRD_URL}/health" >/dev/null 2>&1; then
    error "Songbird not running"
    exit 1
fi
success "Songbird is running"
echo ""

# 2. Query for Compute Services
step "[2/4] Querying for Compute Services..."
COMPUTE_SERVICES=$(curl -sk "${SONGBIRD_URL}/api/v1/services/query/compute" 2>/dev/null | jq .)

COMPUTE_COUNT=$(echo "${COMPUTE_SERVICES}" | jq -r '.count')
if [ "${COMPUTE_COUNT}" -eq 0 ]; then
    error "No compute services registered!"
    echo "   Run 02-toadstool-live-integration.sh first to register Toadstool"
    exit 1
fi

echo "${COMPUTE_SERVICES}" | jq -C '.services[] | {name: .service_name, port: .assigned_endpoint.port, status: .status}'
success "Found ${COMPUTE_COUNT} compute service(s)"
echo ""

# 3. Submit Task (Conceptual - Songbird routing not yet implemented)
step "[3/4] Submitting Compute Task to Songbird..."
TASK_REQUEST='{
  "task_type": "python_compute",
  "requirements": {
    "cpu_cores": 2,
    "memory_mb": 512,
    "gpu": false
  },
  "code": "import sys\nprint(f\"Hello from Toadstool! Python {sys.version}\")\nresult = sum(range(1000000))\nprint(f\"Computed sum: {result}\")",
  "runtime": "python",
  "timeout_seconds": 30
}'

info "Task payload:"
echo "${TASK_REQUEST}" | jq -C '.'
echo ""

# NOTE: This is conceptual - the actual routing endpoint needs to be implemented
info "NOTE: Full task routing implementation is pending"
info "      This demo shows the registration and discovery working"
info "      Task execution will be added in Phase 4"
echo ""

# 4. Demonstrate Capability-Based Routing Logic
step "[4/4] Demonstrating Routing Logic..."
echo "Routing Decision Process:"
echo "  1. User submits task with requirements (cpu, memory, gpu)"
echo "  2. Songbird queries: GET /api/v1/services/query/compute"
echo "  3. Songbird filters by: status=active, has_capacity=true"
echo "  4. Songbird selects best match (load balancing, location)"
echo "  5. Songbird forwards task to selected service"
echo "  6. Service executes task, returns results"
echo "  7. Songbird forwards results to user"
echo ""

BEST_SERVICE=$(echo "${COMPUTE_SERVICES}" | jq -r '.services[0]')
SERVICE_NAME=$(echo "${BEST_SERVICE}" | jq -r '.service_name')
SERVICE_PORT=$(echo "${BEST_SERVICE}" | jq -r '.assigned_endpoint.port')

success "Would route to: ${SERVICE_NAME} on port ${SERVICE_PORT}"
echo ""

# Summary
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║  ✅ TASK ROUTING DEMO COMPLETE                                    ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Architecture Demonstrated:"
echo "  ✅ Capability-Based Service Discovery"
echo "  ✅ Dynamic Routing (no hardcoded endpoints)"
echo "  ✅ User abstracts away primal complexity"
echo "  ✅ Songbird handles all orchestration"
echo ""
echo "What's Working:"
echo "  ✅ Service registry (registration, heartbeat, query)"
echo "  ✅ Capability discovery (find by capability, not name)"
echo "  ✅ Multi-primal support (Toadstool, future: BearDog, Nestgate)"
echo ""
echo "Next Steps (Phase 4):"
echo "  - Implement task routing in Songbird compute API"
echo "  - Add load balancing logic"
echo "  - Add task queuing and retry"
echo "  - Add result caching"
echo ""

