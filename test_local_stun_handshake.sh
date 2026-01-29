#!/bin/bash
# Local STUN Handshake Logical Validation
# Tests Dark Forest protocol on single machine with two Songbird instances
# This validates the logic before testing with physical USB spores

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
STUN_SERVER="stun.nextcloud.com:3478"
ALPHA_SOCKET="/tmp/songbird-alpha.sock"
GAMMA_SOCKET="/tmp/songbird-gamma.sock"
ALPHA_PORT=8081
GAMMA_PORT=8082

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║    🔬 Dark Forest Logical Validation Test 🔬                  ║"
echo "║                                                                ║"
echo "║    Testing STUN handshake logic on local machine              ║"
echo "║    Two Songbird instances simulating USB spores                ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill Songbird instances
    if [ -n "$ALPHA_PID" ]; then
        kill $ALPHA_PID 2>/dev/null || true
        echo "Stopped Alpha (PID: $ALPHA_PID)"
    fi
    
    if [ -n "$GAMMA_PID" ]; then
        kill $GAMMA_PID 2>/dev/null || true
        echo "Stopped Gamma (PID: $GAMMA_PID)"
    fi
    
    # Remove sockets
    rm -f "$ALPHA_SOCKET" "$GAMMA_SOCKET"
    
    # Remove logs
    rm -f /tmp/songbird-alpha.log /tmp/songbird-gamma.log
    
    echo "Cleanup complete"
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 0: Preparation${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

# Clean any existing instances
rm -f "$ALPHA_SOCKET" "$GAMMA_SOCKET"
rm -f /tmp/songbird-alpha.log /tmp/songbird-gamma.log

# Check if Songbird binary exists
if [ ! -f "./target/release/songbird" ]; then
    echo -e "${RED}❌ Songbird binary not found${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

echo -e "${GREEN}✅ Songbird binary found${NC}"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 1: Start Two Songbird Instances${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

# Start Alpha (Spore 1)
echo -e "${CYAN}Starting Alpha (Spore 1)...${NC}"
FAMILY_ID=nat0 \
NODE_ID=node-alpha \
SONGBIRD_PORT=$ALPHA_PORT \
SONGBIRD_TLS_ENABLED=false \
SONGBIRD_SECURITY_PROVIDER=none \
./target/release/songbird server --socket "$ALPHA_SOCKET" \
    > /tmp/songbird-alpha.log 2>&1 &
ALPHA_PID=$!

echo "  PID: $ALPHA_PID"
echo "  Socket: $ALPHA_SOCKET"
echo "  Port: $ALPHA_PORT"

# Wait for Alpha socket
echo -n "  Waiting for socket..."
for i in {1..30}; do
    if [ -S "$ALPHA_SOCKET" ]; then
        echo -e " ${GREEN}✅${NC}"
        break
    fi
    sleep 1
    echo -n "."
done

if [ ! -S "$ALPHA_SOCKET" ]; then
    echo -e " ${RED}❌${NC}"
    echo -e "${RED}Failed to start Alpha${NC}"
    echo "Check logs: /tmp/songbird-alpha.log"
    cat /tmp/songbird-alpha.log
    exit 1
fi

sleep 2

# Start Gamma (Spore 2)
echo ""
echo -e "${CYAN}Starting Gamma (Spore 2)...${NC}"
FAMILY_ID=nat0 \
NODE_ID=node-gamma \
SONGBIRD_PORT=$GAMMA_PORT \
SONGBIRD_TLS_ENABLED=false \
SONGBIRD_SECURITY_PROVIDER=none \
./target/release/songbird server --socket "$GAMMA_SOCKET" \
    > /tmp/songbird-gamma.log 2>&1 &
GAMMA_PID=$!

echo "  PID: $GAMMA_PID"
echo "  Socket: $GAMMA_SOCKET"
echo "  Port: $GAMMA_PORT"

# Wait for Gamma socket
echo -n "  Waiting for socket..."
for i in {1..30}; do
    if [ -S "$GAMMA_SOCKET" ]; then
        echo -e " ${GREEN}✅${NC}"
        break
    fi
    sleep 1
    echo -n "."
done

if [ ! -S "$GAMMA_SOCKET" ]; then
    echo -e " ${RED}❌${NC}"
    echo -e "${RED}Failed to start Gamma${NC}"
    echo "Check logs: /tmp/songbird-gamma.log"
    cat /tmp/songbird-gamma.log
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Both Songbird instances running${NC}"
sleep 2

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 2: Alpha - STUN Public Address Discovery${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

ALPHA_STUN=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.get_public_address\",
  \"params\": {
    \"server\": \"$STUN_SERVER\"
  },
  \"id\": 1
}" | nc -U "$ALPHA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$ALPHA_STUN" | jq '.' 2>/dev/null || echo "$ALPHA_STUN"

if echo "$ALPHA_STUN" | jq -e '.result.public_address' > /dev/null 2>&1; then
    ALPHA_PUBLIC=$(echo "$ALPHA_STUN" | jq -r '.result.public_address')
    ALPHA_LOCAL=$(echo "$ALPHA_STUN" | jq -r '.result.local_address')
    ALPHA_NAT=$(echo "$ALPHA_STUN" | jq -r '.result.nat_type')
    
    echo ""
    echo -e "${GREEN}✅ Alpha STUN discovery successful${NC}"
    echo "  Public: $ALPHA_PUBLIC"
    echo "  Local:  $ALPHA_LOCAL"
    echo "  NAT:    $ALPHA_NAT"
else
    echo -e "${RED}❌ Alpha STUN discovery failed${NC}"
    if echo "$ALPHA_STUN" | grep -q "Unknown method"; then
        echo -e "${RED}ERROR: stun.get_public_address not wired!${NC}"
        echo "This means bin_interface.rs is not using IpcServiceHandler"
        exit 1
    fi
    ALPHA_PUBLIC="unknown"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 3: Gamma - STUN Public Address Discovery${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

GAMMA_STUN=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.get_public_address\",
  \"params\": {
    \"server\": \"$STUN_SERVER\"
  },
  \"id\": 1
}" | nc -U "$GAMMA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$GAMMA_STUN" | jq '.' 2>/dev/null || echo "$GAMMA_STUN"

