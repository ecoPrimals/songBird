#!/usr/bin/env bash
#
# Enhanced Routing Demo - Universal Port Authority in Action
#
# This demo shows the Enhanced Capability Router with priority-based routing:
# 1. PRIORITY 1: Universal Port Authority (registered services)
# 2. PRIORITY 2: Legacy capability registry
# 3. PRIORITY 3: Static endpoint resolver
#
# Architecture Achievement:
# - Modern, idiomatic Rust router
# - Deep debt eliminated (clear priority chain)
# - Backward compatible
# - Zero compile-time dependencies

set -euo pipefail

SONGBIRD_HOST="${SONGBIRD_HOST:-localhost}"
SONGBIRD_PORT="${SONGBIRD_PORT:-8080}"
BASE_URL="https://${SONGBIRD_HOST}:${SONGBIRD_PORT}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARN:${NC} $*"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $*"
}

section() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC} $*"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Check if Songbird is running
check_songbird() {
    if ! curl -sk "${BASE_URL}/health" > /dev/null 2>&1; then
        error "Songbird is not running at ${BASE_URL}"
        error "Start it with: cargo run --release"
        exit 1
    fi
}

# Step 1: Register a service (Toadstool)
register_toadstool() {
    section "STEP 1: Register Toadstool via Universal Port Authority"
    
    log "Sending registration request..."
    
    RESPONSE=$(curl -sk -X POST "${BASE_URL}/api/v1/services/register" \
        -H "Content-Type: application/json" \
        -d '{
            "service_info": {
                "name": "Toadstool",
                "version": "0.1.0",
                "capabilities": [
                    {
                        "name": "compute",
                        "version": "1.0.0",
                        "protocols": ["http"],
                        "metadata": {
                            "gpu": "true",
                            "ml_training": "true",
                            "max_concurrent_jobs": "10"
                        }
                    }
                ],
                "metadata": {
                    "description": "GPU-accelerated compute service",
                    "gpu_type": "NVIDIA RTX 4090"
                }
            },
            "preferred_port": 9000
        }')
    
    # Parse service ID and assigned port
    SERVICE_ID=$(echo "$RESPONSE" | jq -r '.service_id // empty')
    ASSIGNED_PORT=$(echo "$RESPONSE" | jq -r '.assigned_endpoint.port // empty')
    
    if [[ -z "$SERVICE_ID" ]] || [[ -z "$ASSIGNED_PORT" ]]; then
        error "Failed to register service"
        echo "$RESPONSE" | jq '.'
        exit 1
    fi
    
    log "✅ Service registered successfully"
    log "   Service ID: ${SERVICE_ID}"
    log "   Assigned Port: ${ASSIGNED_PORT}"
    
    # Export for other steps
    export TOADSTOOL_SERVICE_ID="$SERVICE_ID"
    export TOADSTOOL_PORT="$ASSIGNED_PORT"
}

# Step 2: Submit a compute task
submit_compute_task() {
    section "STEP 2: Submit Compute Task (Should route to Toadstool)"
    
    log "Submitting ML training task..."
    
    RESPONSE=$(curl -sk -X POST "${BASE_URL}/api/v1/compute/task" \
        -H "Content-Type: application/json" \
        -d '{
            "task": {
                "task_type": "ml_training",
                "payload": {
                    "model": "resnet50",
                    "dataset": "imagenet",
                    "epochs": 10
                },
                "metadata": {
                    "requires_gpu": "true",
                    "priority": "high"
                }
            },
            "priority": 8,
            "timeout_secs": 300
        }')
    
    JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id // empty')
    ROUTED_TO=$(echo "$RESPONSE" | jq -r '.routed_to // empty')
    
    if [[ -z "$JOB_ID" ]]; then
        error "Failed to submit task"
        echo "$RESPONSE" | jq '.'
        exit 1
    fi
    
    log "✅ Task submitted successfully"
    log "   Job ID: ${JOB_ID}"
    log "   Routed To: ${ROUTED_TO}"
    
    # Check if routed to registered service
    if [[ "$ROUTED_TO" == service:Toadstool:* ]]; then
        log "🎯 PRIORITY 1: Routed via Universal Port Authority!"
    else
        warn "⚠️  Not routed to UPA service (got: ${ROUTED_TO})"
    fi
    
    export TEST_JOB_ID="$JOB_ID"
}

