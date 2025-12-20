#!/bin/bash
# Songbird Tower - One-Touch Startup
# Universal script for ANY tower (eastgate, westgate, strandgate, etc.)
# Zero manual configuration - secure by default

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎵 Songbird Tower - One-Touch Startup${NC}"
echo "========================================"
echo ""

# Detect tower name (or use environment variable)
TOWER_NAME="${SONGBIRD_TOWER_NAME:-$(hostname)}"
echo -e "${GREEN}Tower Name:${NC} $TOWER_NAME"
echo ""

# Check if we're in the songbird directory
if [ ! -f "Cargo.toml" ] || ! grep -q "songbird-orchestrator" Cargo.toml 2>/dev/null; then
    echo -e "${RED}❌ Error: Not in songbird directory${NC}"
    echo "Please run this script from the songbird root directory"
    exit 1
fi

# Check if binary exists
if [ ! -f "target/release/songbird-orchestrator" ]; then
    echo -e "${YELLOW}⚠️  Binary not found. Building...${NC}"
    cargo build --release
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
fi

# Kill any existing songbird processes
echo "🧹 Cleaning up existing processes..."
pkill -f songbird-orchestrator 2>/dev/null && echo "  Stopped existing orchestrator" || echo "  No existing processes"
sleep 2
echo ""

# Create logs directory
mkdir -p logs

# Set log file
LOG_FILE="logs/${TOWER_NAME}-$(date +%Y%m%d-%H%M%S).log"

echo "🚀 Starting Songbird Orchestrator..."
echo -e "${GREEN}Configuration:${NC}"
echo "  🔒 TLS: Enabled (secure by default)"
echo "  🌐 Discovery: Anonymous UDP broadcast"
echo "  🤝 Federation: Enabled with zero-trust"
echo "  📊 Trust: Progressive escalation"
echo "  🎯 Network: Auto-detected (zero-config)"
echo ""
echo -e "${YELLOW}Log file:${NC} $LOG_FILE"
echo ""

# Start orchestrator with secure defaults
# ✨ ZERO MANUAL CONFIGURATION ✨
# Songbird auto-detects optimal network binding!
# No more SONGBIRD_BIND_ADDRESS required
SONGBIRD_TLS_ENABLED=true \
SONGBIRD_FEDERATION_ENABLED=true \
SONGBIRD_ANONYMOUS_DISCOVERY=true \
SONGBIRD_NODE_NAME="$TOWER_NAME" \
SONGBIRD_TOWER_NAME="$TOWER_NAME" \
nohup ./target/release/songbird-orchestrator > "$LOG_FILE" 2>&1 &

ORCHESTRATOR_PID=$!
echo -e "${GREEN}✅ Orchestrator started${NC}"
echo "  PID: $ORCHESTRATOR_PID"
echo ""

# Wait a moment for startup
echo "⏳ Waiting for services to initialize..."
sleep 5

# Check if process is still running
if ! ps -p $ORCHESTRATOR_PID > /dev/null 2>&1; then
    echo -e "${RED}❌ Error: Orchestrator failed to start${NC}"
    echo "Check log file: $LOG_FILE"
    tail -20 "$LOG_FILE"
    exit 1
fi

# Detect services
echo "🔍 Detecting services..."
echo ""

# Find HTTPS port
HTTPS_PORT=$(sudo lsof -i -P -n 2>/dev/null | grep "$ORCHESTRATOR_PID" | grep TCP | grep LISTEN | head -1 | awk '{print $9}' | cut -d':' -f2)
if [ -n "$HTTPS_PORT" ]; then
    echo -e "  ${GREEN}✅ HTTPS Server:${NC} Port $HTTPS_PORT"
else
    echo -e "  ${YELLOW}⏳ HTTPS Server:${NC} Starting..."
fi

# Check UDP discovery
if sudo lsof -i UDP:2300 -P -n 2>/dev/null | grep -q "$ORCHESTRATOR_PID"; then
    echo -e "  ${GREEN}✅ Discovery:${NC} UDP port 2300 (broadcasting & listening)"
else
    echo -e "  ${YELLOW}⏳ Discovery:${NC} Initializing..."
fi

echo ""

# Get local IP
LOCAL_IP=$(hostname -I | awk '{print $1}')
echo -e "${BLUE}📡 Tower Information:${NC}"
echo "  Name: $TOWER_NAME"
echo "  IP: $LOCAL_IP"
if [ -n "$HTTPS_PORT" ]; then
    echo "  HTTPS: https://$LOCAL_IP:$HTTPS_PORT"
fi
echo "  Discovery: UDP broadcast on port 2300"
echo ""

# Federation info
echo -e "${BLUE}🌐 Federation Status:${NC}"
echo "  Mode: Zero-trust with progressive escalation"
echo "  Discovery: Automatic (no manual configuration)"
echo "  Connection: Peers will discover automatically"
echo "  Trust: Anonymous → Capability → Identity → Hardware"
echo ""

echo -e "${GREEN}✅ Tower is ready!${NC}"
echo ""
echo "📋 Monitoring:"
echo "  View logs:    tail -f $LOG_FILE"
echo "  Filter logs:  tail -f $LOG_FILE | grep -i discovery"
echo "  Federation:   curl -k https://localhost:${HTTPS_PORT:-8080}/api/federation/status"
echo "  Stop tower:   pkill -f songbird-orchestrator"
echo ""
echo "🤝 Other towers will discover this tower automatically!"
echo "   Just run this same script on other machines."
echo ""
echo "========================================"
echo -e "${BLUE}Songbird Tower: $TOWER_NAME - OPERATIONAL${NC}"



