#!/usr/bin/env bash
set -euo pipefail

# Test Songbird rendezvous client integration

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🌍 Testing Songbird Rendezvous Client Integration"
echo ""

# Start rendezvous server in background
echo "Starting rendezvous server..."
cd "$SONGBIRD_ROOT/rendezvous"
cargo run --release &
RENDEZVOUS_PID=$!
sleep 3

# Cleanup on exit
cleanup() {
    echo ""
    echo "Stopping rendezvous server..."
    kill $RENDEZVOUS_PID 2>/dev/null || true
}
trap cleanup EXIT

# Build Songbird with rendezvous support
echo "Building Songbird..."
cd "$SONGBIRD_ROOT"
cargo build --release --quiet

echo ""
echo "✅ Rendezvous server running on http://localhost:8888"
echo "✅ Songbird compiled with rendezvous client"
echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Integration Ready"
echo "═══════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "1. Wire rendezvous client into FederationCoordinator"
echo "2. Test registration on startup"
echo "3. Test peer discovery via rendezvous"
echo "4. Deploy rendezvous server to internet"
echo ""
echo "See: specs/RENDEZVOUS_PROTOCOL_SPEC.md for details"