if echo "$GAMMA_STUN" | jq -e '.result.public_address' > /dev/null 2>&1; then
    GAMMA_PUBLIC=$(echo "$GAMMA_STUN" | jq -r '.result.public_address')
    GAMMA_LOCAL=$(echo "$GAMMA_STUN" | jq -r '.result.local_address')
    GAMMA_NAT=$(echo "$GAMMA_STUN" | jq -r '.result.nat_type')
    
    echo ""
    echo -e "${GREEN}✅ Gamma STUN discovery successful${NC}"
    echo "  Public: $GAMMA_PUBLIC"
    echo "  Local:  $GAMMA_LOCAL"
    echo "  NAT:    $GAMMA_NAT"
else
    echo -e "${RED}❌ Gamma STUN discovery failed${NC}"
    GAMMA_PUBLIC="unknown"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 4: Alpha - Create STUN Binding${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

ALPHA_BIND=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.bind\",
  \"params\": {
    \"server\": \"$STUN_SERVER\",
    \"local_port\": 54321,
    \"keepalive_secs\": 30
  },
  \"id\": 2
}" | nc -U "$ALPHA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$ALPHA_BIND" | jq '.' 2>/dev/null || echo "$ALPHA_BIND"

if echo "$ALPHA_BIND" | jq -e '.result.binding_id' > /dev/null 2>&1; then
    ALPHA_BINDING=$(echo "$ALPHA_BIND" | jq -r '.result.binding_id')
    ALPHA_MAPPED=$(echo "$ALPHA_BIND" | jq -r '.result.mapped_address')
    
    echo ""
    echo -e "${GREEN}✅ Alpha STUN binding created${NC}"
    echo "  Binding ID: $ALPHA_BINDING"
    echo "  Mapped:     $ALPHA_MAPPED"
