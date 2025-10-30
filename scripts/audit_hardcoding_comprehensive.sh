#!/bin/bash
# Comprehensive Hardcoding Audit Script
# Generated: October 30, 2025
# Purpose: Identify all hardcoded values for migration

set -e

REPORT_FILE="HARDCODING_AUDIT_REPORT_$(date +%Y%m%d_%H%M%S).md"

echo "# 🔍 Hardcoding Audit Report" > "$REPORT_FILE"
echo "**Generated**: $(date)" >> "$REPORT_FILE"
echo "**Location**: Songbird Codebase" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Count functions
count_and_report() {
    local pattern="$1"
    local description="$2"
    local path="${3:-crates/*/src}"
    
    echo "Scanning: $description..."
    local count=$(grep -r "$pattern" $path --include="*.rs" 2>/dev/null | wc -l)
    echo "- **$description**: $count instances" >> "$REPORT_FILE"
    
    if [ $count -gt 0 ]; then
        echo "" >> "$REPORT_FILE"
        echo "### Details: $description" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        grep -rn "$pattern" $path --include="*.rs" 2>/dev/null | head -20 >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        echo "" >> "$REPORT_FILE"
    fi
}

echo "## 📊 Hardcoding Summary" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Port Numbers
count_and_report ":\s*[0-9]{4,5}\b" "Port Numbers"

# IP Addresses
count_and_report "127\.0\.0\.1\|localhost\|0\.0\.0\.0" "Localhost/IP Addresses"

# Primal Names
count_and_report "beardog\|toadstool\|squirrel\|nestgate" "Primal Service Names"

# HTTP/HTTPS URLs
count_and_report "http://\|https://" "Hardcoded URLs"

# Common Constants
count_and_report "const\s\+[A-Z_]+:\s*u" "Numeric Constants"
count_and_report "const\s\+[A-Z_]+:\s*&str\s*=\s*\"" "String Constants"

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "## 🎯 Migration Priority" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "### P0 - Critical (This Week)" >> "$REPORT_FILE"
echo "1. Production code port hardcoding" >> "$REPORT_FILE"
echo "2. Service endpoint hardcoding" >> "$REPORT_FILE"
echo "3. Primal name hardcoding in adapters" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "### P1 - High (Next 2 Weeks)" >> "$REPORT_FILE"
echo "1. Configuration default values" >> "$REPORT_FILE"
echo "2. Test helper hardcoding" >> "$REPORT_FILE"
echo "3. URL construction patterns" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "### P2 - Medium (Month 1)" >> "$REPORT_FILE"
echo "1. Constant consolidation" >> "$REPORT_FILE"
echo "2. Environment variable migration" >> "$REPORT_FILE"
echo "3. Documentation updates" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "## 🔧 Recommended Actions" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "1. **Create configuration abstraction layer**" >> "$REPORT_FILE"
echo "   - Centralize all constants in config module" >> "$REPORT_FILE"
echo "   - Use environment variables with defaults" >> "$REPORT_FILE"
echo "   - Implement discovery-first approach" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "2. **Implement capability-based discovery**" >> "$REPORT_FILE"
echo "   - Replace hardcoded service names with capability queries" >> "$REPORT_FILE"
echo "   - Use service registry for endpoint resolution" >> "$REPORT_FILE"
echo "   - Dynamic port allocation where possible" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "3. **Add configuration validation**" >> "$REPORT_FILE"
echo "   - Validate all environment variables at startup" >> "$REPORT_FILE"
echo "   - Provide helpful error messages for missing config" >> "$REPORT_FILE"
echo "   - Document all configuration options" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "## 📈 Progress Tracking" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "- [ ] Week 1: Reduce by 25% (1,577 → 1,183)" >> "$REPORT_FILE"
echo "- [ ] Week 2: Reduce by 50% (1,183 → 789)" >> "$REPORT_FILE"
echo "- [ ] Week 4: Reduce by 75% (789 → 394)" >> "$REPORT_FILE"
echo "- [ ] Week 6: Reduce by 90% (394 → 157)" >> "$REPORT_FILE"
echo "- [ ] Week 8: Reduce to <100 (157 → <100)" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "**Report Generated**: $(date)" >> "$REPORT_FILE"
echo "**Next Run**: Run weekly to track progress" >> "$REPORT_FILE"

echo ""
echo "✅ Hardcoding audit complete!"
echo "📄 Report saved to: $REPORT_FILE"
echo ""
echo "Next steps:"
echo "1. Review the report: cat $REPORT_FILE"
echo "2. Prioritize P0 items for this week"
echo "3. Create migration plan for top 20 instances"
echo ""

