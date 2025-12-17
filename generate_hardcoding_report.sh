#!/bin/bash
# Comprehensive Hardcoding Audit Report Generator

OUTPUT="hardcoding_detailed_audit_dec16_2025.txt"

echo "═══════════════════════════════════════════════════════════" > "$OUTPUT"
echo "  SONGBIRD HARDCODING AUDIT - December 16, 2025" >> "$OUTPUT"
echo "═══════════════════════════════════════════════════════════" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Scanning for IP addresses and localhost references..." >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "CATEGORY 1: PRODUCTION CODE HARDCODING (CRITICAL - P0)" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "" >> "$OUTPUT"

grep -rn "127\.0\.0\.1\|localhost\|192\.168\.\|10\.\|::1" \
  crates/*/src/ \
  --include="*.rs" \
  --exclude-dir=tests \
  --exclude="*test*.rs" \
  --exclude="*fixture*.rs" \
  --exclude="*mock*.rs" \
  2>/dev/null | head -100 >> "$OUTPUT"

echo "" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "CATEGORY 2: DEPRECATED CONSTANTS MODULE (P1)" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "" >> "$OUTPUT"

grep -rn "config::constants::" crates/ --include="*.rs" 2>/dev/null | head -50 >> "$OUTPUT"

echo "" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "CATEGORY 3: CANONICAL CONSTANTS (P1)" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "" >> "$OUTPUT"

grep -n "127\.0\.0\.1\|localhost" \
  crates/songbird-config/src/canonical/constants.rs \
  2>/dev/null >> "$OUTPUT"

echo "" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "CATEGORY 4: HARDCODED ELIMINATION MODULE (P0 - IRONIC!)" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "" >> "$OUTPUT"

grep -n "127\.0\.0\.1\|localhost\|:8[0-9][0-9][0-9]\|:9[0-9][0-9][0-9]" \
  crates/songbird-config/src/canonical/hardcoded_elimination.rs \
  2>/dev/null >> "$OUTPUT"

echo "" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "SUMMARY STATISTICS" >> "$OUTPUT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >> "$OUTPUT"
echo "" >> "$OUTPUT"

TOTAL=$(grep -r "127\.0\.0\.1\|localhost" crates/*/src --include="*.rs" 2>/dev/null | wc -l)
PROD=$(grep -r "127\.0\.0\.1\|localhost" crates/*/src --include="*.rs" --exclude-dir=tests --exclude="*test*.rs" 2>/dev/null | wc -l)
TEST=$(grep -r "127\.0\.0\.1\|localhost" crates/*/tests --include="*.rs" 2>/dev/null | wc -l)

echo "Total hardcoded instances: $TOTAL" >> "$OUTPUT"
echo "Production code: $PROD" >> "$OUTPUT"
echo "Test code: $TEST" >> "$OUTPUT"
echo "" >> "$OUTPUT"

echo "Report generated: $(date)" >> "$OUTPUT"
echo "Location: $OUTPUT" >> "$OUTPUT"

chmod +x "$0"