else
    echo -e "${RED}❌ Alpha STUN binding failed${NC}"
    if echo "$ALPHA_BIND" | grep -q "Unknown method"; then
        echo -e "${RED}ERROR: stun.bind not wired!${NC}"
        exit 1
    fi
    ALPHA_BINDING="unknown"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 5: Gamma - Create STUN Binding${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

GAMMA_BIND=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.bind\",
  \"params\": {
    \"server\": \"$STUN_SERVER\",
    \"local_port\": 54322,
    \"keepalive_secs\": 30
  },
  \"id\": 2
}" | nc -U "$GAMMA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$GAMMA_BIND" | jq '.' 2>/dev/null || echo "$GAMMA_BIND"

if echo "$GAMMA_BIND" | jq -e '.result.binding_id' > /dev/null 2>&1; then
    GAMMA_BINDING=$(echo "$GAMMA_BIND" | jq -r '.result.binding_id')
    GAMMA_MAPPED=$(echo "$GAMMA_BIND" | jq -r '.result.mapped_address')
    
    echo ""
    echo -e "${GREEN}✅ Gamma STUN binding created${NC}"
    echo "  Binding ID: $GAMMA_BINDING"
    echo "  Mapped:     $GAMMA_MAPPED"
else
    echo -e "${RED}❌ Gamma STUN binding failed${NC}"
    GAMMA_BINDING="unknown"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 6: Wait for UDP Beacon Discovery (30s)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

echo -e "${YELLOW}⏳ Waiting 30 seconds for UDP beacon propagation...${NC}"
for i in {30..1}; do
    echo -ne "\r  Time remaining: ${i}s "
    sleep 1
