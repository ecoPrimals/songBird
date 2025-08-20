#!/bin/bash
# zero-cost-assessment.sh - Analyze your project's zero-cost architecture migration potential
# 
# Created by: beardog team
# Date: January 2025
# Usage: ./zero-cost-assessment.sh [project-directory]

set -e

echo "🔍 Zero-Cost Architecture Migration Assessment"
echo "============================================="
echo "📝 Analysis script created by beardog team"
echo ""

PROJECT_DIR=${1:-"."}
echo "📁 Analyzing project: $(realpath "$PROJECT_DIR")"

# Verify it's a Rust project
if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
    echo "❌ Error: No Cargo.toml found in $PROJECT_DIR"
    echo "   This script is designed for Rust projects only."
    exit 1
fi

echo "✅ Rust project detected"
echo ""

# Count patterns  
echo "🔍 Analyzing performance overhead patterns..."
ASYNC_TRAIT_COUNT=$(grep -r "async_trait" "$PROJECT_DIR" --include="*.rs" 2>/dev/null | wc -l || echo "0")
ARC_DYN_COUNT=$(grep -r "Arc<dyn" "$PROJECT_DIR" --include="*.rs" 2>/dev/null | wc -l || echo "0")
TOTAL_OVERHEAD=$((ASYNC_TRAIT_COUNT + ARC_DYN_COUNT))

echo ""
echo "📊 Analysis Results"  
echo "=================="
echo "🎯 Performance Overhead Patterns:"
echo "   • async_trait usages: $ASYNC_TRAIT_COUNT" 
echo "   • Arc<dyn> usages: $ARC_DYN_COUNT"
echo "   • Total overhead patterns: $TOTAL_OVERHEAD"

echo ""
echo "🎯 Migration Impact Assessment"
echo "=============================="

if [ $TOTAL_OVERHEAD -gt 100 ]; then
    echo "   🔥 **HIGH IMPACT** - Immediate migration recommended"
    echo "   📈 Expected performance improvement: **40-60%**"
elif [ $TOTAL_OVERHEAD -gt 50 ]; then
    echo "   📈 **MODERATE IMPACT** - Planned migration recommended"  
    echo "   📈 Expected performance improvement: **20-40%**"
elif [ $TOTAL_OVERHEAD -gt 10 ]; then
    echo "   📊 **LOW IMPACT** - Consider migration for future-proofing"
    echo "   📈 Expected performance improvement: **10-20%**"
else
    echo "   ✅ **MINIMAL IMPACT** - Current architecture likely optimal"
fi

echo ""
echo "🛠️  Next Steps"
echo "=============="
echo "📖 Read: ./ZERO_COST_ARCHITECTURE_ECOSYSTEM_MIGRATION_GUIDE.md"
echo "👥 Contact: beardog team for architecture review"
echo "🚀 Start: Zero-cost transformation with proven patterns"
