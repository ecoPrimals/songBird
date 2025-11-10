#!/bin/bash
# Config Audit Script - Generates inventory of all config structs
# Part of Songbird Unification Initiative - Nov 10, 2025

set -e

OUTPUT_FILE="CONFIG_INVENTORY.md"

echo "🔍 Auditing Songbird configuration structures..."
echo ""

# Create header
cat > "$OUTPUT_FILE" << 'EOF'
# Songbird Configuration Inventory
**Generated**: $(date)  
**Total Configs**: TBD  
**Status**: 🔄 Audit in progress

---

## 📊 Configuration Definitions by Location

EOF

# Find all config structs
echo "Scanning for 'pub struct.*Config' patterns..."
echo "" >> "$OUTPUT_FILE"
echo "### All Config Definitions" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

find crates -name "*.rs" -type f -exec grep -Hn "pub struct.*Config" {} \; | \
  sort | while IFS=: read -r file line content; do
    config_name=$(echo "$content" | sed 's/.*pub struct \([^ {<]*\).*/\1/')
    echo "- [ ] **$config_name** - \`$file:$line\` - [TAG NEEDED]" >> "$OUTPUT_FILE"
  done

# Count total
TOTAL=$(grep -c "^- \[" "$OUTPUT_FILE" || echo "0")

# Update header
sed -i "s/TBD/$TOTAL/g" "$OUTPUT_FILE"

# Add statistics section
cat >> "$OUTPUT_FILE" << EOF

---

## 📈 Statistics

- **Total configs found**: $TOTAL
- **Target after consolidation**: ~120 (80% reduction)

---

## 🏷️ Categorization Tags

Please review each config and add one of these tags:

- **[CANONICAL]** - Already in canonical location, keep as-is
- **[MIGRATE]** - Move to canonical location in songbird-types/src/config/
- **[DUPLICATE]** - Exact duplicate, remove and update imports
- **[DOMAIN]** - Domain-specific, document and keep
- **[REMOVE]** - Unused/deprecated, safe to delete

---

## 📋 Next Steps

1. Review each config definition above
2. Add appropriate tag in brackets
3. Run \`02_consolidate_configs.sh\` to execute migrations
4. Verify with \`cargo check --workspace\`

EOF

# Generate per-crate breakdown
echo "" >> "$OUTPUT_FILE"
echo "### Breakdown by Crate" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

for crate_dir in crates/songbird-*/; do
    if [ -d "$crate_dir" ]; then
        crate_name=$(basename "$crate_dir")
        count=$(find "$crate_dir" -name "*.rs" -exec grep -h "pub struct.*Config" {} \; 2>/dev/null | wc -l || echo "0")
        if [ "$count" -gt 0 ]; then
            echo "- **$crate_name**: $count configs" >> "$OUTPUT_FILE"
        fi
    fi
done

echo ""
echo "✅ Audit complete!"
echo "📄 Results written to: $OUTPUT_FILE"
echo "📊 Total configs found: $TOTAL"
echo ""
echo "🎯 Next steps:"
echo "   1. Review $OUTPUT_FILE"
echo "   2. Add [CANONICAL/MIGRATE/DUPLICATE/DOMAIN/REMOVE] tags"
echo "   3. Run ./scripts/unification/02_consolidate_configs.sh"

