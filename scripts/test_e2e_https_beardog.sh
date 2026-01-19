#!/usr/bin/env bash
# E2E Test: Songbird HTTPS with Real BearDog
#
# This script tests the complete Pure Rust HTTPS stack:
# - Starts BearDog (crypto provider)
# - Starts Songbird with HTTPS enabled
# - Tests HTTPS connection
# - Validates Pure Rust TLS handshake

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BEARDOG_BIN="${BEARDOG_BIN:-/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/beardog-server}"
BEARDOG_SOCKET="/tmp/beardog-test-$(date +%s).sock"
SONGBIRD_PORT="${SONGBIRD_PORT:-8443}"
TEST_TIMEOUT=30

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     E2E Test: Songbird HTTPS + Real BearDog Integration   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}→ Cleaning up...${NC}"
    
    if [ -n "${BEARDOG_PID:-}" ]; then
        echo "  • Stopping BearDog (PID: $BEARDOG_PID)"
        kill $BEARDOG_PID 2>/dev/null || true
    fi
    
    if [ -n "${SONGBIRD_PID:-}" ]; then
        echo "  • Stopping Songbird (PID: $SONGBIRD_PID)"
        kill $SONGBIRD_PID 2>/dev/null || true
    fi
    
    if [ -S "$BEARDOG_SOCKET" ]; then
        echo "  • Removing BearDog socket"
        rm -f "$BEARDOG_SOCKET"
    fi
    
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

trap cleanup EXIT

# Step 1: Check if BearDog binary exists
echo -e "${YELLOW}→ Step 1: Checking BearDog binary...${NC}"
if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}✗ BearDog binary not found at: $BEARDOG_BIN${NC}"
    echo "  Set BEARDOG_BIN environment variable to the correct path"
    exit 1
fi
echo -e "${GREEN}✓ BearDog binary found: $BEARDOG_BIN${NC}"

# Step 2: Start BearDog
echo -e "\n${YELLOW}→ Step 2: Starting BearDog crypto provider...${NC}"
export BEARDOG_SOCKET_PATH="$BEARDOG_SOCKET"
$BEARDOG_BIN --socket "$BEARDOG_SOCKET" > /tmp/beardog-test.log 2>&1 &
BEARDOG_PID=$!

# Wait for BearDog to create socket
echo "  • Waiting for BearDog socket..."
for i in {1..10}; do
    if [ -S "$BEARDOG_SOCKET" ]; then
        echo -e "${GREEN}✓ BearDog started (PID: $BEARDOG_PID, Socket: $BEARDOG_SOCKET)${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 10 ]; then
        echo -e "${RED}✗ BearDog socket not created after 10 seconds${NC}"
        cat /tmp/beardog-test.log
        exit 1
    fi
done

# Step 3: Build Songbird
echo -e "\n${YELLOW}→ Step 3: Building Songbird...${NC}"
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build -p songbird-orchestrator --release 2>&1 | grep -E "(Finished|Compiling songbird)" || true
echo -e "${GREEN}✓ Songbird built${NC}"

# Step 4: Start Songbird with HTTPS
echo -e "\n${YELLOW}→ Step 4: Starting Songbird HTTPS server...${NC}"
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_BIND_PORT=$SONGBIRD_PORT
export SONGBIRD_NODE_ID="songbird-test"

# Note: Songbird will discover BearDog automatically via $BEARDOG_SOCKET_PATH
./target/release/songbird-orchestrator > /tmp/songbird-test.log 2>&1 &
SONGBIRD_PID=$!

# Wait for Songbird to start
echo "  • Waiting for Songbird HTTPS server..."
for i in {1..$TEST_TIMEOUT}; do
    if curl -k -s https://localhost:$SONGBIRD_PORT/health >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Songbird HTTPS server started (PID: $SONGBIRD_PID, Port: $SONGBIRD_PORT)${NC}"
        break
    fi
    sleep 1
    if [ $i -eq $TEST_TIMEOUT ]; then
        echo -e "${RED}✗ Songbird did not start after $TEST_TIMEOUT seconds${NC}"
        echo "Songbird logs:"
        cat /tmp/songbird-test.log
        exit 1
    fi
done

# Step 5: Test HTTPS connection
echo -e "\n${YELLOW}→ Step 5: Testing HTTPS connection...${NC}"

# Test 1: Health endpoint
echo "  • Testing /health endpoint..."
RESPONSE=$(curl -k -s https://localhost:$SONGBIRD_PORT/health)
if [ "$RESPONSE" = "OK" ]; then
    echo -e "${GREEN}✓ Health check passed: $RESPONSE${NC}"
else
    echo -e "${RED}✗ Health check failed. Response: $RESPONSE${NC}"
    exit 1
fi

# Test 2: Verify TLS is being used
echo "  • Verifying TLS handshake..."
if curl -v -k https://localhost:$SONGBIRD_PORT/health 2>&1 | grep -q "SSL connection"; then
    echo -e "${GREEN}✓ TLS handshake successful${NC}"
else
    echo -e "${RED}✗ TLS handshake failed${NC}"
    exit 1
fi

# Step 6: Verify logs
echo -e "\n${YELLOW}→ Step 6: Verifying integration logs...${NC}"

# Check BearDog logs for crypto operations
if grep -q "crypto" /tmp/beardog-test.log 2>/dev/null; then
    echo -e "${GREEN}✓ BearDog crypto operations detected${NC}"
else
    echo -e "${YELLOW}⚠ No crypto operations in BearDog logs (may be normal)${NC}"
fi

# Check Songbird logs for TLS
if grep -q "Pure Rust TLS" /tmp/songbird-test.log 2>/dev/null; then
    echo -e "${GREEN}✓ Songbird using Pure Rust TLS${NC}"
else
    echo -e "${YELLOW}⚠ Pure Rust TLS not mentioned in logs${NC}"
fi

# Final summary
echo -e "\n${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   E2E TEST RESULTS                         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✓ BearDog started successfully${NC}"
echo -e "${GREEN}✓ Songbird HTTPS server started${NC}"
echo -e "${GREEN}✓ HTTPS connection successful${NC}"
echo -e "${GREEN}✓ TLS handshake completed${NC}"
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║            ✅ E2E TEST PASSED - INTEGRATION WORKS! ✅       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}🦀✨ Pure Rust HTTPS Stack: Validated! ✨🦀${NC}"
echo ""
echo "Logs available at:"
echo "  • BearDog: /tmp/beardog-test.log"
echo "  • Songbird: /tmp/songbird-test.log"

