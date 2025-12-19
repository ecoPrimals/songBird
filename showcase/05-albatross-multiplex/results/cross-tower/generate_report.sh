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
