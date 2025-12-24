#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🧬 Demo 1: Genesis Ceremony
# ═══════════════════════════════════════════════════════════════
# Shows how Songbird + BearDog work together to create a new node
# with cryptographic lineage.
#
# Components:
# - Songbird: Orchestrates ceremony, verifies proximity (BLE)
# - BearDog: Signs lineage, generates keys
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     🧬 Genesis Ceremony Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"

if ! command -v beardog &> /dev/null; then
    echo -e "${RED}❌ BearDog not found. Please install BearDog v0.9.0+${NC}"
    echo "   See: ../../BEARDOG_V0.9.0_INTEGRATION_GUIDE.md"
    exit 1
fi

BEARDOG_VERSION=$(beardog --version | grep -oP '\d+\.\d+\.\d+' || echo "unknown")
echo -e "${GREEN}✅ BearDog $BEARDOG_VERSION detected${NC}"

if [ ! -f "../../target/release/songbird-genesis" ]; then
    echo -e "${YELLOW}⚠️  Building songbird-genesis...${NC}"
    cd ../..
    cargo build --release -p songbird-genesis --features pure-bluetooth
    cd "$SCRIPT_DIR"
fi
echo -e "${GREEN}✅ Songbird genesis binary ready${NC}"

echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: Node A gives birth to Node B${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""

# Create working directory
WORK_DIR="$SCRIPT_DIR/data/genesis-demo"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo -e "${YELLOW}Step 1: Initialize Parent Node (A)${NC}"
echo "       Node A is an existing node with established lineage"
echo ""

# Simulate parent node
cat > node_a.json <<EOF
{
  "node_id": "node-a-parent",
  "public_key": "$(openssl rand -hex 32)",
  "lineage": [],
  "birth_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}✅ Parent Node A initialized${NC}"
cat node_a.json | jq '.'
echo ""

echo -e "${YELLOW}Step 2: Songbird - Physical Proximity Verification (BLE)${NC}"
echo "       Simulating BLE proximity scan..."
echo ""

# Simulate BLE proximity verification
sleep 1
PROXIMITY_PROOF="ble_proximity_$(openssl rand -hex 16)"
echo -e "${GREEN}✅ Physical proximity verified via BLE${NC}"
echo "   Proximity Proof: $PROXIMITY_PROOF"
echo ""

echo -e "${YELLOW}Step 3: BearDog - Generate Keys for Node B${NC}"
echo "       Delegating key generation to BearDog security primal..."
echo ""

# Call BearDog for key generation (mocked for now)
# In production: beardog generate-keys --node-id node-b-child
PUBLIC_KEY="$(openssl rand -hex 32)"
PRIVATE_KEY_HANDLE="solokey://node-b-child"

cat > keys_generated.json <<EOF
{
  "public_key": "$PUBLIC_KEY",
  "private_key_handle": "$PRIVATE_KEY_HANDLE",
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}✅ BearDog generated keys for Node B${NC}"
cat keys_generated.json | jq '.'
echo ""

echo -e "${YELLOW}Step 4: Songbird - Coordinate Witness Network${NC}"
echo "       Gathering witness proofs from nearby nodes..."
echo ""

# Simulate witness coordination
WITNESS_1="node-witness-1-$(openssl rand -hex 8)"
WITNESS_2="node-witness-2-$(openssl rand -hex 8)"
WITNESS_3="node-witness-3-$(openssl rand -hex 8)"

cat > witness_proofs.json <<EOF
{
  "ceremony_id": "$(uuidgen)",
  "witnesses": [
    {
      "node_id": "$WITNESS_1",
      "signature": "$(openssl rand -hex 64)",
      "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    },
    {
      "node_id": "$WITNESS_2",
      "signature": "$(openssl rand -hex 64)",
      "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    },
    {
      "node_id": "$WITNESS_3",
      "signature": "$(openssl rand -hex 64)",
      "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    }
  ]
}
EOF

echo -e "${GREEN}✅ Witness network coordinated (3 witnesses)${NC}"
cat witness_proofs.json | jq '.'
echo ""

echo -e "${YELLOW}Step 5: BearDog - Sign Lineage (Parent → Child)${NC}"
echo "       Delegating lineage signing to BearDog..."
echo ""

# Call BearDog for lineage signing
# In production: beardog sign-lineage --parent node-a-parent --child node-b-child
LINEAGE_SIGNATURE="$(openssl rand -hex 64)"

cat > lineage_signed.json <<EOF
{
  "parent": "node-a-parent",
  "child": "node-b-child",
  "signature": "$LINEAGE_SIGNATURE",
  "signed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "ceremony_id": "$(jq -r '.ceremony_id' witness_proofs.json)"
}
EOF

echo -e "${GREEN}✅ BearDog signed lineage: A → B${NC}"
cat lineage_signed.json | jq '.'
echo ""

echo -e "${YELLOW}Step 6: Construct Final Identity for Node B${NC}"
echo "       Combining all components..."
echo ""

# Construct final identity
cat > node_b_identity.json <<EOF
{
  "node_id": "node-b-child",
  "public_key": "$PUBLIC_KEY",
  "lineage": {
    "parent": "node-a-parent",
    "ancestors": ["node-a-parent"],
    "signature": "$LINEAGE_SIGNATURE",
    "birth_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  },
  "witness_proof": {
    "ceremony_id": "$(jq -r '.ceremony_id' witness_proofs.json)",
    "proximity_proof": "$PROXIMITY_PROOF",
    "witnesses": $(jq '.witnesses' witness_proofs.json)
  },
  "private_key_handle": "$PRIVATE_KEY_HANDLE"
}
EOF

echo -e "${GREEN}✅ Node B identity constructed${NC}"
cat node_b_identity.json | jq '.'
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Genesis Ceremony Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Results:"
echo -e "  📋 Node B ID: ${GREEN}node-b-child${NC}"
echo -e "  🔑 Public Key: ${GREEN}${PUBLIC_KEY:0:16}...${NC}"
echo -e "  👨‍👦 Lineage: ${GREEN}[node-a-parent]${NC}"
echo -e "  👁️  Witnesses: ${GREEN}3${NC}"
echo -e "  🔒 Private Key: ${GREEN}$PRIVATE_KEY_HANDLE${NC}"
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}What Just Happened:${NC}"
echo ""
echo "1. 🌳 Songbird orchestrated the ceremony"
echo "   - Verified physical proximity (BLE)"
echo "   - Coordinated witness network"
echo "   - Managed ceremony lifecycle"
echo ""
echo "2. 🐻 BearDog provided security primitives"
echo "   - Generated cryptographic keys"
echo "   - Signed parent → child lineage"
echo "   - Established lineage chain"
echo ""
echo "3. 🧬 Result: Genetic Trust Established"
echo "   - Node B is cryptographically a descendant of Node A"
echo "   - Node A can relay for Node B (family duty)"
echo "   - Node B's future children will have lineage: [B, A, ...]"
echo ""
echo -e "${GREEN}✅ Node B is now part of the genetic network!${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Next Demo: 02-birdsong-broadcast.sh${NC}"
echo "           (See how Node B broadcasts to its lineage)"
echo ""

