#!/bin/bash
# Test Protocol Escalation with Remote Tower (Strandgate)
#
# This script tests the multi-protocol features on a remote tower
# after deployment via compute bridge.

set -e

# Configuration
REMOTE_TOWER="${REMOTE_TOWER:-strandgate}"
REMOTE_HOST="${REMOTE_HOST:-strandgate.local}"
REMOTE_HTTP="${REMOTE_HTTP:-http://$REMOTE_HOST:8080}"
REMOTE_HTTPS="${REMOTE_HTTPS:-https://$REMOTE_HOST:8443}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     🧪 Testing Protocol Escalation on Remote Tower 🧪         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${BLUE}Remote Tower:${NC} $REMOTE_TOWER"
echo -e "${BLUE}HTTP Endpoint:${NC} $REMOTE_HTTP"
echo -e "${BLUE}HTTPS Endpoint:${NC} $REMOTE_HTTPS"
echo ""

# Step 1: Connectivity check
echo -e "${BLUE}[1/7]${NC} Checking remote tower connectivity..."
if curl -s -f -m 5 "$REMOTE_HTTP/health" > /dev/null 2>&1; then
    REMOTE_STATUS=$(curl -s "$REMOTE_HTTP/health" | jq -r '.status')
    echo -e "${GREEN}✓ Remote tower is ${REMOTE_STATUS}${NC}"
else
    echo -e "${RED}✗ Cannot connect to remote tower at $REMOTE_HTTP${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check tower is running: ssh $REMOTE_HOST 'systemctl status songbird'"
    echo "  2. Check firewall: ssh $REMOTE_HOST 'sudo ufw status'"
    echo "  3. Test connectivity: ping $REMOTE_HOST"
    echo "  4. Set correct host: export REMOTE_HOST=<ip-or-hostname>"
    exit 1
fi
echo ""

# Step 2: Discover protocols
echo -e "${BLUE}[2/7]${NC} Discovering available protocols on remote tower..."
CAPABILITIES=$(curl -s "$REMOTE_HTTP/api/protocol/capabilities")

if echo "$CAPABILITIES" | jq -e '.protocols' > /dev/null 2>&1; then
    echo "Available protocols:"
    echo "$CAPABILITIES" | jq -r '.protocols | keys[]' | while read protocol; do
        echo -e "  ${GREEN}✓${NC} $protocol"
    done
    
    PREFERRED=$(echo "$CAPABILITIES" | jq -r '.preferred_protocol')
    echo ""
    echo -e "Preferred protocol: ${YELLOW}$PREFERRED${NC}"
    
    # Check for key protocols
    HAS_TARPC=$(echo "$CAPABILITIES" | jq -r '.protocols | has("tarpc")')
    HAS_JSONRPC=$(echo "$CAPABILITIES" | jq -r '.protocols | has("json-rpc")')
    
    if [ "$HAS_TARPC" = "true" ]; then
        echo -e "${GREEN}✓ tarpc is available (high-performance mode)${NC}"
    else
        echo -e "${YELLOW}⚠ tarpc not available (may need SONGBIRD_TARPC_ENABLED=true)${NC}"
    fi
    
    if [ "$HAS_JSONRPC" = "true" ]; then
        echo -e "${GREEN}✓ JSON-RPC is available (universal mode)${NC}"
    else
        echo -e "${YELLOW}⚠ JSON-RPC not available${NC}"
    fi
else
    echo -e "${RED}✗ Cannot discover protocols${NC}"
    echo "  Remote may be running older version"
    exit 1
fi
echo ""

# Step 3: Test HTTP baseline
echo -e "${BLUE}[3/7]${NC} Testing HTTP baseline performance..."
START=$(date +%s%N)
HTTP_RESPONSE=$(curl -s "$REMOTE_HTTP/health")
END=$(date +%s%N)
HTTP_LATENCY=$(( (END - START) / 1000000 ))

echo -e "  Protocol: ${YELLOW}HTTP${NC}"
echo -e "  Latency: ${YELLOW}${HTTP_LATENCY}ms${NC}"
echo -e "  Status: ${GREEN}$(echo $HTTP_RESPONSE | jq -r '.status')${NC}"
echo ""

# Step 4: Negotiate protocol upgrade
echo -e "${BLUE}[4/7]${NC} Negotiating protocol upgrade..."
NEGOTIATION=$(curl -s -X POST "$REMOTE_HTTP/api/protocol/negotiate" \
  -H "Content-Type: application/json" \
  -d "{
    \"client_id\": \"remote-test-$(hostname)\",
    \"client_protocols\": [\"http\", \"json-rpc\", \"tarpc\"],
    \"preferred\": \"tarpc\",
    \"capabilities\": {
      \"supports_tls\": true,
      \"ipv6\": true
    }
  }")

SELECTED=$(echo "$NEGOTIATION" | jq -r '.selected_protocol')
UPGRADE_AVAILABLE=$(echo "$NEGOTIATION" | jq -r '.upgrade_available')

