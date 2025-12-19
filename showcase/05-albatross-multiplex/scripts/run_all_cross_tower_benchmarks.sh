#!/bin/bash
# Comprehensive Cross-Tower Benchmark Suite
# Tests all protocols between Eastgate and Strandgate

set -e

STRANDGATE_IP="192.168.1.134"
EASTGATE_IP="192.168.1.144"
BENCHMARK_DIR="../benchmark"
RESULTS_DIR="../results/cross-tower"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║     🌐 CROSS-TOWER BENCHMARK SUITE 🌐                           ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Eastgate (local):   $EASTGATE_IP"
echo "Strandgate (remote): $STRANDGATE_IP"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Pre-flight checks
echo -e "${BLUE}[0/7]${NC} Running pre-flight checks..."
echo ""

# Check Strandgate reachability
if ! ping -c 3 "$STRANDGATE_IP" > /dev/null 2>&1; then
    echo -e "${RED}❌ Cannot reach Strandgate ($STRANDGATE_IP)${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Strandgate reachable"

# Measure baseline latency
LATENCY=$(ping -c 10 "$STRANDGATE_IP" | grep avg | awk -F'/' '{print $5}')
echo -e "${GREEN}✓${NC} Network latency: ${LATENCY}ms"

# Check Songbird on Strandgate
if curl -k -s --max-time 2 "https://$STRANDGATE_IP:8081/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Songbird running on Strandgate"
else
    echo -e "${YELLOW}⚠${NC}  Songbird not responding on Strandgate:8081"
fi

# Check tarpc servers on Strandgate
TARPC_ONLINE=0
for port in 8091 8092 8093; do
    if nc -z -w2 "$STRANDGATE_IP" $port 2>/dev/null; then
        ((TARPC_ONLINE++))
    fi
done

if [ $TARPC_ONLINE -eq 0 ]; then
    echo -e "${YELLOW}⚠${NC}  No tarpc servers found on Strandgate"
    echo "   Run: ./deploy_tarpc_to_strandgate.sh"
else
    echo -e "${GREEN}✓${NC} tarpc servers: $TARPC_ONLINE/3 online"
fi

echo ""
read -p "Continue with benchmarks? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi
echo ""

# Benchmark 1: HTTP Single Connection
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  [1/7] HTTP Single Connection: Eastgate → Strandgate            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

cd "$BENCHMARK_DIR"
cargo run --release --bin bench-http -- \
    -t "https://$STRANDGATE_IP:8081" \
    -n 5000 \
    -w 50 \
    2>&1 | tee "$RESULTS_DIR/http_cross_tower.log" | grep -v "Compiling" | tail -30

mv results_http.json "$RESULTS_DIR/results_http_cross_tower.json"
echo ""

# Benchmark 2: JSON-RPC Single Connection
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  [2/7] JSON-RPC Single: Eastgate → Strandgate                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

cargo run --release --bin bench-jsonrpc -- \
    -t "https://$STRANDGATE_IP:8081" \
    -n 5000 \
    -w 50 \
    2>&1 | tee "$RESULTS_DIR/jsonrpc_cross_tower.log" | grep -v "Compiling" | tail -30

mv results_jsonrpc.json "$RESULTS_DIR/results_jsonrpc_cross_tower.json"
echo ""

# Benchmark 3: tarpc Single Connection
if [ $TARPC_ONLINE -gt 0 ]; then
    echo "╔══════════════════════════════════════════════════════════════════╗"
    echo "║  [3/7] tarpc Single: Eastgate → Strandgate                      ║"
    echo "╚══════════════════════════════════════════════════════════════════╝"
    echo ""

    cargo run --release --bin bench-tarpc-single -- \
        -t "$STRANDGATE_IP:8091" \
        -n 5000 \
        -w 50 \
        2>&1 | tee "$RESULTS_DIR/tarpc_single_cross_tower.log" | grep -v "Compiling" | tail -30

    mv results_tarpc_single.json "$RESULTS_DIR/results_tarpc_single_cross_tower.json"
    echo ""
else
    echo -e "${YELLOW}⚠${NC}  Skipping tarpc single (no servers)"
    echo ""
fi

