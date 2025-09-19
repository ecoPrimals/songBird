#!/bin/bash
# 🔍 Comprehensive System Validation Script
# 
# Validates all aspects of the Songbird Universal Orchestrator for production readiness
# Checks build system, testing infrastructure, performance, and integration capabilities

set -e

echo "🔍 Songbird Comprehensive System Validation"
echo "==========================================="

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Validation counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNING_CHECKS=0

# Function to print status
print_status() {
    echo -e "${BLUE}[VALIDATION]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✅ PASS]${NC} $1"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
}

print_warning() {
    echo -e "${YELLOW}[⚠️ WARN]${NC} $1"
    WARNING_CHECKS=$((WARNING_CHECKS + 1))
}

print_error() {
    echo -e "${RED}[❌ FAIL]${NC} $1"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
}

print_info() {
    echo -e "${PURPLE}[ℹ️ INFO]${NC} $1"
}

# Function to run a validation check
run_check() {
    local check_name="$1"
    local check_command="$2"
    local success_message="$3"
    local failure_message="$4"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    print_status "Running: $check_name"
    
    if eval "$check_command" >/dev/null 2>&1; then
        print_success "$success_message"
        return 0
    else
        print_error "$failure_message"
        return 1
    fi
}

# Function to run a validation check with output capture
run_check_with_output() {
    local check_name="$1"
    local check_command="$2"
    local success_message="$3"
    local failure_message="$4"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    print_status "Running: $check_name"
    
    if output=$(eval "$check_command" 2>&1); then
        print_success "$success_message"
        return 0
    else
        print_error "$failure_message"
        print_info "Output: $output"
        return 1
    fi
}

print_status "Starting comprehensive system validation..."

# Phase 1: Build System Validation
print_status "Phase 1: Build System Validation"

run_check "Cargo availability" \
    "command -v cargo" \
    "Cargo build system available" \
    "Cargo not found - Rust toolchain missing"

run_check "Project structure validation" \
    "test -f Cargo.toml && test -d crates/" \
    "Project structure is valid" \
    "Invalid project structure - missing Cargo.toml or crates directory"

run_check "Debug build compilation" \
    "cargo check --all-targets" \
    "Debug build compiles successfully" \
    "Debug build compilation failed"

run_check "Release build compilation" \
    "cargo build --release --all-targets" \
    "Release build compiles successfully" \
    "Release build compilation failed"

run_check "Clippy linting (warnings allowed)" \
    "cargo clippy --all-targets --all-features" \
    "Clippy linting completed (warnings may exist)" \
    "Clippy linting failed with errors"

run_check "Code formatting check" \
    "cargo fmt --check" \
    "Code formatting is consistent" \
    "Code formatting inconsistencies found"

# Phase 2: Testing Infrastructure Validation
print_status "Phase 2: Testing Infrastructure Validation"

run_check "Unit tests execution" \
    "cargo test --lib --all" \
    "Unit tests pass successfully" \
    "Unit tests failed"

run_check "Integration tests execution" \
    "cargo test --tests --all" \
    "Integration tests pass successfully" \
    "Integration tests failed"

run_check "Chaos engineering tests" \
    "cargo test -p songbird-test-utils --tests chaos_activation_test" \
    "Chaos engineering framework operational" \
    "Chaos engineering tests failed"

run_check "End-to-end workflow tests" \
    "cargo test -p songbird-test-utils --tests e2e_workflow_tests" \
    "End-to-end workflows validated" \
    "End-to-end workflow tests failed"

run_check "Test utilities validation" \
    "cargo test -p songbird-test-utils --tests comprehensive_test_utils_tests" \
    "Test utilities framework operational" \
    "Test utilities validation failed"

# Phase 3: Core Functionality Validation
print_status "Phase 3: Core Functionality Validation"

run_check "Configuration loading" \
    "cargo test -p songbird-types --lib config" \
    "Configuration system operational" \
    "Configuration loading tests failed"

run_check "Error handling system" \
    "cargo test -p songbird-types --lib errors" \
    "Error handling system validated" \
    "Error handling tests failed"

run_check "Type system validation" \
    "cargo test -p songbird-types --lib" \
    "Type system integrity confirmed" \
    "Type system validation failed"

# Phase 4: Performance Validation
print_status "Phase 4: Performance Validation"

