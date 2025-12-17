#!/usr/bin/env bash
# Coverage generation script
# Usage: ./scripts/coverage.sh [html|text|summary]

set -e

MODE="${1:-html}"

echo "🔍 Generating coverage report (mode: $MODE)..."

case "$MODE" in
  html)
    echo "📊 Generating HTML coverage report..."
    cargo llvm-cov --all-features --workspace --html
    echo "✅ HTML report generated: target/llvm-cov/html/index.html"
    echo "📖 Open with: open target/llvm-cov/html/index.html"
    ;;
  
  text)
    echo "📊 Generating text coverage report..."
    cargo llvm-cov --all-features --workspace
    ;;
  
  summary)
    echo "📊 Coverage summary:"
    cargo llvm-cov --all-features --workspace --summary-only
    ;;
  
  lcov)
    echo "📊 Generating lcov report..."
    cargo llvm-cov --all-features --workspace --lcov --output-path coverage.lcov
    echo "✅ LCOV report generated: coverage.lcov"
    ;;
  
  by-crate)
    echo "📊 Coverage by crate:"
    for crate in crates/*/; do
      crate_name=$(basename "$crate")
      echo ""
      echo "📦 $crate_name:"
      cargo llvm-cov --all-features -p "songbird-${crate_name#songbird-}" --summary-only 2>/dev/null || echo "  (no tests or coverage)"
    done
    ;;
  
  *)
    echo "❌ Unknown mode: $MODE"
    echo "Usage: $0 [html|text|summary|lcov|by-crate]"
    exit 1
    ;;
esac

echo ""
echo "✅ Coverage report complete!"

