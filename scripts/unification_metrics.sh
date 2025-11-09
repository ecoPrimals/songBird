#!/bin/bash
# Songbird Unification Metrics Tracker
# Tracks progress on technical debt elimination
# Usage: ./scripts/unification_metrics.sh [--json]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

OUTPUT_JSON=false
if [ "$1" = "--json" ]; then
    OUTPUT_JSON=true
fi

# Colors for terminal output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to count and evaluate against target
count_metric() {
    local name="$1"
    local command="$2"
    local target="$3"
    local direction="$4"  # "lower" or "higher"
    
    local count=$(eval "$command" 2>/dev/null || echo "0")
    
    if [ "$OUTPUT_JSON" = true ]; then
        echo "  \"$name\": { \"count\": $count, \"target\": $target },"
    else
        local color="$NC"
        local status="◯"
        
        if [ "$direction" = "lower" ]; then
            if [ "$count" -le "$target" ]; then
                color="$GREEN"
                status="✅"
            elif [ "$count" -le $((target * 2)) ]; then
                color="$YELLOW"
                status="🟡"
            else
                color="$RED"
                status="🔴"
            fi
        else
            if [ "$count" -ge "$target" ]; then
                color="$GREEN"
                status="✅"
            elif [ "$count" -ge $((target / 2)) ]; then
                color="$YELLOW"
                status="🟡"
            else
                color="$RED"
                status="🔴"
            fi
        fi
        
        printf "${color}${status} %-40s %5d / %-5d${NC}\n" "$name" "$count" "$target"
    fi
}

if [ "$OUTPUT_JSON" = true ]; then
    echo "{"
    echo "  \"timestamp\": \"$(date -Iseconds)\","
    echo "  \"metrics\": {"
else
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║     SONGBIRD UNIFICATION METRICS REPORT                    ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "📅 Date: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "📂 Project: $PROJECT_ROOT"
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo " Status  Metric                                 Current / Target"
    echo "════════════════════════════════════════════════════════════"
fi

# Core Metrics
count_metric "Config Structs" \
    "grep -r 'struct.*Config' --include='*.rs' crates/*/src | wc -l" \
    "50" "lower"

count_metric "Legacy Patterns (legacy|shim|wrapper|compat)" \
    "grep -ri 'legacy\|shim\|wrapper\|compat' --include='*.rs' crates/*/src | wc -l" \
    "0" "lower"

count_metric "Deprecated Items (#[deprecated])" \
    "grep -r '#\[deprecated' --include='*.rs' crates/*/src | wc -l" \
    "0" "lower"

count_metric "Error Enums (pub enum.*Error)" \
    "grep -r 'pub enum.*Error' --include='*.rs' crates/ | wc -l" \
    "3" "lower"

count_metric "Provider Traits (pub trait.*Provider)" \
    "grep -r 'pub trait.*Provider' --include='*.rs' crates/*/src | grep -v test | wc -l" \
    "10" "lower"

count_metric "Result Type Aliases" \
    "grep -r 'pub type.*Result.*=.*Result<' --include='*.rs' crates/ | wc -l" \
    "1" "lower"

count_metric "Constants (const [A-Z])" \
    "grep -r 'const ' --include='*.rs' crates/ | grep -v '//' | wc -l" \
    "50" "lower"

count_metric "Files Over 2000 Lines" \
    "find crates/ -name '*.rs' -exec wc -l {} + | grep -v total | awk '\$1 > 2000' | wc -l" \
    "0" "lower"

# Additional Insights
if [ "$OUTPUT_JSON" = false ]; then
    echo "════════════════════════════════════════════════════════════"
    echo ""
    echo "📊 Additional Insights:"
    echo ""
    
    TOTAL_RS_FILES=$(find crates/ -name '*.rs' | wc -l)
    TOTAL_LINES=$(find crates/ -name '*.rs' -exec wc -l {} + | tail -1 | awk '{print $1}')
    AVG_LINES=$((TOTAL_LINES / TOTAL_RS_FILES))
    
    echo "   📄 Total Rust Files: $TOTAL_RS_FILES"
    echo "   📏 Total Lines of Code: $(printf "%'d" $TOTAL_LINES)"
    echo "   📐 Average Lines per File: $AVG_LINES"
    echo ""
    
    # Top 5 largest files
    echo "   📦 Largest Files (Top 5):"
    find crates/ -name '*.rs' -exec wc -l {} + | sort -rn | head -6 | tail -5 | \
        awk '{ printf "      %5d lines: %s\n", $1, $2 }'
    echo ""
    
    # Files with most legacy patterns
    echo "   ⚠️  Files with Most Legacy Patterns (Top 5):"
    for file in $(grep -ri 'legacy\|shim\|wrapper\|compat' --include='*.rs' crates/*/src -l 2>/dev/null | head -5); do
        count=$(grep -i 'legacy\|shim\|wrapper\|compat' "$file" 2>/dev/null | wc -l)
        printf "      %3d patterns: %s\n" "$count" "$file"
    done
    echo ""
    
    # Crates with most configs
    echo "   🔧 Crates with Most Config Structs (Top 5):"
    for crate_dir in crates/*/; do
        crate=$(basename "$crate_dir")
        count=$(grep -r 'struct.*Config' --include='*.rs' "$crate_dir/src" 2>/dev/null | wc -l)
        if [ "$count" -gt 0 ]; then
            echo "$count $crate"
        fi
    done | sort -rn | head -5 | awk '{ printf "      %3d configs: %s\n", $1, $2 }'
    echo ""
    
    echo "════════════════════════════════════════════════════════════"
    echo ""
    echo "💡 Quick Actions:"
    echo ""
    echo "   1. Remove deprecated items:"
    echo "      grep -r '#\[deprecated' --include='*.rs' crates/ -l"
    echo ""
    echo "   2. Find legacy patterns:"
    echo "      ./scripts/detect_legacy.sh"
    echo ""
    echo "   3. See config duplication:"
    echo "      ./scripts/audit_configs.sh"
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo ""
else
    # Close JSON
    echo "  }"
    echo "}"
fi

