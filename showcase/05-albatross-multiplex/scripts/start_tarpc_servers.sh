#!/bin/bash
# Start 3 standalone tarpc servers for Albatross benchmarking

TARPC_BIN="/home/eastgate/Development/ecoPrimals/songbird/target/release/tarpc-server"
LOG_DIR="/home/eastgate/Development/ecoPrimals/songbird/showcase/05-albatross-multiplex/logs"

echo "🚀 Starting 3 tarpc servers..."

# Stop existing
pkill -f tarpc-server 2>/dev/null

# Start 3 instances
RUST_LOG=info "$TARPC_BIN" 0.0.0.0:8091 > "$LOG_DIR/tarpc-8091.log" 2>&1 &
echo "  tarpc-8091 (PID: $!)"

RUST_LOG=info "$TARPC_BIN" 0.0.0.0:8092 > "$LOG_DIR/tarpc-8092.log" 2>&1 &
echo "  tarpc-8092 (PID: $!)"

RUST_LOG=info "$TARPC_BIN" 0.0.0.0:8093 > "$LOG_DIR/tarpc-8093.log" 2>&1 &
echo "  tarpc-8093 (PID: $!)"

sleep 2

# Verify
if ss -tlnp | grep -q 8091 && ss -tlnp | grep -q 8092 && ss -tlnp | grep -q 8093; then
    echo "✅ All 3 tarpc servers running"
else
    echo "⚠️  Some servers may not have started"
fi