# Quick performance check - build time
BUILD_START=$(date +%s)
if cargo build --release --all-targets >/dev/null 2>&1; then
    BUILD_END=$(date +%s)
    BUILD_TIME=$((BUILD_END - BUILD_START))
    
    if [ $BUILD_TIME -le 300 ]; then
        print_success "Build performance acceptable: ${BUILD_TIME}s (≤300s)"
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        print_warning "Build performance slow: ${BUILD_TIME}s (>300s)"
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
    fi
else
    print_error "Build performance test failed"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Quick test execution time
TEST_START=$(date +%s)
if cargo test --all --tests --quiet >/dev/null 2>&1; then
    TEST_END=$(date +%s)
    TEST_TIME=$((TEST_END - TEST_START))
    
    if [ $TEST_TIME -le 60 ]; then
        print_success "Test execution performance acceptable: ${TEST_TIME}s (≤60s)"
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        print_warning "Test execution performance slow: ${TEST_TIME}s (>60s)"
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        WARNING_CHECKS=$((WARNING_CHECKS + 1))
    fi
else
    print_error "Test execution performance check failed"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Phase 5: Documentation and Compliance Validation
print_status "Phase 5: Documentation and Compliance Validation"

run_check "Documentation generation" \
    "cargo doc --no-deps --all-features" \
    "Documentation generates successfully" \
    "Documentation generation failed"

run_check "README file existence" \
    "test -f README.md" \
    "README.md exists" \
    "README.md missing"

run_check "License file existence" \
    "test -f LICENSE || test -f LICENSE.md || test -f LICENSE.txt" \
    "License file exists" \
    "License file missing"

run_check "Cargo.toml metadata completeness" \
    "grep -q 'name.*=.*\"songbird\"' Cargo.toml && grep -q 'version.*=' Cargo.toml" \
    "Cargo.toml metadata is complete" \
    "Cargo.toml metadata incomplete"

# Phase 6: File Structure and Standards Validation
print_status "Phase 6: File Structure and Standards Validation"

# Check for excessively large files (>1000 lines)
LARGE_FILES=$(find . -name "*.rs" -type f -exec wc -l {} + | awk '$1 > 1000 {print $2}' | head -5)
if [ -z "$LARGE_FILES" ]; then
    print_success "All Rust files comply with 1000-line limit"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
else
    print_warning "Some files exceed 1000-line limit: $(echo $LARGE_FILES | tr '\n' ' ')"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    WARNING_CHECKS=$((WARNING_CHECKS + 1))
fi

# Check for TODO/FIXME items
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" --include="*.rs" . 2>/dev/null | wc -l)
if [ $TODO_COUNT -eq 0 ]; then
    print_success "No TODO/FIXME items found"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
elif [ $TODO_COUNT -le 10 ]; then
    print_warning "Found $TODO_COUNT TODO/FIXME items (manageable)"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    WARNING_CHECKS=$((WARNING_CHECKS + 1))
else
    print_error "Found $TODO_COUNT TODO/FIXME items (requires attention)"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Phase 7: Security and Safety Validation
print_status "Phase 7: Security and Safety Validation"

# Check for unsafe code usage
UNSAFE_COUNT=$(grep -r "unsafe" --include="*.rs" . 2>/dev/null | wc -l)
if [ $UNSAFE_COUNT -eq 0 ]; then
    print_success "No unsafe code detected"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
elif [ $UNSAFE_COUNT -le 20 ]; then
    print_warning "Found $UNSAFE_COUNT unsafe code blocks (review recommended)"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    WARNING_CHECKS=$((WARNING_CHECKS + 1))
else
    print_error "Found $UNSAFE_COUNT unsafe code blocks (requires security review)"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
fi

# Check for hardcoded secrets or sensitive data
SENSITIVE_PATTERNS="password|secret|key|token|api_key"
SENSITIVE_COUNT=$(grep -ri "$SENSITIVE_PATTERNS" --include="*.rs" . 2>/dev/null | grep -v "test\|example\|demo" | wc -l)
if [ $SENSITIVE_COUNT -eq 0 ]; then
    print_success "No hardcoded sensitive data detected"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
else
    print_warning "Found $SENSITIVE_COUNT potential sensitive data references (review recommended)"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    WARNING_CHECKS=$((WARNING_CHECKS + 1))