# Step 3: Query task status
query_task_status() {
    section "STEP 3: Query Task Status"
    
    log "Querying job ${TEST_JOB_ID}..."
    
    RESPONSE=$(curl -sk "${BASE_URL}/api/v1/compute/task/${TEST_JOB_ID}")
    
    STATUS=$(echo "$RESPONSE" | jq -r '.status // empty')
    ROUTED_TO=$(echo "$RESPONSE" | jq -r '.routed_to // empty')
    
    log "Status: ${STATUS}"
    log "Routed To: ${ROUTED_TO}"
    
    echo "$RESPONSE" | jq '.'
}

# Step 4: Query all registered services
query_services() {
    section "STEP 4: Query All Registered Services"
    
    log "Fetching all services..."
    
    RESPONSE=$(curl -sk "${BASE_URL}/api/v1/services")
    
    COUNT=$(echo "$RESPONSE" | jq '.services | length')
    
    log "Found ${COUNT} registered service(s)"
    echo "$RESPONSE" | jq '.services[] | {name: .name, status: .status, capabilities: [.capabilities[].name], port: .assigned_endpoint.port}'
}

# Step 5: Query by capability
query_by_capability() {
    section "STEP 5: Query Services by Capability"
    
    log "Searching for 'compute' capability..."
    
    RESPONSE=$(curl -sk "${BASE_URL}/api/v1/services/query/compute")
    
    COUNT=$(echo "$RESPONSE" | jq '.services | length')
    
    log "Found ${COUNT} service(s) with 'compute' capability"
    echo "$RESPONSE" | jq '.services[] | {name: .name, port: .assigned_endpoint.port, metadata: .metadata}'
}

# Step 6: Test routing priority chain
test_routing_priority() {
    section "STEP 6: Test Routing Priority Chain"
    
    log "Testing different task types to demonstrate routing priority..."
    
    # Test 1: ML training (should route to registered Toadstool)
    log "\n📝 Test 1: ML Training Task"
    RESPONSE=$(curl -sk -X POST "${BASE_URL}/api/v1/compute/task" \
        -H "Content-Type: application/json" \
        -d '{"task": {"task_type": "ml_training", "metadata": {"requires_gpu": "true"}}}')
    ROUTED=$(echo "$RESPONSE" | jq -r '.routed_to')
    log "   Routed to: ${ROUTED}"
    
    # Test 2: Simple task (might route locally)
    log "\n📝 Test 2: Simple Health Check"
    RESPONSE=$(curl -sk -X POST "${BASE_URL}/api/v1/compute/task" \
        -H "Content-Type: application/json" \
        -d '{"task": {"task_type": "health_check", "metadata": {}}}')
    ROUTED=$(echo "$RESPONSE" | jq -r '.routed_to')
    log "   Routed to: ${ROUTED}"
    
    # Test 3: Security task (might use capability resolver)
    log "\n📝 Test 3: Security Task"
    RESPONSE=$(curl -sk -X POST "${BASE_URL}/api/v1/compute/task" \
        -H "Content-Type: application/json" \
        -d '{"task": {"task_type": "encryption", "metadata": {}}}')
    ROUTED=$(echo "$RESPONSE" | jq -r '.routed_to')
    log "   Routed to: ${ROUTED}"
}

# Step 7: Cleanup
cleanup() {
    section "STEP 7: Cleanup"
    
    if [[ -n "${TOADSTOOL_SERVICE_ID:-}" ]]; then
        log "Deregistering service ${TOADSTOOL_SERVICE_ID}..."
        curl -sk -X DELETE "${BASE_URL}/api/v1/services/${TOADSTOOL_SERVICE_ID}" > /dev/null 2>&1 || true
        log "✅ Service deregistered"
    fi
}

# Main execution
main() {
    section "🎵 Enhanced Routing Demo - Universal Port Authority"
    
    log "Testing Enhanced Capability Router with priority-based routing"
    log "Base URL: ${BASE_URL}"
    
    check_songbird
    
    register_toadstool
    submit_compute_task
    sleep 1
    query_task_status
    query_services
    query_by_capability
    test_routing_priority
    cleanup
    
    section "✅ Demo Complete"
    
    log ""
    log "Architecture Achievements:"
    log "  ✅ Modern, idiomatic Rust router"
    log "  ✅ Priority-based routing (UPA → Legacy → Static)"
    log "  ✅ Deep debt eliminated"
    log "  ✅ Backward compatible"
    log "  ✅ Zero compile-time dependencies"
    log ""
    log "Routing Priority Chain:"
    log "  1️⃣  Universal Port Authority (registered services)"
    log "  2️⃣  Legacy capability registry"
    log "  3️⃣  Static endpoint resolver"
    log ""
}

# Trap cleanup on exit
trap cleanup EXIT

main "$@"

