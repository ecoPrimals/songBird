#!/bin/bash
# audit_configs.sh - Comprehensive configuration struct audit
# Part of the Unification & Modernization initiative

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$PROJECT_ROOT/reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create reports directory
mkdir -p "$REPORT_DIR"

echo "🔍 Songbird Configuration Audit"
echo "================================"
echo ""

# Count all config structs
echo "📊 Counting configuration structs..."
TOTAL_CONFIGS=$(grep -r "struct.*Config\s*{" "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l)
echo "   Total config structs found: $TOTAL_CONFIGS"

# Find config locations by crate
echo ""
echo "📂 Configuration by crate:"
grep -r "struct.*Config\s*{" "$PROJECT_ROOT/crates"/*/src --include="*.rs" | \
  awk -F: '{print $1}' | \
  xargs -n1 dirname | \
  awk -F/ '{print $(NF-1)}' | \
  sort | uniq -c | sort -rn | \
  while read count crate; do
    printf "   %-30s %3d structs\n" "$crate" "$count"
  done

# Generate detailed report
REPORT_FILE="$REPORT_DIR/config_audit_$TIMESTAMP.txt"
echo ""
echo "📝 Generating detailed report: $REPORT_FILE"

{
  echo "Songbird Configuration Struct Audit"
  echo "Generated: $(date)"
  echo ""
  echo "========================================"
  echo ""
  
  # List all config structs with file locations
  echo "ALL CONFIG STRUCTS:"
  echo "-------------------"
  grep -rn "struct.*Config\s*{" "$PROJECT_ROOT/crates"/*/src --include="*.rs" | \
    sed 's|'"$PROJECT_ROOT"'/||' | \
    sort
  
  echo ""
  echo "========================================"
  echo ""
  echo "SUMMARY BY DOMAIN:"
  echo "------------------"
  
  # Network configs
  echo ""
  echo "NETWORK CONFIGURATION:"
  grep -r "struct.*Network.*Config\|struct.*Peer.*Config\|struct.*Port.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
  # Security configs
  echo ""
  echo "SECURITY CONFIGURATION:"
  grep -r "struct.*Security.*Config\|struct.*Auth.*Config\|struct.*Crypto.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
  # Discovery configs
  echo ""
  echo "DISCOVERY CONFIGURATION:"
  grep -r "struct.*Discovery.*Config\|struct.*Registry.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
  # Service configs
  echo ""
  echo "SERVICE CONFIGURATION:"
  grep -r "struct.*Service.*Config\|struct.*Provider.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
  # Gaming configs
  echo ""
  echo "GAMING CONFIGURATION:"
  grep -r "struct.*Gaming.*Config\|struct.*Game.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
  # Orchestration configs
  echo ""
  echo "ORCHESTRATION CONFIGURATION:"
  grep -r "struct.*Orchestr.*Config\|struct.*Deploy.*Config" \
    "$PROJECT_ROOT/crates"/*/src --include="*.rs" | wc -l | \
    xargs -I {} echo "  {} structs found"
  
} > "$REPORT_FILE"

echo "✅ Audit complete!"
echo ""
echo "📄 Full report saved to: $REPORT_FILE"
echo ""

# Check canonical configs
echo "🎯 Canonical Configuration Status:"
CANONICAL_DIR="$PROJECT_ROOT/crates/songbird-config/src/canonical"
if [ -d "$CANONICAL_DIR" ]; then
  CANONICAL_COUNT=$(find "$CANONICAL_DIR" -name "*.rs" -not -name "mod.rs" | wc -l)
  echo "   Canonical config modules: $CANONICAL_COUNT"
  echo "   Files:"
  find "$CANONICAL_DIR" -name "*.rs" -not -name "mod.rs" | \
    xargs -n1 basename | sed 's/\.rs$//' | sed 's/^/     - /'
else
  echo "   ⚠️  Canonical directory not found"
fi

echo ""
echo "🎯 RECOMMENDED ACTIONS:"
echo "   1. Review report: cat $REPORT_FILE"
echo "   2. Identify duplicate configs in each domain"
echo "   3. Create canonical configs for high-frequency domains"
echo "   4. Use scripts/migrate_config_domain.sh to consolidate"
echo ""

