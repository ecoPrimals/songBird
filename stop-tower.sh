#!/bin/bash
# Stop Songbird Tower
# Universal script for ANY tower

echo "🛑 Stopping Songbird Tower..."
echo "=============================="
echo ""

# Find and kill songbird processes
PIDS=$(pgrep -f songbird-orchestrator)

if [ -z "$PIDS" ]; then
    echo "No songbird processes found."
    exit 0
fi

echo "Found processes:"
ps aux | grep songbird-orchestrator | grep -v grep
echo ""

echo "Stopping gracefully..."
pkill -TERM -f songbird-orchestrator
sleep 2

# Check if still running
REMAINING=$(pgrep -f songbird-orchestrator)
if [ -n "$REMAINING" ]; then
    echo "⚠️  Some processes still running, forcing..."
    pkill -9 -f songbird-orchestrator
    sleep 1
fi

# Final check
if pgrep -f songbird-orchestrator > /dev/null; then
    echo "❌ Failed to stop all processes"
    ps aux | grep songbird-orchestrator | grep -v grep
    exit 1
else
    echo "✅ All songbird processes stopped"
fi

echo ""
echo "Tower stopped successfully."

