#!/bin/bash
# Track Songbird Production Readiness Progress

set -e

echo "🔍 SONGBIRD PRODUCTION READINESS METRICS"
echo "========================================"
echo ""
echo "📅 Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Test Coverage
echo "📊 TEST COVERAGE:"
if command -v cargo-tarpaulin &> /dev/null; then
    echo "   Running coverage analysis..."
    cargo tarpaulin --timeout 300 --out Json 2>/dev/null | jq -r '.coverage' 2>/dev/null || echo "   ⚠️  Run: cargo tarpaulin --out Html"
else
    echo "   ⚠️  Install tarpaulin: cargo install cargo-tarpaulin"
fi
echo ""

# Hardcoding Count
echo "🔧 HARDCODING INSTANCES:"
PORTS=$(grep -r "8080\|8081\|3000\|9090\|5000" crates/*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l)
LOCALHOST=$(grep -r "localhost\|127\.0\.0\.1" crates/*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l)
ENDPOINTS=$(grep -r "http://localhost" crates/*/src --include="*.rs" 2>/dev/null | wc -l)
PRIMAL_NAMES=$(grep -rE "beardog|nestgate|toadstool|squirrel" crates/*/src --include="*.rs" 2>/dev/null | grep -vi "mock\|test" | wc -l)

TOTAL_HARDCODING=$((PORTS + LOCALHOST + ENDPOINTS + PRIMAL_NAMES))

echo "   Ports (8080, etc):        $PORTS"
echo "   Localhost/IPs:            $LOCALHOST"
echo "   Hardcoded endpoints:      $ENDPOINTS"
echo "   Primal names:             $PRIMAL_NAMES"
echo "   ---"
echo "   TOTAL:                    $TOTAL_HARDCODING"
echo "   TARGET:                   <50"
echo ""

# Unwrap/Expect Count
echo "⚠️  UNWRAP/EXPECT CALLS:"
UNWRAPS=$(grep -r "\.unwrap()" crates/*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l)
EXPECTS=$(grep -r "\.expect" crates/*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l)
TOTAL_ERROR=$((UNWRAPS + EXPECTS))

echo "   Production unwraps:       $UNWRAPS"
echo "   Production expects:       $EXPECTS"
echo "   ---"
echo "   TOTAL:                    $TOTAL_ERROR"
echo "   TARGET:                   <100"
echo ""

# File Size Compliance
echo "📏 FILE SIZE COMPLIANCE:"
OVERSIZED=$(find crates -name "*.rs" -exec wc -l {} \; 2>/dev/null | awk '$1 > 1000 {count++} END {print count+0}')
TOTAL_FILES=$(find crates -name "*.rs" 2>/dev/null | wc -l)

echo "   Files over 1000 lines:    $OVERSIZED"
echo "   Total files:              $TOTAL_FILES"
echo "   Compliance:               $(awk "BEGIN {printf \"%.1f%%\", (($TOTAL_FILES - $OVERSIZED) / $TOTAL_FILES) * 100}")"
echo ""

# Build Status
echo "🏗️  BUILD STATUS:"
if cargo build --all-features 2>&1 | grep -q "Finished"; then
    echo "   ✅ Build: SUCCESS"
else
    echo "   ❌ Build: FAILED"
fi
echo ""

# Test Status
echo "🧪 TEST STATUS:"
TEST_OUTPUT=$(cargo test --workspace 2>&1 | grep "test result:")
if echo "$TEST_OUTPUT" | grep -q "ok"; then
    PASSED=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= passed)' | head -1)
    FAILED=$(echo "$TEST_OUTPUT" | grep -oP '\d+(?= failed)' | head -1)
    echo "   Passed:                   ${PASSED:-0}"
    echo "   Failed:                   ${FAILED:-0}"
else
    echo "   ⚠️  Run: cargo test --workspace"
fi
echo ""

# Production Readiness Score
echo "🎯 PRODUCTION READINESS:"
COVERAGE_SCORE=0  # Will be calculated from tarpaulin
HARDCODING_SCORE=$((100 - (TOTAL_HARDCODING * 100 / 650)))  # Out of 650 original
ERROR_HANDLING_SCORE=$((100 - (TOTAL_ERROR * 100 / 1000)))  # Out of 1000 original
FILE_SIZE_SCORE=$((($TOTAL_FILES - $OVERSIZED) * 100 / $TOTAL_FILES))

AVG_SCORE=$(((HARDCODING_SCORE + ERROR_HANDLING_SCORE + FILE_SIZE_SCORE) / 3))

echo "   Hardcoding:               ${HARDCODING_SCORE}%"
echo "   Error Handling:           ${ERROR_HANDLING_SCORE}%"
echo "   File Size:                ${FILE_SIZE_SCORE}%"
echo "   ---"
echo "   OVERALL:                  ${AVG_SCORE}% (excl. coverage)"
echo ""

# Production Ready?
echo "✅ PRODUCTION READY?"
if [ $TOTAL_HARDCODING -lt 50 ] && [ $TOTAL_ERROR -lt 100 ] && [ $OVERSIZED -eq 0 ]; then
    echo "   🎉 YES - Ready for production!"
else
    echo "   ⚠️  NOT YET - Blockers remaining:"
    [ $TOTAL_HARDCODING -ge 50 ] && echo "      - Hardcoding: $TOTAL_HARDCODING (need <50)"
    [ $TOTAL_ERROR -ge 100 ] && echo "      - Unwrap/expect: $TOTAL_ERROR (need <100)"
    [ $OVERSIZED -gt 0 ] && echo "      - Oversized files: $OVERSIZED (need 0)"
    echo "      - Test coverage: Check with cargo tarpaulin (need 90%)"
fi
echo ""

echo "📋 NEXT ACTIONS:"
echo "   1. Fix remaining hardcoding: $(crates/songbird-config/src/config/constants.rs)"
echo "   2. Migrate unwraps: Focus on songbird-config, songbird-types"
echo "   3. Add tests: Target songbird-types (0% coverage)"
echo "   4. Check coverage: cargo tarpaulin --out Html"
echo ""
echo "========================================"

