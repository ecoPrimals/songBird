#!/usr/bin/env bash
#
# Quick start script for ToadStool distributed ML testing
#
# This script starts Songbird in HTTP mode for easy cross-tower communication
# without TLS certificate issues.
#
# Usage:
#   ./scripts/toadstool-quick-start.sh [tower-name]
#
# Examples:
#   ./scripts/toadstool-quick-start.sh eastgate    # Tower A
#   ./scripts/toadstool-quick-start.sh strandgate  # Tower B

set -euo pipefail

TOWER_NAME=${1:-"local"}

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  🎵 Songbird Quick Start for ToadStool Distributed ML${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}Tower: ${TOWER_NAME}${NC}"
echo ""

# Disable TLS for easy cross-tower communication
export SONGBIRD_TLS_ENABLED=false

# Set node ID
export SONGBIRD_NODE_ID="songbird-${TOWER_NAME}"

# Port configuration
export SONGBIRD_PORT=8081

# Bind to all interfaces for cross-tower access
export SONGBIRD_BIND_ADDRESS="[::]"

# Enable verbose logging for debugging
export RUST_LOG=${RUST_LOG:-"info,songbird=debug"}

echo -e "${YELLOW}⚠️  TLS DISABLED for development${NC}"
echo -e "${YELLOW}   This is OK for local network testing.${NC}"
echo -e "${YELLOW}   Enable TLS for production!${NC}"
echo ""

echo -e "${GREEN}Configuration:${NC}"
echo "  Node ID: $SONGBIRD_NODE_ID"
echo "  Port: $SONGBIRD_PORT"
echo "  TLS: Disabled (HTTP only)"
echo "  Bind: All interfaces (0.0.0.0)"
echo ""

# Detect local IP
LOCAL_IP=$(hostname -I | awk '{print $1}' || echo "unknown")
echo -e "${GREEN}Local IP: ${LOCAL_IP}${NC}"
echo ""

echo -e "${GREEN}Endpoints:${NC}"
echo "  Health: http://localhost:$SONGBIRD_PORT/health"
echo "  Health (network): http://${LOCAL_IP}:$SONGBIRD_PORT/health"
echo "  Compute API: http://${LOCAL_IP}:$SONGBIRD_PORT/api/compute/task"
echo "  WebSocket: ws://${LOCAL_IP}:$SONGBIRD_PORT/api/ws/tasks"
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Starting Songbird orchestrator...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Start Songbird
cargo run --release --bin songbird-orchestrator

