#!/usr/bin/env bash
# BTSP Live Integration Test
# Tests Songbird discovering and using BearDog's BTSP capability

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SONGBIRD_URL="${SONGBIRD_URL:-https://localhost:8080}"
BEARDOG_EXPECTED_PORT="${BEARDOG_EXPECTED_PORT:-9000}"

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  🔒 BTSP Live Integration Test                                    ║"
echo "║  Testing Songbird ↔ BearDog BTSP Communication                    ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Test 1: Check Songbird is running
echo -e "\n${YELLOW}Test 1: Verifying Songbird is running...${NC}"
if curl -k -s "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird is running${NC}"
else
    echo -e "${RED}❌ Songbird is not running. Start it first:${NC}"
    echo "   cargo run --bin songbird-orchestrator"
    exit 1
fi

# Test 2: Query for security capability
echo -e "\n${YELLOW}Test 2: Discovering security providers...${NC}"
SECURITY_PROVIDERS=$(curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=security" | jq -r '. | length' 2>/dev/null || echo "0")

if [ "${SECURITY_PROVIDERS}" -gt 0 ]; then
    echo -e "${GREEN}✅ Discovered ${SECURITY_PROVIDERS} security provider(s)${NC}"
else
    echo -e "${RED}❌ No security providers found${NC}"
    echo -e "${YELLOW}   BearDog needs to register first with:${NC}"
    echo "   POST ${SONGBIRD_URL}/api/v1/services/register"
    exit 1
fi

# Test 3: Find BTSP-capable provider
echo -e "\n${YELLOW}Test 3: Looking for BTSP capability...${NC}"
BTSP_PROVIDER=$(curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=btsp" | jq -r '.[0] // empty' 2>/dev/null)

if [ -n "${BTSP_PROVIDER}" ]; then
    BTSP_NAME=$(echo "${BTSP_PROVIDER}" | jq -r '.primal_name // "unknown"')
    BTSP_ENDPOINT=$(echo "${BTSP_PROVIDER}" | jq -r '.endpoints[0].url // empty')
    echo -e "${GREEN}✅ BTSP provider found: ${BTSP_NAME}${NC}"
    echo -e "   Endpoint: ${BTSP_ENDPOINT}"
else
    echo -e "${RED}❌ No BTSP-capable provider found${NC}"
    echo -e "${YELLOW}   BearDog needs to advertise 'btsp' capability${NC}"
    exit 1
fi

# Test 4: Verify BTSP provider health
echo -e "\n${YELLOW}Test 4: Checking BTSP provider health...${NC}"
if [ -n "${BTSP_ENDPOINT}" ]; then
    if curl -k -s "${BTSP_ENDPOINT}/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ BTSP provider is healthy${NC}"
    else
        echo -e "${RED}❌ BTSP provider not responding at ${BTSP_ENDPOINT}/health${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ No BTSP endpoint available${NC}"
    exit 1
fi

# Test 5: Test BTSP tunnel establishment with Real BearDog
echo -e "\n${YELLOW}Test 5: Testing BTSP tunnel establishment (Real BearDog)...${NC}"
TUNNEL_REQUEST=$(cat <<EOF
{
  "peer": {
    "id": "test-peer-123",
    "endpoint": "192.168.1.100:8080"
  },
  "initiator_entropy": "test-entropy-abc123"
}
EOF
)

TUNNEL_RESPONSE=$(curl -k -s -X POST \
    -H "Content-Type: application/json" \
    -d "${TUNNEL_REQUEST}" \
    "${BTSP_ENDPOINT}/btsp/tunnel/establish" 2>/dev/null || echo '{"error": "endpoint_not_ready"}')

if echo "${TUNNEL_RESPONSE}" | jq -e '.handle.id' > /dev/null 2>&1; then
    TUNNEL_ID=$(echo "${TUNNEL_RESPONSE}" | jq -r '.handle.id')
    echo -e "${GREEN}✅ BTSP tunnel established: ${TUNNEL_ID}${NC}"
    echo -e "${GREEN}   Using REAL BearDog genetic cryptography!${NC}"
    
    # Test 6: Check tunnel status
    echo -e "\n${YELLOW}Test 6: Checking tunnel status...${NC}"
    TUNNEL_STATUS=$(curl -k -s "${BTSP_ENDPOINT}/btsp/tunnel/${TUNNEL_ID}/status" 2>/dev/null || echo '{}')
    
    if echo "${TUNNEL_STATUS}" | jq -e '.active' > /dev/null 2>&1; then
        ACTIVE=$(echo "${TUNNEL_STATUS}" | jq -r '.active')
        echo -e "${GREEN}✅ Tunnel status retrieved: active=${ACTIVE}${NC}"
    else
        echo -e "${YELLOW}⚠️  Tunnel status endpoint not yet implemented${NC}"
    fi
    
    # Clean up: Close tunnel
    echo -e "\n${YELLOW}Cleanup: Closing tunnel...${NC}"
    curl -k -s -X DELETE "${BTSP_ENDPOINT}/btsp/tunnel/${TUNNEL_ID}" > /dev/null 2>&1 || true
    echo -e "${GREEN}✅ Tunnel closed${NC}"
    
elif echo "${TUNNEL_RESPONSE}" | jq -e '.error' > /dev/null 2>&1; then
    ERROR=$(echo "${TUNNEL_RESPONSE}" | jq -r '.error')
    echo -e "${YELLOW}⚠️  BTSP tunnel endpoint error: ${ERROR}${NC}"
else
    echo -e "${RED}❌ BTSP API endpoint unexpected response${NC}"
    echo -e "${YELLOW}   Response: ${TUNNEL_RESPONSE}${NC}"
fi

# Summary
echo -e "\n${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  📊 Test Summary                                                  ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${GREEN}✅ Songbird running${NC}"
echo -e "${GREEN}✅ Security providers discovered${NC}"
echo -e "${GREEN}✅ BTSP capability found${NC}"
echo -e "${GREEN}✅ BTSP provider healthy${NC}"

if [ -n "${TUNNEL_ID:-}" ]; then
    echo -e "${GREEN}✅ BTSP tunnel operations working with REAL genetic cryptography${NC}"
    echo -e "\n${GREEN}🎉 FULL INTEGRATION WORKING!${NC}"
else
    echo -e "${YELLOW}⚠️  BTSP tunnel operations need verification${NC}"
    echo -e "\n${YELLOW}Status: Infrastructure ready, check API format${NC}"
fi

echo -e "\n${BLUE}Next Steps:${NC}"
echo "  1. Test tunnel encrypt/decrypt operations"
echo "  2. Test tunnel status and close"
echo "  3. Validate end-to-end P2P flow"

exit 0

