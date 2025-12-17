#!/bin/bash
# Quick Start Script - Next Session
# Run this to begin the next phase of work

echo "🚀 Songbird - Next Phase Quick Start"
echo "======================================"
echo ""

echo "📊 Step 1: Verify test compilation..."
cargo test --workspace --no-run
if [ $? -eq 0 ]; then
    echo "✅ All tests compile successfully!"
else
    echo "❌ Test compilation failed. Check errors above."
    exit 1
fi

echo ""
echo "📈 Step 2: Run test suite (this may take a few minutes)..."
cargo test --workspace --lib -- --test-threads=4 2>&1 | tee test_results.log
echo "✅ Test results saved to test_results.log"

echo ""
echo "📊 Step 3: Measure code coverage..."
echo "Running: cargo llvm-cov --workspace --lcov --output-path coverage.lcov"
cargo llvm-cov --workspace --lcov --output-path coverage.lcov 2>&1 | tee coverage.log

if [ -f coverage.lcov ]; then
    echo "✅ Coverage report generated: coverage.lcov"
    echo ""
    echo "📊 Coverage summary:"
    cargo llvm-cov --workspace --summary-only
else
    echo "⚠️  Coverage generation incomplete. Check coverage.log for details."
fi

echo ""
echo "📋 Step 4: Generate coverage HTML report..."
cargo llvm-cov --workspace --html
if [ $? -eq 0 ]; then
    echo "✅ HTML coverage report generated in: target/llvm-cov/html/index.html"
    echo "   Open it with: xdg-open target/llvm-cov/html/index.html"
fi

echo ""
echo "✅ Quick start complete!"
echo ""
echo "📚 Next steps:"
echo "1. Review coverage report"
echo "2. Check CAPABILITY_BASED_MIGRATION_GUIDE.md for migration patterns"
echo "3. Start migrating 1-2 modules to capability-based discovery"
echo "4. Follow 15-week roadmap in COMPREHENSIVE_AUDIT_REPORT_DEC_12_2025.md"
echo ""
echo "📁 Key documentation:"
echo "   - COMPREHENSIVE_AUDIT_REPORT_DEC_12_2025.md"
echo "   - CAPABILITY_BASED_MIGRATION_GUIDE.md"
echo "   - SESSION_COMPLETE_DELIVERABLES.md"
echo ""
echo "🎯 Current status: All blockers resolved, ready for systematic execution!"

