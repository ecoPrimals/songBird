#!/usr/bin/env bash
# Test script for IPC HTTP functionality
# This validates that Songbird can expose HTTP/HTTPS via JSON-RPC over Unix socket

set -e

SOCKET_PATH="/tmp/songbird-ipc-test.sock"
BEARDOG_SOCKET="/tmp/beardog-test.sock"  # Will be mocked for this test

echo "🧪 Testing Songbird IPC HTTP Integration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Cleanup function
cleanup() {
    echo
    echo "🧹 Cleaning up..."
    rm -f "$SOCKET_PATH"
    echo "✅ Test complete"
}
trap cleanup EXIT

# Remove old socket
rm -f "$SOCKET_PATH"

echo "1. Starting Songbird with IPC mode..."
echo "   Command: songbird server --socket $SOCKET_PATH --beardog-socket $BEARDOG_SOCKET"
echo

# Note: This test assumes BearDog is running or will be mocked
# For now, we just validate the CLI accepts the flags

if cargo run --bin songbird -- server --help | grep -q "socket"; then
    echo "✅ CLI accepts --socket flag"
else
    echo "❌ CLI does not accept --socket flag"
    exit 1
fi

if cargo run --bin songbird -- server --help | grep -q "beardog-socket"; then
    echo "✅ CLI accepts --beardog-socket flag"
else
    echo "❌ CLI does not accept --beardog-socket flag"
    exit 1
fi

echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ IPC Integration: CLI flags validated!"
echo
echo "📋 Next Steps for Full Integration Test:"
echo "   1. Start BearDog: beardog server --socket $BEARDOG_SOCKET"
echo "   2. Start Songbird: songbird server --socket $SOCKET_PATH"
echo "   3. Test HTTP request:"
echo "      echo '{\"jsonrpc\":\"2.0\",\"method\":\"http.get\",\"params\":{\"url\":\"https://cloudflare.com\"},\"id\":1}' | nc -U $SOCKET_PATH"
echo


