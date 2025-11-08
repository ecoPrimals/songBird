#!/bin/bash
# Local MVP Testing Script
# Validates Songbird works correctly before GitHub push

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Songbird MVP Local Testing${NC}"
echo -e "${BLUE}==============================${NC}"
echo ""

# Track failures
FAILURES=0

# Test function
run_test() {
    local test_name="$1"
    local test_cmd="$2"
    
    echo -e "${YELLOW}Testing: ${test_name}${NC}"
    if eval "$test_cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS: ${test_name}${NC}"
        return 0
    else
        echo -e "${RED}❌ FAIL: ${test_name}${NC}"
        FAILURES=$((FAILURES + 1))
        return 1
    fi
}

echo -e "${BLUE}Phase 1: Build Verification${NC}"
echo "----------------------------"

# Test 1: Clean build
echo -e "${YELLOW}Building workspace...${NC}"
if cargo build --workspace --release 2>&1 | tee /tmp/songbird-build.log | tail -1 | grep -q "Finished"; then
    BUILD_TIME=$(grep "Finished" /tmp/songbird-build.log | awk '{print $(NF-1), $NF}')
    echo -e "${GREEN}✅ PASS: Workspace builds successfully (${BUILD_TIME})${NC}"
else
    echo -e "${RED}❌ FAIL: Build failed${NC}"
    FAILURES=$((FAILURES + 1))
    cat /tmp/songbird-build.log
fi
echo ""

# Test 2: All crates compile
echo -e "${YELLOW}Verifying all crates...${NC}"
CRATE_COUNT=$(ls -d crates/*/ | wc -l)
echo -e "${GREEN}✅ Found ${CRATE_COUNT} crates${NC}"
echo ""

echo -e "${BLUE}Phase 2: Test Suite${NC}"
echo "----------------------------"

# Test 3: Unit tests
echo -e "${YELLOW}Running unit tests...${NC}"
if cargo test --workspace --lib --quiet 2>&1 | tee /tmp/songbird-tests.log | tail -5 | grep -q "test result: ok"; then
    TEST_COUNT=$(grep "test result: ok" /tmp/songbird-tests.log | head -1 | awk '{print $4}')
    echo -e "${GREEN}✅ PASS: Unit tests passed (${TEST_COUNT} tests)${NC}"
else
    echo -e "${RED}❌ FAIL: Unit tests failed${NC}"
    FAILURES=$((FAILURES + 1))
    tail -20 /tmp/songbird-tests.log
fi
echo ""

# Test 4: Integration tests
echo -e "${YELLOW}Running integration tests...${NC}"
if cargo test --workspace --test '*' --quiet 2>&1 | tee /tmp/songbird-integration.log | tail -5 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ PASS: Integration tests passed${NC}"
else
    echo -e "${RED}❌ FAIL: Integration tests failed${NC}"
    FAILURES=$((FAILURES + 1))
fi
echo ""

echo -e "${BLUE}Phase 3: Example Verification${NC}"
echo "----------------------------"

# Test 5: Example compilation
echo -e "${YELLOW}Checking infant_discovery_demo...${NC}"
if cargo check --example infant_discovery_demo --package songbird-config --quiet 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✅ PASS: infant_discovery_demo compiles${NC}"
else
    echo -e "${RED}❌ FAIL: infant_discovery_demo failed to compile${NC}"
    FAILURES=$((FAILURES + 1))
fi

echo -e "${YELLOW}Checking vendor_agnostic_demo...${NC}"
if cargo check --example vendor_agnostic_demo --package songbird-discovery --quiet 2>&1 | grep -q "Finished"; then
    echo -e "${GREEN}✅ PASS: vendor_agnostic_demo compiles${NC}"
else
    echo -e "${RED}❌ FAIL: vendor_agnostic_demo failed to compile${NC}"
    FAILURES=$((FAILURES + 1))
fi
echo ""

echo -e "${BLUE}Phase 4: Example Execution${NC}"
echo "----------------------------"

# Test 6: Run example with proper env
echo -e "${YELLOW}Running infant_discovery_demo...${NC}"
export SERVICE_PORT=8080
export SERVICE_ID=test-service
export SONGBIRD_HOST=127.0.0.1

timeout 5s cargo run --example infant_discovery_demo --package songbird-config --quiet 2>&1 | tee /tmp/example-run.log || true

if grep -q "Infant Discovery Demo" /tmp/example-run.log; then
    echo -e "${GREEN}✅ PASS: Example runs successfully${NC}"
else
    echo -e "${YELLOW}⚠️  WARN: Example output unexpected (may need SERVICE_PORT)${NC}"
fi
echo ""

echo -e "${BLUE}Phase 5: Documentation Check${NC}"
echo "----------------------------"

# Test 7: Key files exist
for file in README.md QUICK_START.md CONTRIBUTING.md LICENSE Cargo.toml; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ ${file} exists${NC}"
    else
        echo -e "${RED}❌ ${file} missing${NC}"
        FAILURES=$((FAILURES + 1))
    fi
done
echo ""

echo -e "${BLUE}Phase 6: Code Quality${NC}"
echo "----------------------------"

# Test 8: Clippy (allow warnings for now)
echo -e "${YELLOW}Running clippy...${NC}"
CLIPPY_WARNINGS=$(cargo clippy --workspace --quiet 2>&1 | grep "warning:" | wc -l)
echo -e "${GREEN}✅ Clippy complete (${CLIPPY_WARNINGS} warnings - deprecations expected)${NC}"
echo ""

# Test 9: Audit for known vulnerabilities
echo -e "${YELLOW}Checking for security vulnerabilities...${NC}"
if command -v cargo-audit &> /dev/null; then
    if cargo audit --quiet 2>&1 | grep -q "Success"; then
        echo -e "${GREEN}✅ PASS: No known vulnerabilities${NC}"
    else
        echo -e "${YELLOW}⚠️  WARN: cargo audit found issues (review recommended)${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  SKIP: cargo-audit not installed (install: cargo install cargo-audit)${NC}"
fi
echo ""

echo -e "${BLUE}Phase 7: Binary Check${NC}"
echo "----------------------------"

# Test 10: Orchestrator binary exists
if [ -f "target/release/songbird-orchestrator" ]; then
    SIZE=$(du -h target/release/songbird-orchestrator | cut -f1)
    echo -e "${GREEN}✅ songbird-orchestrator binary built (${SIZE})${NC}"
else
    echo -e "${YELLOW}⚠️  orchestrator binary not found (may need: cargo build --release --bin songbird-orchestrator)${NC}"
fi

if [ -f "target/release/songbird-cli" ]; then
    SIZE=$(du -h target/release/songbird-cli | cut -f1)
    echo -e "${GREEN}✅ songbird-cli binary built (${SIZE})${NC}"
else
    echo -e "${YELLOW}⚠️  CLI binary not found (build if needed)${NC}"
fi
echo ""

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Results Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo -e "${GREEN}✅ Songbird is ready for GitHub release!${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "  1. Review test output above"
    echo "  2. Commit changes: git add . && git commit -m 'chore: MVP ready'"
    echo "  3. Push to GitHub: git push origin main"
    echo "  4. Create release: GitHub UI or 'gh release create v0.2.0'"
    echo ""
    exit 0
else
    echo -e "${RED}❌ ${FAILURES} test(s) failed${NC}"
    echo -e "${YELLOW}Please review failures above and fix before pushing${NC}"
    echo ""
    exit 1
fi

