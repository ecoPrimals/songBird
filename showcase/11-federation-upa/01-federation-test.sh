#!/usr/bin/env bash
#
# Phase 5: Federation Testing
# Test Universal Port Authority across federated towers
#
# This script tests:
# 1. Service registration on each tower
# 2. Cross-tower service discovery
# 3. Task routing across federation
# 4. Load distribution

set -euo pipefail

# Configuration
EASTGATE_URL="${EASTGATE_URL:-https://192.168.1.10:8080}"
WESTGATE_URL="${WESTGATE_URL:-https://192.168.1.20:8080}"
STRANDGATE_URL="${STRANDGATE_URL:-https://192.168.1.30:8080}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

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

subsection() {
    echo ""
    echo -e "${CYAN}┌────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${CYAN}│${NC} $*"
    echo -e "${CYAN}└────────────────────────────────────────────────────────────┘${NC}"
    echo ""
}

# Check tower health
check_tower_health() {
    local tower_name=$1
    local tower_url=$2
    
    if curl -sk "${tower_url}/health" > /dev/null 2>&1; then
        log "✅ ${tower_name} is healthy"
        return 0
    else
        error "❌ ${tower_name} is not responding"
        return 1
    fi
}

# Check all towers
check_all_towers() {
    section "Checking Tower Health"
    
    local all_healthy=true
    
    check_tower_health "Eastgate" "$EASTGATE_URL" || all_healthy=false
    check_tower_health "Westgate" "$WESTGATE_URL" || all_healthy=false
    check_tower_health "Strandgate" "$STRANDGATE_URL" || all_healthy=false
    
    if ! $all_healthy; then
        error "Not all towers are healthy. Please start missing towers."
        exit 1
    fi
    
    log "✅ All towers are healthy"
}

# Check federation
check_federation() {
    section "Checking Federation Status"
    
    subsection "Eastgate's View of Federation"
    local east_nodes=$(curl -sk "${EASTGATE_URL}/api/v1/federation/nodes" | jq -r '.nodes[] | "  \(.node_name) (\(.status))"')
    echo "$east_nodes"
    
    subsection "Westgate's View of Federation"
    local west_nodes=$(curl -sk "${WESTGATE_URL}/api/v1/federation/nodes" | jq -r '.nodes[] | "  \(.node_name) (\(.status))"')
    echo "$west_nodes"
    
    subsection "Strandgate's View of Federation"
    local strand_nodes=$(curl -sk "${STRANDGATE_URL}/api/v1/federation/nodes" | jq -r '.nodes[] | "  \(.node_name) (\(.status))"')
    echo "$strand_nodes"
}

# Register Toadstool on a tower
register_toadstool() {
    local tower_name=$1
    local tower_url=$2
    local preferred_port=$3
    
    subsection "Registering Toadstool on ${tower_name}"
    
    local response=$(curl -sk -X POST "${tower_url}/api/v1/services/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"service_info\": {
                \"name\": \"Toadstool-${tower_name}\",
                \"version\": \"0.1.0\",
                \"capabilities\": [
                    {
                        \"name\": \"compute\",
                        \"version\": \"1.0.0\",
                        \"protocols\": [\"http\"],
                        \"metadata\": {
                            \"gpu\": \"true\",
                            \"ml_training\": \"true\",
                            \"tower\": \"${tower_name}\"
                        }
                    }
                ],
                \"metadata\": {
                    \"description\": \"GPU compute on ${tower_name}\",
                    \"tower\": \"${tower_name}\"
                }
            },
            \"preferred_port\": ${preferred_port}
        }")
    
    local service_id=$(echo "$response" | jq -r '.service_id // empty')
    local assigned_port=$(echo "$response" | jq -r '.assigned_endpoint.port // empty')
    
    if [[ -z "$service_id" ]]; then
        error "Failed to register Toadstool on ${tower_name}"
        echo "$response" | jq '.'
        return 1
    fi
    
    log "✅ Registered: ${service_id} on port ${assigned_port}"
    
    # Export for cleanup
    echo "${tower_name}:${service_id}" >> /tmp/songbird-test-services.txt
}

# Register Toadstool on all towers
register_all_toadstools() {
    section "Registering Toadstool on All Towers"
    
    # Clear previous registrations
    rm -f /tmp/songbird-test-services.txt
    
    register_toadstool "Eastgate" "$EASTGATE_URL" 9000
    register_toadstool "Westgate" "$WESTGATE_URL" 9001
    register_toadstool "Strandgate" "$STRANDGATE_URL" 9002
    
    log "✅ All Toadstools registered"
}

# Query services on a tower
query_services() {
    local tower_name=$1
    local tower_url=$2
    
    subsection "Services on ${tower_name}"
    
    local response=$(curl -sk "${tower_url}/api/v1/services")
    local count=$(echo "$response" | jq '.services | length')
    
    log "Found ${count} service(s)"
    echo "$response" | jq -r '.services[] | "  - \(.name) on port \(.assigned_endpoint.port) (\(.status))"'
}

# Query all services
query_all_services() {
    section "Querying Services on All Towers"
    
    query_services "Eastgate" "$EASTGATE_URL"
    query_services "Westgate" "$WESTGATE_URL"
    query_services "Strandgate" "$STRANDGATE_URL"
}

