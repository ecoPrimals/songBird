#!/usr/bin/env bash
# Quick Start: Songbird ↔ ToadStool ML Orchestration
# Gets you running in under 5 minutes

set -euo pipefail

SHOWCASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}🚀 Songbird ↔ ToadStool Quick Start${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo

# Step 1: Check builds
echo -e "${YELLOW}[1/4] Checking builds...${NC}"

SONGBIRD_BIN="/home/eastgate/Development/ecoPrimals/songbird/target/release/songbird-orchestrator"
TOADSTOOL_BIN="/home/eastgate/Development/ecoPrimals/toadstool/target/release/toadstool-server"

if [[ ! -f "$SONGBIRD_BIN" ]]; then
    echo -e "${YELLOW}⚠️  Songbird not built, building now...${NC}"
    cd /home/eastgate/Development/ecoPrimals/songbird
    cargo build --release --bin songbird-orchestrator
    echo -e "${GREEN}✅ Songbird built${NC}"
else
    echo -e "${GREEN}✅ Songbird already built${NC}"
fi

if [[ ! -f "$TOADSTOOL_BIN" ]]; then
    echo -e "${YELLOW}⚠️  ToadStool not built${NC}"
    echo "   Build it with:"
    echo "   cd /home/eastgate/Development/ecoPrimals/toadstool"
    echo "   cargo build --release"
    echo
    echo -e "${YELLOW}   Continuing without ToadStool for now...${NC}"
else
    echo -e "${GREEN}✅ ToadStool already built${NC}"
fi

echo

# Step 2: Start Songbird
echo -e "${YELLOW}[2/4] Starting Songbird orchestrator...${NC}"

if pgrep -f "songbird-orchestrator" > /dev/null; then
    echo -e "${GREEN}✅ Songbird already running${NC}"
else
    mkdir -p "$SHOWCASE_DIR/logs"
    
    # Start with federation enabled
    nohup "$SONGBIRD_BIN" \
        --config "$SHOWCASE_DIR/configs/songbird-orchestrator.toml" \
        > "$SHOWCASE_DIR/logs/songbird.log" 2>&1 &
    
    SONGBIRD_PID=$!
    echo "$SONGBIRD_PID" > "$SHOWCASE_DIR/logs/songbird.pid"
    
    sleep 2
    
    if ps -p $SONGBIRD_PID > /dev/null; then
        echo -e "${GREEN}✅ Songbird started (PID: $SONGBIRD_PID)${NC}"
    else
        echo -e "${RED}❌ Songbird failed to start${NC}"
        tail -20 "$SHOWCASE_DIR/logs/songbird.log"
        exit 1
    fi
fi

echo

# Step 3: Start ToadStool (if available)
echo -e "${YELLOW}[3/4] Starting ToadStool (if available)...${NC}"

if [[ -f "$TOADSTOOL_BIN" ]]; then
    if pgrep -f "toadstool-server" > /dev/null; then
        echo -e "${GREEN}✅ ToadStool already running${NC}"
    else
        nohup "$TOADSTOOL_BIN" \
            --config "$SHOWCASE_DIR/configs/toadstool-server.toml" \
            --register-with-songbird "http://localhost:8080" \
            > "$SHOWCASE_DIR/logs/toadstool.log" 2>&1 &
        
        TOADSTOOL_PID=$!
        echo "$TOADSTOOL_PID" > "$SHOWCASE_DIR/logs/toadstool.pid"
        
        sleep 2
        
        if ps -p $TOADSTOOL_PID > /dev/null; then
            echo -e "${GREEN}✅ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
        else
            echo -e "${YELLOW}⚠️  ToadStool failed to start (continuing anyway)${NC}"
        fi
    fi
else
    echo -e "${YELLOW}⚠️  ToadStool not available (skipping)${NC}"
fi

echo

# Step 4: Verify mesh
echo -e "${YELLOW}[4/4] Verifying mesh...${NC}"

sleep 2

HEALTH_CHECK=$(curl -sf http://localhost:8080/health 2>/dev/null || echo "")
if [[ -n "$HEALTH_CHECK" ]]; then
    echo -e "${GREEN}✅ Songbird API responding${NC}"
else
    echo -e "${RED}❌ Songbird API not responding${NC}"
    exit 1
fi

SERVICES=$(curl -sf http://localhost:8080/api/federation/services 2>/dev/null || echo "{}")
SERVICE_COUNT=$(echo "$SERVICES" | jq -r '.services | length' 2>/dev/null || echo "0")

echo -e "${BLUE}📊 Mesh Status:${NC}"
echo "   Songbird: ✅ Running"
if [[ "$SERVICE_COUNT" -gt 0 ]]; then
    echo "   ToadStool: ✅ $SERVICE_COUNT instance(s) discovered"
else
    echo "   ToadStool: ⚠️  No instances (manual registration may be needed)"
fi

echo
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Quick Start Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo
echo -e "${BLUE}🚀 Next Steps:${NC}"
echo
echo "   1. Run your first demo:"
echo "      cd $SHOWCASE_DIR"
echo "      ./demos/01-simple-inference.sh"
echo
echo "   2. Check the mesh status:"
echo "      curl http://localhost:8080/api/federation/services | jq"
echo
echo "   3. View logs:"
echo "      tail -f $SHOWCASE_DIR/logs/songbird.log"
echo "      tail -f $SHOWCASE_DIR/logs/toadstool.log"
echo
echo "   4. Stop everything:"
echo "      ./scripts/stop-all.sh"
echo
echo -e "${YELLOW}📚 Read the full guide: $SHOWCASE_DIR/README.md${NC}"
echo

