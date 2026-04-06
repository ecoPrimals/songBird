#!/usr/bin/env bash
# Songbird Security Provider Health Monitor
# Continuously monitors registered security providers for health and performance

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SONGBIRD_URL="${SONGBIRD_URL:-https://localhost:8080}"
CHECK_INTERVAL="${CHECK_INTERVAL:-30}"
ALERT_THRESHOLD_MS="${ALERT_THRESHOLD_MS:-1000}"

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  Songbird Security Provider Health Monitor                         ║"
echo "║  Real-time monitoring of security providers                       ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo "Configuration:"
echo "  Songbird URL: ${SONGBIRD_URL}"
echo "  Check Interval: ${CHECK_INTERVAL}s"
echo "  Alert Threshold: ${ALERT_THRESHOLD_MS}ms"
echo ""

# Function to check endpoint health
check_endpoint_health() {
    local endpoint=$1
    local start_time=$(date +%s%3N)
    
    # Try to reach health endpoint
    local http_code=$(curl -k -s -o /dev/null -w "%{http_code}" "${endpoint}/health" 2>/dev/null || echo "000")
    
    local end_time=$(date +%s%3N)
    local response_time=$((end_time - start_time))
    
    echo "${http_code}:${response_time}"
}

# Function to query for security providers
get_security_providers() {
    curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=crypto.delegate" 2>/dev/null || echo "[]"
}

# Function to display health status
display_health() {
    local provider_name=$1
    local endpoint=$2
    local http_code=$3
    local response_time=$4
    
    if [ "${http_code}" == "200" ]; then
        if [ "${response_time}" -lt "${ALERT_THRESHOLD_MS}" ]; then
            echo -e "${GREEN}✅ ${provider_name} - HEALTHY${NC} (${endpoint}, ${response_time}ms)"
        else
            echo -e "${YELLOW}⚠️  ${provider_name} - SLOW${NC} (${endpoint}, ${response_time}ms - threshold: ${ALERT_THRESHOLD_MS}ms)"
        fi
    elif [ "${http_code}" == "000" ]; then
        echo -e "${RED}❌ ${provider_name} - UNREACHABLE${NC} (${endpoint})"
    else
        echo -e "${RED}❌ ${provider_name} - ERROR${NC} (${endpoint}, HTTP ${http_code})"
    fi
}

# Main monitoring loop
echo -e "${BLUE}Starting continuous monitoring...${NC}\n"

iteration=0
while true; do
    iteration=$((iteration + 1))
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}Check #${iteration} - ${timestamp}${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    
    # Query for security providers
    providers=$(get_security_providers)
    provider_count=$(echo "${providers}" | jq '. | length' 2>/dev/null || echo "0")
    
    if [ "${provider_count}" -eq 0 ]; then
        echo -e "${YELLOW}⚠️  No security providers registered${NC}"
        echo -e "${YELLOW}   Waiting for security provider (crypto.delegate capability)...${NC}"
    else
        echo -e "${GREEN}Found ${provider_count} BTSP provider(s):${NC}\n"
        
        # Check each provider
        for i in $(seq 0 $((provider_count - 1))); do
            provider=$(echo "${providers}" | jq -r ".[$i]")
            provider_name=$(echo "${provider}" | jq -r '.primal_name // "unknown"')
            endpoint=$(echo "${provider}" | jq -r '.endpoints[0].url // empty')
            
            if [ -n "${endpoint}" ]; then
                # Check health
                health_result=$(check_endpoint_health "${endpoint}")
                http_code=$(echo "${health_result}" | cut -d':' -f1)
                response_time=$(echo "${health_result}" | cut -d':' -f2)
                
                display_health "${provider_name}" "${endpoint}" "${http_code}" "${response_time}"
                
                # Check security provider endpoints
                echo -e "   ${BLUE}Security Provider Capabilities:${NC}"
                
                # Check tunnel establishment endpoint
                tunnel_check=$(curl -k -s -o /dev/null -w "%{http_code}" "${endpoint}/security/tunnel/establish" -X POST -H "Content-Type: application/json" -d '{}' 2>/dev/null || echo "000")
                if [ "${tunnel_check}" == "400" ] || [ "${tunnel_check}" == "200" ]; then
                    echo -e "     ${GREEN}✅ Tunnel establishment endpoint active${NC}"
                else
                    echo -e "     ${YELLOW}⚠️  Tunnel establishment endpoint: HTTP ${tunnel_check}${NC}"
                fi
                
                # Check lineage endpoint (if BirdSong capable)
                if echo "${provider}" | jq -e '.capabilities[] | select(.name == "lineage")' > /dev/null 2>&1; then
                    lineage_check=$(curl -k -s -o /dev/null -w "%{http_code}" "${endpoint}/lineage/generate" -X POST -H "Content-Type: application/json" -d '{}' 2>/dev/null || echo "000")
                    if [ "${lineage_check}" == "400" ] || [ "${lineage_check}" == "200" ]; then
                        echo -e "     ${GREEN}✅ Lineage endpoint active${NC}"
                    else
                        echo -e "     ${YELLOW}⚠️  Lineage endpoint: HTTP ${lineage_check}${NC}"
                    fi
                fi
                
                # Check BirdSong endpoint
                if echo "${provider}" | jq -e '.capabilities[] | select(.name == "birdsong")' > /dev/null 2>&1; then
                    birdsong_check=$(curl -k -s -o /dev/null -w "%{http_code}" "${endpoint}/birdsong/encrypt" -X POST -H "Content-Type: application/json" -d '{}' 2>/dev/null || echo "000")
                    if [ "${birdsong_check}" == "400" ] || [ "${birdsong_check}" == "200" ]; then
                        echo -e "     ${GREEN}✅ BirdSong endpoint active${NC}"
                    else
                        echo -e "     ${YELLOW}⚠️  BirdSong endpoint: HTTP ${birdsong_check}${NC}"
                    fi
                fi
                
            else
                echo -e "${RED}❌ ${provider_name} - NO ENDPOINT${NC}"
            fi
            echo ""
        done
    fi
    
    # Wait for next check
    echo -e "${BLUE}Next check in ${CHECK_INTERVAL} seconds...${NC}\n"
    sleep "${CHECK_INTERVAL}"
done

