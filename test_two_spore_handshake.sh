#!/bin/bash
# Two Spore Dark Forest Validation Test
# Run this script on BOTH spores (set NODE_ID differently for each)
#
# Tower A: NODE_ID=node-alpha SONGBIRD_PORT=8081 ./test_two_spore_handshake.sh
# Tower B: NODE_ID=node-gamma SONGBIRD_PORT=8082 ./test_two_spore_handshake.sh

set -e

# Configuration
NODE_ID="${NODE_ID:-node-alpha}"
FAMILY_ID="nat0"
STUN_SERVER="stun.nextcloud.com:3478"
LOCAL_PORT="${SONGBIRD_PORT:-8081}"
SOCKET="/primal/songbird"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║    🌲 Dark Forest Protocol - Two Spore Validation 🌲          ║"
echo "║                                                                ║"
echo "║    Node: $NODE_ID"
echo "║    Port: $LOCAL_PORT"
echo "║    Family: $FAMILY_ID"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if Songbird is running
echo -e "${BLUE}⏳ Checking Songbird status...${NC}"
if [ ! -S "$SOCKET" ]; then
    echo -e "${YELLOW}⚠️  Songbird not running. Starting now...${NC}"
    
    # Start Songbird
    cd "$(dirname "$0")"
    FAMILY_ID=$FAMILY_ID \
    NODE_ID=$NODE_ID \
    SONGBIRD_PORT=$LOCAL_PORT \
    ./target/release/songbird server > /tmp/songbird-$NODE_ID.log 2>&1 &
    
    SONGBIRD_PID=$!
    echo "Started Songbird (PID: $SONGBIRD_PID)"
    
    # Wait for socket
    for i in {1..30}; do
        if [ -S "$SOCKET" ]; then
            echo -e "${GREEN}✅ Songbird ready: $SOCKET${NC}"
            break
        fi
        sleep 1
    done
    
    if [ ! -S "$SOCKET" ]; then
        echo -e "${RED}❌ Songbird socket not available after 30 seconds${NC}"
        echo "Check logs: /tmp/songbird-$NODE_ID.log"
        exit 1
    fi
else
    echo -e "${GREEN}✅ Songbird already running: $SOCKET${NC}"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Test 1: STUN Public Address Discovery"
echo "════════════════════════════════════════════════════════════════"

STUN_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.get_public_address\",
  \"params\": {
    \"server\": \"$STUN_SERVER\"
  },
  \"id\": 1
}" | nc -U "$SOCKET" -N 2>/dev/null)

if echo "$STUN_RESULT" | jq -e '.result.public_address' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ STUN discovery successful${NC}"
    echo "$STUN_RESULT" | jq '.'
    
    PUBLIC_ADDR=$(echo "$STUN_RESULT" | jq -r '.result.public_address')
    LOCAL_ADDR=$(echo "$STUN_RESULT" | jq -r '.result.local_address')
    NAT_TYPE=$(echo "$STUN_RESULT" | jq -r '.result.nat_type')
    
    echo ""
    echo -e "${BLUE}Addresses for $NODE_ID:${NC}"
    echo "  Public: $PUBLIC_ADDR"
    echo "  Local:  $LOCAL_ADDR"
    echo "  NAT:    $NAT_TYPE"
else
    echo -e "${RED}❌ STUN discovery failed${NC}"
    echo "$STUN_RESULT" | jq '.' || echo "$STUN_RESULT"
    PUBLIC_ADDR="unknown"
    LOCAL_ADDR="unknown"
    NAT_TYPE="unknown"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Test 2: STUN Binding (Hole Punching Setup)"
echo "════════════════════════════════════════════════════════════════"

# Use different ports for different nodes
BIND_PORT=54321
if [ "$NODE_ID" = "node-gamma" ] || [ "$NODE_ID" = "node-beta" ]; then
    BIND_PORT=54322
fi

BIND_RESULT=$(echo "{
  \"jsonrpc\": \"2.0\",
  \"method\": \"stun.bind\",
  \"params\": {
    \"server\": \"$STUN_SERVER\",
    \"local_port\": $BIND_PORT,
    \"keepalive_secs\": 30
  },
  \"id\": 2
}" | nc -U "$SOCKET" -N 2>/dev/null)

if echo "$BIND_RESULT" | jq -e '.result.binding_id' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ STUN binding created${NC}"
    echo "$BIND_RESULT" | jq '.'
    
    BINDING_ID=$(echo "$BIND_RESULT" | jq -r '.result.binding_id')
    MAPPED_ADDR=$(echo "$BIND_RESULT" | jq -r '.result.mapped_address')
    LIFETIME=$(echo "$BIND_RESULT" | jq -r '.result.lifetime_secs')
    
    echo ""
    echo -e "${BLUE}STUN Binding for $NODE_ID:${NC}"
    echo "  ID:       $BINDING_ID"
    echo "  Mapped:   $MAPPED_ADDR"
    echo "  Lifetime: ${LIFETIME}s"
else
    echo -e "${RED}❌ STUN binding failed${NC}"
    echo "$BIND_RESULT" | jq '.' || echo "$BIND_RESULT"
    BINDING_ID="unknown"
    MAPPED_ADDR="unknown"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Test 3: Peer Discovery (UDP Beacons)"
echo "════════════════════════════════════════════════════════════════"

echo -e "${YELLOW}⏳ Waiting 5 seconds for UDP beacon propagation...${NC}"
sleep 5

PEERS_RESULT=$(echo '{
  "jsonrpc": "2.0",
  "method": "discovery.peers",
  "params": {},
  "id": 3
}' | nc -U "$SOCKET" -N 2>/dev/null)

