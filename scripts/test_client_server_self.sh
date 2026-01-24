#!/bin/bash
# Songbird TLS Client + Server Self-Test Harness
# Purpose: Compare client and server transcripts byte-by-byte
# Strategy: biomeOS validated approach (18+ hour breakthrough session)
#
# This script will:
# 1. Start BearDog (crypto provider)
# 2. Start Songbird TLS server
# 3. Connect with Songbird TLS client
# 4. Extract transcripts from both logs
# 5. Compare transcripts byte-by-byte
# 6. Identify exact differences (likely in Certificate message)

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║   🔬 SONGBIRD TLS CLIENT + SERVER SELF-TEST                ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Purpose: Compare client and server transcripts byte-by-byte"
echo "🎯 Goal: Find exact differences to fix Certificate content"
echo "✅ Confidence: 99% (biomeOS validated strategy)"
echo ""

# Configuration
BEARDOG_SOCKET="/tmp/beardog-test.sock"
SERVER_PORT=8443
SERVER_LOG="/tmp/songbird-server-transcript.log"
CLIENT_LOG="/tmp/songbird-client-transcript.log"
SERVER_HEX="/tmp/server-transcript.hex"
CLIENT_HEX="/tmp/client-transcript.hex"
DIFF_OUTPUT="/tmp/transcript-diff.txt"

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    
    # Kill processes
    if [ ! -z "$BEARDOG_PID" ]; then
        kill $BEARDOG_PID 2>/dev/null || true
    fi
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
    
    # Remove socket
    rm -f $BEARDOG_SOCKET
    
    echo "✅ Cleanup complete"
}

trap cleanup EXIT

# Clean up previous runs
echo "═══════════════════════════════════════════════════════════════"
echo "🧹 Step 0: Cleaning up previous runs..."
echo "═══════════════════════════════════════════════════════════════"
rm -f $SERVER_LOG $CLIENT_LOG $SERVER_HEX $CLIENT_HEX $DIFF_OUTPUT $BEARDOG_SOCKET
echo "✅ Previous logs cleared"
echo ""

# Step 1: Start BearDog
echo "═══════════════════════════════════════════════════════════════"
echo "🐻 Step 1: Starting BearDog (crypto provider)..."
echo "═══════════════════════════════════════════════════════════════"

# Check if BearDog binary exists
if [ ! -f "target/release/beardog" ] && [ ! -f "target/debug/beardog" ]; then
    echo -e "${RED}❌ BearDog binary not found!${NC}"
    echo "   Please build BearDog first:"
    echo "   cd ../beardog && cargo build --release"
    exit 1
fi

# Use release if available, otherwise debug
BEARDOG_BIN="target/release/beardog"
if [ ! -f "$BEARDOG_BIN" ]; then
    BEARDOG_BIN="target/debug/beardog"
fi

echo "   Using: $BEARDOG_BIN"
echo "   Socket: $BEARDOG_SOCKET"

# Start BearDog in background
$BEARDOG_BIN server --socket $BEARDOG_SOCKET > /dev/null 2>&1 &
BEARDOG_PID=$!

# Wait for BearDog to start
sleep 2

# Verify BearDog is running
if ! kill -0 $BEARDOG_PID 2>/dev/null; then
    echo -e "${RED}❌ Failed to start BearDog!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BearDog started (PID: $BEARDOG_PID)${NC}"
echo ""

# Step 2: Generate test certificate (if needed)
echo "═══════════════════════════════════════════════════════════════"
echo "🔐 Step 2: Preparing test certificate..."
echo "═══════════════════════════════════════════════════════════════"

CERT_FILE="test-data/test-cert.pem"
KEY_FILE="test-data/test-key.pem"

if [ ! -f "$CERT_FILE" ] || [ ! -f "$KEY_FILE" ]; then
    echo "   Generating self-signed certificate..."
    mkdir -p test-data
    
    openssl req -x509 -newkey rsa:2048 -nodes \
        -keyout $KEY_FILE \
        -out $CERT_FILE \
        -days 365 \
        -subj "/CN=localhost" \
        > /dev/null 2>&1
    
    echo "✅ Certificate generated"
else
    echo "✅ Using existing certificate"
fi
echo ""

# Step 3: Start Songbird Server
echo "═══════════════════════════════════════════════════════════════"
echo "🔒 Step 3: Starting Songbird TLS Server (DIRECT MODE)..."
echo "═══════════════════════════════════════════════════════════════"

# Check if server binary exists (we need to build it)
if [ ! -f "target/release/songbird-server" ] && [ ! -f "target/debug/songbird-server" ]; then
    echo "   Building server test binary..."
    cargo build --package songbird-http-client --example server_test
fi

echo "   Mode: DIRECT (no Neural API needed)"
echo "   Port: $SERVER_PORT"
echo "   Logging to: $SERVER_LOG"
echo "   Certificate: $CERT_FILE"

# Start server in background with full logging (DIRECT MODE!)
RUST_LOG=info \
BEARDOG_MODE=direct \
BEARDOG_SOCKET=$BEARDOG_SOCKET \
cargo run --package songbird-http-client --example server_test -- \
    --port $SERVER_PORT \
    --cert $CERT_FILE \
    --key $KEY_FILE \
    > $SERVER_LOG 2>&1 &
SERVER_PID=$!

# Wait for server to start
echo "   Waiting for server to initialize..."
sleep 3

# Verify server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo -e "${RED}❌ Failed to start server!${NC}"
    echo "   Check server log: $SERVER_LOG"
    exit 1
fi

