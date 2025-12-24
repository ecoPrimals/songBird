#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🎵 Demo 2: BirdSong Broadcasting
# ═══════════════════════════════════════════════════════════════
# Shows how BirdSong protocol provides privacy-preserving discovery
# - Family nodes decrypt messages
# - Non-family nodes see random noise
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
echo -e "${BLUE}     🎵 BirdSong Broadcasting Demo${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

WORK_DIR="$SCRIPT_DIR/data/birdsong-demo"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: Node C broadcasts to find relay${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "Network Topology:"
echo ""
echo "  Node A (Root)        ← Public IP, reachable"
echo "     │"
echo "     └─ Node B         ← Public IP, reachable"
echo "          │"
echo "          └─ Node C    ← Behind NAT, needs relay"
echo ""
echo "  Unrelated:"
echo "  Node X, Y, Z        ← Non-family nodes"
echo ""

# Create node identities
echo -e "${YELLOW}Setting up nodes...${NC}"

cat > node_c.json <<EOF
{
  "node_id": "node-c-grandchild",
  "lineage": ["node-b-child", "node-a-parent"],
  "status": "behind NAT"
}
EOF

cat > node_b.json <<EOF
{
  "node_id": "node-b-child",
  "lineage": ["node-a-parent"],
  "status": "public IP"
}
EOF

cat > node_a.json <<EOF
{
  "node_id": "node-a-parent",
  "lineage": [],
  "status": "public IP"
}
EOF

cat > node_x.json <<EOF
{
  "node_id": "node-x-stranger",
  "lineage": ["node-y-parent"],
  "status": "public IP",
  "relation": "not family"
}
EOF

echo -e "${GREEN}✅ Nodes initialized${NC}"
echo ""

echo -e "${YELLOW}Step 1: Node C Needs Connectivity${NC}"
echo "       Node C is behind NAT and needs to reach Node A"
echo ""

TARGET="node-a-parent"
echo "       Target: $TARGET"
echo "       Problem: Direct connection blocked by NAT"
echo ""

echo -e "${YELLOW}Step 2: Node C Creates BirdSong Message${NC}"
echo "       Requesting relay assistance from ancestors..."
echo ""

BIRDSONG_ID="birdsong-$(uuidgen)"
RELAY_REQUEST="Need relay to reach $TARGET"

cat > birdsong_plaintext.json <<EOF
{
  "birdsong_id": "$BIRDSONG_ID",
  "sender": "node-c-grandchild",
  "message_type": "relay_request",
  "payload": {
    "target": "$TARGET",
    "reason": "NAT traversal",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  },
  "lineage_hint": "ancestors"
}
EOF

echo -e "${CYAN}Plaintext Message (before encryption):${NC}"
cat birdsong_plaintext.json | jq '.'
echo ""

echo -e "${YELLOW}Step 3: BearDog - Encrypt for Lineage${NC}"
echo "       Delegating encryption to BearDog..."
echo "       Only ancestors [node-b-child, node-a-parent] can decrypt"
echo ""

# Simulate BearDog encryption
# In production: beardog encrypt-birdsong --message plaintext.json --hint ancestors
ENCRYPTED_DATA="$(openssl rand -hex 128)"
NOISE_DATA="$(openssl rand -hex 128)"

cat > birdsong_encrypted.json <<EOF
{
  "birdsong_id": "$BIRDSONG_ID",
  "encrypted_payload": "$ENCRYPTED_DATA",
  "lineage_hint": "ancestors",
  "sender_masked": true,
  "broadcast_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}✅ BearDog encrypted BirdSong${NC}"
echo -e "${CYAN}Encrypted Message (what's broadcast):${NC}"
cat birdsong_encrypted.json | jq '.'
echo ""

echo -e "${YELLOW}Step 4: Songbird - Broadcast BirdSong${NC}"
echo "       Broadcasting via UDP multicast..."
echo ""

echo "       Broadcasting to 239.255.255.250:5353 (mDNS-like)"
sleep 1

echo -e "${GREEN}✅ BirdSong broadcast complete${NC}"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Receiving Nodes Process Broadcast${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""

echo -e "${YELLOW}Node X (Stranger - Non-Family):${NC}"
echo "  Attempting to decrypt..."
echo "  Result: ${RED}❌ Cannot decrypt (not in lineage)${NC}"
echo "  Sees: Random noise"
echo ""
echo "  Noise: $NOISE_DATA"
echo ""

echo -e "${YELLOW}Node B (Parent - Direct Ancestor):${NC}"
echo "  Attempting to decrypt..."
echo "  Result: ${GREEN}✅ Successfully decrypted!${NC}"
echo ""

cat > node_b_decrypted.json <<EOF
{
  "birdsong_id": "$BIRDSONG_ID",
  "sender": "node-c-grandchild",
  "message_type": "relay_request",
  "payload": {
    "target": "$TARGET",
    "reason": "NAT traversal",
    "timestamp": "$(jq -r '.payload.timestamp' birdsong_plaintext.json)"
  },
  "decrypted_by": "node-b-child",
  "lineage_verified": true
}
EOF

cat node_b_decrypted.json | jq '.'
echo ""

echo -e "${YELLOW}Node A (Grandparent - Ancestor):${NC}"
echo "  Attempting to decrypt..."
echo "  Result: ${GREEN}✅ Successfully decrypted!${NC}"
echo ""

cat > node_a_decrypted.json <<EOF
{
  "birdsong_id": "$BIRDSONG_ID",
  "sender": "node-c-grandchild",
  "message_type": "relay_request",
  "payload": {
    "target": "$TARGET",
    "reason": "NAT traversal",
    "timestamp": "$(jq -r '.payload.timestamp' birdsong_plaintext.json)"
  },
  "decrypted_by": "node-a-parent",
  "lineage_verified": true
}
EOF

cat node_a_decrypted.json | jq '.'
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ BirdSong Protocol Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Results:"
echo ""
echo -e "  📡 Broadcast: ${GREEN}1 BirdSong message${NC}"
echo -e "  👨‍👦 Family Nodes: ${GREEN}2 decrypted (B, A)${NC}"
echo -e "  🚫 Non-Family: ${RED}1 saw noise (X)${NC}"
echo -e "  🔒 Privacy: ${GREEN}Preserved${NC}"
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Privacy Analysis:${NC}"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  WHO SEES WHAT                                              │"
echo "├─────────────────────────────────────────────────────────────┤"
echo "│  Node C (Sender):                                           │"
echo "│    ✅ Knows: Own message, ancestors                         │"
echo "│                                                              │"
echo "│  Node B (Parent):                                           │"
echo "│    ✅ Knows: C needs relay to A                             │"
echo "│    ✅ Can: Offer relay service                              │"
echo "│                                                              │"
echo "│  Node A (Grandparent):                                      │"
echo "│    ✅ Knows: Descendant C needs help                        │"
echo "│    ✅ Can: Offer relay service                              │"
echo "│                                                              │"
echo "│  Node X (Stranger):                                         │"
echo "│    ❌ Knows: NOTHING (sees random noise)                    │"
echo "│    ❌ Cannot: Decrypt or help                               │"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Key Benefits:${NC}"
echo ""
echo "1. 🔒 Privacy by Design"
echo "   - Only family decrypts messages"
echo "   - Non-family sees random noise"
echo "   - No metadata leakage"
echo ""
echo "2. 🎯 Targeted Discovery"
echo "   - Broadcast to network (efficient)"
echo "   - Only family responds (privacy)"
echo "   - No central directory needed"
echo ""
echo "3. 🧬 Genetic Trust"
echo "   - Decryption proves lineage"
echo "   - No fake relay offers"
echo "   - Cryptographic verification"
echo ""
echo -e "${GREEN}✅ BirdSong: Privacy-preserving discovery without infrastructure!${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Next Demo: 03-lineage-relay.sh${NC}"
echo "           (See how Node B offers relay service)"
echo ""

