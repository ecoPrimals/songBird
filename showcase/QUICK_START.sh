#!/usr/bin/env bash
#
# Songbird Showcase - Quick Start
# Get started with demos in < 2 minutes
#

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

clear

echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${MAGENTA}        🎵 Songbird Showcase - Quick Start 🎵${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${BLUE}World-class orchestration with zero-configuration mesh${NC}"
echo
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Check if Songbird is built
SONGBIRD_BIN="../target/release/songbird-orchestrator"

if [[ ! -f "$SONGBIRD_BIN" ]]; then
    echo -e "${YELLOW}⚠️  Songbird not built yet${NC}"
    echo
    echo -e "${BLUE}Building Songbird... (this may take a few minutes)${NC}"
    cd .. && cargo build --release
    
    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✅ Songbird built successfully${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
    echo
    cd showcase
fi

echo -e "${GREEN}✅ Songbird is ready${NC}"
echo

# Show demo options
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Choose your adventure:${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "  ${MAGENTA}1)${NC} ${YELLOW}★ THE MAGIC${NC} - Friend Joins LAN (5 min, zero-config)"
echo -e "     ${GREEN}→ The killer demo that shows the vision${NC}"
echo
echo -e "  ${BLUE}2)${NC} Hello Songbird (2 min, beginner-friendly)"
echo -e "     ${GREEN}→ Perfect starting point, understand basics${NC}"
echo
echo -e "  ${BLUE}3)${NC} Full Learning Path (30-60 min, comprehensive)"
echo -e "     ${GREEN}→ All 3 phases, complete understanding${NC}"
echo
echo -e "  ${BLUE}4)${NC} Read Documentation"
echo -e "     ${GREEN}→ Understand before running${NC}"
echo
echo -e "  ${BLUE}5)${NC} Exit"
echo
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

read -p "$(echo -e ${YELLOW}'Enter choice (1-5): '${NC})" choice

case $choice in
    1)
        echo
        echo -e "${MAGENTA}🌟 Running THE MAGIC - LAN Join Demo...${NC}"
        echo
        sleep 1
        cd 03-inter-primal/demos
        ./03-lan-join-demo.sh
        ;;
    2)
        echo
        echo -e "${BLUE}🎵 Running Hello Songbird...${NC}"
        echo
        sleep 1
        cd 01-isolated/demos
        ./01-hello-songbird.sh
        ;;
    3)
        echo
        echo -e "${BLUE}📚 Full Learning Path${NC}"
        echo
        echo -e "${YELLOW}Phase 1: Isolated Instance${NC}"
        echo -e "   Read: 01-isolated/README.md"
        echo -e "   Demo: 01-isolated/demos/01-hello-songbird.sh"
        echo
        echo -e "${YELLOW}Phase 2: Federation${NC}"
        echo -e "   Read: 02-federation/README.md"
        echo -e "   Demos: 02-federation/demos/"
        echo
        echo -e "${YELLOW}Phase 3: Inter-Primal${NC}"
        echo -e "   Read: 03-inter-primal/README.md"
        echo -e "   Demo: 03-inter-primal/demos/03-lan-join-demo.sh"
        echo
        read -p "$(echo -e ${YELLOW}'Press Enter to start with Phase 1...'${NC})"
        cd 01-isolated/demos
        ./01-hello-songbird.sh
        ;;
    4)
        echo
        echo -e "${BLUE}📚 Documentation${NC}"
        echo
        echo -e "  ${GREEN}Main README:${NC}     README.md"
        echo -e "  ${GREEN}Index:${NC}           SHOWCASE_INDEX.md"
        echo -e "  ${GREEN}Summary:${NC}         SHOWCASE_SUMMARY.md"
        echo
        echo -e "  ${GREEN}Phase 1:${NC}         01-isolated/README.md"
        echo -e "  ${GREEN}Phase 2:${NC}         02-federation/README.md"
        echo -e "  ${GREEN}Phase 3:${NC}         03-inter-primal/README.md"
        echo
        echo -e "${YELLOW}💡 Tip: Start with README.md, then dive into phases${NC}"
        ;;
    5)
        echo
        echo -e "${GREEN}👋 Come back anytime!${NC}"
        exit 0
        ;;
    *)
        echo
        echo -e "${YELLOW}Invalid choice. Run again and choose 1-5${NC}"
        exit 1
        ;;
esac

echo
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Thanks for exploring Songbird Showcase!${NC}"
echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${BLUE}Questions? Check the READMEs or visit:${NC}"
echo -e "   ${GREEN}../docs/${NC}"
echo -e "   ${GREEN}../specs/${NC}"
echo