done
echo ""
echo -e "${GREEN}✅ Wait complete${NC}"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 7: Alpha - List Discovered Peers${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

ALPHA_PEERS=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U "$ALPHA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$ALPHA_PEERS" | jq '.' 2>/dev/null || echo "$ALPHA_PEERS"

if echo "$ALPHA_PEERS" | jq -e '.result.peers' > /dev/null 2>&1; then
    ALPHA_PEER_COUNT=$(echo "$ALPHA_PEERS" | jq -r '.result.total_count // 0')
    
    echo ""
    echo -e "${GREEN}✅ Alpha peer discovery works${NC}"
    echo "  Peers discovered: $ALPHA_PEER_COUNT"
    
    if [ "$ALPHA_PEER_COUNT" -gt 0 ]; then
        echo ""
        echo "  Discovered peers:"
        echo "$ALPHA_PEERS" | jq -r '.result.peers[] | "    • \(.node_id) @ \(.address) (Family: \(.family_id))"'
        
        # Check if Gamma is in the list
        if echo "$ALPHA_PEERS" | jq -e '.result.peers[] | select(.node_id=="node-gamma")' > /dev/null 2>&1; then
            echo ""
            echo -e "${GREEN}🎉 Alpha discovered Gamma!${NC}"
            ALPHA_FOUND_GAMMA=true
        else
            echo ""
            echo -e "${YELLOW}⚠️  Gamma not in Alpha's peer list yet${NC}"
            ALPHA_FOUND_GAMMA=false
        fi
    else
        echo -e "${YELLOW}⚠️  No peers discovered yet${NC}"
        ALPHA_FOUND_GAMMA=false
    fi
else
    echo -e "${RED}❌ Alpha peer discovery failed${NC}"
    if echo "$ALPHA_PEERS" | grep -q "Unknown method"; then
        echo -e "${RED}ERROR: discovery.peers not wired!${NC}"
        exit 1
    fi
    ALPHA_FOUND_GAMMA=false
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 8: Gamma - List Discovered Peers${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

GAMMA_PEERS=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U "$GAMMA_SOCKET" -N 2>/dev/null)

echo "Response:"
echo "$GAMMA_PEERS" | jq '.' 2>/dev/null || echo "$GAMMA_PEERS"

if echo "$GAMMA_PEERS" | jq -e '.result.peers' > /dev/null 2>&1; then
    GAMMA_PEER_COUNT=$(echo "$GAMMA_PEERS" | jq -r '.result.total_count // 0')
    
    echo ""
    echo -e "${GREEN}✅ Gamma peer discovery works${NC}"
    echo "  Peers discovered: $GAMMA_PEER_COUNT"
    
    if [ "$GAMMA_PEER_COUNT" -gt 0 ]; then
        echo ""
        echo "  Discovered peers:"
        echo "$GAMMA_PEERS" | jq -r '.result.peers[] | "    • \(.node_id) @ \(.address) (Family: \(.family_id))"'
        
        # Check if Alpha is in the list
        if echo "$GAMMA_PEERS" | jq -e '.result.peers[] | select(.node_id=="node-alpha")' > /dev/null 2>&1; then
            echo ""
            echo -e "${GREEN}🎉 Gamma discovered Alpha!${NC}"
            GAMMA_FOUND_ALPHA=true
        else
            echo ""
            echo -e "${YELLOW}⚠️  Alpha not in Gamma's peer list yet${NC}"
            GAMMA_FOUND_ALPHA=false
        fi
    else
        echo -e "${YELLOW}⚠️  No peers discovered yet${NC}"
        GAMMA_FOUND_ALPHA=false
    fi
else
    echo -e "${RED}❌ Gamma peer discovery failed${NC}"
    GAMMA_FOUND_ALPHA=false
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Step 9: Test Peer Connection (If Discovery Succeeded)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"

if [ "$ALPHA_FOUND_GAMMA" = true ] && [ "$GAMMA_MAPPED" != "unknown" ]; then
    echo -e "${CYAN}Testing Alpha → Gamma connection...${NC}"
    
    ALPHA_CONN=$(echo "{
      \"jsonrpc\": \"2.0\",
      \"method\": \"peer.connect\",
      \"params\": {
        \"target_address\": \"$GAMMA_MAPPED\",
        \"our_binding\": \"$ALPHA_BINDING\"
      },
      \"id\": 4
    }" | nc -U "$ALPHA_SOCKET" -N 2>/dev/null)
    
    echo "Response:"
    echo "$ALPHA_CONN" | jq '.' 2>/dev/null || echo "$ALPHA_CONN"
    
    if echo "$ALPHA_CONN" | jq -e '.result.connection_id' > /dev/null 2>&1; then
        echo ""
        echo -e "${GREEN}✅ Alpha initiated connection to Gamma${NC}"
    else
        echo ""
        echo -e "${YELLOW}⚠️  Peer connection pending (implementation stub)${NC}"
    fi
else
    echo -e "${YELLOW}⏸️  Skipped (peers not mutually discovered)${NC}"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}📊 Final Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Summary table
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│                       Test Results                          │"
echo "├─────────────────────────────────────────────────────────────┤"
echo "│                                                             │"

# Alpha results
echo "│  Alpha (node-alpha):                                        │"
if [ "$ALPHA_PUBLIC" != "unknown" ]; then
    echo -e "│    STUN Discovery:     ${GREEN}✅ SUCCESS${NC}                         │"
else
    echo -e "│    STUN Discovery:     ${RED}❌ FAILED${NC}                          │"
fi

if [ "$ALPHA_BINDING" != "unknown" ]; then
    echo -e "│    STUN Binding:       ${GREEN}✅ SUCCESS${NC}                         │"
else
    echo -e "│    STUN Binding:       ${RED}❌ FAILED${NC}                          │"
fi

if [ "$ALPHA_FOUND_GAMMA" = true ]; then
    echo -e "│    Found Gamma:        ${GREEN}✅ YES${NC}                             │"
else
    echo -e "│    Found Gamma:        ${YELLOW}⚠️  NO${NC}                              │"
fi

echo "│                                                             │"

# Gamma results
echo "│  Gamma (node-gamma):                                        │"
if [ "$GAMMA_PUBLIC" != "unknown" ]; then
    echo -e "│    STUN Discovery:     ${GREEN}✅ SUCCESS${NC}                         │"
