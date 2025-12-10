#!/usr/bin/env bash
# 🎵 Songbird Federation - Local Setup
# Sets up a 3-node mesh on localhost for testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BINARY="$PROJECT_ROOT/target/release/songbird-orchestrator"

echo "🎵 Songbird Federation - Local Setup"
echo "===================================="
echo

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found: $BINARY"
    echo "Building Songbird..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin songbird-orchestrator
    echo "✅ Build complete"
fi

# Clean up any existing instances
echo "🧹 Cleaning up any existing instances..."
killall -q songbird-orchestrator 2>/dev/null || true
sleep 1

# Create data directories for each tower
echo "📁 Creating data directories..."
mkdir -p "$SCRIPT_DIR/data/tower-a"
mkdir -p "$SCRIPT_DIR/data/tower-b"
mkdir -p "$SCRIPT_DIR/data/tower-c"

# Create log directories
mkdir -p "$SCRIPT_DIR/logs"

echo "✅ Setup complete!"
echo
echo "Next steps:"
echo "  1. Run demos individually: cd demos && ./01-mesh-formation.sh"
echo "  2. Run all demos: ./run-all-demos.sh"
echo "  3. Manual setup: See README.md for multi-machine configuration"
echo