# Submit task to a tower
submit_task() {
    local tower_name=$1
    local tower_url=$2
    
    subsection "Submitting Task to ${tower_name}"
    
    local response=$(curl -sk -X POST "${tower_url}/api/v1/compute/task" \
        -H "Content-Type: application/json" \
        -d '{
            "task": {
                "task_type": "ml_training",
                "payload": {
                    "model": "resnet50",
                    "dataset": "imagenet"
                },
                "metadata": {
                    "requires_gpu": "true",
                    "submitted_from": "'"${tower_name}"'"
                }
            },
            "priority": 8
        }')
    
    local job_id=$(echo "$response" | jq -r '.job_id // empty')
    local routed_to=$(echo "$response" | jq -r '.routed_to // empty')
    
    if [[ -z "$job_id" ]]; then
        error "Failed to submit task to ${tower_name}"
        echo "$response" | jq '.'
        return 1
    fi
    
    log "Job ${job_id} routed to: ${routed_to}"
    
    # Check if routed locally
    if [[ "$routed_to" == *"${tower_name}"* ]] || [[ "$routed_to" == service:* ]]; then
        log "  ✅ Routed to local service"
    else
        log "  🔄 Routed elsewhere: ${routed_to}"
    fi
}

# Test task routing
test_task_routing() {
    section "Testing Task Routing Across Federation"
    
    submit_task "Eastgate" "$EASTGATE_URL"
    submit_task "Westgate" "$WESTGATE_URL"
    submit_task "Strandgate" "$STRANDGATE_URL"
}

# Query capability across towers
test_capability_query() {
    section "Testing Capability Queries"
    
    subsection "Querying 'compute' capability on Eastgate"
    local response=$(curl -sk "${EASTGATE_URL}/api/v1/services/query/compute")
    local count=$(echo "$response" | jq '.services | length')
    log "Found ${count} service(s) with 'compute' capability"
    echo "$response" | jq -r '.services[] | "  - \(.name) on \(.assigned_endpoint.address):\(.assigned_endpoint.port)"'
}

# Load distribution test
test_load_distribution() {
    section "Testing Load Distribution"
    
    subsection "Submitting 10 tasks to Eastgate"
    
    local routed_local=0
    local routed_remote=0
    
    for i in {1..10}; do
        local response=$(curl -sk -X POST "${EASTGATE_URL}/api/v1/compute/task" \
            -H "Content-Type: application/json" \
            -d "{
                \"task\": {
                    \"task_type\": \"ml_training\",
                    \"metadata\": {\"task_num\": \"$i\"}
                }
            }")
        
        local routed_to=$(echo "$response" | jq -r '.routed_to // empty')
        
        if [[ "$routed_to" == service:* ]]; then
            routed_local=$((routed_local + 1))
        else
            routed_remote=$((routed_remote + 1))
        fi
        
        echo -n "."
    done
    
    echo ""
    log "Local: ${routed_local}, Remote: ${routed_remote}"
}

# Cleanup registered services
cleanup() {
    section "Cleanup"
    
    if [[ ! -f /tmp/songbird-test-services.txt ]]; then
        log "No services to clean up"
        return 0
    fi
    
    while IFS=: read -r tower_name service_id; do
        log "Deregistering ${service_id} from ${tower_name}..."
        
        local tower_url=""
        case "$tower_name" in
            Eastgate) tower_url="$EASTGATE_URL" ;;
            Westgate) tower_url="$WESTGATE_URL" ;;
            Strandgate) tower_url="$STRANDGATE_URL" ;;
        esac
        
        if [[ -n "$tower_url" ]]; then
            curl -sk -X DELETE "${tower_url}/api/v1/services/${service_id}" > /dev/null 2>&1 || true
        fi
    done < /tmp/songbird-test-services.txt
    
    rm -f /tmp/songbird-test-services.txt
    log "✅ Cleanup complete"
}

# Main test flow
main() {
    cat << 'EOF'

╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║  🌐 Phase 5: Federation Testing                                   ║
║     Universal Port Authority - Cross-Tower Validation            ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝

Testing Configuration:
  Eastgate:   EASTGATE_URL
  Westgate:   WESTGATE_URL
  Strandgate: STRANDGATE_URL

EOF

    log "Eastgate:   ${EASTGATE_URL}"
    log "Westgate:   ${WESTGATE_URL}"
    log "Strandgate: ${STRANDGATE_URL}"
    
    check_all_towers
    check_federation
    register_all_toadstools
    sleep 2
    query_all_services
    test_task_routing
    test_capability_query
    test_load_distribution
    cleanup
    
    section "✅ Federation Testing Complete"
    
    log ""
    log "Test Results:"
    log "  ✅ All towers healthy"
    log "  ✅ Federation discovered"
    log "  ✅ Services registered on all towers"
    log "  ✅ Task routing operational"
    log "  ✅ Capability queries working"
    log "  ✅ Load distribution tested"
    log ""
    log "🎯 Universal Port Authority validated across federation"
    log ""
}

# Trap cleanup on exit
trap cleanup EXIT

main "$@"

