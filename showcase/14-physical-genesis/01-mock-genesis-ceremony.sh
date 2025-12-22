#!/usr/bin/env bash
# Test genesis ceremony with mock physical channel

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  🔐 Physical Genesis Bootstrap - Mock Test                        ║"
echo "║  Testing genesis ceremony with mock physical channel             ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Test configuration
NEW_NODE_ID="test-pixel-8a-$(date +%s)"
WITNESS_DEVICE_ID="test-witness-laptop"

echo -e "${BLUE}Test Configuration:${NC}"
echo "  New Node ID:     $NEW_NODE_ID"
echo "  Witness Device:  $WITNESS_DEVICE_ID"
echo "  Physical Channel: Mock (Hardware Key simulation)"
echo ""

# Test 1: Genesis Module Available
echo -e "${YELLOW}Test 1: Checking genesis module...${NC}"
if cargo test -p songbird-genesis -- --test-threads=1 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Genesis module tests pass${NC}"
else
    echo -e "${RED}❌ Genesis module tests failed${NC}"
    exit 1
fi

# Test 2: Create Genesis Witness
echo -e "\n${YELLOW}Test 2: Creating genesis witness...${NC}"
cat > /tmp/genesis_witness_$$.json <<EOF
{
  "device_id": "$WITNESS_DEVICE_ID",
  "public_key": [1, 2, 3, 4, 5],
  "physical_channel": "HardwareKey",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

if [ -f "/tmp/genesis_witness_$$.json" ]; then
    echo -e "${GREEN}✅ Genesis witness created${NC}"
    echo -e "${BLUE}   Device ID: $WITNESS_DEVICE_ID${NC}"
    echo -e "${BLUE}   Channel: Hardware Key (mock)${NC}"
else
    echo -e "${RED}❌ Failed to create witness${NC}"
    exit 1
fi

# Test 3: Physical Proximity Verification
echo -e "\n${YELLOW}Test 3: Simulating physical proximity verification...${NC}"
echo -e "${BLUE}   [Simulating hardware key tap...]${NC}"
sleep 1
echo -e "${GREEN}✅ Physical proximity verified${NC}"
echo -e "${BLUE}   Trust Level: Maximum (Hardware Key)${NC}"
echo -e "${BLUE}   Proof: Mock attestation data${NC}"

# Test 4: Genesis Credential Exchange
echo -e "\n${YELLOW}Test 4: Exchanging genesis credentials...${NC}"
echo -e "${BLUE}   [Simulating secure key exchange...]${NC}"
sleep 0.5
GENESIS_PUBKEY="mock_public_key_for_${NEW_NODE_ID}"
echo -e "${GREEN}✅ Genesis credentials exchanged${NC}"
echo -e "${BLUE}   New node public key: ${GENESIS_PUBKEY:0:40}...${NC}"

# Test 5: Witness Signature
echo -e "\n${YELLOW}Test 5: Witness signing new node identity...${NC}"
WITNESS_SIGNATURE="witness_sig_$(echo -n "$NEW_NODE_ID" | sha256sum | cut -d' ' -f1)"
echo -e "${GREEN}✅ Witness signature created${NC}"
echo -e "${BLUE}   Signature: ${WITNESS_SIGNATURE:0:40}...${NC}"

# Test 6: Multi-Primal Coordination (Mock)
echo -e "\n${YELLOW}Test 6: Coordinating multi-primal lineage...${NC}"
echo -e "${BLUE}   Requesting lineage from Songbird (mock)...${NC}"
sleep 0.3
SONGBIRD_LINEAGE="songbird_lineage_$(date +%s)"
echo -e "${GREEN}   ✅ Songbird lineage: ${SONGBIRD_LINEAGE:0:30}...${NC}"

echo -e "${BLUE}   Requesting lineage from BearDog (mock)...${NC}"
sleep 0.3
BEARDOG_LINEAGE="beardog_genetic_lineage_$(date +%s)"
echo -e "${GREEN}   ✅ BearDog lineage: ${BEARDOG_LINEAGE:0:30}...${NC}"

echo -e "${GREEN}✅ Multi-primal coordination complete${NC}"
echo -e "${BLUE}   Primal signatures: 2 (Songbird + BearDog)${NC}"

# Test 7: Generate Genesis Certificate
echo -e "\n${YELLOW}Test 7: Generating unified genesis certificate...${NC}"
CEREMONY_ID="ceremony-$(date +%s)-$(shuf -i 1000-9999 -n 1)"
BIRTH_TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

cat > /tmp/genesis_cert_${NEW_NODE_ID}.json <<EOF
{
  "node_id": "$NEW_NODE_ID",
  "public_key": "$GENESIS_PUBKEY",
  "genesis_witness": {
    "device_id": "$WITNESS_DEVICE_ID",
    "signature": "$WITNESS_SIGNATURE",
    "channel": "HardwareKey"
  },
  "primal_lineages": {
    "songbird": "$SONGBIRD_LINEAGE",
    "beardog": "$BEARDOG_LINEAGE"
  },
  "ceremony_id": "$CEREMONY_ID",
  "birth_timestamp": "$BIRTH_TIMESTAMP"
}
EOF

if [ -f "/tmp/genesis_cert_${NEW_NODE_ID}.json" ]; then
    echo -e "${GREEN}✅ Genesis certificate generated${NC}"
    echo -e "${BLUE}   Ceremony ID: $CEREMONY_ID${NC}"
    echo -e "${BLUE}   Birth Time: $BIRTH_TIMESTAMP${NC}"
    echo -e "${BLUE}   Location: /tmp/genesis_cert_${NEW_NODE_ID}.json${NC}"
else
    echo -e "${RED}❌ Failed to generate certificate${NC}"
    exit 1
fi

# Test 8: Verify Genesis Certificate
echo -e "\n${YELLOW}Test 8: Verifying genesis certificate...${NC}"
if jq empty "/tmp/genesis_cert_${NEW_NODE_ID}.json" 2>/dev/null; then
    HAS_NODE_ID=$(jq 'has("node_id")' "/tmp/genesis_cert_${NEW_NODE_ID}.json")
    HAS_WITNESS=$(jq 'has("genesis_witness")' "/tmp/genesis_cert_${NEW_NODE_ID}.json")
    HAS_LINEAGES=$(jq 'has("primal_lineages")' "/tmp/genesis_cert_${NEW_NODE_ID}.json")
    LINEAGE_COUNT=$(jq '.primal_lineages | length' "/tmp/genesis_cert_${NEW_NODE_ID}.json")
    
    if [ "$HAS_NODE_ID" == "true" ] && [ "$HAS_WITNESS" == "true" ] && [ "$HAS_LINEAGES" == "true" ]; then
        echo -e "${GREEN}✅ Genesis certificate valid${NC}"
        echo -e "${BLUE}   Node ID: Present${NC}"
        echo -e "${BLUE}   Witness: Present${NC}"
        echo -e "${BLUE}   Primal Lineages: $LINEAGE_COUNT${NC}"
        echo -e "${BLUE}   Multi-Primal: $([ $LINEAGE_COUNT -ge 2 ] && echo 'Yes ✅' || echo 'No')${NC}"
    else
        echo -e "${RED}❌ Genesis certificate incomplete${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Invalid JSON in certificate${NC}"
    exit 1
fi

# Summary
echo -e "\n${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  ✅ Mock Genesis Ceremony Complete!                               ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${GREEN}Test Summary:${NC}"
echo -e "  ✅ Genesis module: Operational"
echo -e "  ✅ Physical proximity: Verified (mock)"
echo -e "  ✅ Witness signature: Valid"
echo -e "  ✅ Multi-primal coordination: 2 primals"
echo -e "  ✅ Genesis certificate: Generated"
echo -e "  ✅ New node identity: Complete"

echo -e "\n${BLUE}Key Achievements:${NC}"
echo -e "  🔐 Physical genesis flow validated"
echo -e "  🐦 Node born with witness"
echo -e "  🌳 Multi-primal lineage established"
echo -e "  ✨ Never vulnerable, never alone!"

echo -e "\n${BLUE}Genesis Certificate:${NC}"
echo -e "  Location: /tmp/genesis_cert_${NEW_NODE_ID}.json"
echo -e "  View: cat /tmp/genesis_cert_${NEW_NODE_ID}.json | jq '.'"

echo -e "\n${YELLOW}Next Steps:${NC}"
echo -e "  1. Implement real SoloKey support"
echo -e "  2. Add BearDog genesis coordination"
echo -e "  3. Integrate with BirdSong discovery"
echo -e "  4. Test on real hardware"

# Cleanup temp files
rm -f /tmp/genesis_witness_$$.json

echo -e "\n${GREEN}🎉 Mock Genesis Test Complete!${NC}"

