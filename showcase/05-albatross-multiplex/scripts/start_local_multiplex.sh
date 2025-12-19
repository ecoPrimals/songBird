#!/bin/bash
# Start Local Songbird Multiplex for Albatross Benchmarking
# Starts 3 Songbirds + 1 Toadstool on local machine

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

SONGBIRD_BIN="/home/eastgate/Development/ecoPrimals/songbird/target/release/songbird-orchestrator"
TOADSTOOL_BIN="$(pwd)/simple_toadstool"
LOG_DIR="$(pwd)/logs"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║       🦅 STARTING ALBATROSS LOCAL MULTIPLEX 🦅                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "This will start:"
echo "  • 3 Songbird instances (ports 8443, 8444, 8445)"
echo "  • 1 Toadstool instance (port 7878)"
echo ""

# Create log directory
mkdir -p "$LOG_DIR"

# Check if binaries exist
if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${YELLOW}Building Songbird...${NC}"
    CURRENT_DIR=$(pwd)
    cd /home/eastgate/Development/ecoPrimals/songbird
    cargo build --release --bin songbird-orchestrator
    cd "$CURRENT_DIR"
fi

if [ ! -f "$TOADSTOOL_BIN" ]; then
    echo -e "${YELLOW}Building simple Toadstool...${NC}"
    if [ -f "simple_toadstool.rs" ]; then
        rustc simple_toadstool.rs -o simple_toadstool
    else
        echo -e "${YELLOW}❌ simple_toadstool.rs not found${NC}"
        echo "Run this from showcase/05-albatross-multiplex/"
        exit 1
    fi
fi

# Stop any existing instances
echo -e "${BLUE}[1/5]${NC} Stopping existing instances..."
pkill -f "songbird-orchestrator.*844[3-5]" 2>/dev/null || true
pkill -f "simple_toadstool" 2>/dev/null || true
sleep 1
echo -e "${GREEN}✅ Cleaned up${NC}"
echo ""

# Start Songbird A (master)
echo -e "${BLUE}[2/5]${NC} Starting Songbird A (master) on port 8443..."
SONGBIRD_PORT=8443 SONGBIRD_TARPC_PORT=8091 SONGBIRD_TARPC_ENABLED=true RUST_LOG=info \
  "$SONGBIRD_BIN" > "$LOG_DIR/songbird-a.log" 2>&1 &
SONGBIRD_A_PID=$!
echo "   PID: $SONGBIRD_A_PID"
sleep 3

if curl -k -s https://localhost:8443/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird A running${NC}"
else
    echo -e "${YELLOW}⚠️  Songbird A not responding${NC}"
    tail -20 "$LOG_DIR/songbird-a.log"
fi
echo ""

# Start Songbird B
echo -e "${BLUE}[3/5]${NC} Starting Songbird B on port 8444..."
SONGBIRD_PORT=8444 SONGBIRD_TARPC_PORT=8092 SONGBIRD_TARPC_ENABLED=true RUST_LOG=info \
  "$SONGBIRD_BIN" > "$LOG_DIR/songbird-b.log" 2>&1 &
SONGBIRD_B_PID=$!
echo "   PID: $SONGBIRD_B_PID"
sleep 3

if curl -k -s https://localhost:8444/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird B running${NC}"
else
    echo -e "${YELLOW}⚠️  Songbird B not responding${NC}"
    tail -20 "$LOG_DIR/songbird-b.log"
fi
echo ""

# Start Songbird C
echo -e "${BLUE}[4/5]${NC} Starting Songbird C on port 8445..."
SONGBIRD_PORT=8445 SONGBIRD_TARPC_PORT=8093 SONGBIRD_TARPC_ENABLED=true RUST_LOG=info \
  "$SONGBIRD_BIN" > "$LOG_DIR/songbird-c.log" 2>&1 &
SONGBIRD_C_PID=$!
echo "   PID: $SONGBIRD_C_PID"
sleep 3

if curl -k -s https://localhost:8445/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird C running${NC}"
else
    echo -e "${YELLOW}⚠️  Songbird C not responding${NC}"
    tail -20 "$LOG_DIR/songbird-c.log"
fi
echo ""

# Start Toadstool
echo -e "${BLUE}[5/5]${NC} Starting Toadstool on port 7878..."
"$TOADSTOOL_BIN" > "$LOG_DIR/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
echo "   PID: $TOADSTOOL_PID"
sleep 2

if curl -s http://localhost:7878/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Toadstool running${NC}"
else
    echo -e "${YELLOW}⚠️  Toadstool not responding${NC}"
    tail -20 "$LOG_DIR/toadstool.log"
fi
echo ""

# Summary
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║             ✨ LOCAL MULTIPLEX STARTED ✨                        ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${CYAN}Songbird Instances:${NC}"
echo "  A (master): https://localhost:8443 (PID: $SONGBIRD_A_PID, tarpc: 8091)"
echo "  B:          https://localhost:8444 (PID: $SONGBIRD_B_PID, tarpc: 8092)"
echo "  C:          https://localhost:8445 (PID: $SONGBIRD_C_PID, tarpc: 8093)"
echo ""
echo -e "${CYAN}Compute:${NC}"
echo "  Toadstool:  http://localhost:7878 (PID: $TOADSTOOL_PID)"
echo ""
echo -e "${CYAN}Logs:${NC}"
echo "  $LOG_DIR/songbird-{a,b,c}.log"
echo "  $LOG_DIR/toadstool.log"
echo ""
echo -e "${CYAN}To stop:${NC}"
echo "  ./scripts/stop_local_multiplex.sh"
echo ""
echo -e "${CYAN}To verify:${NC}"
echo "  ./scripts/verify_multiplex.sh"
echo ""
echo -e "${CYAN}To benchmark:${NC}"
echo "  ./demo_albatross.sh"
echo ""

# Save PIDs
cat > "$LOG_DIR/pids.txt" << EOF
SONGBIRD_A_PID=$SONGBIRD_A_PID
SONGBIRD_B_PID=$SONGBIRD_B_PID
SONGBIRD_C_PID=$SONGBIRD_C_PID
TOADSTOOL_PID=$TOADSTOOL_PID
EOF

echo "🦅 Albatross multiplex ready for benchmarking!"
echo ""

