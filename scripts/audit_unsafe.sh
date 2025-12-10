#!/usr/bin/env bash
# Audit unsafe blocks in production code

echo "# Unsafe Block Audit Report"
echo "Generated: $(date)"
echo
echo "## Production Code (crates/*/src)"
echo

find crates/*/src -name "*.rs" -type f | while read f; do
    # Skip test files
    if echo "$f" | grep -q "test"; then
        continue
    fi
    
    count=$(grep -c "unsafe" "$f" 2>/dev/null || echo "0")
    if [ "$count" -gt "0" ]; then
        echo "- $f: $count blocks"
    fi
done | sort -t: -k2 -rn | head -30

echo
echo "## Summary"
total=$(find crates/*/src -name "*.rs" -type f ! -path "*/test*" -exec grep -c "unsafe" {} \; | awk '{s+=$1} END {print s}')
files=$(find crates/*/src -name "*.rs" -type f ! -path "*/test*" -exec grep -l "unsafe" {} \; | wc -l)
echo "Total unsafe blocks in production: $total"
echo "Files with unsafe blocks: $files"
