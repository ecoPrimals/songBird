#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🚀 Songbird + BearDog Backbone - Quick Start
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

clear

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   🌳🐻 Songbird + BearDog: The P2P Backbone${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${CYAN}The Foundation of Sovereign Interprimal Communication${NC}"
echo ""

echo -e "${YELLOW}Available Demonstrations:${NC}"
echo ""
echo "  1. Genesis Ceremony         - How nodes are born with lineage"
echo "  2. BirdSong Broadcasting    - Privacy-preserving discovery"
echo "  3. Lineage Relay            - NAT traversal without TURN"
echo "  4. Multi-Primal Coordination - Primals working together"
echo "  5. Hardware Root of Trust   - Genesis with SoloKey"
echo "  6. Full Integration Test    - Complete end-to-end scenario"
echo ""
echo "  A. Run All Demos (sequential)"
echo "  Q. Quit"
echo ""

read -p "Select a demo (1-6, A, Q): " choice

case "$choice" in
    1)
        echo ""
        echo -e "${GREEN}Running Demo 1: Genesis Ceremony${NC}"
        ./01-genesis-ceremony.sh
        ;;
    2)
        echo ""
        echo -e "${GREEN}Running Demo 2: BirdSong Broadcasting${NC}"
        ./02-birdsong-broadcast.sh
        ;;
    3)
        echo ""
        echo -e "${GREEN}Running Demo 3: Lineage Relay${NC}"
        ./03-lineage-relay.sh
        ;;
    4)
        echo ""
        echo -e "${GREEN}Running Demo 4: Multi-Primal Coordination${NC}"
        ./04-multi-primal.sh
        ;;
    5)
        echo ""
        echo -e "${GREEN}Running Demo 5: Hardware Root of Trust${NC}"
        ./05-hardware-genesis.sh
        ;;
    6)
        echo ""
        echo -e "${GREEN}Running Demo 6: Full Integration Test${NC}"
        ./06-full-integration.sh
        ;;
    [Aa])
        echo ""
        echo -e "${GREEN}Running All Demos...${NC}"
        echo ""
        
        for demo in 01-genesis-ceremony.sh 02-birdsong-broadcast.sh 03-lineage-relay.sh 04-multi-primal.sh 05-hardware-genesis.sh 06-full-integration.sh; do
            echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
            echo -e "${YELLOW}Running: $demo${NC}"
            echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
            ./$demo
            echo ""
            echo -e "${CYAN}Press Enter to continue to next demo...${NC}"
            read
        done
        
        echo ""
        echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}     ✅ All Demos Complete!${NC}"
        echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
        ;;
    [Qq])
        echo ""
        echo -e "${CYAN}Thanks for exploring the backbone!${NC}"
        exit 0
        ;;
    *)
        echo ""
        echo -e "${RED}Invalid choice. Please run again and select 1-6, A, or Q.${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}What to do next:${NC}"
echo ""
echo "  📚 Read the README.md for detailed architecture"
echo "  🔍 Explore the demo scripts to see implementation"
echo "  🧪 Run integration tests with real BearDog v0.9.0"
echo "  🚀 Integrate into your primal"
echo ""
echo -e "${GREEN}Run ./QUICK_START.sh again to explore more demos!${NC}"
echo ""

