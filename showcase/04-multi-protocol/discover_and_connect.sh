#!/bin/bash
# Use Songbird's built-in discovery to find Strandgate automatically
# This leverages Songbird's mDNS, capability-based discovery, and federation APIs

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║    🔍 Auto-Discover Strandgate Using Songbird APIs 🔍         ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if local Songbird is running (we'll use it for discovery)
LOCAL_SONGBIRD="${LOCAL_SONGBIRD:-http://localhost:8080}"

echo -e "${BLUE}[1/5]${NC} Checking if local Songbird orchestrator is running..."
if ! curl -s -f "$LOCAL_SONGBIRD/health" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Local Songbird not running at $LOCAL_SONGBIRD${NC}"
    echo ""
    echo "To use Songbird's discovery, you need to start a local instance:"
    echo "  cargo run --release --bin songbird-orchestrator"
    echo ""
    echo "Or set LOCAL_SONGBIRD to an existing instance:"
    echo "  export LOCAL_SONGBIRD=http://your-songbird:8080"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Local Songbird is running${NC}"
echo ""

# Use Songbird's federation discovery API
echo -e "${BLUE}[2/5]${NC} Using Songbird's federation discovery to find towers..."

# Check if federation API exists
TOWERS_RESPONSE=$(curl -s "$LOCAL_SONGBIRD/api/federation/towers" 2>/dev/null || echo "[]")

TOWER_COUNT=$(echo "$TOWERS_RESPONSE" | jq '. | length' 2>/dev/null || echo "0")

if [ "$TOWER_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Found $TOWER_COUNT federated tower(s)!${NC}"
    echo ""
    echo "Discovered towers:"
    echo "$TOWERS_RESPONSE" | jq -r '.[] | "  • \(.node_name) @ \(.address) - \(.status)"'
    echo ""
    
    # Look for Strandgate specifically
    STRANDGATE=$(echo "$TOWERS_RESPONSE" | jq -r '.[] | select(.node_name | contains("strandgate")) | .address' | head -1)
    
    if [ -n "$STRANDGATE" ] && [ "$STRANDGATE" != "null" ]; then
        echo -e "${GREEN}✓✓✓ Found Strandgate at: $STRANDGATE${NC}"
        FOUND_VIA="federation"
    fi
else
    echo -e "${YELLOW}ℹ No towers in federation yet${NC}"
fi

# If not found via federation, try capability-based discovery
if [ -z "$STRANDGATE" ] || [ "$STRANDGATE" = "null" ]; then
    echo ""
    echo -e "${BLUE}[3/5]${NC} Trying capability-based discovery..."
    
    # Use Songbird's capability discovery (looks for orchestration capability)
    DISCOVERED=$(curl -s "$LOCAL_SONGBIRD/api/discovery/capabilities?capability=orchestration" 2>/dev/null || echo "[]")
    
    DISCOVERED_COUNT=$(echo "$DISCOVERED" | jq '. | length' 2>/dev/null || echo "0")
    
    if [ "$DISCOVERED_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $DISCOVERED_COUNT service(s) with orchestration capability${NC}"
        echo ""
        
        # Look for Strandgate
        STRANDGATE=$(echo "$DISCOVERED" | jq -r '.[] | select(.name | contains("strandgate")) | .endpoint' | head -1)
        
        if [ -n "$STRANDGATE" ] && [ "$STRANDGATE" != "null" ]; then
            echo -e "${GREEN}✓✓✓ Found Strandgate at: $STRANDGATE${NC}"
            FOUND_VIA="capability-discovery"
        fi
    fi
fi

# If still not found, use mDNS discovery (via Songbird's CLI if available)
if [ -z "$STRANDGATE" ] || [ "$STRANDGATE" = "null" ]; then
    echo ""
    echo -e "${BLUE}[4/5]${NC} Trying mDNS discovery..."
    
    # Check if songbird CLI is available
    if command -v songbird &> /dev/null; then
        echo "Using Songbird CLI for mDNS discovery..."
        MDNS_RESULT=$(songbird discover --format json 2>/dev/null || echo "[]")
        
        STRANDGATE=$(echo "$MDNS_RESULT" | jq -r '.[] | select(.name | contains("strandgate")) | .endpoint' | head -1)
        
        if [ -n "$STRANDGATE" ] && [ "$STRANDGATE" != "null" ]; then
            echo -e "${GREEN}✓✓✓ Found Strandgate via mDNS: $STRANDGATE${NC}"
            FOUND_VIA="mdns"
        fi
    else
        echo -e "${YELLOW}ℹ Songbird CLI not available, skipping mDNS${NC}"
    fi
fi

# Final check - did we find it?
if [ -z "$STRANDGATE" ] || [ "$STRANDGATE" = "null" ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}⚠ Could not auto-discover Strandgate${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "This could mean:"
    echo "  1. Strandgate is not running"
    echo "  2. Strandgate is not broadcasting via mDNS"
    echo "  3. Strandgate is not in the federation"
    echo "  4. Strandgate is on a different network"
    echo ""
    echo "Manual discovery options:"
    echo "  • Use check_strandgate.sh for manual scan"
    echo "  • SSH to Strandgate and check Songbird status"
    echo "  • Join Strandgate to federation manually"
    echo ""
    echo "To manually join Strandgate to federation:"
    echo "  1. Find Strandgate's IP and port"
    echo "  2. POST to: $LOCAL_SONGBIRD/api/federation/register"
    echo "  3. Or use: curl -X POST $LOCAL_SONGBIRD/api/federation/join \\
         -d '{\"bootstrap\":\"strandgate-ip:port\"}'"
    echo ""
    exit 1
fi

# Success! We found Strandgate
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Successfully discovered Strandgate!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "Endpoint: ${YELLOW}$STRANDGATE${NC}"
echo -e "Discovery method: ${YELLOW}$FOUND_VIA${NC}"
echo ""

# Extract host and port
STRANDGATE_HOST=$(echo "$STRANDGATE" | sed 's|http[s]*://||' | cut -d':' -f1)
STRANDGATE_PORT=$(echo "$STRANDGATE" | sed 's|http[s]*://||' | cut -d':' -f2)

# Test connection
echo -e "${BLUE}[5/5]${NC} Testing connection to Strandgate..."
STRANDGATE_HEALTH=$(curl -s "$STRANDGATE/health" 2>/dev/null || echo '{"status":"unreachable"}')
STRANDGATE_STATUS=$(echo "$STRANDGATE_HEALTH" | jq -r '.status')

if [ "$STRANDGATE_STATUS" = "healthy" ]; then
    echo -e "${GREEN}✓ Strandgate is healthy and responsive${NC}"
    
    # Check for multi-protocol support
    CAPS=$(curl -s "$STRANDGATE/api/protocol/capabilities" 2>/dev/null || echo "{}")
    
    if echo "$CAPS" | jq -e '.protocols' > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Multi-protocol support detected${NC}"
        echo ""
        echo "Available protocols:"
        echo "$CAPS" | jq -r '.protocols | keys[]' | while read proto; do
            echo -e "  ${GREEN}✓${NC} $proto"
        done
    else
        echo -e "${YELLOW}⚠ Multi-protocol API not detected - needs update${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Strandgate status: $STRANDGATE_STATUS${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Next steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Deploy multi-protocol update to Strandgate:"
echo "   export REMOTE_HOST=$STRANDGATE_HOST"
echo "   export REMOTE_PORT=$STRANDGATE_PORT"
echo "   export COMPUTE_BRIDGE=$STRANDGATE"
echo "   ./showcase/04-multi-protocol/deploy_to_remote_tower.sh"
echo ""
echo "2. Test protocol escalation:"
echo "   export REMOTE_HOST=$STRANDGATE_HOST"
echo "   ./showcase/04-multi-protocol/test_remote_protocol_escalation.sh"
echo ""
echo "3. Monitor the connection:"
echo "   watch -n 2 'curl -s $STRANDGATE/api/protocol/capabilities | jq .'"
echo ""

