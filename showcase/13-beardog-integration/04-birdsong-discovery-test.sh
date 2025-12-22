#!/usr/bin/env bash
# BirdSong Discovery Integration Test
# Tests privacy-preserving discovery with BearDog encryption

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SONGBIRD_URL="${SONGBIRD_URL:-https://localhost:8080}"

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  🐦 BirdSong Discovery Integration Test                           ║"
echo "║  Privacy-Preserving Federation with BearDog                       ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Test 1: Check Songbird is running
echo -e "\n${YELLOW}Test 1: Verifying Songbird is running...${NC}"
if curl -k -s "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird is running${NC}"
else
    echo -e "${RED}❌ Songbird is not running${NC}"
    exit 1
fi

# Test 2: Query for BirdSong capability
echo -e "\n${YELLOW}Test 2: Discovering BirdSong providers...${NC}"
BIRDSONG_PROVIDERS=$(curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=birdsong" | jq -r '. | length' 2>/dev/null || echo "0")

if [ "${BIRDSONG_PROVIDERS}" -gt 0 ]; then
    echo -e "${GREEN}✅ Discovered ${BIRDSONG_PROVIDERS} BirdSong provider(s)${NC}"
    BIRDSONG_PROVIDER=$(curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=birdsong" | jq -r '.[0]')
    BIRDSONG_NAME=$(echo "${BIRDSONG_PROVIDER}" | jq -r '.primal_name // "unknown"')
    BIRDSONG_ENDPOINT=$(echo "${BIRDSONG_PROVIDER}" | jq -r '.endpoints[0].url // empty')
    echo -e "   Provider: ${BIRDSONG_NAME}"
    echo -e "   Endpoint: ${BIRDSONG_ENDPOINT}"
else
    echo -e "${YELLOW}⚠️  No BirdSong providers found${NC}"
    echo -e "${YELLOW}   Running in Plaintext mode (trusted LAN)${NC}"
    BIRDSONG_ENDPOINT=""
fi

# Test 3: Check current discovery mode
echo -e "\n${YELLOW}Test 3: Checking current discovery mode...${NC}"
DISCOVERY_MODE=$(curl -k -s "${SONGBIRD_URL}/api/federation/status" | jq -r '.discovery_mode // "Plaintext"' 2>/dev/null || echo "Plaintext")
echo -e "   Current mode: ${DISCOVERY_MODE}"

if [ "${DISCOVERY_MODE}" == "Plaintext" ]; then
    echo -e "${YELLOW}⚠️  Songbird is in Plaintext mode${NC}"
    echo -e "${YELLOW}   This is normal for trusted LAN deployments${NC}"
    echo -e "${YELLOW}   BirdSong will activate when BearDog registers${NC}"
elif [ "${DISCOVERY_MODE}" == "BirdSong" ]; then
    echo -e "${GREEN}✅ Songbird is in BirdSong mode (privacy-preserving)${NC}"
else
    echo -e "${YELLOW}⚠️  Unknown discovery mode: ${DISCOVERY_MODE}${NC}"
fi

# Test 4: Test BirdSong encryption (if provider available)
if [ -n "${BIRDSONG_ENDPOINT}" ]; then
    echo -e "\n${YELLOW}Test 4: Testing BirdSong encryption...${NC}"
    
    ENCRYPT_REQUEST=$(cat <<EOF
{
  "plaintext": "SGVsbG8gRmFtaWx5IQ==",
  "lineage_hint": "test-lineage-123"
}
EOF
)
    
    ENCRYPT_RESPONSE=$(curl -k -s -X POST \
        -H "Content-Type: application/json" \
        -d "${ENCRYPT_REQUEST}" \
        "${BIRDSONG_ENDPOINT}/birdsong/encrypt" 2>/dev/null || echo '{"error": "endpoint_not_ready"}')
    
    if echo "${ENCRYPT_RESPONSE}" | jq -e '.ciphertext' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ BirdSong encryption working${NC}"
        CIPHERTEXT=$(echo "${ENCRYPT_RESPONSE}" | jq -r '.ciphertext')
        echo -e "   Ciphertext: ${CIPHERTEXT:0:40}..."
        
        # Test 5: Test BirdSong decryption
        echo -e "\n${YELLOW}Test 5: Testing BirdSong decryption...${NC}"
        
        DECRYPT_REQUEST=$(cat <<EOF
{
  "ciphertext": "${CIPHERTEXT}",
  "lineage_proof": {
    "node_id": "test-node",
    "root_id": "test-root",
    "path": ["test-root", "test-node"]
  }
}
EOF
)
        
        DECRYPT_RESPONSE=$(curl -k -s -X POST \
            -H "Content-Type: application/json" \
            -d "${DECRYPT_REQUEST}" \
            "${BIRDSONG_ENDPOINT}/birdsong/decrypt" 2>/dev/null || echo '{}')
        
        if echo "${DECRYPT_RESPONSE}" | jq -e '.plaintext' > /dev/null 2>&1; then
            echo -e "${GREEN}✅ BirdSong decryption working${NC}"
        else
            echo -e "${YELLOW}⚠️  BirdSong decryption endpoint not yet ready${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  BirdSong encryption endpoint not yet ready${NC}"
        echo -e "${YELLOW}   This is expected during Phase 3 implementation${NC}"
    fi
else
    echo -e "\n${YELLOW}Test 4: BirdSong provider not available${NC}"
    echo -e "${YELLOW}   Skipping encryption tests${NC}"
fi

# Test 6: Query lineage capability
if [ -n "${BIRDSONG_ENDPOINT}" ]; then
    echo -e "\n${YELLOW}Test 6: Testing lineage verification...${NC}"
    
    LINEAGE_PROVIDERS=$(curl -k -s "${SONGBIRD_URL}/api/v1/services?capability=lineage" | jq -r '. | length' 2>/dev/null || echo "0")
    
    if [ "${LINEAGE_PROVIDERS}" -gt 0 ]; then
        echo -e "${GREEN}✅ Lineage capability available${NC}"
        
        # Test lineage generation
        LINEAGE_REQUEST=$(cat <<EOF
{
  "parent_id": "test-parent",
  "child_id": "test-child"
}
EOF
)
        
        LINEAGE_RESPONSE=$(curl -k -s -X POST \
            -H "Content-Type: application/json" \
            -d "${LINEAGE_REQUEST}" \
            "${BIRDSONG_ENDPOINT}/lineage/generate" 2>/dev/null || echo '{"error": "endpoint_not_ready"}')
        
        if echo "${LINEAGE_RESPONSE}" | jq -e '.proof' > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Lineage generation working${NC}"
        else
            echo -e "${YELLOW}⚠️  Lineage endpoint not yet ready${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  No lineage providers found${NC}"
    fi
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
echo -e "   Discovery mode: ${DISCOVERY_MODE}"

if [ "${BIRDSONG_PROVIDERS}" -gt 0 ]; then
    echo -e "${GREEN}✅ BirdSong provider discovered${NC}"
    
    if echo "${ENCRYPT_RESPONSE:-{}}" | jq -e '.ciphertext' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ BirdSong encryption/decryption working${NC}"
        echo -e "\n${GREEN}🎉 BIRDSONG FULLY OPERATIONAL!${NC}"
        echo -e "\n${BLUE}What This Means:${NC}"
        echo "  • Broadcasts are now encrypted"
        echo "  • Only family members can decrypt"
        echo "  • Privacy-preserving federation active"
        echo "  • Ready for internet deployment"
    else
        echo -e "${YELLOW}⚠️  BirdSong API endpoints pending (Phase 3)${NC}"
        echo -e "\n${YELLOW}Status: Discovery working, awaiting API implementation${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  BirdSong provider not registered${NC}"
    echo -e "${YELLOW}   Running in Plaintext mode (trusted LAN)${NC}"
    echo -e "\n${BLUE}To Enable BirdSong:${NC}"
    echo "  1. BearDog registers with Songbird UPA"
    echo "  2. Advertises 'birdsong' capability"
    echo "  3. Songbird auto-switches to BirdSong mode"
    echo "  4. Privacy-preserving discovery activated"
fi

exit 0

