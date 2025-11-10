#!/bin/bash
# Find Duplicate Config/Type/Trait Definitions
# Part of Songbird Unification Toolset

set -euo pipefail

echo "🔍 Songbird Duplicate Definition Finder"
echo "========================================"
echo ""

REPORT_FILE="DUPLICATE_DEFINITIONS_REPORT.md"

# Initialize report
cat > "$REPORT_FILE" << 'EOF'
# Duplicate Definitions Report
**Generated**: $(date)
**Purpose**: Identify exact duplicates for consolidation

---

## Config Struct Duplicates

EOF

echo "📊 Finding duplicate Config structs..."

# Find all config struct names and count occurrences
grep -rh "pub struct.*Config" crates --include="*.rs" | \
    sed 's/.*pub struct \([^ {<]*\).*/\1/' | \
    sort | uniq -c | sort -rn | \
    awk '$1 > 1 {print}' > /tmp/duplicate_configs.txt

if [ -s /tmp/duplicate_configs.txt ]; then
    DUPLICATE_COUNT=$(wc -l < /tmp/duplicate_configs.txt)
    echo "Found $DUPLICATE_COUNT config names with multiple definitions"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Summary
EOF
    echo "" >> "$REPORT_FILE"
    cat /tmp/duplicate_configs.txt >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Detailed Locations
EOF
    echo "" >> "$REPORT_FILE"
    
    # For each duplicate, show all locations
    while read count name; do
        echo "" >> "$REPORT_FILE"
        echo "#### $name ($count definitions)" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        grep -rn "pub struct $name" crates --include="*.rs" | grep -v "test" >> "$REPORT_FILE" || true
        echo '```' >> "$REPORT_FILE"
    done < /tmp/duplicate_configs.txt
else
    echo "✅ No duplicate config structs found"
    echo "No duplicate config names found." >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Find duplicate traits
echo "" 
echo "📊 Finding duplicate trait definitions..."

cat >> "$REPORT_FILE" << 'EOF'
## Trait Duplicates

EOF

grep -rh "pub trait [A-Z]" crates --include="*.rs" | \
    sed 's/.*pub trait \([^ {<:]*\).*/\1/' | \
    sort | uniq -c | sort -rn | \
    awk '$1 > 1 {print}' > /tmp/duplicate_traits.txt

if [ -s /tmp/duplicate_traits.txt ]; then
    DUPLICATE_COUNT=$(wc -l < /tmp/duplicate_traits.txt)
    echo "Found $DUPLICATE_COUNT trait names with multiple definitions"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Summary
EOF
    echo "" >> "$REPORT_FILE"
    cat /tmp/duplicate_traits.txt >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Detailed Locations
EOF
    echo "" >> "$REPORT_FILE"
    
    # For each duplicate, show all locations
    while read count name; do
        echo "" >> "$REPORT_FILE"
        echo "#### $name ($count definitions)" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        grep -rn "pub trait $name" crates --include="*.rs" | grep -v "test" >> "$REPORT_FILE" || true
        echo '```' >> "$REPORT_FILE"
    done < /tmp/duplicate_traits.txt
else
    echo "✅ No duplicate trait definitions found"
    echo "No duplicate trait names found." >> "$REPORT_FILE"
fi

echo "" >> "$REPORT_FILE"
echo "---" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Find duplicate error types
echo ""
echo "📊 Finding duplicate error types..."

cat >> "$REPORT_FILE" << 'EOF'
## Error Type Duplicates

EOF

grep -rh "pub enum.*Error" crates --include="*.rs" | \
    sed 's/.*pub enum \([^ {<]*\).*/\1/' | \
    sort | uniq -c | sort -rn | \
    awk '$1 > 1 {print}' > /tmp/duplicate_errors.txt

if [ -s /tmp/duplicate_errors.txt ]; then
    DUPLICATE_COUNT=$(wc -l < /tmp/duplicate_errors.txt)
    echo "Found $DUPLICATE_COUNT error names with multiple definitions"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Summary
EOF
    echo "" >> "$REPORT_FILE"
    cat /tmp/duplicate_errors.txt >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    
    cat >> "$REPORT_FILE" << 'EOF'
### Detailed Locations
EOF
    echo "" >> "$REPORT_FILE"
    
    # For each duplicate, show all locations
    while read count name; do
        echo "" >> "$REPORT_FILE"
        echo "#### $name ($count definitions)" >> "$REPORT_FILE"
        echo '```' >> "$REPORT_FILE"
        grep -rn "pub enum $name" crates --include="*.rs" | grep -v "test" >> "$REPORT_FILE" || true
        echo '```' >> "$REPORT_FILE"
    done < /tmp/duplicate_errors.txt
else
    echo "✅ No duplicate error types found"
    echo "No duplicate error names found." >> "$REPORT_FILE"
fi

# Add recommendations
cat >> "$REPORT_FILE" << 'EOF'

---

## Recommendations

### Config Consolidation Process
1. For each duplicate config name above:
   - Identify the canonical version (usually in `canonical/` directory)
   - Compare definitions to ensure they're actually duplicates
   - Update all imports to use canonical version
   - Remove duplicate definitions

### Trait Consolidation Process
1. For each duplicate trait:
   - Determine if it should be in `songbird-types/src/traits/canonical.rs`
   - Or if it's domain-specific and belongs in the domain crate
   - Consolidate or clarify naming to indicate purpose

### Error Consolidation Process
1. Most errors should use `SongbirdError` from `songbird-types`
2. Domain-specific errors should be clearly named and documented
3. Consider if error variants can be added to canonical error instead

---

**Next Steps**: Review this report and create consolidation plan for each duplicate.
EOF

# Cleanup
rm -f /tmp/duplicate_configs.txt /tmp/duplicate_traits.txt /tmp/duplicate_errors.txt

echo ""
echo "✅ Report generated: $REPORT_FILE"
echo ""
echo "📈 Summary:"
echo "   - Review the report to identify true duplicates"
echo "   - Use CONFIG_CONSOLIDATION_PLAN.md for step-by-step consolidation"
echo "   - Track progress with ./scripts/unification/track_progress.sh"
echo ""

