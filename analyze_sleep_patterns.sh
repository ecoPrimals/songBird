#!/usr/bin/env bash
# Script to systematically modernize tests - eliminate sleep() calls
# and replace with deterministic time control patterns

set -euo pipefail

echo "🚀 MODERNIZATION: Eliminating sleep() calls and modernizing tests"
echo "================================================================"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Count total sleep calls
TOTAL_SLEEPS=$(rg 'tokio::time::sleep|std::thread::sleep' --type rust | wc -l)
echo -e "${YELLOW}Total sleep() calls found: ${TOTAL_SLEEPS}${NC}"

# Analyze by category
echo ""
echo "📊 Analysis by Category:"
echo "------------------------"

CIRCUIT_BREAKER=$(rg 'tokio::time::sleep' crates/songbird-universal/tests/circuit_breaker*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "Circuit Breaker tests: ${CIRCUIT_BREAKER} sleeps"

INTEGRATION=$(rg 'tokio::time::sleep' crates/*/tests/*integration*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "Integration tests: ${INTEGRATION} sleeps"

E2E=$(rg 'tokio::time::sleep' tests/e2e/*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "E2E tests: ${E2E} sleeps"

CHAOS=$(rg 'tokio::time::sleep' tests/chaos/*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "Chaos tests: ${CHAOS} sleeps ${GREEN}(acceptable)${NC}"

EXAMPLES=$(rg 'tokio::time::sleep' examples/*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "Examples: ${EXAMPLES} sleeps ${YELLOW}(demo code)${NC}"

PROD_CODE=$(rg 'tokio::time::sleep' crates/*/src/**/*.rs --type rust 2>/dev/null | wc -l || echo "0")
echo -e "${RED}Production code: ${PROD_CODE} sleeps ⚠️${NC}"

echo ""
echo "🎯 Modernization Strategy:"
echo "-------------------------"
echo "1. Circuit breaker tests: Replace with tokio::time::pause/advance"
echo "2. Integration tests: Replace with proper sync primitives"
echo "3. E2E tests: Replace with polling/waiting patterns"
echo "4. Chaos tests: Keep (intentional timing simulation)"
echo "5. Production code: Audit for legitimate vs testable delays"

echo ""
echo "📝 Modern Pattern Example:"
echo "-------------------------"
cat << 'EOF'
// ❌ OLD (slow, non-deterministic)
#[tokio::test]
async fn test_timeout() {
    start_operation();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(is_complete());
}

// ✅ NEW (instant, deterministic)
#[tokio::test]
async fn test_timeout() {
    tokio::time::pause();
    start_operation();
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::time::resume();
    assert!(is_complete());
}
EOF

echo ""
echo -e "${GREEN}✅ Analysis complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Run test suite to identify breaking tests"
echo "2. Apply modern patterns systematically"
echo "3. Verify all tests pass concurrently"

# Create a report file
REPORT_FILE="MODERNIZATION_SLEEP_AUDIT_$(date +%Y%m%d_%H%M%S).txt"
{
    echo "SLEEP AUDIT REPORT"
    echo "=================="
    echo "Date: $(date)"
    echo ""
    echo "Total sleeps: ${TOTAL_SLEEPS}"
    echo "Circuit breaker: ${CIRCUIT_BREAKER}"
    echo "Integration: ${INTEGRATION}"
    echo "E2E: ${E2E}"
    echo "Chaos: ${CHAOS}"
    echo "Examples: ${EXAMPLES}"
    echo "Production: ${PROD_CODE}"
    echo ""
    echo "DETAILED LOCATIONS:"
    echo "==================="
    rg 'tokio::time::sleep|std::thread::sleep' --type rust -n || true
} > "${REPORT_FILE}"

echo -e "${GREEN}Report saved to: ${REPORT_FILE}${NC}"

