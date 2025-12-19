#!/usr/bin/env bash
#
# Start Songbird in HTTP mode (TLS disabled) for local development
#
# This is useful for:
# - Local testing
# - Cross-tower development (like ToadStool distributed ML)
# - Debugging without TLS complexity
#
# ⚠️  WARNING: This disables encryption! Use only on trusted networks.

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Starting Songbird in HTTP mode (TLS disabled)${NC}"
echo -e "${RED}⚠️  WARNING: Connections will NOT be encrypted!${NC}"
echo -e "${YELLOW}   Use only for local development on trusted networks.${NC}"
echo ""

# Disable TLS
export SONGBIRD_TLS_ENABLED=false

# Use default HTTP port
export SONGBIRD_PORT=${SONGBIRD_PORT:-8081}

# Set bind address (default to all interfaces for cross-tower)
export SONGBIRD_BIND_ADDRESS=${SONGBIRD_BIND_ADDRESS:-"[::]"}

echo -e "${GREEN}Configuration:${NC}"
echo "  TLS: Disabled"
echo "  Port: $SONGBIRD_PORT"
echo "  Bind: $SONGBIRD_BIND_ADDRESS"
echo ""
echo -e "${GREEN}Endpoints:${NC}"
echo "  Health: http://localhost:$SONGBIRD_PORT/health"
echo "  API: http://localhost:$SONGBIRD_PORT/api/"
echo ""

# Start Songbird
echo -e "${GREEN}Starting Songbird orchestrator...${NC}"
cargo run --release --bin songbird-orchestrator

