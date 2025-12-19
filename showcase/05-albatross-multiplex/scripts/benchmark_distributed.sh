#!/bin/bash
# Run distributed benchmarks: Eastgate → Strandgate
# Tests tarpc performance over LAN to compare with localhost

set -e

STRANDGATE_TARGETS="192.168.1.134:8091,192.168.1.134:8092,192.168.1.134:8093"
BENCHMARK_DIR="../benchmark"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        📡 DISTRIBUTED BENCHMARK: EASTGATE → STRANDGATE 📡       ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Network: LAN (192.168.1.0/24)"
echo "Source: Eastgate (192.168.1.144)"
echo "Target: Strandgate (192.168.1.134)"
echo ""

# Verify Strandgate is reachable
echo -e "${BLUE}[0/3]${NC} Verifying Strandgate connectivity..."
if ! ping -c 1 192.168.1.134 > /dev/null 2>&1; then
    echo -e "${RED}❌ Cannot reach Strandgate (192.168.1.134)${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Strandgate reachable${NC}"
echo ""

# Test tarpc ports
echo -e "${BLUE}[1/3]${NC} Checking tarpc servers..."
for port in 8091 8092 8093; do
    if nc -z -w2 192.168.1.134 $port 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Port $port: Open"
    else
        echo -e "  ${YELLOW}⚠${NC}  Port $port: Not responding (may need deployment)"
    fi
done
echo ""

# Run tarpc single connection benchmark
echo -e "${BLUE}[2/3]${NC} Running tarpc single connection (5000 requests)..."
echo "════════════════════════════════════════════════════════════════════"
cd "$BENCHMARK_DIR"
cargo run --release --bin bench-tarpc-single -- \
    -t 192.168.1.134:8091 \
    -n 5000 \
    -w 50 \
    2>&1 | grep -v "Compiling" | tail -30

mv results_tarpc_single.json results_tarpc_single_distributed.json
cd - > /dev/null
echo ""

# Run tarpc multiplex benchmark
echo -e "${BLUE}[3/3]${NC} Running tarpc multiplex (5000 requests, 30 connections)..."
echo "════════════════════════════════════════════════════════════════════"
cd "$BENCHMARK_DIR"
cargo run --release --bin bench-tarpc-multiplex -- \
    -t "$STRANDGATE_TARGETS" \
    -c 10 \
    -n 5000 \
    -w 50 \
    2>&1 | grep -v "Compiling" | tail -40

mv results_tarpc_multiplex.json results_tarpc_multiplex_distributed.json
cd - > /dev/null
echo ""

# Generate comparison report
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              📊 LOCALHOST vs NETWORK COMPARISON 📊              ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Extract results from JSON files
echo "Results will be in:"
echo "  • $BENCHMARK_DIR/results_tarpc_single_distributed.json"
echo "  • $BENCHMARK_DIR/results_tarpc_multiplex_distributed.json"
echo ""

# Create comparison script
cat << 'EOF' > /tmp/compare_distributed.sh
#!/bin/bash
# Compare localhost vs distributed results

LOCAL_SINGLE=$(jq -r '.requests_per_second' benchmark/results_tarpc_single.json 2>/dev/null || echo "0")
DIST_SINGLE=$(jq -r '.requests_per_second' benchmark/results_tarpc_single_distributed.json 2>/dev/null || echo "0")

LOCAL_MULTI=$(jq -r '.requests_per_second' benchmark/results_tarpc_multiplex.json 2>/dev/null || echo "0")
DIST_MULTI=$(jq -r '.requests_per_second' benchmark/results_tarpc_multiplex_distributed.json 2>/dev/null || echo "0")

echo "Comparison: Localhost vs Network"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
printf "%-20s %-15s %-15s %s\n" "Configuration" "Localhost" "Network" "Ratio"
echo "────────────────────────────────────────────────────────────────"
printf "%-20s %-15s %-15s %.2fx\n" "Single connection" "${LOCAL_SINGLE%.*}" "${DIST_SINGLE%.*}" "$(echo "scale=2; $LOCAL_SINGLE / $DIST_SINGLE" | bc)"
printf "%-20s %-15s %-15s %.2fx\n" "30 connections" "${LOCAL_MULTI%.*}" "${DIST_MULTI%.*}" "$(echo "scale=2; $LOCAL_MULTI / $DIST_MULTI" | bc)"
echo ""

# Scaling efficiency
LOCAL_EFF=$(echo "scale=2; ($LOCAL_MULTI / $LOCAL_SINGLE) / 30 * 100" | bc)
DIST_EFF=$(echo "scale=2; ($DIST_MULTI / $DIST_SINGLE) / 30 * 100" | bc)

echo "Scaling Efficiency (30 connections):"
printf "  Localhost: %.1f%%\n" "$LOCAL_EFF"
printf "  Network:   %.1f%%\n" "$DIST_EFF"
echo ""

if (( $(echo "$DIST_EFF > $LOCAL_EFF" | bc -l) )); then
    echo "✅ Network scaling is BETTER than localhost (as expected!)"
else
    echo "⚠️  Network scaling is not better (investigate)"
fi
EOF

chmod +x /tmp/compare_distributed.sh
/tmp/compare_distributed.sh

echo ""
echo "🎯 Distributed benchmarking complete!"
echo ""

