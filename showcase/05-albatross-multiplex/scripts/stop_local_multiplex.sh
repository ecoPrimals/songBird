#!/bin/bash
# Stop Albatross Local Multiplex

GREEN='\033[0;32m'
NC='\033[0m'

echo "🦅 Stopping Albatross multiplex..."
echo ""

LOG_DIR="./logs"

# Read PIDs if available
if [ -f "$LOG_DIR/pids.txt" ]; then
    source "$LOG_DIR/pids.txt"
    
    [ ! -z "$SONGBIRD_A_PID" ] && kill $SONGBIRD_A_PID 2>/dev/null && echo "  Stopped Songbird A (PID: $SONGBIRD_A_PID)"
    [ ! -z "$SONGBIRD_B_PID" ] && kill $SONGBIRD_B_PID 2>/dev/null && echo "  Stopped Songbird B (PID: $SONGBIRD_B_PID)"
    [ ! -z "$SONGBIRD_C_PID" ] && kill $SONGBIRD_C_PID 2>/dev/null && echo "  Stopped Songbird C (PID: $SONGBIRD_C_PID)"
    [ ! -z "$TOADSTOOL_PID" ] && kill $TOADSTOOL_PID 2>/dev/null && echo "  Stopped Toadstool (PID: $TOADSTOOL_PID)"
    
    rm "$LOG_DIR/pids.txt"
fi

# Fallback: kill by pattern
pkill -f "songbird-orchestrator.*844[3-5]" 2>/dev/null
pkill -f "simple_toadstool" 2>/dev/null

echo ""
echo -e "${GREEN}✅ Multiplex stopped${NC}"

