#!/usr/bin/env bash
#
# Demo 01: Hello Songbird
# The simplest demo - start Songbird and verify it's healthy
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SONGBIRD_PORT=${SONGBIRD_PORT:-8000}
SONGBIRD_BIN=${SONGBIRD_BIN:-"../../../target/release/songbird-orchestrator"}
SONGBIRD_HOST=${SONGBIRD_HOST:-"localhost"}

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🎵 Songbird Showcase - Phase 1: Hello Songbird${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Check if Songbird binary exists
if [[ ! -f "$SONGBIRD_BIN" ]]; then
    echo -e "${RED}❌ Songbird binary not found at: $SONGBIRD_BIN${NC}"
    echo -e "${YELLOW}💡 Build it first: cargo build --release${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Songbird binary found${NC}"
echo

# Check if port is available
if lsof -Pi :$SONGBIRD_PORT -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo -e "${YELLOW}⚠️  Port $SONGBIRD_PORT is already in use${NC}"
    echo -e "${YELLOW}💡 Stop existing service or use: SONGBIRD_PORT=8080 $0${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Port $SONGBIRD_PORT is available${NC}"
echo

# Start Songbird in background
echo -e "${BLUE}🚀 Starting Songbird on http://$SONGBIRD_HOST:$SONGBIRD_PORT${NC}"
echo

# Set environment for Songbird
export RUST_LOG=${RUST_LOG:-info}
export SONGBIRD_PORT=$SONGBIRD_PORT
export SONGBIRD_HOST=$SONGBIRD_HOST

# Start in background and capture PID
$SONGBIRD_BIN > /tmp/songbird-demo.log 2>&1 &
SONGBIRD_PID=$!

echo -e "${YELLOW}⏳ Waiting for Songbird to start...${NC}"
sleep 3

# Check if process is still running
if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}❌ Songbird failed to start${NC}"
    echo -e "${YELLOW}📋 Check logs: tail -f /tmp/songbird-demo.log${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Songbird process running (PID: $SONGBIRD_PID)${NC}"
echo

# Check health endpoint
echo -e "${BLUE}🔍 Checking health endpoint...${NC}"
echo

HEALTH_RESPONSE=$(curl -s "http://$SONGBIRD_HOST:$SONGBIRD_PORT/health" 2>&1)
if [[ "$HEALTH_RESPONSE" == "OK" ]]; then
    echo -e "${GREEN}✅ Songbird is healthy!${NC}"
    echo
    echo -e "${BLUE}📊 Health Response:${NC} $HEALTH_RESPONSE"
    echo
else
    echo -e "${RED}❌ Health check failed${NC}"
    echo -e "${YELLOW}Response: $HEALTH_RESPONSE${NC}"
    echo -e "${YELLOW}📋 Check logs: tail -f /tmp/songbird-demo.log${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 Success! Songbird is running${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${BLUE}📍 Endpoints:${NC}"
echo -e "   Health:     http://$SONGBIRD_HOST:$SONGBIRD_PORT/health"
echo -e "   Discovery:  http://$SONGBIRD_HOST:$SONGBIRD_PORT/api/v1/discovery"
echo -e "   Metrics:    http://$SONGBIRD_HOST:$SONGBIRD_PORT/api/v1/metrics"
echo
echo -e "${BLUE}📋 Useful Commands:${NC}"
echo -e "   Check health:  curl http://$SONGBIRD_HOST:$SONGBIRD_PORT/health | jq"
echo -e "   View logs:     tail -f /tmp/songbird-demo.log"
echo -e "   Stop Songbird: kill $SONGBIRD_PID"
echo
echo -e "${YELLOW}💡 Tip: Leave Songbird running and try other demos!${NC}"
echo
echo -e "${BLUE}Press Ctrl+C to stop Songbird${NC}"
echo

# Wait for user interrupt
trap "echo; echo -e '${YELLOW}🛑 Stopping Songbird...${NC}'; kill $SONGBIRD_PID 2>/dev/null || true; echo -e '${GREEN}✅ Songbird stopped${NC}'; exit 0" INT TERM

# Keep running
wait $SONGBIRD_PID