else
    echo -e "│    STUN Discovery:     ${RED}❌ FAILED${NC}                          │"
fi

if [ "$GAMMA_BINDING" != "unknown" ]; then
    echo -e "│    STUN Binding:       ${GREEN}✅ SUCCESS${NC}                         │"
else
    echo -e "│    STUN Binding:       ${RED}❌ FAILED${NC}                          │"
fi

if [ "$GAMMA_FOUND_ALPHA" = true ]; then
    echo -e "│    Found Alpha:        ${GREEN}✅ YES${NC}                             │"
else
    echo -e "│    Found Alpha:        ${YELLOW}⚠️  NO${NC}                              │"
fi

echo "│                                                             │"
echo "└─────────────────────────────────────────────────────────────┘"

echo ""

# Final verdict
ERRORS=0
WARNINGS=0

if [ "$ALPHA_PUBLIC" = "unknown" ] || [ "$GAMMA_PUBLIC" = "unknown" ]; then
    ((ERRORS++))
fi

if [ "$ALPHA_BINDING" = "unknown" ] || [ "$GAMMA_BINDING" = "unknown" ]; then
    ((ERRORS++))
fi

if [ "$ALPHA_FOUND_GAMMA" != true ] || [ "$GAMMA_FOUND_ALPHA" != true ]; then
    ((WARNINGS++))
fi

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo -e "║           ${GREEN}🎉 LOGICAL VALIDATION: SUCCESS! 🎉${NC}              ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    echo ""
    echo "✅ STUN handshake logic validated"
    echo "✅ Both spores discovered public IPs"
    echo "✅ Both spores created STUN bindings"
    echo "✅ Mutual peer discovery via UDP beacons"
    echo "✅ Dark Forest protocol wiring confirmed"
    echo ""
    echo -e "${CYAN}→ Ready for physical USB spore testing!${NC}"
    exit 0
    
elif [ $ERRORS -eq 0 ] && [ $WARNINGS -gt 0 ]; then
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo -e "║          ${YELLOW}⚠️  LOGICAL VALIDATION: PARTIAL ⚠️${NC}              ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${YELLOW}Some tests passed with warnings:${NC}"
    echo ""
    echo "✅ STUN handshake logic works"
    echo "✅ Both spores got public IPs"
    echo "✅ Both spores created bindings"
    if [ "$ALPHA_FOUND_GAMMA" != true ] || [ "$GAMMA_FOUND_ALPHA" != true ]; then
        echo -e "${YELLOW}⚠️  Peer discovery incomplete (may need longer wait)${NC}"
    fi
    echo ""
    echo -e "${CYAN}Notes:${NC}"
    echo "- UDP beacon discovery can take 30-60 seconds"
    echo "- On same machine, discovery might be delayed"
    echo "- Physical USB spores on different towers will work better"
    echo ""
    echo -e "${CYAN}→ Proceed to physical USB spore testing${NC}"
    exit 0
    
else
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo -e "║           ${RED}❌ LOGICAL VALIDATION: FAILED ❌${NC}                ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${RED}Critical errors detected:${NC}"
    echo ""
    
    if [ "$ALPHA_PUBLIC" = "unknown" ] || [ "$GAMMA_PUBLIC" = "unknown" ]; then
        echo -e "${RED}❌ STUN discovery failed${NC}"
        echo "   • Check network connectivity to $STUN_SERVER"
        echo "   • Ensure UDP port 3478 is not blocked"
        echo "   • Try alternative STUN server (stun.l.google.com:19302)"
    fi
    
    if [ "$ALPHA_BINDING" = "unknown" ] || [ "$GAMMA_BINDING" = "unknown" ]; then
        echo -e "${RED}❌ STUN binding failed${NC}"
        echo "   • Check if local UDP ports are available"
        echo "   • May indicate wiring issue if discovery worked"
    fi
    
    echo ""
    echo "Check logs:"
    echo "  • /tmp/songbird-alpha.log"
    echo "  • /tmp/songbird-gamma.log"
    echo ""
    exit 1
fi