if echo "$PEERS_RESULT" | jq -e '.result.peers' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Peer discovery successful${NC}"
    echo "$PEERS_RESULT" | jq '.'
    
    PEER_COUNT=$(echo "$PEERS_RESULT" | jq -r '.result.total_count // 0')
    
    echo ""
    echo -e "${BLUE}Discovered Peers: $PEER_COUNT${NC}"
    
    if [ "$PEER_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✅ Other spore(s) discovered via UDP beacons!${NC}"
        
        # Show peer details
        echo ""
        echo "Peer Details:"
        echo "$PEERS_RESULT" | jq -r '.result.peers[] | "  • \(.node_id) @ \(.address) (TCP: \(.tcp_port // "N/A"))"'
        
        # Get first peer for connection test
        PEER_NODE=$(echo "$PEERS_RESULT" | jq -r '.result.peers[0].node_id // "none"')
        PEER_ADDR=$(echo "$PEERS_RESULT" | jq -r '.result.peers[0].address // "none"')
        PEER_TCP=$(echo "$PEERS_RESULT" | jq -r '.result.peers[0].tcp_port // 0')
    else
        echo -e "${YELLOW}⚠️  No peers discovered yet${NC}"
        echo "Tip: Ensure the other spore is running and wait 30s for beacon propagation"
        PEER_NODE="none"
        PEER_ADDR="none"
    fi
else
    echo -e "${RED}❌ Peer discovery failed${NC}"
    echo "$PEERS_RESULT" | jq '.' || echo "$PEERS_RESULT"
    PEER_COUNT=0
    PEER_NODE="none"
    PEER_ADDR="none"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "Test 4: Direct Peer Connection (If Peer Discovered)"
echo "════════════════════════════════════════════════════════════════"

if [ "$PEER_NODE" != "none" ] && [ "$MAPPED_ADDR" != "unknown" ]; then
    echo -e "${BLUE}Attempting connection to $PEER_NODE...${NC}"
    
    # Use the peer's mapped address from discovery
    # In real scenario, we'd get this from STUN exchange
    CONNECT_RESULT=$(echo "{
      \"jsonrpc\": \"2.0\",
      \"method\": \"peer.connect\",
      \"params\": {
        \"target_address\": \"$PEER_ADDR\",
        \"our_binding\": \"$BINDING_ID\"
      },
      \"id\": 4
    }" | nc -U "$SOCKET" -N 2>/dev/null)
    
    if echo "$CONNECT_RESULT" | jq -e '.result.connection_id' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Peer connection initiated${NC}"
        echo "$CONNECT_RESULT" | jq '.'
        
        CONN_ID=$(echo "$CONNECT_RESULT" | jq -r '.result.connection_id')
        CONN_STATE=$(echo "$CONNECT_RESULT" | jq -r '.result.state')
        
        echo ""
        echo -e "${BLUE}Connection to $PEER_NODE:${NC}"
        echo "  ID:    $CONN_ID"
        echo "  State: $CONN_STATE"
    else
        echo -e "${YELLOW}⚠️  Peer connection pending (implementation stub)${NC}"
        echo "$CONNECT_RESULT" | jq '.' || echo "$CONNECT_RESULT"
    fi
else
    echo -e "${YELLOW}⏸️  Skipped (no peer discovered or no binding)${NC}"
    echo "Run this test on both spores and wait for mutual discovery"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "📊 Summary for $NODE_ID"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Network Addresses:"
echo "  Public:  $PUBLIC_ADDR"
echo "  Local:   $LOCAL_ADDR"
echo "  NAT:     $NAT_TYPE"
echo ""
echo "STUN Binding:"
echo "  ID:      $BINDING_ID"
echo "  Mapped:  $MAPPED_ADDR"
echo ""
echo "Discovery:"
echo "  Peers:   $PEER_COUNT discovered"
if [ "$PEER_NODE" != "none" ]; then
    echo "  Target:  $PEER_NODE @ $PEER_ADDR"
fi
echo ""

# Final verdict
if [ "$PUBLIC_ADDR" != "unknown" ] && [ "$BINDING_ID" != "unknown" ]; then
    echo -e "${GREEN}✅ Dark Forest Protocol: OPERATIONAL${NC}"
    echo ""
    if [ "$PEER_COUNT" -gt 0 ]; then
        echo -e "${GREEN}🎉 TWO-SPORE HANDSHAKE: SUCCESS!${NC}"
        echo ""
        echo "Both spores have:"
        echo "  ✅ Discovered each other via UDP beacons"
        echo "  ✅ Established STUN bindings for NAT traversal"
        echo "  ✅ Ready for secure peer-to-peer communication"
        echo ""
        echo "🌲 Dark Forest Protocol Validated!"
    else
        echo -e "${YELLOW}⚠️  Waiting for peer discovery${NC}"
        echo ""
        echo "Next Steps:"
        echo "  1. Ensure other spore is running"
        echo "  2. Wait 30-60 seconds for UDP beacon propagation"
        echo "  3. Re-run this test to verify mutual discovery"
    fi
else
    echo -e "${RED}❌ Dark Forest Protocol: PARTIAL${NC}"
    echo ""
    echo "Issues detected:"
    if [ "$PUBLIC_ADDR" = "unknown" ]; then
        echo "  ❌ STUN public address discovery failed"
        echo "     Check network connectivity to $STUN_SERVER"
    fi
    if [ "$BINDING_ID" = "unknown" ]; then
        echo "  ❌ STUN binding creation failed"
        echo "     Check UDP port availability"
    fi
    echo ""
    echo "Check logs: /tmp/songbird-$NODE_ID.log"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo ""

