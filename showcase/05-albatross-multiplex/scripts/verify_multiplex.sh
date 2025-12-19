#!/bin/bash
# Verify Albatross Multiplex Status

GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║         🔍 VERIFYING ALBATROSS MULTIPLEX 🔍                      ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

SUCCESS=0
TOTAL=0

# Check Songbird A
TOTAL=$((TOTAL + 1))
echo -n "Songbird A (8443): "
if curl -k -s https://localhost:8443/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Running${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}❌ Not responding${NC}"
fi

# Check Songbird B
TOTAL=$((TOTAL + 1))
echo -n "Songbird B (8444): "
if curl -k -s https://localhost:8444/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Running${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}❌ Not responding${NC}"
fi

# Check Songbird C
TOTAL=$((TOTAL + 1))
echo -n "Songbird C (8445): "
if curl -k -s https://localhost:8445/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Running${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}❌ Not responding${NC}"
fi

# Check Toadstool
TOTAL=$((TOTAL + 1))
echo -n "Toadstool  (7878): "
if curl -s http://localhost:7878/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Running${NC}"
    SUCCESS=$((SUCCESS + 1))
else
    echo -e "${RED}❌ Not responding${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${CYAN}Status: $SUCCESS/$TOTAL services running${NC}"

if [ $SUCCESS -eq $TOTAL ]; then
    echo -e "${GREEN}✅ Multiplex is ready for Albatross benchmarking!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some services not responding${NC}"
    echo "Check logs in: ./logs/"
    exit 1
fi