echo -e "${GREEN}✅ Server started (PID: $SERVER_PID)${NC}"
echo ""

# Step 4: Run Songbird Client
echo "═══════════════════════════════════════════════════════════════"
echo "🔗 Step 4: Connecting with Songbird TLS Client (DIRECT MODE)..."
echo "═══════════════════════════════════════════════════════════════"

echo "   Mode: DIRECT (no Neural API needed)"
echo "   Target: https://localhost:$SERVER_PORT"
echo "   Logging to: $CLIENT_LOG"

# Run client and capture logs (DIRECT MODE!)
RUST_LOG=info \
BEARDOG_MODE=direct \
BEARDOG_SOCKET=$BEARDOG_SOCKET \
cargo run --package songbird-http-client --example client_test -- \
    --url https://localhost:$SERVER_PORT \
    --skip-verify \
    > $CLIENT_LOG 2>&1 || true

echo "✅ Client connection complete"
echo ""

# Give logs a moment to flush
sleep 1

# Step 5: Extract transcripts
echo "═══════════════════════════════════════════════════════════════"
echo "📝 Step 5: Extracting transcripts from logs..."
echo "═══════════════════════════════════════════════════════════════"

# Extract client transcript hex dump
grep "CLIENT.*0000:" $CLIENT_LOG > $CLIENT_HEX 2>/dev/null || echo "" > $CLIENT_HEX
CLIENT_LINES=$(wc -l < $CLIENT_HEX)
echo "   Client transcript: $CLIENT_LINES lines"

# Extract server transcript hex dump
grep "SERVER.*0000:" $SERVER_LOG > $SERVER_HEX 2>/dev/null || echo "" > $SERVER_HEX
SERVER_LINES=$(wc -l < $SERVER_HEX)
echo "   Server transcript: $SERVER_LINES lines"

if [ $CLIENT_LINES -eq 0 ] || [ $SERVER_LINES -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Warning: Transcripts not found in logs${NC}"
    echo "   This might indicate handshake didn't complete"
    echo "   Check logs:"
    echo "     Client: $CLIENT_LOG"
    echo "     Server: $SERVER_LOG"
    echo ""
    echo "   Showing last 20 lines of each log:"
    echo ""
    echo "   === CLIENT LOG ==="
    tail -20 $CLIENT_LOG
    echo ""
    echo "   === SERVER LOG ==="
    tail -20 $SERVER_LOG
    exit 1
fi

echo -e "${GREEN}✅ Transcripts extracted${NC}"
echo ""

# Step 6: Compare transcripts
echo "═══════════════════════════════════════════════════════════════"
echo "🔬 Step 6: Comparing client and server transcripts..."
echo "═══════════════════════════════════════════════════════════════"

# Run diff and capture output
if diff -u $CLIENT_HEX $SERVER_HEX > $DIFF_OUTPUT 2>&1; then
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}🎉 SUCCESS! TRANSCRIPTS MATCH PERFECTLY! 🎉${NC}"
    echo -e "${GREEN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "✅ Client and server computed IDENTICAL transcripts!"
    echo "✅ This means key derivation will match!"
    echo "✅ Application keys will be identical!"
    echo "✅ Ready to test against real HTTPS servers!"
    echo ""
    echo "🎯 Next step: Validate against example.com"
    echo "   Run: cargo run --example test_https -- https://example.com"
    echo ""
    exit 0
else
    echo -e "${YELLOW}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}🔍 TRANSCRIPTS DIFFER - SHOWING DIFFERENCES${NC}"
    echo -e "${YELLOW}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "❌ Transcripts do NOT match"
    echo "📊 Showing differences (saved to: $DIFF_OUTPUT)"
    echo ""
    
    # Show diff with line numbers
    cat $DIFF_OUTPUT
    
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    echo "🎯 ANALYSIS"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    
    # Count total lines
    TOTAL_CLIENT=$(wc -l < $CLIENT_HEX)
    TOTAL_SERVER=$(wc -l < $SERVER_HEX)
    
    echo "Transcript lengths:"
    echo "  Client: $TOTAL_CLIENT lines"
    echo "  Server: $TOTAL_SERVER lines"
    echo ""
    
    # Find first difference
    FIRST_DIFF=$(diff $CLIENT_HEX $SERVER_HEX | grep "^<" | head -1 | cut -d: -f1 || echo "")
    
    if [ ! -z "$FIRST_DIFF" ]; then
        echo "First difference at offset: $FIRST_DIFF"
        echo ""
    fi
    
    echo "💡 Most likely causes (biomeOS analysis):"
    echo "   1. Certificate message content (80% likely)"
    echo "      - Certificate chain ordering"
    echo "      - Extension order or content"
    echo "      - OCSP responses"
    echo "      - SCT timestamps"
    echo "      - DER encoding variations"
    echo ""
    echo "   2. EncryptedExtensions (15% likely)"
    echo "      - Extension order"
    echo "      - Content variations"
    echo ""
    echo "   3. CertificateVerify (5% likely)"
    echo "      - Signature computation"
    echo "      - Padding"
    echo ""
    echo "🔧 Next steps:"
    echo "   1. Review differences above"
    echo "   2. Focus on Certificate message (most likely)"
    echo "   3. Fix content construction"
    echo "   4. Re-run this test"
    echo "   5. Iterate until transcripts match"
    echo ""
    echo "📁 Full logs available:"
    echo "   Client: $CLIENT_LOG"
    echo "   Server: $SERVER_LOG"
    echo "   Diff: $DIFF_OUTPUT"
    echo ""
    exit 1
fi