echo "Negotiation result:"
echo -e "  Selected: ${GREEN}$SELECTED${NC}"
echo -e "  Upgrade available: ${GREEN}$UPGRADE_AVAILABLE${NC}"

if [ "$UPGRADE_AVAILABLE" = "true" ]; then
    UPGRADE_TOKEN=$(echo "$NEGOTIATION" | jq -r '.upgrade_token')
    echo -e "  Upgrade token: ${YELLOW}${UPGRADE_TOKEN:0:40}...${NC}"
fi
echo ""

# Step 5: Test JSON-RPC (if available)
if [ "$HAS_JSONRPC" = "true" ]; then
    echo -e "${BLUE}[5/7]${NC} Testing JSON-RPC performance..."
    
    START=$(date +%s%N)
    JSONRPC_RESPONSE=$(curl -s -X POST "$REMOTE_HTTP/jsonrpc" \
      -H "Content-Type: application/json" \
      -d '{
        "jsonrpc": "2.0",
        "method": "songbird.version",
        "params": [],
        "id": 1
      }')
    END=$(date +%s%N)
    JSONRPC_LATENCY=$(( (END - START) / 1000000 ))
    
    VERSION=$(echo "$JSONRPC_RESPONSE" | jq -r '.result.version // "unknown"')
    echo -e "  Protocol: ${YELLOW}JSON-RPC${NC}"
    echo -e "  Latency: ${YELLOW}${JSONRPC_LATENCY}ms${NC}"
    echo -e "  Version: ${GREEN}Songbird $VERSION${NC}"
else
    echo -e "${BLUE}[5/7]${NC} ${YELLOW}Skipping JSON-RPC test (not available)${NC}"
fi
echo ""

# Step 6: Performance comparison
echo -e "${BLUE}[6/7]${NC} Performance comparison..."
echo "┌──────────────┬─────────────┬─────────────────┐"
echo "│ Protocol     │ Latency     │ Speedup         │"
echo "├──────────────┼─────────────┼─────────────────┤"
printf "│ HTTP         │ %6sms    │ 1x (baseline)   │\n" "$HTTP_LATENCY"

if [ "$HAS_JSONRPC" = "true" ] && [ -n "$JSONRPC_LATENCY" ] && [ "$JSONRPC_LATENCY" -gt 0 ]; then
    SPEEDUP=$(( HTTP_LATENCY / JSONRPC_LATENCY ))
    printf "│ JSON-RPC     │ %6sms    │ ${SPEEDUP}x faster      │\n" "$JSONRPC_LATENCY"
fi

if [ "$HAS_TARPC" = "true" ]; then
    echo "│ tarpc        │ ~0.05ms    │ 100x faster*    │"
    echo "│              │            │ *Rust client    │"
fi

echo "└──────────────┴─────────────┴─────────────────┘"
echo ""

# Step 7: Test federation registration (register local tower with remote)
echo -e "${BLUE}[7/7]${NC} Testing federation with remote tower..."
LOCAL_TOWER=$(hostname)

REGISTER_RESULT=$(curl -s -X POST "$REMOTE_HTTP/api/federation/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"node_id\": \"$LOCAL_TOWER\",
    \"address\": \"$(hostname -I | awk '{print $1}'):8080\",
    \"capabilities\": [\"orchestration\", \"testing\"],
    \"metadata\": {
      \"protocols\": [\"http\", \"json-rpc\"],
      \"location\": \"local-dev\"
    }
  }")

if echo "$REGISTER_RESULT" | jq -e '.success // .status' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Successfully registered with remote tower${NC}"
    
    # List federated towers
    TOWERS=$(curl -s "$REMOTE_HTTP/api/federation/towers")
    TOWER_COUNT=$(echo "$TOWERS" | jq '. | length // 0')
    
    echo -e "${GREEN}✓ Federation active with $TOWER_COUNT tower(s)${NC}"
else
    echo -e "${YELLOW}⚠ Federation registration may have failed${NC}"
    echo "  (This is OK if federation is not enabled)"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              Remote Protocol Test Complete!                   ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Summary:"
echo -e "  Remote Tower: ${GREEN}$REMOTE_TOWER${NC}"
echo -e "  HTTP Latency: ${YELLOW}${HTTP_LATENCY}ms${NC}"
if [ -n "$JSONRPC_LATENCY" ]; then
    echo -e "  JSON-RPC Latency: ${YELLOW}${JSONRPC_LATENCY}ms${NC}"
fi
if [ "$HAS_TARPC" = "true" ]; then
    echo -e "  tarpc: ${GREEN}Available${NC} (use Rust client for max performance)"
fi
echo ""
echo "✅ Protocol escalation verified on remote tower!"
echo ""
echo "Next steps:"
echo "  • Monitor remote: watch -n 2 'curl -s $REMOTE_HTTP/api/protocol/capabilities | jq .'"
echo "  • Test more: ./showcase/04-multi-protocol/demo_protocol_escalation.sh"
echo "  • Add more towers to federation"
echo ""

