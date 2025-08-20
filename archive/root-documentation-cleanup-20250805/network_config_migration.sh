#!/bin/bash

echo "=== NETWORK CONFIG MIGRATION SCRIPT ==="
echo

# Step 1: Analyze NetworkConfig patterns
echo "🔍 ANALYZING NETWORKCONFIG USAGE PATTERNS:"
echo "==========================================="

echo "NetworkConfig struct definitions:"
grep -r "struct NetworkConfig" crates/ --include="*.rs"

echo
echo "Import patterns:"
grep -r "use.*NetworkConfig" crates/ --include="*.rs" | head -5

echo
echo "Field access patterns:"
grep -r "\.bind_address\|\.port\|\.timeout" crates/ --include="*.rs" | head -5

echo
echo "🎯 MIGRATION MAPPING:"
echo "===================="
echo "NetworkConfig → UnifiedNetworkConfig fields:"
echo "- .bind_address → .bind_address"  
echo "- .port → .port"
echo "- .timeout → .connection_timeout_secs"
echo "- .enable_tls → .ssl.enabled"
echo "- .gaming → .gaming.*"

echo
echo "🔧 STARTING MIGRATION:"
echo "====================="

# Step 2: Replace imports  
echo "Step 1: Replacing NetworkConfig imports..."
find crates/ -name "*.rs" -exec sed -i 's/use.*NetworkConfig/use songbird_config::UnifiedNetworkConfig/g' {} \;

echo "Step 2: Updating field patterns..."
# Note: More complex field mappings will need manual adjustment

echo "Step 3: Checking progress..."
remaining_configs=$(grep -r "struct NetworkConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining NetworkConfig struct definitions: $remaining_configs"

