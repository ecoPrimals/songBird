#!/bin/bash

echo "=== PERFORMANCE CONFIG MIGRATION SCRIPT ==="
echo

echo "🚨 MASSIVE FRAGMENTATION DETECTED:"
echo "=================================="
echo "Found 8 PerformanceConfig struct definitions!"
echo "This demonstrates why unification is critical."
echo

echo "PerformanceConfig locations:"
grep -r "struct PerformanceConfig" crates/ --include="*.rs"

echo
echo "🎯 MIGRATION TO UnifiedPerformanceConfig:"
echo "========================================"
echo "UnifiedPerformanceConfig fields:"
echo "- discovery_timeout_secs: u64"  
echo "- max_concurrent_operations: usize"
echo "- circuit_breaker_threshold: u32"
echo "- retry_backoff_base_ms: u64"
echo "- request_batch_size: usize"

echo
echo "🔧 STARTING MIGRATION:"
echo "====================="

# Step 1: Replace imports
echo "Step 1: Replacing PerformanceConfig imports..."
find crates/ -name "*.rs" -exec sed -i 's/use.*PerformanceConfig[^;]*;/use songbird_config::UnifiedPerformanceConfig;/g' {} \;

# Step 2: Update field patterns
echo "Step 2: Updating field access patterns..."
find crates/ -name "*.rs" -exec sed -i 's/\.timeout_ms/.discovery_timeout_secs/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.max_operations/.max_concurrent_operations/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.circuit_breaker/.circuit_breaker_threshold/g' {} \;

echo "Step 3: Checking progress..."
remaining_configs=$(grep -r "struct PerformanceConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining PerformanceConfig struct definitions: $remaining_configs"

