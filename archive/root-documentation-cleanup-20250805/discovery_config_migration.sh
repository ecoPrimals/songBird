#!/bin/bash

echo "=== DISCOVERY CONFIG MIGRATION SCRIPT ==="
echo

# Step 1: Find all DiscoveryConfig usages for analysis
echo "🔍 ANALYZING DISCOVERYCONFIG USAGE PATTERNS:"
echo "============================================="

echo "Import patterns:"
grep -r "use.*DiscoveryConfig" crates/ --include="*.rs" | head -10

echo
echo "Field access patterns:"
grep -r "\.backend\|\.health_check_interval\|\.connection_timeout" crates/ --include="*.rs" | head -10

echo
echo "🔧 STARTING MIGRATION:"
echo "====================="

# Step 2: Replace imports
echo "Step 1: Replacing DiscoveryConfig imports..."
find crates/ -name "*.rs" -exec sed -i 's/use.*DiscoveryConfig/use songbird_config::UnifiedDiscoveryConfig/g' {} \;

# Step 3: Update common field access patterns (duration to seconds)
echo "Step 2: Updating field access patterns..."
find crates/ -name "*.rs" -exec sed -i 's/\.health_check_interval/.health_check_interval_secs/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.connection_timeout/.discovery_timeout/g' {} \;

echo "Step 3: Checking migration progress..."
remaining_old_configs=$(grep -r "struct DiscoveryConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining DiscoveryConfig struct definitions: $remaining_old_configs"

if [ "$remaining_old_configs" -gt 0 ]; then
    echo "📍 Remaining DiscoveryConfig definitions to remove:"
    grep -r "struct DiscoveryConfig" crates/ --include="*.rs"
fi

echo
echo "✅ Phase 1 migration completed!"
echo "Next: Manual verification and struct definition removal"
