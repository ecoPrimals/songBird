#!/bin/bash
# Songbird USB Live Spore Launcher
# genomeBin-compliant portable launcher for USB/removable media
#
# Usage:
#   ./launch-songbird.sh [family_id]
#
# Features:
#   - Fully portable (no system installation required)
#   - XDG-compliant runtime directories
#   - Automatic cleanup on exit
#   - Zero hardcoding (runtime discovery)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory (USB mount point)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_ROOT="$(dirname "$SCRIPT_DIR")"

# Family ID (default: "usb-spore")
FAMILY_ID="${1:-usb-spore}"

# Portable directories (on USB)
USB_BIN="${USB_ROOT}/bin"
USB_DATA="${USB_ROOT}/data/songbird-${FAMILY_ID}"
USB_LOGS="${USB_ROOT}/logs/songbird-${FAMILY_ID}"

# Runtime directory (XDG-compliant, ephemeral)
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/songbird-runtime-$$}"
RUNTIME_DIR="${XDG_RUNTIME_DIR}/songbird-${FAMILY_ID}"

# Songbird binary path
SONGBIRD_BIN="${USB_BIN}/songbird"

# PID file
PID_FILE="${RUNTIME_DIR}/songbird.pid"

# Cleanup function (called on exit)
cleanup() {
    local exit_code=$?
    
    echo -e "${YELLOW}[CLEANUP]${NC} Stopping Songbird USB Live Spore..."
    
    # Kill Songbird process if running
    if [ -f "${PID_FILE}" ]; then
        local pid=$(cat "${PID_FILE}")
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${BLUE}[INFO]${NC} Sending SIGTERM to PID $pid..."
            kill -TERM "$pid" 2>/dev/null || true
            
            # Wait for graceful shutdown (max 10 seconds)
            local count=0
            while kill -0 "$pid" 2>/dev/null && [ $count -lt 10 ]; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "${RED}[WARN]${NC} Graceful shutdown failed, force killing..."
                kill -KILL "$pid" 2>/dev/null || true
            fi
        fi
        rm -f "${PID_FILE}"
    fi
    
    # Clean up runtime directory
    if [ -d "${RUNTIME_DIR}" ]; then
        echo -e "${BLUE}[INFO]${NC} Cleaning up runtime directory..."
        rm -rf "${RUNTIME_DIR}"
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} Songbird USB Live Spore stopped."
    exit $exit_code
}

# Register cleanup handler
trap cleanup EXIT INT TERM

# Print banner
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC}  ${GREEN}🌍 Songbird USB Live Spore Launcher${NC}                      ${BLUE}║${NC}"
echo -e "${BLUE}║${NC}  ${YELLOW}genomeBin-compliant portable deployment${NC}                  ${BLUE}║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verify binary exists
if [ ! -f "${SONGBIRD_BIN}" ]; then
    echo -e "${RED}[ERROR]${NC} Songbird binary not found: ${SONGBIRD_BIN}"
    echo -e "${YELLOW}[HINT]${NC} Expected directory structure:"
    echo "  ${USB_ROOT}/"
    echo "    ├── bin/songbird (x86_64-unknown-linux-musl)"
    echo "    ├── deployment/usb-live-spore/launch-songbird.sh"
    echo "    ├── data/ (persistent data)"
    echo "    └── logs/ (log files)"
    exit 1
fi

# Verify binary is executable
if [ ! -x "${SONGBIRD_BIN}" ]; then
    echo -e "${YELLOW}[WARN]${NC} Binary not executable, fixing permissions..."
    chmod +x "${SONGBIRD_BIN}"
fi

# Create directories
echo -e "${BLUE}[INFO]${NC} Creating directories..."
mkdir -p "${USB_DATA}" "${USB_LOGS}" "${RUNTIME_DIR}"

# Display configuration
echo -e "${BLUE}[INFO]${NC} Configuration:"
echo "  • Family ID: ${FAMILY_ID}"
echo "  • USB Root: ${USB_ROOT}"
echo "  • Binary: ${SONGBIRD_BIN}"
echo "  • Data: ${USB_DATA}"
echo "  • Logs: ${USB_LOGS}"
echo "  • Runtime: ${RUNTIME_DIR}"
echo ""

# Verify binary architecture
if file "${SONGBIRD_BIN}" | grep -q "statically linked"; then
    echo -e "${GREEN}[OK]${NC} Binary is statically linked (portable!)"
else
    echo -e "${YELLOW}[WARN]${NC} Binary may have dynamic dependencies"
    echo "  This may cause issues on different systems."
fi
echo ""

# Set environment variables (zero hardcoding!)
export SONGBIRD_FAMILY_ID="${FAMILY_ID}"
export SONGBIRD_MODE="usb-live-spore"
export RUST_LOG="${RUST_LOG:-info}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR}"

# Optional: BearDog integration (auto-discover)
if [ -f "${USB_BIN}/beardog" ]; then
    export BEARDOG_SOCKET="${RUNTIME_DIR}/beardog.sock"
    echo -e "${GREEN}[INFO]${NC} BearDog binary found, integration enabled"
    echo "  Socket: ${BEARDOG_SOCKET}"
fi

# Launch Songbird
echo -e "${GREEN}[START]${NC} Launching Songbird USB Live Spore..."
echo ""

# Run in background and capture PID
"${SONGBIRD_BIN}" > "${USB_LOGS}/songbird-$(date +%Y%m%d-%H%M%S).log" 2>&1 &
SONGBIRD_PID=$!

# Save PID
echo "$SONGBIRD_PID" > "${PID_FILE}"

# Wait a moment for startup
sleep 2

# Verify process is running
if ! kill -0 "$SONGBIRD_PID" 2>/dev/null; then
    echo -e "${RED}[ERROR]${NC} Songbird failed to start!"
    echo -e "${YELLOW}[HINT]${NC} Check logs: ${USB_LOGS}"
    tail -20 "${USB_LOGS}/songbird-"*.log | tail -20
    exit 1
fi

# Display running status
echo -e "${GREEN}[SUCCESS]${NC} Songbird USB Live Spore is running!"
echo ""
echo -e "${BLUE}[STATUS]${NC}"
echo "  • PID: ${SONGBIRD_PID}"
echo "  • Socket: ${RUNTIME_DIR}/songbird.sock"
echo "  • Logs: ${USB_LOGS}"
echo ""
echo -e "${YELLOW}[INFO]${NC} Press Ctrl+C to stop Songbird and cleanup."
echo ""

# Wait for process (allows Ctrl+C to trigger cleanup)
wait "$SONGBIRD_PID"
