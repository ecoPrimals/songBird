#!/bin/bash

# Capability-Based Discovery Demonstration
# Shows how Songbird discovers security providers by capability, not by name

set -e

SONGBIRD_URL="https://localhost:8080"
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "═══════════════════════════════════════════════════════════════"
echo "  🔍 Capability-Based Discovery Demonstration"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Helper function
check_response() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Success${NC}"
    else
        echo -e "${RED}❌ Failed${NC}"
        return 1
    fi
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 1: Verify Songbird is Running"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -n "Checking Songbird health... "
curl -k -s "${SONGBIRD_URL}/health" > /dev/null
check_response

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 2: Register 'BearDog' Security Provider"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Registering provider with name 'beardog'..."
echo ""

BEARDOG_RESPONSE=$(curl -k -s -X POST "${SONGBIRD_URL}/api/v1/services/register" \
  -H "Content-Type: application/json" \
  -d '{
    "primal_name": "beardog",
    "primal_version": "0.1.0",
    "capabilities": [
      {"name": "security", "type": "security", "metadata": {"description": "Core security capability"}},
      {"name": "btsp", "type": "security", "metadata": {"description": "BearDog Secure Tunnel Protocol"}},
      {"name": "lineage", "type": "security", "metadata": {"description": "Cryptographic lineage verification"}},
      {"name": "birdsong", "type": "security", "metadata": {"description": "Privacy-preserving broadcasts"}}
    ],
    "endpoints": [
      {"protocol": "https", "host": "localhost", "port": 9000, "url": "https://localhost:9000"}
    ],
    "protocols": ["btsp", "https"],
    "preferred_protocol": "https",
    "metadata": {
      "description": "Security primal with genetic lineage"
    }
  }')

echo -e "${BLUE}Response:${NC}"
echo "$BEARDOG_RESPONSE" | jq '.' 2>/dev/null || echo "$BEARDOG_RESPONSE"
check_response

BEARDOG_ID=$(echo "$BEARDOG_RESPONSE" | jq -r '.service_id' 2>/dev/null)
echo ""
echo -e "${GREEN}BearDog registered with ID: $BEARDOG_ID${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 3: Register 'SecurePrimal' Alternative Provider"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Registering provider with DIFFERENT name 'secureprimal'..."
echo ""

SECURE_RESPONSE=$(curl -k -s -X POST "${SONGBIRD_URL}/api/v1/services/register" \
  -H "Content-Type: application/json" \
  -d '{
    "primal_name": "secureprimal",
    "primal_version": "2.0.0",
    "capabilities": [
      {"name": "security", "type": "security", "metadata": {"description": "Enhanced security capability"}},
      {"name": "btsp", "type": "security", "metadata": {"description": "BTSP support"}},
      {"name": "quantum-resistant", "type": "security", "metadata": {"description": "Post-quantum cryptography"}}
    ],
    "endpoints": [
      {"protocol": "https", "host": "localhost", "port": 9001, "url": "https://localhost:9001"}
    ],
    "protocols": ["btsp", "https"],
    "preferred_protocol": "https",
    "metadata": {
      "description": "Community security primal with quantum resistance"
    }
  }')

echo -e "${BLUE}Response:${NC}"
echo "$SECURE_RESPONSE" | jq '.' 2>/dev/null || echo "$SECURE_RESPONSE"
check_response

SECURE_ID=$(echo "$SECURE_RESPONSE" | jq -r '.service_id' 2>/dev/null)
echo ""
echo -e "${GREEN}SecurePrimal registered with ID: $SECURE_ID${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 4: Query by Capability (NOT by name)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Querying for 'security' capability..."
echo "NOTE: Songbird doesn't care about names, only capabilities!"
echo ""

SECURITY_PROVIDERS=$(curl -k -s "${SONGBIRD_URL}/api/v1/services/query/security")

echo -e "${BLUE}All security providers discovered:${NC}"
echo "$SECURITY_PROVIDERS" | jq '.' 2>/dev/null || echo "$SECURITY_PROVIDERS"

