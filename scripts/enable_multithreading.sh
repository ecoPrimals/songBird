#!/bin/bash
# Enable multi-threading for all tokio tests
# Part of Concurrent Evolution Plan - December 10, 2025

set -e

echo "🚀 Enabling multi-threaded tokio tests..."
echo ""

# Counter
count=0
total=0

# Find all Rust test files
find crates -name "*.rs" -type f | while read file; do
    # Check if file has tokio tests
    if grep -q "#\[tokio::test\]$" "$file" 2>/dev/null; then
        total=$((total + 1))
        
        # Replace single-threaded with multi-threaded
        if sed -i 's/#\[tokio::test\]$/#[tokio::test(flavor = "multi_thread", worker_threads = 4)]/g' "$file"; then
            count=$((count + 1))
            echo "✅ Updated: $file"
        fi
    fi
done

echo ""
echo "📊 Summary:"
echo "  Files processed: $count"
echo "  Multi-threading enabled for all tokio tests!"
echo ""
echo "🧪 Running tests to verify..."
cargo test --workspace --lib

echo ""
echo "✅ Multi-threading enablement complete!"

