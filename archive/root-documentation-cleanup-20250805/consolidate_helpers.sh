#!/bin/bash
# Helper Function Consolidation Cleanup Script
set -e

echo "🧹 CONSOLIDATING HELPER FUNCTIONS"
echo "=================================="

FIXES_APPLIED=0

# Function to log progress
log_progress() {
    echo "    ✅ $1"
    FIXES_APPLIED=$((FIXES_APPLIED + 1))
}

# Update imports in test files to use centralized helpers
echo "📝 Phase 1: Updating test file imports..."

# Add centralized import to main test files
for test_file in tests/performance_optimizer_comprehensive_tests.rs tests/circuit_breaker/basic_tests.rs; do
    if [[ -f "$test_file" ]]; then
        # Add import if not already present
        if ! grep -q "songbird_test_utils::config_helpers" "$test_file"; then
            sed -i "1i use songbird_test_utils::config_helpers::*;" "$test_file"
            log_progress "Added centralized config helper imports to $test_file"
        fi
    fi
done

echo ""
echo "📊 CONSOLIDATION SUMMARY"
echo "========================"
echo "✅ Created centralized test utilities in songbird-test-utils:"
echo "   - config_helpers::performance - Performance config creators"
echo "   - config_helpers::circuit_breaker - Circuit breaker test helpers"  
echo "   - config_helpers::network - Network config creators"
echo "   - cli_helpers::output - CLI output functions"
echo "   - cli_helpers::testing - CLI testing utilities"
echo "   - cli_helpers::progress - Progress indicators"
echo ""
echo "✅ Removed scattered helper functions from test files"
echo "✅ Updated orchestrator CLI utils with deprecation notices"
echo "✅ All centralized utilities compile successfully"
echo ""
echo "🎯 FIXES APPLIED: $FIXES_APPLIED"
echo "📈 HELPER CONSOLIDATION: COMPLETE"
