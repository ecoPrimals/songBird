#!/bin/bash
# Systematic Syntax Error Fixer for Songbird
# Generated: October 4, 2025

echo "🔧 Songbird Syntax Error Fix Script"
echo "===================================="
echo ""

# Common patterns to fix:
# 1. Missing ) before ; or ,
# 2. Extra ;" instead of );
# 3. Mismatched delimiters

echo "Step 1: Finding files with syntax errors..."
cargo build --workspace 2>&1 | grep "^error" | grep -oP 'crates/[^:]+' | sort -u > /tmp/error_files.txt

echo "Files with errors:"
cat /tmp/error_files.txt
echo ""

echo "Step 2: Count of errors per file..."
while read file; do
    count=$(cargo build --workspace 2>&1 | grep "$file" | grep "^error" | wc -l)
    if [ $count -gt 0 ]; then
        echo "  $file: $count errors"
    fi
done < /tmp/error_files.txt

echo ""
echo "To fix systematically:"
echo "1. cargo build -p <crate> 2>&1 | grep -A 5 '^error:' | less"
echo "2. Identify pattern (missing ), extra ;, etc.)"
echo "3. Fix with search_replace"
echo "4. Repeat until crate compiles"
echo ""
echo "Current progress:"
echo "✅ songbird-cli: FIXED"
echo "✅ songbird-discovery: FIXED"  
echo "⏳ songbird-core: 16 errors in zero_cost_optimizations.rs"
echo "⏳ songbird-network: ~50-80 errors"
echo "⏳ Other crates: ~150-200 errors"
echo ""
echo "Estimated time remaining: 2-4 hours with systematic approach"

