#!/usr/bin/env bash
# Count remaining technical debt

set -euo pipefail

cd "$(dirname "$0")/../.."

echo "=== 📊 TECHNICAL DEBT METRICS ==="
echo ""

# Sleeps
SLEEP_COUNT=$(rg 'sleep|Sleep' crates --type rust | wc -l || echo "0")
echo "⏰ Sleeps:     $SLEEP_COUNT (target: 0 except chaos tests)"

# Unwraps
UNWRAP_COUNT=$(rg '\.unwrap\(|\.expect\(' crates --type rust | wc -l || echo "0")
echo "💥 Unwraps:    $UNWRAP_COUNT (target: test-only)"

# Clones
CLONE_COUNT=$(rg '\.clone\(\)' crates --type rust | wc -l || echo "0")
echo "📋 Clones:     $CLONE_COUNT (target: <500)"

# TODOs/FIXMEs
TODO_COUNT=$(rg 'TODO|FIXME|XXX|HACK' . --type rust ! -path "*/target/*" | wc -l || echo "0")
echo "📝 TODOs:      $TODO_COUNT"

# Unsafe blocks
UNSAFE_COUNT=$(rg 'unsafe' crates --type rust | wc -l || echo "0")
echo "⚠️  Unsafe:     $UNSAFE_COUNT (acceptable for systems code)"

# Mock usage
MOCK_COUNT=$(rg 'mock|Mock|stub|Stub|fake|Fake' crates --type rust | wc -l || echo "0")
echo "🎭 Mocks:      $MOCK_COUNT (test infrastructure)"

echo ""
echo "=== 📈 QUALITY METRICS ==="
echo ""

# File count
RUST_FILES=$(find crates -name "*.rs" -type f | wc -l)
echo "📁 Rust files: $RUST_FILES"

# Test count  
TEST_COUNT=$(rg '#\[test\]|#\[tokio::test\]' crates --type rust | wc -l || echo "0")
echo "🧪 Tests:      $TEST_COUNT"

# Lines of code
LOC=$(find crates -name "*.rs" -type f -exec wc -l {} + | tail -1 | awk '{print $1}' || echo "0")
echo "📏 Total LoC:  $LOC"

# Average file size
AVG_SIZE=$((LOC / RUST_FILES))
echo "📐 Avg file:   $AVG_SIZE lines"

# Largest files
echo ""
echo "=== 🔍 LARGEST FILES (Top 5) ===" 
find crates -name "*.rs" -type f ! -path "*/target/*" -exec wc -l {} + | sort -rn | head -6 | tail -5 | awk '{printf "   %4d lines: %s\n", $1, $2}'

echo ""
echo "✅ Run 'cargo llvm-cov --workspace --html' for coverage report"

