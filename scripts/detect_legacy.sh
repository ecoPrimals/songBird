#!/bin/bash
# detect_legacy.sh - Find and categorize legacy patterns
# Part of the Unification & Modernization initiative

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create reports directory
mkdir -p "$REPORT_DIR"

echo "🔍 Songbird Legacy Pattern Detection"
echo "====================================="
echo ""

# Find all legacy patterns
echo "📊 Scanning for legacy patterns..."
LEGACY_FILE="$REPORT_DIR/legacy_patterns.txt"
grep -rn "legacy\|shim\|wrapper" -i "$PROJECT_ROOT/crates"/*/src --include="*.rs" > "$LEGACY_FILE" || true

TOTAL_MATCHES=$(wc -l < "$LEGACY_FILE")
echo "   Total matches found: $TOTAL_MATCHES"

if [ "$TOTAL_MATCHES" -eq 0 ]; then
  echo "✅ No legacy patterns found!"
  exit 0
fi

# Categorize patterns
echo ""
echo "📂 Categorization:"

REEXPORTS=$(grep -c "pub use.*legacy\|pub use.*shim\|pub use.*wrapper" "$LEGACY_FILE" || echo "0")
echo "   Re-exports:        $REEXPORTS"

WRAPPERS=$(grep -c "struct.*Wrapper\|struct.*Shim\|struct.*Legacy" "$LEGACY_FILE" || echo "0")
echo "   Wrapper structs:   $WRAPPERS"

FUNCTIONS=$(grep -c "fn.*legacy\|fn.*migrate\|fn.*shim" "$LEGACY_FILE" || echo "0")
echo "   Migration funcs:   $FUNCTIONS"

COMMENTS=$(grep -c "//.*legacy\|//.*shim\|//.*wrapper" "$LEGACY_FILE" || echo "0")
echo "   Comments only:     $COMMENTS"

# Files affected
echo ""
echo "📄 Top 10 files needing cleanup:"
awk -F: '{print $1}' "$LEGACY_FILE" | \
  sed 's|'"$PROJECT_ROOT"'/||' | \
  sort | uniq -c | sort -rn | head -10 | \
  while read count file; do
    printf "   %3d matches: %s\n" "$count" "$file"
  done

# Crates affected
echo ""
echo "📦 Patterns by crate:"
awk -F: '{print $1}' "$LEGACY_FILE" | \
  awk -F/ '{print $(NF-2)}' | \
  sort | uniq -c | sort -rn | \
  while read count crate; do
    printf "   %-30s %3d patterns\n" "$crate" "$count"
  done

# Generate detailed report
REPORT_FILE="$REPORT_DIR/legacy_audit_$TIMESTAMP.txt"
echo ""
echo "📝 Generating detailed report: $REPORT_FILE"

{
  echo "Songbird Legacy Pattern Audit"
  echo "Generated: $(date)"
  echo ""
  echo "========================================"
  echo ""
  echo "SUMMARY:"
  echo "--------"
  echo "Total matches: $TOTAL_MATCHES"
  echo "Re-exports: $REEXPORTS"
  echo "Wrapper structs: $WRAPPERS"
  echo "Migration functions: $FUNCTIONS"
  echo "Comments only: $COMMENTS"
  echo ""
  echo "========================================"
  echo ""
  echo "ALL MATCHES:"
  echo "------------"
  cat "$LEGACY_FILE" | sed 's|'"$PROJECT_ROOT"'/||'
  echo ""
  echo "========================================"
  echo ""
  echo "CLEANUP RECOMMENDATIONS:"
  echo "------------------------"
  echo ""
  echo "1. RE-EXPORTS (Priority: HIGH)"
  echo "   Remove 'pub use' statements that expose legacy types"
  echo "   Update consumers to use modern types directly"
  echo ""
  echo "2. WRAPPER STRUCTS (Priority: HIGH)"
  echo "   Remove adapter/wrapper structs"
  echo "   Implement modern traits directly"
  echo ""
  echo "3. MIGRATION FUNCTIONS (Priority: MEDIUM)"
  echo "   Remove temporary migration helpers"
  echo "   Use From/Into trait implementations"
  echo ""
  echo "4. COMMENTS (Priority: LOW)"
  echo "   Update or remove legacy-related comments"
  echo "   Update documentation to reflect modern patterns"
  echo ""
} > "$REPORT_FILE"

echo "✅ Detection complete!"
echo ""
echo "📄 Full report saved to: $REPORT_FILE"
echo ""

# Extract specific patterns for manual review
echo "🎯 SPECIFIC PATTERNS TO REVIEW:"
echo ""

echo "   1. Discovery crate (highest impact):"
grep "songbird-discovery" "$LEGACY_FILE" | wc -l | \
  xargs -I {} echo "      {} patterns found"

echo ""
echo "   2. Universal crate adapters:"
grep "songbird-universal.*adapter" "$LEGACY_FILE" | wc -l | \
  xargs -I {} echo "      {} patterns found"

echo ""
echo "   3. Orchestrator BiomeOS wrappers:"
grep "songbird-orchestrator.*biomeos" "$LEGACY_FILE" | wc -l | \
  xargs -I {} echo "      {} patterns found"

echo ""
echo "🎯 RECOMMENDED ACTIONS:"
echo "   1. Review full report: cat $REPORT_FILE"
echo "   2. Start with songbird-discovery (highest count)"
echo "   3. Use scripts/cleanup_legacy_file.sh <file> for systematic cleanup"
echo "   4. Test after each batch of changes"
echo ""

