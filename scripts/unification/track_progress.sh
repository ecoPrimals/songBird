#!/bin/bash
# Unification Progress Tracker - Monitors all unification metrics
# Part of Songbird Unification Initiative - Nov 10, 2025

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║     SONGBIRD UNIFICATION PROGRESS DASHBOARD           ║"
echo "║     $(date +'%Y-%m-%d %H:%M:%S')                          ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Function to calculate percentage
calc_progress() {
    local current=$1
    local target=$2
    local inverse=$3  # if true, progress = (target-current)/target
    
    if [ "$inverse" = "true" ]; then
        echo $(( (target - current) * 100 / target ))
    else
        echo $(( current * 100 / target ))
    fi
}

# Function to show progress bar
progress_bar() {
    local percent=$1
    local width=30
    local filled=$(( percent * width / 100 ))
    local empty=$(( width - filled ))
    
    printf "["
    printf "%${filled}s" | tr ' ' '█'
    printf "%${empty}s" | tr ' ' '░'
    printf "] %3d%%" "$percent"
}

echo "📊 CORE METRICS"
echo "───────────────────────────────────────────────────────"
echo ""

# 1. Config Structs
TOTAL_CONFIGS=$(grep -r "pub struct.*Config" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
CONFIG_TARGET=120
CONFIG_PROGRESS=$(calc_progress "$TOTAL_CONFIGS" 662 true)
echo "Config Consolidation:"
echo "  Current: $TOTAL_CONFIGS structs (target: $CONFIG_TARGET)"
echo "  Progress: $(progress_bar $CONFIG_PROGRESS)"
echo ""

# 2. unwrap() calls
UNWRAPS=$(grep -r "\.unwrap()\|\.expect(" crates/songbird-*/src --include="*.rs" 2>/dev/null | grep -v test | wc -l || echo "0")
UNWRAP_PROGRESS=$(calc_progress "$UNWRAPS" 116 true)
if [ "$UNWRAPS" -eq 0 ]; then
    echo "✅ Production Safety: COMPLETE!"
    echo "  unwrap() calls: 0 🎉"
    echo "  Progress: $(progress_bar 100)"
else
    echo "⚠️  Production Safety:"
    echo "  unwrap() calls: $UNWRAPS (target: 0)"
    echo "  Progress: $(progress_bar $UNWRAP_PROGRESS)"
fi
echo ""

# 3. async_trait usage
ASYNC_TRAIT=$(grep -r "#\[async_trait\]" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
ASYNC_TARGET=15
ASYNC_PROGRESS=$(calc_progress "$ASYNC_TRAIT" 43 true)
echo "Performance Modernization:"
echo "  async_trait instances: $ASYNC_TRAIT (target: $ASYNC_TARGET)"
echo "  Progress: $(progress_bar $ASYNC_PROGRESS)"
echo ""

# 4. Legacy files
LEGACY=$(find crates -name "*.rs" -type f -exec grep -l "legacy\|compat\|shim" {} \; 2>/dev/null | wc -l || echo "0")
LEGACY_PROGRESS=$(calc_progress "$LEGACY" 50 true)
echo "Legacy Code Cleanup:"
echo "  Legacy files: $LEGACY (target: <10)"
echo "  Progress: $(progress_bar $LEGACY_PROGRESS)"
echo ""

# 5. Deprecated items
DEPRECATED=$(grep -r "#\[deprecated" crates --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$DEPRECATED" -eq 0 ]; then
    echo "✅ Deprecated Code: CLEAN!"
    echo "  Deprecated items: 0 🎉"
else
    DEPRECATED_PROGRESS=$(calc_progress "$DEPRECATED" 7 true)
    echo "Deprecated Code:"
    echo "  Deprecated items: $DEPRECATED (target: 0)"
    echo "  Progress: $(progress_bar $DEPRECATED_PROGRESS)"
fi
echo ""

# 6. TODO comments
TODOS=$(grep -r "TODO\|FIXME\|HACK" crates/songbird-*/src --include="*.rs" 2>/dev/null | wc -l || echo "0")
if [ "$TODOS" -eq 0 ]; then
    echo "✅ Code Debt Markers: CLEAN!"
    echo "  TODO/FIXME: 0 🎉"
else
    TODO_PROGRESS=$(calc_progress "$TODOS" 14 true)
    echo "Code Debt Markers:"
    echo "  TODO/FIXME/HACK: $TODOS (target: 0)"
    echo "  Progress: $(progress_bar $TODO_PROGRESS)"
fi
echo ""

echo "───────────────────────────────────────────────────────"
echo ""

# Calculate overall progress
OVERALL=$(( (CONFIG_PROGRESS + UNWRAP_PROGRESS + ASYNC_PROGRESS + LEGACY_PROGRESS) / 4 ))

echo "🎯 OVERALL UNIFICATION PROGRESS"
echo "  $(progress_bar $OVERALL)"
echo ""

# Grade calculation
if [ "$OVERALL" -ge 95 ]; then
    GRADE="A+ (EXCELLENT)"
    COLOR="🟢"
elif [ "$OVERALL" -ge 90 ]; then
    GRADE="A (VERY GOOD)"
    COLOR="🟢"
elif [ "$OVERALL" -ge 85 ]; then
    GRADE="B+ (GOOD)"
    COLOR="🟡"
elif [ "$OVERALL" -ge 80 ]; then
    GRADE="B (SATISFACTORY)"
    COLOR="🟡"
else
    GRADE="C (NEEDS WORK)"
    COLOR="🔴"
fi

echo "Current Grade: $COLOR $GRADE"
echo ""

# Build health
echo "───────────────────────────────────────────────────────"
echo "🔨 BUILD HEALTH"
echo "───────────────────────────────────────────────────────"
echo ""

# Check if cargo is available
if command -v cargo &> /dev/null; then
    echo "Running cargo check..."
    if cargo check --workspace --quiet 2>&1 | grep -q "error\|warning"; then
        echo "⚠️  Compilation issues detected"
        echo "   Run 'cargo check --workspace' for details"
    else
        echo "✅ Compilation: CLEAN"
    fi
else
    echo "⚠️  Cargo not available, skipping build check"
fi
echo ""

# File size compliance
MAX_FILE_LINES=$(find crates -name "*.rs" -type f -exec wc -l {} + 2>/dev/null | sort -rn | head -1 | awk '{print $1}' || echo "0")
if [ "$MAX_FILE_LINES" -lt 2000 ]; then
    echo "✅ File Size Compliance: EXCELLENT"
    echo "   Largest file: $MAX_FILE_LINES lines (limit: 2000)"
else
    echo "⚠️  File Size Compliance: NEEDS ATTENTION"
    echo "   Largest file: $MAX_FILE_LINES lines (limit: 2000)"
fi
echo ""

echo "───────────────────────────────────────────────────────"
echo "📈 TREND ANALYSIS"
echo "───────────────────────────────────────────────────────"
echo ""

# Save current metrics to history file
HISTORY_FILE=".unification_history.csv"
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,configs,unwraps,async_trait,legacy,deprecated,todos,overall" > "$HISTORY_FILE"
fi

echo "$(date +%s),$TOTAL_CONFIGS,$UNWRAPS,$ASYNC_TRAIT,$LEGACY,$DEPRECATED,$TODOS,$OVERALL" >> "$HISTORY_FILE"

# Show last 5 entries
if [ $(wc -l < "$HISTORY_FILE") -gt 6 ]; then
    echo "Recent Progress (last 5 measurements):"
    echo ""
    echo "Date                 Configs  Unwraps  AsyncTrait  Overall"
    echo "─────────────────────────────────────────────────────────"
    tail -5 "$HISTORY_FILE" | while IFS=, read -r ts configs unwraps async legacy depr todos overall; do
        if [ "$ts" != "timestamp" ]; then
            date_str=$(date -d "@$ts" "+%Y-%m-%d %H:%M" 2>/dev/null || echo "N/A")
            printf "%-20s %-8s %-8s %-11s %s%%\n" "$date_str" "$configs" "$unwraps" "$async" "$overall"
        fi
    done
else
    echo "Tracking started. Run this script regularly to see trends."
fi
echo ""

echo "───────────────────────────────────────────────────────"
echo "🎯 NEXT ACTIONS"
echo "───────────────────────────────────────────────────────"
echo ""

# Generate recommendations based on metrics
if [ "$UNWRAPS" -gt 0 ]; then
    echo "🚨 CRITICAL: Eliminate $UNWRAPS unwrap() calls"
    echo "   Run: ./scripts/unification/02_eliminate_unwraps.sh"
    echo ""
fi

if [ "$TOTAL_CONFIGS" -gt 200 ]; then
    echo "📊 HIGH: Config consolidation needed ($TOTAL_CONFIGS → $CONFIG_TARGET)"
    echo "   Run: ./scripts/unification/01_audit_configs.sh"
    echo ""
fi

if [ "$ASYNC_TRAIT" -gt 20 ]; then
    echo "⚡ MEDIUM: async_trait optimization opportunity"
    echo "   Run: ./scripts/unification/03_analyze_async_trait.sh"
    echo ""
fi

if [ "$LEGACY" -gt 10 ]; then
    echo "🧹 LOW: Legacy code cleanup recommended"
    echo "   Review: FILES_WITH_LEGACY_PATTERNS.txt"
    echo ""
fi

if [ "$OVERALL" -ge 95 ]; then
    echo "🎉 CONGRATULATIONS!"
    echo "   Songbird has achieved 95%+ unification!"
    echo "   Consider documenting achievements and lessons learned."
fi

echo "───────────────────────────────────────────────────────"
echo ""
echo "Report generated: $(date)"
echo "Re-run this script weekly to track progress."
echo ""

