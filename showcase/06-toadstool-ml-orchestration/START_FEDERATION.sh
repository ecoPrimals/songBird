#!/usr/bin/env bash
# Start 2-Tower Federation: Eastgate + Strandgate
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}🎵 Starting 2-Tower Federation${NC}"
echo -e "${BLUE}   Eastgate (192.168.1.144) + Strandgate (192.168.1.134)${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo

# Tower status
echo -e "${YELLOW}📊 Checking tower status...${NC}"
echo -n "   Eastgate (localhost:8000): "
if curl -sk https://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Online${NC}"
else
    echo -e "${YELLOW}⚠️  Offline - Starting...${NC}"
    cd /home/eastgate/Development/ecoPrimals/songbird/showcase/02-federation
    ./scripts/start-tower.sh eastgate 8000 &
    sleep 3
fi

echo -n "   Strandgate (192.168.1.134:8081): "
if curl -sk https://192.168.1.134:8081/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Online${NC}"
else
    echo -e "${YELLOW}⚠️  Offline${NC}"
    echo "   Start Strandgate separately"
fi

echo
echo -e "${YELLOW}🤝 Registering Eastgate with federation...${NC}"

curl -sk -X POST https://localhost:8000/api/federation/join \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "tower-eastgate",
    "node_name": "Eastgate",
    "node_address": "192.168.1.144:8000",
    "cpu_cores": 12,
    "memory_gb": 32,
    "capabilities": ["compute", "ml-training", "ml-inference", "gpu-rtx-2070"],
    "metadata": {"gpu": "NVIDIA RTX 2070", "gpu_memory_gb": 8, "location": "eastgate"}
  }' | jq . || true

echo
echo -e "${GREEN}✅ Federation ready!${NC}"
echo
echo -e "${BLUE}📊 Federation Status:${NC}"
curl -sk https://localhost:8000/api/federation/services | jq .

echo
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 2-Tower Federation LIVE!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo
echo "Next: Run distributed ML workload"
echo "  ./demos/01-simple-inference.sh"

