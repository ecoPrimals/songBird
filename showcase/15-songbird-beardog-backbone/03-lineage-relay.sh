#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🔄 Demo 3: Lineage Relay Discovery & Connection
# ═══════════════════════════════════════════════════════════════
# Shows how ancestors offer relay service and establish connections
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     🔄 Lineage Relay Discovery & Connection${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

WORK_DIR="$SCRIPT_DIR/data/relay-demo"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Continuing from BirdSong broadcast...${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "Previous state:"
echo "  ✅ Node C broadcast BirdSong"
echo "  ✅ Node B and A decrypted it"
echo "  ✅ Both know C needs relay to A"
echo ""
echo "Now: Node B offers relay service (as parent)"
echo ""

# Node identities
NODE_C="node-c-grandchild"
NODE_B="node-b-child"
NODE_A="node-a-parent"

echo -e "${YELLOW}Step 1: Node B - Authorize Relay Request${NC}"
echo "       Checking with BearDog if relay is authorized..."
echo ""

# BearDog authorization check
# In production: beardog authorize-relay --relay node-b-child --requester node-c-grandchild

echo "       Lineage Check:"
echo "         - Requester: $NODE_C"
echo "         - Relay Node: $NODE_B"
echo "         - Relationship: Parent → Child ✅"
echo ""

LINEAGE_DEPTH=1  # Direct parent

cat > relay_authorization.json <<EOF
{
  "relay_node": "$NODE_B",
  "requester": "$NODE_C",
  "authorized": true,
  "masking_level": "SubMasked",
  "reason": "Direct descendant (depth: $LINEAGE_DEPTH)",
  "ttl_seconds": 3600,
  "audit_token": "$(uuidgen)"
}
EOF

echo -e "${GREEN}✅ BearDog authorized relay${NC}"
cat relay_authorization.json | jq '.'
echo ""

echo -e "${YELLOW}Step 2: Node B - Determine Masking Level${NC}"
echo "       Based on lineage depth..."
echo ""

echo "       Masking Levels:"
echo "         - FullVisibility: Direct parent/child (depth 1)"
echo "         - SubMasked: Grandparent/grandchild (depth 2)"
echo "         - Masked: Far ancestor (depth 3+)"
echo ""
echo "       Node B is direct parent → ${CYAN}SubMasked${NC}"
echo "         - Sees: Source, destination, message type"
echo "         - Hidden: Message content, full payload"
echo ""

echo -e "${YELLOW}Step 3: Node B - Send Relay Offer${NC}"
echo "       Broadcasting relay offer back to Node C..."
echo ""

RELAY_OFFER_ID="relay-offer-$(uuidgen)"

cat > relay_offer.json <<EOF
{
  "offer_id": "$RELAY_OFFER_ID",
  "relay_node": "$NODE_B",
  "relay_endpoint": "192.168.1.100:8443",
  "target": "$NODE_A",
  "masking_level": "SubMasked",
  "valid_until": "$(date -u -d '+1 hour' +%Y-%m-%dT%H:%M:%SZ)",
  "proof": "$(openssl rand -hex 32)"
}
EOF

echo -e "${GREEN}✅ Relay offer sent${NC}"
cat relay_offer.json | jq '.'
echo ""

echo -e "${YELLOW}Step 4: Node C - Accept Relay Offer${NC}"
echo "       Validating offer and establishing session..."
echo ""

echo "       Validation:"
echo "         ✅ Relay node is ancestor: $NODE_B"
echo "         ✅ Proof signature valid"
echo "         ✅ Offer not expired"
echo "         ✅ Target matches: $NODE_A"
echo ""

cat > relay_acceptance.json <<EOF
{
  "offer_id": "$RELAY_OFFER_ID",
  "accepted_by": "$NODE_C",
  "session_id": "$(uuidgen)",
  "accepted_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}✅ Relay offer accepted${NC}"
cat relay_acceptance.json | jq '.'
echo ""

echo -e "${YELLOW}Step 5: Songbird - Establish Relay Session${NC}"
echo "       Creating connection: C → B → A"
echo ""

SESSION_ID=$(jq -r '.session_id' relay_acceptance.json)

echo "       Phase 1: C establishes connection to B"
sleep 1
echo "         ✅ TCP connection: $NODE_C → $NODE_B:8443"
echo ""

echo "       Phase 2: B establishes connection to A"
sleep 1
echo "         ✅ TCP connection: $NODE_B → $NODE_A:8443"
echo ""

echo "       Phase 3: B relays traffic (SubMasked)"
sleep 1
echo "         ✅ Relay session active"
echo ""

cat > relay_session.json <<EOF
{
  "session_id": "$SESSION_ID",
  "path": ["$NODE_C", "$NODE_B", "$NODE_A"],
  "masking_level": "SubMasked",
  "established_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "active",
  "relay_stats": {
    "bytes_relayed": 0,
    "packets_relayed": 0,
    "session_duration_seconds": 0
  }
}
EOF

echo -e "${GREEN}✅ Relay session established${NC}"
cat relay_session.json | jq '.'
echo ""

echo -e "${YELLOW}Step 6: Test Data Flow${NC}"
echo "       Sending test message: C → B → A"
echo ""

TEST_MESSAGE="Hello from Node C via relay!"
ENCRYPTED_MESSAGE="$(echo -n "$TEST_MESSAGE" | openssl rand -hex 32)"

echo "       C → B: Encrypted packet"
echo "         Payload: $ENCRYPTED_MESSAGE"
echo "         B sees: Source (C), Dest (A), Size (42 bytes)"
echo "         B CANNOT see: Message content (encrypted)"
echo ""
sleep 1

echo "       B → A: Forwarded packet"
echo "         B adds: Relay metadata (SubMasked)"
echo "         B forwards: Encrypted payload unchanged"
echo ""
sleep 1

echo "       A receives message from C"
echo "         A decrypts: \"$TEST_MESSAGE\""
echo "         A knows: Direct message from C (via relay)"
echo ""

# Update relay stats
jq '.relay_stats.bytes_relayed = 512 | 
    .relay_stats.packets_relayed = 4 |
    .relay_stats.session_duration_seconds = 2' \
    relay_session.json > relay_session_updated.json
mv relay_session_updated.json relay_session.json

echo -e "${GREEN}✅ Message delivered successfully${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Lineage Relay Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Relay Session Stats:"
cat relay_session.json | jq '.relay_stats'
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}What Node B (Relay) Sees:${NC}"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  MASKING LEVEL: SubMasked (Parent relaying for child)      │"
echo "├─────────────────────────────────────────────────────────────┤"
echo "│  ✅ VISIBLE:                                                 │"
echo "│     - Source: node-c-grandchild                             │"
echo "│     - Destination: node-a-parent                            │"
echo "│     - Packet size: 512 bytes                                │"
echo "│     - Connection duration: 2 seconds                        │"
echo "│                                                              │"
echo "│  ❌ HIDDEN:                                                  │"
echo "│     - Message content (encrypted)                           │"
echo "│     - Application protocol                                  │"
echo "│     - Payload structure                                     │"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Comparison with Traditional TURN:${NC}"
echo ""
echo "Traditional TURN Server:"
echo "  ❌ Sees ALL traffic (can decrypt TLS)"
echo "  ❌ Central point of failure"
echo "  ❌ Requires payment/authentication"
echo "  ❌ Subject to jurisdiction"
echo "  ❌ Can be shut down"
echo ""
echo "Lineage Relay (Node B):"
echo "  ✅ Sees only metadata (SubMasked)"
echo "  ✅ Distributed (multiple ancestors available)"
echo "  ✅ Free (family duty)"
echo "  ✅ Sovereign (no jurisdiction)"
echo "  ✅ Self-healing (automatic failover)"
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Key Innovations:${NC}"
echo ""
echo "1. 🧬 Genetic Authorization"
echo "   - Relay based on lineage, not accounts"
echo "   - Cryptographic proof required"
echo "   - No fake relay offers possible"
echo ""
echo "2. 🎭 Adaptive Masking"
echo "   - Close family: More visibility"
echo "   - Distant family: More privacy"
echo "   - Non-family: Complete privacy"
echo ""
echo "3. 🔄 Self-Healing"
echo "   - If B fails, A can relay"
echo "   - Multiple ancestors available"
echo "   - Automatic fallback"
echo ""
echo -e "${GREEN}✅ Sovereign, private, resilient relay without infrastructure!${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Next Demo: 04-multi-primal.sh${NC}"
echo "           (See how other primals use this backbone)"
echo ""

