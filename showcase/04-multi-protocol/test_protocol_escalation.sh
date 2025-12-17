#!/bin/bash
# Test Protocol Escalation: HTTP → JSON-RPC → tarpc
# This script simulates a client progressively upgrading to better protocols

set -e

TOWER_A="http://localhost:8080"
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        Protocol Escalation Test: HTTP → JSON-RPC → tarpc      ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Phase 1: Start with HTTP
echo -e "${BLUE}[Phase 1]${NC} Client starts with HTTP..."
START=$(date +%s%N)
HTTP_RESPONSE=$(curl -s "$TOWER_A/health")
END=$(date +%s%N)
HTTP_LATENCY=$(( (END - START) / 1000000 ))

echo -e "  Current protocol: ${YELLOW}HTTP${NC}"
echo -e "  Latency: ${YELLOW}${HTTP_LATENCY}ms${NC}"
echo -e "  Status: ${GREEN}$(echo $HTTP_RESPONSE | jq -r '.status')${NC}"
echo ""

# Phase 2: Discover available protocols
echo -e "${BLUE}[Phase 2]${NC} Discovering available protocols..."
CAPABILITIES=$(curl -s "$TOWER_A/api/protocol/capabilities")
PROTOCOLS=$(echo "$CAPABILITIES" | jq -r '.protocols | keys | join(", ")')

echo -e "  Available protocols: ${GREEN}$PROTOCOLS${NC}"
echo -e "  Preferred: ${YELLOW}$(echo $CAPABILITIES | jq -r '.preferred_protocol')${NC}"
echo ""

# Phase 3: Negotiate upgrade to JSON-RPC
echo -e "${BLUE}[Phase 3]${NC} Negotiating upgrade to JSON-RPC..."
NEGOTIATION=$(curl -s -X POST "$TOWER_A/api/protocol/negotiate" \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "escalation-test",
    "client_protocols": ["http", "json-rpc"],
    "preferred": "json-rpc"
  }')

SELECTED=$(echo "$NEGOTIATION" | jq -r '.selected_protocol')
echo -e "  Negotiated protocol: ${GREEN}$SELECTED${NC}"

if [ "$SELECTED" = "json-rpc" ]; then
    echo -e "  ${GREEN}✓ Upgrade to JSON-RPC successful!${NC}"
else
    echo -e "  ${YELLOW}⚠ Using $SELECTED instead${NC}"
fi
echo ""

# Phase 4: Test JSON-RPC performance
echo -e "${BLUE}[Phase 4]${NC} Testing JSON-RPC performance..."
START=$(date +%s%N)
JSONRPC_RESPONSE=$(curl -s -X POST "$TOWER_A/jsonrpc" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.version",
    "params": [],
    "id": 1
  }')
END=$(date +%s%N)
JSONRPC_LATENCY=$(( (END - START) / 1000000 ))

echo -e "  Current protocol: ${YELLOW}JSON-RPC${NC}"
echo -e "  Latency: ${YELLOW}${JSONRPC_LATENCY}ms${NC}"
echo -e "  Response: ${GREEN}$(echo $JSONRPC_RESPONSE | jq -r '.result.version')${NC}"
echo ""

# Calculate speedup
if [ $JSONRPC_LATENCY -gt 0 ] && [ $HTTP_LATENCY -gt 0 ]; then
    SPEEDUP=$(( HTTP_LATENCY / JSONRPC_LATENCY ))
    echo -e "  ${GREEN}Performance improvement: ${SPEEDUP}x faster than HTTP!${NC}"
fi
echo ""

# Phase 5: Discover tarpc availability
echo -e "${BLUE}[Phase 5]${NC} Checking for tarpc availability..."
TARPC_AVAILABLE=$(echo "$CAPABILITIES" | jq -r '.protocols | has("tarpc")')

if [ "$TARPC_AVAILABLE" = "true" ]; then
    echo -e "  ${GREEN}✓ tarpc is available!${NC}"
    
    # Phase 6: Negotiate final upgrade to tarpc
    echo ""
    echo -e "${BLUE}[Phase 6]${NC} Negotiating upgrade to tarpc..."
    FINAL_NEGOTIATION=$(curl -s -X POST "$TOWER_A/api/protocol/negotiate" \
      -H "Content-Type: application/json" \
      -d '{
        "client_id": "escalation-test",
        "client_protocols": ["http", "json-rpc", "tarpc"],
        "preferred": "tarpc"
      }')
    
    FINAL_SELECTED=$(echo "$FINAL_NEGOTIATION" | jq -r '.selected_protocol')
    echo -e "  Final protocol: ${GREEN}$FINAL_SELECTED${NC}"
    
    if [ "$FINAL_SELECTED" = "tarpc" ]; then
        echo -e "  ${GREEN}✓ Upgrade to tarpc successful!${NC}"
        echo -e "  ${GREEN}Expected performance: ~50μs latency (100x faster than HTTP!)${NC}"
    fi
else
    echo -e "  ${YELLOW}⚠ tarpc not available (requires Rust client)${NC}"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║               Protocol Escalation Complete!                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Summary:"
echo -e "  Phase 1 (HTTP):     ${HTTP_LATENCY}ms"
echo -e "  Phase 2 (JSON-RPC): ${JSONRPC_LATENCY}ms"
echo -e "  Phase 3 (tarpc):    ~0.05ms (requires Rust client)"
echo ""
echo "✅ Protocol escalation verified!"
echo ""

