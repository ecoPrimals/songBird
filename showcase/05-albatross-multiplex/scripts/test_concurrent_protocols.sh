#!/bin/bash
# Test concurrent multi-protocol usage
# Runs HTTP and tarpc simultaneously to measure interference

set -e

STRANDGATE="192.168.1.134"
RESULTS_DIR="../results/concurrent"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║        🔀 CONCURRENT MULTI-PROTOCOL TEST 🔀                     ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Testing: Can HTTP and tarpc run concurrently without interference?"
echo ""

mkdir -p "$RESULTS_DIR"

# Test 1: HTTP alone (baseline)
echo -e "${BLUE}[1/5]${NC} Baseline: HTTP alone (1000 requests)..."
cd ../benchmark
cargo run --release --bin bench-http -- \
    -t "https://$STRANDGATE:8081" \
    -n 1000 \
    -w 20 \
    2>&1 | grep "Requests/Second" | tee "$RESULTS_DIR/http_alone.txt"
HTTP_ALONE=$(grep -oP '\d+' "$RESULTS_DIR/http_alone.txt" | head -1)
cd - > /dev/null
echo ""

# Test 2: tarpc alone (baseline)
echo -e "${BLUE}[2/5]${NC} Baseline: tarpc alone (1000 requests)..."
cd ../benchmark
cargo run --release --bin bench-tarpc-single -- \
    -t "$STRANDGATE:8091" \
    -n 1000 \
    -w 20 \
    2>&1 | grep "Requests/Second" | tee "$RESULTS_DIR/tarpc_alone.txt"
TARPC_ALONE=$(grep -oP '\d+' "$RESULTS_DIR/tarpc_alone.txt" | head -1)
cd - > /dev/null
echo ""

# Test 3: Both concurrent
echo -e "${BLUE}[3/5]${NC} Concurrent: HTTP + tarpc simultaneously..."
echo "Starting both benchmarks..."

cd ../benchmark

# Start tarpc in background
cargo run --release --bin bench-tarpc-single -- \
    -t "$STRANDGATE:8091" \
    -n 1000 \
    -w 20 \
    2>&1 | grep "Requests/Second" > "$RESULTS_DIR/tarpc_concurrent.txt" &
TARPC_PID=$!

# Start HTTP in background
cargo run --release --bin bench-http -- \
    -t "https://$STRANDGATE:8081" \
    -n 1000 \
    -w 20 \
    2>&1 | grep "Requests/Second" > "$RESULTS_DIR/http_concurrent.txt" &
HTTP_PID=$!

# Wait for both to complete
wait $TARPC_PID
wait $HTTP_PID

HTTP_CONCURRENT=$(grep -oP '\d+' "$RESULTS_DIR/http_concurrent.txt" | head -1)
TARPC_CONCURRENT=$(grep -oP '\d+' "$RESULTS_DIR/tarpc_concurrent.txt" | head -1)

cd - > /dev/null
echo ""

# Analysis
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              📊 CONCURRENT PROTOCOL RESULTS 📊                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

printf "%-20s %-15s %-15s %-15s\n" "Protocol" "Alone" "Concurrent" "Interference"
echo "────────────────────────────────────────────────────────────────"

# HTTP analysis
if [ -n "$HTTP_ALONE" ] && [ -n "$HTTP_CONCURRENT" ]; then
    HTTP_RATIO=$(echo "scale=2; 100 - ($HTTP_CONCURRENT * 100 / $HTTP_ALONE)" | bc)
    printf "%-20s %-15s %-15s %-15s\n" "HTTP" "${HTTP_ALONE} req/s" "${HTTP_CONCURRENT} req/s" "${HTTP_RATIO}%"
fi

# tarpc analysis
if [ -n "$TARPC_ALONE" ] && [ -n "$TARPC_CONCURRENT" ]; then
    TARPC_RATIO=$(echo "scale=2; 100 - ($TARPC_CONCURRENT * 100 / $TARPC_ALONE)" | bc)
    printf "%-20s %-15s %-15s %-15s\n" "tarpc" "${TARPC_ALONE} req/s" "${TARPC_CONCURRENT} req/s" "${TARPC_RATIO}%"
fi

echo ""
echo "Summary:"

# Determine result
AVG_INTERFERENCE=$(echo "scale=1; ($HTTP_RATIO + $TARPC_RATIO) / 2" | bc)

if (( $(echo "$AVG_INTERFERENCE < 10" | bc -l) )); then
    echo -e "${GREEN}✅ Minimal interference (<10%)${NC}"
    echo "   Protocols can run concurrently without significant impact!"
elif (( $(echo "$AVG_INTERFERENCE < 20" | bc -l) )); then
    echo -e "${YELLOW}⚠️  Moderate interference (10-20%)${NC}"
    echo "   Some impact but still usable concurrently"
else
    echo -e "${YELLOW}⚠️  Significant interference (>20%)${NC}"
    echo "   May want to optimize or sequence operations"
fi

echo ""
echo "Interpretation:"
echo "  • <10% interference: Excellent concurrent operation"
echo "  • 10-20% interference: Good, expected on 1Gb NIC"
echo "  • >20% interference: May indicate network saturation"
echo ""

# Save summary
cat > "$RESULTS_DIR/summary.txt" << EOF
Concurrent Multi-Protocol Test Results
=======================================

HTTP:
  Alone: ${HTTP_ALONE} req/s
  Concurrent: ${HTTP_CONCURRENT} req/s
  Interference: ${HTTP_RATIO}%

tarpc:
  Alone: ${TARPC_ALONE} req/s
  Concurrent: ${TARPC_CONCURRENT} req/s
  Interference: ${TARPC_RATIO}%

Average Interference: ${AVG_INTERFERENCE}%

Conclusion: $(if (( $(echo "$AVG_INTERFERENCE < 10" | bc -l) )); then echo "Concurrent operation works well"; elif (( $(echo "$AVG_INTERFERENCE < 20" | bc -l) )); then echo "Moderate interference, still usable"; else echo "Significant interference"; fi)
EOF

echo "Results saved to: $RESULTS_DIR/summary.txt"
echo ""
echo "🔀 Concurrent protocol test complete!"
echo ""

