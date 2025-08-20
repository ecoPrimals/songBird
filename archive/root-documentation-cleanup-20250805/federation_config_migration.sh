#!/bin/bash

echo "=== FEDERATION CONFIG MIGRATION SCRIPT ==="
echo

echo "🎯 FIELD MAPPING ANALYSIS:"
echo "=========================="
echo "UnifiedFederationConfig fields:"
echo "- enabled: bool"
echo "- mode: String (standalone/client/server/hybrid/clustered)"  
echo "- discovery_port: u16"
echo "- cluster_name: Option<String>"
echo "- max_nodes: usize"
echo "- heartbeat_interval_secs: u64"
echo "- trust_verification_enabled: bool"
echo
echo "Fragmented configs → Unified mapping:"
echo "- cluster_endpoints → cluster_name (simplified)"
echo "- heartbeat_interval → heartbeat_interval_secs"
echo "- node_id/default_cluster_id → cluster_name"
echo "- broadcast_ports/discovery_ports → discovery_port (first port)"
echo "- auto_discovery → moved to discovery config"
echo "- nested configs → respective unified configs"

echo
echo "🔧 STARTING MIGRATION:"
echo "====================="

# Step 1: Replace imports
echo "Step 1: Replacing FederationConfig imports..."
find crates/ -name "*.rs" -exec sed -i 's/use.*FederationConfig[^;]*;/use songbird_config::UnifiedFederationConfig;/g' {} \;

# Step 2: Update field access patterns 
echo "Step 2: Updating field access patterns..."
find crates/ -name "*.rs" -exec sed -i 's/\.heartbeat_interval/.heartbeat_interval_secs/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.cluster_endpoints/.cluster_name/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.node_id/.cluster_name/g' {} \;

echo "Step 3: Checking progress..."
remaining_configs=$(grep -r "struct FederationConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining FederationConfig struct definitions: $remaining_configs"

if [ "$remaining_configs" -gt 0 ]; then
    echo "📍 Remaining FederationConfig definitions:"
    grep -r "struct FederationConfig" crates/ --include="*.rs"
fi