# Benchmark 4: tarpc Multiplex
if [ $TARPC_ONLINE -eq 3 ]; then
    echo "╔══════════════════════════════════════════════════════════════════╗"
    echo "║  [4/7] tarpc Multiplex: Eastgate → Strandgate (30 conn)         ║"
    echo "╚══════════════════════════════════════════════════════════════════╝"
    echo ""

    cargo run --release --bin bench-tarpc-multiplex -- \
        -t "$STRANDGATE_IP:8091,$STRANDGATE_IP:8092,$STRANDGATE_IP:8093" \
        -c 10 \
        -n 5000 \
        -w 50 \
        2>&1 | tee "$RESULTS_DIR/tarpc_multiplex_cross_tower.log" | grep -v "Compiling" | tail -40

    mv results_tarpc_multiplex.json "$RESULTS_DIR/results_tarpc_multiplex_cross_tower.json"
    echo ""
else
    echo -e "${YELLOW}⚠${NC}  Skipping tarpc multiplex (need 3 servers)"
    echo ""
fi

# Benchmark 5: Bidirectional Test (if we have tarpc on both sides)
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  [5/7] Bidirectional Test (requires setup on Strandgate)        ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${YELLOW}ℹ${NC}  This requires tarpc servers running on both towers"
echo -e "${YELLOW}ℹ${NC}  Run manually if desired (see CROSS_TOWER_BENCHMARKS.md)"
echo ""

# Benchmark 6: Real Orchestration (Songbird → Toadstool)
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  [6/7] Real Orchestration: Songbird → Toadstool on Strandgate   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

echo "Testing Toadstool on Strandgate..."
if curl -s --max-time 2 "http://$STRANDGATE_IP:7878/health" | grep -q "ok"; then
    echo -e "${GREEN}✓${NC} Toadstool responding"
    
    # Simple orchestration test
    echo "Running 10 orchestrated tasks..."
    for i in {1..10}; do
        START=$(date +%s%N)
        RESULT=$(curl -s -X POST "http://$STRANDGATE_IP:7878/compute/echo" -d "test $i" 2>/dev/null)
        END=$(date +%s%N)
        DURATION=$(( ($END - $START) / 1000000 ))
        echo "  Task $i: ${DURATION}ms"
    done
else
    echo -e "${YELLOW}⚠${NC}  Toadstool not responding on Strandgate"
fi
echo ""

# Benchmark 7: Latency Distribution Analysis
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  [7/7] Network Latency Distribution Analysis                    ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

echo "Measuring network latency distribution (100 samples)..."
ping -c 100 "$STRANDGATE_IP" | tail -5 > "$RESULTS_DIR/network_latency.txt"
cat "$RESULTS_DIR/network_latency.txt"
echo ""

cd - > /dev/null

# Generate comparison report
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              📊 GENERATING COMPARISON REPORT 📊                  ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

cat << 'EOF' > "$RESULTS_DIR/generate_report.sh"
#!/bin/bash
# Extract and compare results

echo "Cross-Tower Benchmark Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "⚠️  jq not found, showing raw results"
    ls -lh *.json
    exit 0
fi

echo "Protocol Performance (Localhost vs Network):"
echo ""
printf "%-15s %-15s %-15s %-15s\n" "Protocol" "Localhost" "Network" "Ratio"
echo "───────────────────────────────────────────────────────────────"

# HTTP
if [ -f "../benchmark/results_http.json" ] && [ -f "results_http_cross_tower.json" ]; then
    LOCAL=$(jq -r '.requests_per_second' ../benchmark/results_http.json 2>/dev/null | cut -d. -f1)
    NETWORK=$(jq -r '.requests_per_second' results_http_cross_tower.json 2>/dev/null | cut -d. -f1)
    if [ -n "$LOCAL" ] && [ -n "$NETWORK" ]; then
        RATIO=$(echo "scale=2; $LOCAL / $NETWORK" | bc)
        printf "%-15s %-15s %-15s %-15s\n" "HTTP" "${LOCAL}" "${NETWORK}" "${RATIO}x"
    fi
fi

# JSON-RPC
if [ -f "../benchmark/results_jsonrpc.json" ] && [ -f "results_jsonrpc_cross_tower.json" ]; then
    LOCAL=$(jq -r '.requests_per_second' ../benchmark/results_jsonrpc.json 2>/dev/null | cut -d. -f1)
    NETWORK=$(jq -r '.requests_per_second' results_jsonrpc_cross_tower.json 2>/dev/null | cut -d. -f1)
    if [ -n "$LOCAL" ] && [ -n "$NETWORK" ]; then
        RATIO=$(echo "scale=2; $LOCAL / $NETWORK" | bc)
        printf "%-15s %-15s %-15s %-15s\n" "JSON-RPC" "${LOCAL}" "${NETWORK}" "${RATIO}x"
    fi
fi

