#!/bin/bash
# showcase/09-local-compute/04-resource-monitoring.sh
#
# This script demonstrates Songbird's resource monitoring capabilities.
# It shows how Songbird tracks CPU, memory, and system resources during task execution.
#
# Prerequisites:
# - Songbird Orchestrator running locally on its default port (8080).
# - `curl` and `jq` installed.
#
# Usage:
# ./04-resource-monitoring.sh

set -euo pipefail

# --- Configuration ---
SONGBIRD_URL="https://localhost:8080"
STATS_ENDPOINT="/api/federation/stats"
HEALTH_ENDPOINT="/health"
# -------------------

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║       🎵 Songbird Local Compute: Resource Monitoring             ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Helper functions
info() { echo -e "\033[0;36m[INFO]\033[0m $*"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $*"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $*"; }
step() { echo -e "\033[1;35m==> $*\033[0m"; }

# 1. Check if Songbird is running
step "[1/4] Checking Songbird Orchestrator health..."
if ! curl -sk "${SONGBIRD_URL}${HEALTH_ENDPOINT}" >/dev/null 2>&1; then
    error "Songbird Orchestrator is not running at ${SONGBIRD_URL}."
    exit 1
fi
success "Songbird is running."
echo ""

# 2. Get initial resource stats
step "[2/4] Getting baseline resource stats..."
INITIAL_STATS=$(curl -sk "${SONGBIRD_URL}${STATS_ENDPOINT}" 2>/dev/null)

if [ $? -eq 0 ]; then
    echo "${INITIAL_STATS}" | jq -C '.'
    echo ""
    success "Baseline stats collected."
else
    info "Stats endpoint not available (this is normal for local-only mode)."
    echo "   Songbird is running in standalone mode."
fi
echo ""

# 3. Display system information
step "[3/4] System Information..."
echo ""
echo "Host Information:"
echo "  Hostname:    $(hostname)"
echo "  OS:          $(uname -s)"
echo "  Kernel:      $(uname -r)"
echo "  Arch:        $(uname -m)"
echo ""

if command -v nproc &> /dev/null; then
    echo "CPU Information:"
    echo "  CPU Cores:   $(nproc)"
fi

if command -v free &> /dev/null; then
    echo ""
    echo "Memory Information:"
    free -h | head -2
fi

if command -v df &> /dev/null; then
    echo ""
    echo "Disk Usage:"
    df -h / | tail -1
fi

echo ""

# 4. Get service registry stats
step "[4/4] Service Registry Status..."
SERVICES=$(curl -sk "${SONGBIRD_URL}/api/v1/services" 2>/dev/null)

if [ $? -eq 0 ]; then
    echo ""
    echo "Service Registry Statistics:"
    echo "${SERVICES}" | jq -C '.stats'
    echo ""
    
    TOTAL_SERVICES=$(echo "${SERVICES}" | jq -r '.stats.total_services // 0')
    ACTIVE_SERVICES=$(echo "${SERVICES}" | jq -r '.stats.active_services // 0')
    ALLOCATED_PORTS=$(echo "${SERVICES}" | jq -r '.stats.allocated_ports // 0')
    
    echo "Summary:"
    echo "  Total Services:    ${TOTAL_SERVICES}"
    echo "  Active Services:   ${ACTIVE_SERVICES}"
    echo "  Allocated Ports:   ${ALLOCATED_PORTS}"
    
    if [ ${TOTAL_SERVICES} -gt 0 ]; then
        echo ""
        echo "Registered Services:"
        echo "${SERVICES}" | jq -C '.services[] | {name: .service_name, status: .status, port: .assigned_endpoint.port}'
    fi
else
    info "Service registry not available or empty."
fi

echo ""
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Resource Monitoring Features:"
echo "  ✅ System resource tracking"
echo "  ✅ Service registry statistics"
echo "  ✅ Port allocation monitoring"
echo "  ✅ Health status endpoints"
echo ""
echo "Use Cases:"
echo "  • Monitor system capacity"
echo "  • Track service health"
echo "  • Debug resource issues"
echo "  • Capacity planning"
echo ""
echo "Next Steps:"
echo "  • Explore showcase/10-inter-primal-foundation/ for primal integration"
echo "  • Register services with: ./10-inter-primal-foundation/02-toadstool-live-integration.sh"
echo ""

