#!/bin/bash
# Query mesh status

FAMILY_ID="demo-fractal"
ALBATROSS_SOCKET="/tmp/songbird-${FAMILY_ID}-albatross-main.sock"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                     🔍 FRACTAL MESH STATUS                                    ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

if [ ! -S "$ALBATROSS_SOCKET" ]; then
    echo -e "${YELLOW}⚠️  Albatross socket not found${NC}"
    echo "   Is the demo running? Try: ./run-demo.sh"
    exit 1
fi

# Query discovered peers
echo -e "${BLUE}📡 Discovered Peers:${NC}"
RESPONSE=$(echo '{"jsonrpc":"2.0","method":"primal.list_all","id":1}' | nc -U "$ALBATROSS_SOCKET" 2>/dev/null)

if [ $? -eq 0 ]; then
    PEER_COUNT=$(echo "$RESPONSE" | jq -r '.result.total_primals // 0')
    
    echo -e "   Total: ${GREEN}$PEER_COUNT peers${NC}"
    echo ""
    
    # Show breakdown by role
    echo -e "${BLUE}📊 Breakdown by Role:${NC}"
    
    ALBATROSS_COUNT=$(echo "$RESPONSE" | jq -r '[.result.primals[]? | select(.capabilities[]? | contains("multiplexer"))] | length')
    SONGBIRD_COUNT=$(echo "$RESPONSE" | jq -r '[.result.primals[]? | select(.capabilities[]? | contains("orchestrator"))] | length')
    SPARROW_COUNT=$(echo "$RESPONSE" | jq -r '[.result.primals[]? | select(.capabilities[]? | contains("sensor"))] | length')
    
    echo "   🦅 Albatross:  $ALBATROSS_COUNT"
    echo "   🎵 Songbirds:  $SONGBIRD_COUNT"
    echo "   🐦 Sparrows:   $SPARROW_COUNT"
    echo ""
    
    # Show peer details
    echo -e "${BLUE}🗂️  Peer Details:${NC}"
    echo "$RESPONSE" | jq -r '.result.primals[]? | "   • \(.primal_id) - [\(.capabilities | join(", "))]"' 2>/dev/null | head -15
    
    if [ "$PEER_COUNT" -ge 14 ]; then
        echo ""
        echo -e "${GREEN}✅ Full mesh formed successfully!${NC}"
    else
        echo ""
        echo -e "${YELLOW}⚠️  Mesh still forming... (expected 14, got $PEER_COUNT)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Could not query Albatross${NC}"
fi

echo ""