# tarpc single
if [ -f "../benchmark/results_tarpc_single.json" ] && [ -f "results_tarpc_single_cross_tower.json" ]; then
    LOCAL=$(jq -r '.requests_per_second' ../benchmark/results_tarpc_single.json 2>/dev/null | cut -d. -f1)
    NETWORK=$(jq -r '.requests_per_second' results_tarpc_single_cross_tower.json 2>/dev/null | cut -d. -f1)
    if [ -n "$LOCAL" ] && [ -n "$NETWORK" ]; then
        RATIO=$(echo "scale=2; $LOCAL / $NETWORK" | bc)
        printf "%-15s %-15s %-15s %-15s\n" "tarpc (single)" "${LOCAL}" "${NETWORK}" "${RATIO}x"
    fi
fi

# tarpc multiplex
if [ -f "../benchmark/results_tarpc_multiplex.json" ] && [ -f "results_tarpc_multiplex_cross_tower.json" ]; then
    LOCAL=$(jq -r '.requests_per_second' ../benchmark/results_tarpc_multiplex.json 2>/dev/null | cut -d. -f1)
    NETWORK=$(jq -r '.requests_per_second' results_tarpc_multiplex_cross_tower.json 2>/dev/null | cut -d. -f1)
    if [ -n "$LOCAL" ] && [ -n "$NETWORK" ]; then
        RATIO=$(echo "scale=2; $LOCAL / $NETWORK" | bc)
        printf "%-15s %-15s %-15s %-15s\n" "tarpc (30x)" "${LOCAL}" "${NETWORK}" "${RATIO}x"
    fi
fi

echo ""
echo "Multiplex Scaling Efficiency:"
echo ""

# Compare multiplex efficiency
if [ -f "results_tarpc_multiplex_cross_tower.json" ]; then
    TOTAL=$(jq -r '.requests_per_second' results_tarpc_multiplex_cross_tower.json 2>/dev/null | cut -d. -f1)
    SINGLE=$(jq -r '.requests_per_second' results_tarpc_single_cross_tower.json 2>/dev/null | cut -d. -f1)
    if [ -n "$TOTAL" ] && [ -n "$SINGLE" ] && [ "$SINGLE" != "0" ]; then
        PER_CONN=$(echo "scale=0; $TOTAL / 30" | bc)
        EFF=$(echo "scale=1; ($TOTAL / $SINGLE) / 30 * 100" | bc)
        echo "  Network (30 conn):  ${TOTAL} req/s total, ${PER_CONN} req/s per-conn (${EFF}% efficiency)"
    fi
fi

if [ -f "../benchmark/results_tarpc_multiplex.json" ]; then
    TOTAL=$(jq -r '.requests_per_second' ../benchmark/results_tarpc_multiplex.json 2>/dev/null | cut -d. -f1)
    SINGLE=$(jq -r '.requests_per_second' ../benchmark/results_tarpc_single.json 2>/dev/null | cut -d. -f1)
    if [ -n "$TOTAL" ] && [ -n "$SINGLE" ] && [ "$SINGLE" != "0" ]; then
        PER_CONN=$(echo "scale=0; $TOTAL / 30" | bc)
        EFF=$(echo "scale=1; ($TOTAL / $SINGLE) / 30 * 100" | bc)
        echo "  Localhost (30 conn): ${TOTAL} req/s total, ${PER_CONN} req/s per-conn (${EFF}% efficiency)"
    fi
fi

echo ""
echo "Key Insight:"
if [ -f "results_tarpc_multiplex_cross_tower.json" ] && [ -f "../benchmark/results_tarpc_multiplex.json" ]; then
    NET_EFF=$(jq -r '.requests_per_second' results_tarpc_multiplex_cross_tower.json 2>/dev/null)
    LOC_EFF=$(jq -r '.requests_per_second' ../benchmark/results_tarpc_multiplex.json 2>/dev/null)
    if (( $(echo "$NET_EFF > $LOC_EFF" | bc -l) )); then
        echo "  ✅ Network multiplex is BETTER than localhost (as expected!)"
        echo "     Network I/O allows better CPU utilization during waits."
    else
        echo "  ⚠️  Network multiplex is not better than localhost"
        echo "     May indicate network bottleneck or need for optimization."
    fi
fi

echo ""
EOF

chmod +x "$RESULTS_DIR/generate_report.sh"
cd "$RESULTS_DIR"
./generate_report.sh
cd - > /dev/null

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              ✨ BENCHMARKS COMPLETE ✨                            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Results saved to: $RESULTS_DIR/"
echo ""
echo "Files:"
ls -lh "$RESULTS_DIR"/*.json 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To view full report:"
echo "  cd $RESULTS_DIR && cat *.log"
echo ""
echo "🌐 Cross-tower benchmarking complete!"
echo ""

