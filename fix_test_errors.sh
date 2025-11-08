#!/bin/bash
# Systematic Test Error Fix Script
# Fixes the 5 most common test compilation error patterns

echo "🔧 Songbird Test Error Fix Script"
echo "=================================="
echo ""

# Pattern 1: Fix 'cannot find value e in scope' errors
# Change |_| to |e| when e is used in the closure body
echo "📝 Phase 1: Fixing variable scope errors (|_| -> |e| when e is used)..."

find crates/*/tests -name "*.rs" -type f | while read file; do
    # Fix map_err(|_| ... format!(..., e) patterns
    if grep -q 'map_err(|_|.*format!.*[,\s]e[,)]' "$file"; then
        echo "  Fixing: $file"
        sed -i 's/\.map_err(|_|\(.*format!.*\), e\(.*\))/\.map_err(|e|\1, e\2)/g' "$file"
    fi
done

echo "✅ Phase 1 complete"
echo ""

# Pattern 2: Fix duplicate imports
echo "📝 Phase 2: Fixing duplicate imports..."

find crates/*/tests -name "*.rs" -type f | while read file; do
    # Check if file has both single and combined import
    if grep -q "^use songbird_types::SongbirdError;$" "$file" && \
       grep -q "^use songbird_types::{.*SongbirdError" "$file"; then
        echo "  Fixing: $file"
        # Remove the single-line import
        sed -i '/^use songbird_types::SongbirdError;$/d' "$file"
    fi
    
    if grep -q "^use songbird_types::SongbirdResult;$" "$file" && \
       grep -q "^use songbird_types::{.*SongbirdResult" "$file"; then
        echo "  Fixing: $file"
        # Remove the single-line import
        sed -i '/^use songbird_types::SongbirdResult;$/d' "$file"
    fi
done

echo "✅ Phase 2 complete"
echo ""

# Pattern 3: Fix .ok_or_else on Result (should be .map_err)
echo "📝 Phase 3: Fixing Result.ok_or_else -> Result.map_err..."

find crates/*/tests -name "*.rs" -type f | while read file; do
    if grep -q '\.ok_or_else(' "$file"; then
        # Count occurrences
        count=$(grep -c '\.ok_or_else(' "$file")
        if [ "$count" -gt 0 ]; then
            echo "  Found $count instances in: $file"
            echo "  (Manual review needed - context-dependent)"
        fi
    fi
done

echo "✅ Phase 3 complete (manual fixes needed)"
echo ""

# Pattern 4: Add missing imports
echo "📝 Phase 4: Checking for missing imports..."

cargo test --workspace 2>&1 | grep "E0412.*SongbirdResult" | \
    sed 's/.*-->\s*\([^:]*\):.*/\1/' | sort -u | while read file; do
    if [ -f "$file" ]; then
        if ! grep -q "use songbird_types::SongbirdResult" "$file"; then
            echo "  $file needs: use songbird_types::SongbirdResult;"
        fi
    fi
done

echo "✅ Phase 4 complete"
echo ""

# Phase 5: Test compilation
echo "📝 Phase 5: Testing compilation..."
echo ""

cargo test --workspace --no-run 2>&1 | grep -E "Compiling|Finished|error:" | head -50

echo ""
echo "=================================="
echo "🎯 Fix script complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Run tests: cargo test --workspace"
echo "  3. Fix remaining manual issues"
echo ""

