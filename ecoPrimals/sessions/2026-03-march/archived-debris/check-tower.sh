#!/bin/bash
# Check Songbird Tower Status
# Universal script for ANY tower

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔍 Songbird Tower Status Check${NC}"
echo "=============================="
echo ""

# Check if orchestrator is running
PID=$(pgrep -f songbird-orchestrator)
if [ -z "$PID" ]; then
    echo -e "${RED}❌ Status: NOT RUNNING${NC}"
    echo ""
    echo "Start with: ./start-tower.sh"
    exit 1
fi

echo -e "${GREEN}✅ Status: RUNNING${NC}"
echo "  PID: $PID"
echo ""

# Get tower info
TOWER_NAME="${SONGBIRD_TOWER_NAME:-$(hostname)}"
LOCAL_IP=$(hostname -I | awk '{print $1}')

echo -e "${BLUE}📡 Tower Information:${NC}"
echo "  Name: $TOWER_NAME"
echo "  IP: $LOCAL_IP"
echo ""

# Check services
echo -e "${BLUE}🔧 Services:${NC}"

# HTTPS
HTTPS_PORT=$(sudo lsof -i -P -n 2>/dev/null | grep "$PID" | grep TCP | grep LISTEN | head -1 | awk '{print $9}' | cut -d':' -f2)
if [ -n "$HTTPS_PORT" ]; then
    echo -e "  ${GREEN}✅ HTTPS:${NC} Port $HTTPS_PORT"
    
    # Test health endpoint
    if timeout 2 curl -k -s https://localhost:$HTTPS_PORT/health > /dev/null 2>&1; then
        echo -e "     Health check: ${GREEN}OK${NC}"
    else
        echo -e "     Health check: ${YELLOW}Not responding${NC}"
    fi
else
    echo -e "  ${YELLOW}⏳ HTTPS:${NC} Initializing..."
fi

# Discovery
if sudo lsof -i UDP:2300 -P -n 2>/dev/null | grep -q "$PID"; then
    echo -e "  ${GREEN}✅ Discovery:${NC} UDP port 2300"
else
    echo -e "  ${RED}❌ Discovery:${NC} Not listening"
fi

# UDP Broadcaster
BROADCAST_PORT=$(sudo lsof -i UDP -P -n 2>/dev/null | grep "$PID" | grep -v ":2300" | head -1 | awk '{print $9}' | cut -d':' -f2)
if [ -n "$BROADCAST_PORT" ]; then
    echo -e "  ${GREEN}✅ Broadcaster:${NC} UDP port $BROADCAST_PORT"
else
    echo -e "  ${YELLOW}⏳ Broadcaster:${NC} Initializing..."
fi

echo ""

# Federation status
if [ -n "$HTTPS_PORT" ]; then
    echo -e "${BLUE}🌐 Federation Status:${NC}"
    
    FED_STATUS=$(timeout 3 curl -k -s https://localhost:$HTTPS_PORT/api/federation/status 2>/dev/null)
    if [ -n "$FED_STATUS" ]; then
        ACTIVE_NODES=$(echo "$FED_STATUS" | jq -r '.active_nodes' 2>/dev/null || echo "?")
        FEDERATION_ID=$(echo "$FED_STATUS" | jq -r '.federation_id' 2>/dev/null | cut -c1-8)
        
        echo "  Federation ID: $FEDERATION_ID..."
        echo "  Active Nodes: $ACTIVE_NODES"
        
        if [ "$ACTIVE_NODES" = "1" ]; then
            echo -e "  ${YELLOW}⏳ Waiting for peers...${NC}"
        elif [ "$ACTIVE_NODES" -gt "1" ] 2>/dev/null; then
            echo -e "  ${GREEN}✅ Connected to federation!${NC}"
        fi
    else
        echo -e "  ${YELLOW}⏳ Federation API initializing...${NC}"
    fi
fi

echo ""

# Recent logs
LATEST_LOG=$(ls -t logs/*.log 2>/dev/null | head -1)
if [ -n "$LATEST_LOG" ]; then
    echo -e "${BLUE}📋 Recent Activity:${NC}"
    echo "  Log: $LATEST_LOG"
    echo ""
    
    # Show last few log lines with discovery/federation activity
    tail -20 "$LATEST_LOG" | grep -iE "discovery|federation|peer|trust" | tail -5 || echo "  No recent federation activity"
fi

echo ""
echo "=============================="
echo "Commands:"
echo "  View logs:    tail -f $LATEST_LOG"
echo "  Federation:   curl -k https://localhost:${HTTPS_PORT:-8080}/api/federation/status | jq '.'"
echo "  Stop tower:   ./stop-tower.sh"
echo ""

