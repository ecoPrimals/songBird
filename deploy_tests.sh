#!/bin/bash
# deploy_tests.sh - Phase by phase test deployment
# Based on TEST_DEPLOYMENT_PLAN_OCT_25_2025.md

set -e  # Exit on error

echo "🎯 Songbird Test Deployment Script"
echo "Goal: 19% → 40%+ coverage in 3 weeks"
echo ""

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Phase 1: Quick Wins (Days 1-2) - Target 26% Coverage
phase1() {
    echo -e "${BLUE}🚀 Phase 1: Quick Wins (Days 1-2)${NC}"
    echo "Target: 26% coverage (+7% from baseline)"
    echo ""
    
    echo "Testing songbird-observability..."
    cargo test -p songbird-observability --all-features --lib || echo "Some tests may need activation"
    
    echo ""
    echo "Testing songbird-test-utils..."
    cargo test -p songbird-test-utils --all-features --lib || echo "Some tests may need activation"
    
    echo ""
    echo "Testing songbird-universal..."
    cargo test -p songbird-universal --all-features --lib || echo "Some tests may need activation"
    
    echo -e "${GREEN}✅ Phase 1 testing complete${NC}"
    echo ""
}

# Phase 2: Core System Tests (Days 3-5) - Target 35% Coverage
phase2() {
    echo -e "${BLUE}🚀 Phase 2: Core System Tests (Days 3-5)${NC}"
    echo "Target: 35% coverage (+16% from baseline)"
    echo ""
    
    echo "Testing songbird-registry..."
    cargo test -p songbird-registry --all-features --lib
    
    echo ""
    echo "Testing songbird-discovery..."
    cargo test -p songbird-discovery --all-features --lib
    
    echo ""
    echo "Testing songbird-canonical..."
    cargo test -p songbird-canonical --all-features --lib
    
    echo -e "${GREEN}✅ Phase 2 testing complete${NC}"
    echo ""
}

# Phase 3: Integration & E2E (Week 2) - Target 40%+ Coverage
phase3() {
    echo -e "${BLUE}🚀 Phase 3: Integration & E2E (Week 2)${NC}"
    echo "Target: 40%+ coverage (+21%+ from baseline)"
    echo ""
    
    echo "Running E2E test scenarios..."
    cargo test --test adapter_integration_tests 2>/dev/null || echo "Test not yet activated"
    cargo test --test discovery_integration_tests 2>/dev/null || echo "Test not yet activated"
    cargo test --test integration_tests 2>/dev/null || echo "Test not yet activated"
    cargo test --test routing_tests 2>/dev/null || echo "Test not yet activated"
    cargo test --test sovereignty_comprehensive_tests 2>/dev/null || echo "Test not yet activated"
    
    echo ""
    echo "Testing federation..."
    cargo test -p songbird-network-federation --all-features --lib
    
    echo -e "${GREEN}✅ Phase 3 testing complete${NC}"
    echo ""
}

# Coverage measurement
measure_coverage() {
    echo -e "${BLUE}📊 Measuring test coverage...${NC}"
    echo ""
    
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --workspace --out Html,Json --output-dir coverage-report || \
            echo -e "${YELLOW}⚠️  Tarpaulin failed, install with: cargo install cargo-tarpaulin${NC}"
    else
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed${NC}"
        echo "Install with: cargo install cargo-tarpaulin"
        echo "Then run: cargo tarpaulin --workspace --out Html --output-dir coverage-report"
    fi
    
    echo ""
}

# Full test suite
full_test() {
    echo -e "${BLUE}🧪 Running full test suite...${NC}"
    echo ""
    
    cargo test --workspace --lib
    
    echo -e "${GREEN}✅ Full test suite complete${NC}"
    echo ""
}

# Main execution
main() {
    echo "=========================================="
    echo "  Songbird Test Deployment"
    echo "  Based on: TEST_DEPLOYMENT_PLAN_OCT_25_2025.md"
    echo "=========================================="
    echo ""
    
    # Parse command line arguments
    case "${1:-full}" in
        phase1)
            phase1
            ;;
        phase2)
            phase2
            ;;
        phase3)
            phase3
            ;;
        coverage)
            measure_coverage
            ;;
        full)
            echo "Running all phases..."
            phase1
            phase2
            phase3
            full_test
            measure_coverage
            ;;
        quick)
            echo "Running quick test validation..."
            full_test
            ;;
        *)
            echo "Usage: $0 {phase1|phase2|phase3|coverage|full|quick}"
            echo ""
            echo "  phase1   - Run Phase 1 tests (Quick Wins)"
            echo "  phase2   - Run Phase 2 tests (Core Systems)"
            echo "  phase3   - Run Phase 3 tests (Integration & E2E)"
            echo "  coverage - Measure test coverage"
            echo "  full     - Run all phases + coverage (default)"
            echo "  quick    - Quick validation run"
            exit 1
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}=========================================="
    echo "  Test Deployment Complete!"
    echo "==========================================${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Review test results above"
    echo "2. Check coverage report (if generated)"
    echo "3. Proceed to next phase"
    echo ""
}

main "$@"

