#!/bin/bash
# Quick connectivity check for Strandgate tower

set -e

# Try common configurations
HOSTS=(
    "strandgate.local"
    "strandgate"
    "192.168.1.100"
    "10.0.0.100"
)

PORTS=(
    "8080"
    "8000"
    "3000"
)

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           🔍 Checking Strandgate Tower Connection 🔍          ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if user provided host
if [ -n "$REMOTE_HOST" ]; then
    HOSTS=("$REMOTE_HOST")
    echo -e "${BLUE}Using REMOTE_HOST:${NC} $REMOTE_HOST"
    echo ""
fi

FOUND=false

for HOST in "${HOSTS[@]}"; do
    echo -e "${BLUE}Testing host:${NC} $HOST"
    
    # Ping test
    if ping -c 1 -W 2 "$HOST" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} Ping successful"
        
        # Port scan
        for PORT in "${PORTS[@]}"; do
            echo -e "  Testing port $PORT..."
            
            # Try HTTP health endpoint
            if timeout 3 curl -s -f "http://$HOST:$PORT/health" > /dev/null 2>&1; then
                echo -e "  ${GREEN}✓✓✓ FOUND SONGBIRD!${NC}"
                echo ""
                
                ENDPOINT="http://$HOST:$PORT"
                
                # Get details
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo "Songbird Instance Details:"
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                
                # Health check
                HEALTH=$(curl -s "$ENDPOINT/health")
                STATUS=$(echo "$HEALTH" | jq -r '.status // "unknown"')
                VERSION=$(echo "$HEALTH" | jq -r '.version // "unknown"')
                
                echo -e "Status:   ${GREEN}$STATUS${NC}"
                echo -e "Version:  ${YELLOW}$VERSION${NC}"
                echo -e "Endpoint: ${YELLOW}$ENDPOINT${NC}"
                echo ""
                
                # Check for protocol capabilities
                echo "Checking protocol capabilities..."
                CAPS=$(curl -s "$ENDPOINT/api/protocol/capabilities" 2>/dev/null || echo "{}")
                
                if echo "$CAPS" | jq -e '.protocols' > /dev/null 2>&1; then
                    echo -e "${GREEN}✓ Multi-protocol support detected!${NC}"
                    echo ""
                    echo "Available protocols:"
                    echo "$CAPS" | jq -r '.protocols | keys[]' | while read proto; do
                        echo -e "  ${GREEN}✓${NC} $proto"
                    done
                    echo ""
                    PREFERRED=$(echo "$CAPS" | jq -r '.preferred_protocol // "unknown"')
                    echo -e "Preferred protocol: ${YELLOW}$PREFERRED${NC}"
                else
                    echo -e "${YELLOW}⚠ Multi-protocol API not available${NC}"
                    echo "  This tower needs the update!"
                fi
                
                echo ""
                echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                echo ""
                echo "🚀 To deploy the update:"
                echo ""
                echo "  export REMOTE_HOST=$HOST"
                echo "  export REMOTE_PORT=$PORT"
                echo "  export COMPUTE_BRIDGE=http://$HOST:$PORT"
                echo "  ./showcase/04-multi-protocol/deploy_to_remote_tower.sh"
                echo ""
                echo "Or test protocol escalation if already updated:"
                echo ""
                echo "  export REMOTE_HOST=$HOST"
                echo "  ./showcase/04-multi-protocol/test_remote_protocol_escalation.sh"
                echo ""
                
                FOUND=true
                break 2
            fi
        done
    else
        echo -e "  ${RED}✗${NC} Cannot reach host"
    fi
    echo ""
done

if [ "$FOUND" = false ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${RED}✗ Could not find Songbird on any host${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Troubleshooting:"
    echo ""
    echo "1. Check if Strandgate is powered on:"
    echo "   ping strandgate.local"
    echo ""
    echo "2. Check if Songbird is running on Strandgate:"
    echo "   ssh strandgate.local 'systemctl status songbird'"
    echo ""
    echo "3. Check what port Songbird is using:"
    echo "   ssh strandgate.local 'sudo lsof -i -P | grep songbird'"
    echo ""
    echo "4. Try manually specifying host and port:"
    echo "   export REMOTE_HOST=<ip-or-hostname>"
    echo "   export REMOTE_PORT=<port>"
    echo "   ./showcase/04-multi-protocol/check_strandgate.sh"
    echo ""
    echo "5. Check firewall on Strandgate:"
    echo "   ssh strandgate.local 'sudo ufw status'"
    echo ""
    
    exit 1
fi

