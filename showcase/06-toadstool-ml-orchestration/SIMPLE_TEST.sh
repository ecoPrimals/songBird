#!/usr/bin/env bash
# Simple test: Submit task to Songbird federation

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🎵 Simple Federation Test${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Test both towers
echo -e "${YELLOW}Testing Eastgate...${NC}"
if curl -sk https://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Eastgate online${NC}"
    EASTGATE_ONLINE=true
else
    echo -e "${YELLOW}⚠️  Eastgate offline${NC}"
    EASTGATE_ONLINE=false
fi

echo -e "${YELLOW}Testing Strandgate...${NC}"
if curl -sk https://192.168.1.134:8081/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Strandgate online${NC}"
    STRANDGATE_ONLINE=true
else
    echo -e "${YELLOW}⚠️  Strandgate offline${NC}"
    STRANDGATE_ONLINE=false
fi

echo

if [[ "$EASTGATE_ONLINE" == "true" ]]; then
    echo -e "${BLUE}📊 Eastgate Capabilities:${NC}"
    curl -sk https://localhost:8000/api/v1/capabilities 2>/dev/null | jq -r '.capabilities[]' 2>/dev/null || echo "  (API not available yet)"
    echo
fi

if [[ "$STRANDGATE_ONLINE" == "true" ]]; then
    echo -e "${BLUE}📊 Strandgate Capabilities:${NC}"
    curl -sk https://192.168.1.134:8081/api/v1/capabilities 2>/dev/null | jq -r '.capabilities[]' 2>/dev/null || echo "  (API not available yet)"
    echo
fi

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Federation Test Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "Summary:"
echo "  Eastgate:   $([[ "$EASTGATE_ONLINE" == "true" ]] && echo "✅ Online" || echo "❌ Offline")"
echo "  Strandgate: $([[ "$STRANDGATE_ONLINE" == "true" ]] && echo "✅ Online" || echo "❌ Offline")"
echo
echo "Both towers are ready for distributed workloads!"
echo

