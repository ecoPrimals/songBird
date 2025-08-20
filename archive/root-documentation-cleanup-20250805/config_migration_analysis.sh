#!/bin/bash

echo "=== SONGBIRD CONFIG MIGRATION ANALYSIS ==="
echo

# Find all deprecated config struct definitions
echo "📊 DEPRECATED CONFIG STRUCTS BY CRATE:"
echo "======================================"
grep -r "#\[deprecated.*note.*UnifiedSongbirdConfig" crates/ --include="*.rs" -A 2 | \
grep "pub struct.*Config" | \
sed 's|crates/\([^/]*\)/.*pub struct \([^{]*\).*|\1: \2|' | \
sort | uniq -c | sort -nr

echo
echo "🔍 USAGE ANALYSIS:"
echo "=================="

# Create list of deprecated config struct names
grep -r "#\[deprecated.*note.*UnifiedSongbirdConfig" crates/ --include="*.rs" -A 2 | \
grep "pub struct.*Config" | \
sed 's/.*pub struct \([^ {]*\).*/\1/' | \
sort | uniq > /tmp/deprecated_configs.txt

echo "Found $(wc -l < /tmp/deprecated_configs.txt) unique deprecated config structs"
echo

# For each deprecated config, find if it's still used
echo "📈 USAGE FREQUENCY (excluding definitions and deprecation notices):"
echo "=================================================================="
while read config_name; do
    usage_count=$(grep -r "$config_name" crates/ --include="*.rs" | \
                  grep -v "#\[deprecated" | \
                  grep -v "pub struct $config_name" | \
                  grep -v "note.*UnifiedSongbirdConfig" | wc -l)
    if [ "$usage_count" -gt 0 ]; then
        echo "$usage_count usages: $config_name"
    fi
done < /tmp/deprecated_configs.txt | sort -nr

echo
echo "✅ READY FOR IMMEDIATE REMOVAL (0 active usages):"
echo "================================================="
while read config_name; do
    usage_count=$(grep -r "$config_name" crates/ --include="*.rs" | \
                  grep -v "#\[deprecated" | \
                  grep -v "pub struct $config_name" | \
                  grep -v "note.*UnifiedSongbirdConfig" | wc -l)
    if [ "$usage_count" -eq 0 ]; then
        echo "✅ $config_name (ready for deletion)"
    fi
done < /tmp/deprecated_configs.txt

echo
echo "🔧 REQUIRES MIGRATION (active usages):"
echo "======================================"
while read config_name; do
    usage_count=$(grep -r "$config_name" crates/ --include="*.rs" | \
                  grep -v "#\[deprecated" | \
                  grep -v "pub struct $config_name" | \
                  grep -v "note.*UnifiedSongbirdConfig" | wc -l)
    if [ "$usage_count" -gt 0 ]; then
        echo "🔧 $config_name ($usage_count usages need migration)"
    fi
done < /tmp/deprecated_configs.txt

rm -f /tmp/deprecated_configs.txt