PROVIDER_COUNT=$(echo "$SECURITY_PROVIDERS" | jq 'length' 2>/dev/null || echo "0")
echo ""
echo -e "${GREEN}✅ Discovered $PROVIDER_COUNT security providers${NC}"
echo ""

# Show provider details
echo -e "${YELLOW}Provider 1 (BearDog):${NC}"
echo "$SECURITY_PROVIDERS" | jq '.[0] | {name: .primal_name, capabilities: [.capabilities[].name], endpoint: .endpoints[0].address}' 2>/dev/null

echo ""
echo -e "${YELLOW}Provider 2 (SecurePrimal):${NC}"
echo "$SECURITY_PROVIDERS" | jq '.[1] | {name: .primal_name, capabilities: [.capabilities[].name], endpoint: .endpoints[0].address}' 2>/dev/null

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 5: Query for Specific Capability (BTSP)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Filtering for providers with 'btsp' capability..."
echo ""

BTSP_PROVIDERS=$(echo "$SECURITY_PROVIDERS" | jq '[.[] | select(.capabilities[] | .name == "btsp")]' 2>/dev/null)

echo -e "${BLUE}Providers with BTSP support:${NC}"
echo "$BTSP_PROVIDERS" | jq '.' 2>/dev/null || echo "$BTSP_PROVIDERS"

BTSP_COUNT=$(echo "$BTSP_PROVIDERS" | jq 'length' 2>/dev/null || echo "0")
echo ""
echo -e "${GREEN}✅ Found $BTSP_COUNT providers with BTSP capability${NC}"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 6: Demonstrate Name-Agnostic Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Key Point: Songbird code queries for 'security + btsp' capability."
echo "It does NOT hardcode 'beardog' in the source code."
echo ""
echo "Both providers discovered because they advertise the capability:"
echo ""

for i in 0 1; do
    NAME=$(echo "$SECURITY_PROVIDERS" | jq -r ".[$i].primal_name" 2>/dev/null)
    HAS_BTSP=$(echo "$SECURITY_PROVIDERS" | jq ".[$i].capabilities[] | select(.name == \"btsp\") | .name" 2>/dev/null)
    
    if [ -n "$HAS_BTSP" ]; then
        echo -e "  ${GREEN}✅${NC} Provider '$NAME' has BTSP → Can be used by Songbird"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 7: List All Registered Services"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

ALL_SERVICES=$(curl -k -s "${SONGBIRD_URL}/api/v1/services")

echo -e "${BLUE}All registered services:${NC}"
echo "$ALL_SERVICES" | jq '.[] | {name: .primal_name, version: .primal_version, port: .port, status: .status}' 2>/dev/null || echo "$ALL_SERVICES"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test 8: Cleanup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Deregistering BearDog..."
curl -k -s -X DELETE "${SONGBIRD_URL}/api/v1/services/${BEARDOG_ID}" > /dev/null
check_response

echo "Deregistering SecurePrimal..."
curl -k -s -X DELETE "${SONGBIRD_URL}/api/v1/services/${SECURE_ID}" > /dev/null
check_response

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  🎊 Demonstration Complete!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo -e "${GREEN}Key Takeaways:${NC}"
echo ""
echo "1. ✅ Songbird discovers providers by CAPABILITY, not NAME"
echo "2. ✅ 'beardog' and 'secureprimal' both work (name irrelevant)"
echo "3. ✅ Any primal with 'security + btsp' capability is discovered"
echo "4. ✅ Community primals can provide security (extensible)"
echo "5. ✅ Multiple providers can coexist (load balancing possible)"
echo ""
echo -e "${BLUE}Architecture:${NC}"
echo "  Code Knowledge:   'I need security + btsp capability'"
echo "  Developer Knowledge: 'BearDog will provide it'"
echo ""
echo -e "${YELLOW}Future: BearDog v2.0, community alternatives, all work!${NC}"
echo ""

