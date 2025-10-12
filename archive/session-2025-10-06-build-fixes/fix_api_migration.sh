#!/bin/bash
# Automated API Migration Script
# Fixes Result<T, E> → Result<SongbirdResponse<T>, E> patterns

echo "🔧 Starting API Migration Fix..."
echo "================================================"

# Find all Rust files in crates/songbird-network
FILES=$(find crates/songbird-network/src -name "*.rs" -type f)

TOTAL_FIXES=0

for file in $FILES; do
    echo "Processing: $file"
    
    # Count fixes in this file
    FIXES=0
    
    # Fix pattern: Ok(()) → Ok(SongbirdResponse::success(()))
    if grep -q "^        Ok(())$" "$file"; then
        sed -i 's/^        Ok(())$/        Ok(SongbirdResponse::success(()))/' "$file"
        FIXES=$((FIXES + 1))
    fi
    
    # Fix pattern: Ok(value) at end of function → Ok(SongbirdResponse::success(value))
    # This is trickier - we need to be more selective
    
    # Pattern: return Ok(something)
    perl -i -pe 's/return Ok\(([^)]+)\)$/return Ok(SongbirdResponse::success($1))/ if /return Ok\([^)]+\)$/ && !/SongbirdResponse/' "$file"
    
    # Pattern: Ok(value) where value is simple (no nested parens)
    # Only at end of lines followed by }
    # This needs careful handling to avoid breaking already-fixed code
    
    if [ $FIXES -gt 0 ]; then
        TOTAL_FIXES=$((TOTAL_FIXES + FIXES))
        echo "  ✓ Fixed $FIXES patterns"
    fi
done

echo "================================================"
echo "✅ Total patterns fixed: $TOTAL_FIXES"
echo ""
echo "🔍 Now checking build status..."
cargo build --package songbird-network 2>&1 | grep "error\[E" | wc -l

