#!/bin/bash
# Profile HTTP vs JSON-RPC to compare overhead

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

BENCHMARK_DIR="../benchmark"
PROFILE_DIR="."

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        🔥 PROFILING HTTP vs JSON-RPC OVERHEAD 🔥                ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Profile HTTP
echo -e "${BLUE}[1/3]${NC} Profiling HTTP baseline (5000 requests)..."
cd "$BENCHMARK_DIR"
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin bench-http -o "$PROFILE_DIR/flamegraph-http.svg" -- -n 5000 2>&1 | grep -v "Compiling" | tail -5
cd - > /dev/null

if [ -f flamegraph-http.svg ]; then
    echo -e "${GREEN}✅ HTTP flamegraph: flamegraph-http.svg${NC}"
    echo "   Size: $(du -h flamegraph-http.svg | cut -f1)"
else
    echo -e "${YELLOW}⚠️  HTTP flamegraph not created${NC}"
fi
echo ""

# Profile JSON-RPC
echo -e "${BLUE}[2/3]${NC} Profiling JSON-RPC (5000 requests)..."
cd "$BENCHMARK_DIR"
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin bench-jsonrpc -o "$PROFILE_DIR/flamegraph-jsonrpc.svg" -- -n 5000 2>&1 | grep -v "Compiling" | tail -5
cd - > /dev/null

if [ -f flamegraph-jsonrpc.svg ]; then
    echo -e "${GREEN}✅ JSON-RPC flamegraph: flamegraph-jsonrpc.svg${NC}"
    echo "   Size: $(du -h flamegraph-jsonrpc.svg | cut -f1)"
else
    echo -e "${YELLOW}⚠️  JSON-RPC flamegraph not created${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}[3/3]${NC} Generating comparison summary..."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "FLAMEGRAPHS GENERATED"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Files:"
ls -lh *.svg 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To view:"
echo "  firefox flamegraph-http.svg &"
echo "  firefox flamegraph-jsonrpc.svg &"
echo ""
echo "Or copy to local machine:"
echo "  scp $(hostname):$(pwd)/flamegraph-*.svg ."
echo ""

