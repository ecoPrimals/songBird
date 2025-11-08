#!/bin/bash
# Technical Debt Metrics Tracker
# Run this before and after cleanup to measure progress

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════════════════════════"
echo "  SONGBIRD TECHNICAL DEBT METRICS"
echo "  Generated: $(date '+%Y-%m-%d %H:%M:%S')"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print metric with status
print_metric() {
    local name="$1"
    local value="$2"
    local target="$3"
    local status=""
    
    if [ "$value" -le "$target" ]; then
        status="${GREEN}✓${NC}"
    else
        status="${RED}✗${NC}"
    fi
    
    printf "%-45s %6s  (target: ≤ %s) %b\n" "$name:" "$value" "$target" "$status"
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "FILE SIZE COMPLIANCE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
FILES_OVER_2000=$(find crates -name "*.rs" -type f -exec wc -l {} + 2>/dev/null | awk '$1 > 2000 {count++} END {print count+0}')
print_metric "Files exceeding 2000 lines" "$FILES_OVER_2000" "0"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "DEPRECATED IMPORTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
DEPRECATED_CONSTANTS=$(grep -r "use songbird_config::config::constants::" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "config::constants imports (deprecated)" "$DEPRECATED_CONSTANTS" "0"

DEPRECATED_CONFIG=$(grep -r "use songbird_config::config::" crates/ --include="*.rs" | grep -v "canonical" 2>/dev/null | wc -l || echo 0)
print_metric "config::* imports (non-canonical)" "$DEPRECATED_CONFIG" "0"

DEPRECATED_UNIFIED=$(grep -r "use songbird_config::unified::" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "unified::* imports (potential overlap)" "$DEPRECATED_UNIFIED" "50"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "ERROR HANDLING PATTERNS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
UNWRAP_CALLS=$(grep -r "\.unwrap()" crates/ --include="*.rs" | grep -v "test" | grep -v "//" 2>/dev/null | wc -l || echo 0)
print_metric "Production .unwrap() calls" "$UNWRAP_CALLS" "50"

EXPECT_CALLS=$(grep -r "\.expect(" crates/ --include="*.rs" | grep -v "test" | grep -v "//" 2>/dev/null | wc -l || echo 0)
print_metric "Production .expect() calls" "$EXPECT_CALLS" "30"

UNWRAP_DATA=$(grep -r "\.unwrap_data\(\)" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "Deprecated .unwrap_data() calls" "$UNWRAP_DATA" "0"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "TECHNICAL DEBT MARKERS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TODO_COUNT=$(grep -r "TODO" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "TODO comments" "$TODO_COUNT" "100"

FIXME_COUNT=$(grep -r "FIXME" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "FIXME comments" "$FIXME_COUNT" "20"

XXX_COUNT=$(grep -r "XXX" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "XXX comments" "$XXX_COUNT" "10"

DEPRECATED_ATTRS=$(grep -r "#\[deprecated" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
print_metric "Deprecated attributes" "$DEPRECATED_ATTRS" "20"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "DEPRECATED MODULES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check for specific deprecated primals
BEARDOG_USES=$(grep -r "BearDogPrimal\|use.*beardog::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/beardog.rs" | \
    grep -v "^//" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "BearDogPrimal uses (should archive if 0):" "$BEARDOG_USES"

TOADSTOOL_USES=$(grep -r "ToadstoolPrimal\|use.*toadstool::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/toadstool.rs" | \
    grep -v "^//" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "ToadstoolPrimal uses (should archive if 0):" "$TOADSTOOL_USES"

SQUIRREL_USES=$(grep -r "SquirrelPrimal\|use.*squirrel::" crates/ --include="*.rs" | \
    grep -v "crates/songbird-primal-sdk/src/squirrel.rs" | \
    grep -v "^//" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "SquirrelPrimal uses (should archive if 0):" "$SQUIRREL_USES"

# Check for archived directories
if [ -d "crates/songbird-config/src/_archived_q2_2026" ]; then
    echo -e "${YELLOW}⚠${NC}  Q2 2026 archive directory still exists (can be removed)"
else
    echo -e "${GREEN}✓${NC}  Q2 2026 archive directory removed"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CONFIGURATION STRUCTURE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

CANONICAL_CONFIGS=$(find crates/songbird-config/src/canonical -name "*.rs" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Config files in canonical/ (modern):" "$CANONICAL_CONFIGS"

UNIFIED_CONFIGS=$(find crates/songbird-config/src/unified -name "*.rs" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Config files in unified/ (audit needed):" "$UNIFIED_CONFIGS"

LEGACY_CONFIGS=$(find crates/songbird-config/src/config -name "*.rs" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Config files in config/ (deprecated):" "$LEGACY_CONFIGS"

TOTAL_CONFIG_STRUCTS=$(grep -r "pub struct.*Config" crates/songbird-config/src --include="*.rs" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Total Config struct definitions:" "$TOTAL_CONFIG_STRUCTS"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "TRAIT SYSTEM"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

CANONICAL_TRAIT_USES=$(grep -r "use songbird_types::traits::canonical" crates/ --include="*.rs" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Canonical trait imports (good):" "$CANONICAL_TRAIT_USES"

PROVIDER_TRAIT_DEFS=$(grep -r "pub trait.*Provider" crates/ --include="*.rs" | \
    grep -v "songbird-types/src/traits" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s  (target: ≤ 5)\n" "Provider trait definitions (non-canonical):" "$PROVIDER_TRAIT_DEFS"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BUILD HEALTH"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Try to count crates
TOTAL_CRATES=$(find crates -name "Cargo.toml" 2>/dev/null | wc -l || echo 0)
printf "%-45s %6s\n" "Total crates in workspace:" "$TOTAL_CRATES"

# Check if cargo build works (don't actually run it, just report)
echo ""
echo "To verify build health, run:"
echo "  cargo build --workspace"
echo "  cargo test --workspace"
echo "  cargo clippy --workspace -- -D warnings"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SUMMARY SCORE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Calculate a simple health score
SCORE=100

# Deduct points for various issues
[ "$FILES_OVER_2000" -gt 0 ] && SCORE=$((SCORE - FILES_OVER_2000 * 5))
[ "$DEPRECATED_CONSTANTS" -gt 0 ] && SCORE=$((SCORE - DEPRECATED_CONSTANTS / 10))
[ "$UNWRAP_DATA" -gt 0 ] && SCORE=$((SCORE - UNWRAP_DATA * 2))
[ "$TODO_COUNT" -gt 100 ] && SCORE=$((SCORE - (TODO_COUNT - 100) / 20))
[ "$FIXME_COUNT" -gt 20 ] && SCORE=$((SCORE - (FIXME_COUNT - 20)))

# Cap at 0
[ "$SCORE" -lt 0 ] && SCORE=0

# Display score with color
if [ "$SCORE" -ge 90 ]; then
    echo -e "${GREEN}★ Technical Debt Score: $SCORE/100 - EXCELLENT ${NC}"
elif [ "$SCORE" -ge 75 ]; then
    echo -e "${GREEN}✓ Technical Debt Score: $SCORE/100 - GOOD ${NC}"
elif [ "$SCORE" -ge 60 ]; then
    echo -e "${YELLOW}⚠ Technical Debt Score: $SCORE/100 - NEEDS IMPROVEMENT ${NC}"
else
    echo -e "${RED}✗ Technical Debt Score: $SCORE/100 - NEEDS ATTENTION ${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "RECOMMENDATIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$DEPRECATED_CONSTANTS" -gt 50 ]; then
    echo "• HIGH PRIORITY: Migrate $DEPRECATED_CONSTANTS deprecated constant imports"
fi

if [ "$UNWRAP_DATA" -gt 0 ]; then
    echo "• Update $UNWRAP_DATA unwrap_data() calls to modern pattern"
fi

if [ "$BEARDOG_USES" -eq 0 ] && [ -f "crates/songbird-primal-sdk/src/beardog.rs" ]; then
    echo "• Archive beardog.rs (no active users)"
fi

if [ "$TOADSTOOL_USES" -eq 0 ] && [ -f "crates/songbird-primal-sdk/src/toadstool.rs" ]; then
    echo "• Archive toadstool.rs (no active users)"
fi

if [ "$SQUIRREL_USES" -eq 0 ] && [ -f "crates/songbird-primal-sdk/src/squirrel.rs" ]; then
    echo "• Archive squirrel.rs (no active users)"
fi

if [ "$UNIFIED_CONFIGS" -gt 10 ]; then
    echo "• Audit unified/ config files vs canonical/ for duplicates"
fi

if [ "$TODO_COUNT" -gt 200 ]; then
    echo "• Convert TODOs to tracked issues or remove obsolete ones"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Metrics collection complete"
echo "  Save this output to track progress over time"
echo "═══════════════════════════════════════════════════════════════"