fi

# Phase 8: Generate Validation Report
print_status "Phase 8: Generating Validation Report"

VALIDATION_REPORT="system_validation_report_$(date +%Y%m%d_%H%M%S).json"

cat > "$VALIDATION_REPORT" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "validation_summary": {
    "total_checks": $TOTAL_CHECKS,
    "passed_checks": $PASSED_CHECKS,
    "warning_checks": $WARNING_CHECKS,
    "failed_checks": $FAILED_CHECKS,
    "success_rate": $(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc)
  },
  "validation_phases": {
    "build_system": "completed",
    "testing_infrastructure": "completed", 
    "core_functionality": "completed",
    "performance": "completed",
    "documentation": "completed",
    "file_structure": "completed",
    "security_safety": "completed"
  },
  "performance_metrics": {
    "build_time_seconds": $BUILD_TIME,
    "test_execution_seconds": $TEST_TIME
  },
  "code_quality": {
    "todo_count": $TODO_COUNT,
    "unsafe_code_blocks": $UNSAFE_COUNT,
    "sensitive_data_references": $SENSITIVE_COUNT
  }
}
EOF

# Final Results
echo ""
echo "📊 COMPREHENSIVE SYSTEM VALIDATION RESULTS"
echo "=========================================="
echo ""
print_info "Total validation checks: $TOTAL_CHECKS"
print_info "✅ Passed: $PASSED_CHECKS"
print_info "⚠️  Warnings: $WARNING_CHECKS"  
print_info "❌ Failed: $FAILED_CHECKS"

SUCCESS_RATE=$(echo "scale=1; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc)
print_info "📈 Success rate: ${SUCCESS_RATE}%"

echo ""

# Determine overall system status
if [ $FAILED_CHECKS -eq 0 ]; then
    if [ $WARNING_CHECKS -eq 0 ]; then
        print_success "🎊 SYSTEM VALIDATION COMPLETE - ALL CHECKS PASSED!"
        print_success "✅ System is PRODUCTION READY"
        OVERALL_STATUS="PRODUCTION_READY"
        EXIT_CODE=0
    else
        print_warning "⚠️  System validation completed with warnings"
        print_warning "✅ System is PRODUCTION READY with minor issues to address"
        OVERALL_STATUS="PRODUCTION_READY_WITH_WARNINGS"
        EXIT_CODE=0
    fi
else
    if [ $FAILED_CHECKS -le 2 ] && [ $SUCCESS_RATE -gt 80 ]; then
        print_warning "⚠️  System validation completed with minor failures"
        print_warning "🔄 System needs minor fixes before production deployment"
        OVERALL_STATUS="NEEDS_MINOR_FIXES"
        EXIT_CODE=1
    else
        print_error "❌ System validation failed with significant issues"
        print_error "🚫 System NOT ready for production deployment"
        OVERALL_STATUS="NOT_PRODUCTION_READY"
        EXIT_CODE=1
    fi
fi

# Update JSON report with overall status
jq --arg status "$OVERALL_STATUS" --arg rate "$SUCCESS_RATE" \
   '.overall_status = $status | .validation_summary.success_rate = ($rate | tonumber)' \
   "$VALIDATION_REPORT" > "${VALIDATION_REPORT}.tmp" && mv "${VALIDATION_REPORT}.tmp" "$VALIDATION_REPORT" 2>/dev/null || true

print_status "Validation report saved to: $VALIDATION_REPORT"

echo ""
echo "🚀 Next Steps:"
echo "=============="
if [ $FAILED_CHECKS -eq 0 ] && [ $WARNING_CHECKS -eq 0 ]; then
    echo "✅ System is ready for production deployment!"
    echo "✅ All validation checks passed successfully"
    echo "✅ Proceed with deployment confidence"
elif [ $FAILED_CHECKS -eq 0 ]; then
    echo "⚠️  Address warning items for optimal production readiness"
    echo "✅ System can be deployed with minor monitoring"
    echo "📋 Review warnings in the validation report"
else
    echo "🔧 Address failed validation checks before production deployment"
    echo "📋 Review failed items in the validation report"
    echo "🔄 Re-run validation after fixes"
fi

echo ""
exit $EXIT_CODE 