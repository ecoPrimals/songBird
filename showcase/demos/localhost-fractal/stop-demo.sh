#!/bin/bash
# Stop the localhost fractal demo

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo -e "${RED}🛑 Stopping Localhost Fractal Demo...${NC}"
echo ""

# Kill all songbird processes
pkill -f "songbird-orchestrator" 2>/dev/null

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ All Songbird processes terminated${NC}"
else
    echo -e "${GREEN}✅ No Songbird processes running${NC}"
fi

# Clean up sockets
rm -f /tmp/songbird-demo-fractal-*.sock 2>/dev/null
echo -e "${GREEN}✅ Cleaned up Unix sockets${NC}"

# Clean up PID files
rm -f /var/run/songbird/songbird-demo-fractal-*.pid 2>/dev/null
rm -f ~/.local/share/songbird/songbird-demo-fractal-*.pid 2>/dev/null
echo -e "${GREEN}✅ Cleaned up PID files${NC}"

echo ""
echo -e "${GREEN}🎊 Demo stopped successfully!${NC}"
echo ""
echo "Logs preserved at: /tmp/songbird-demo-logs/"
echo "Run ./run-demo.sh to start again"
echo ""

