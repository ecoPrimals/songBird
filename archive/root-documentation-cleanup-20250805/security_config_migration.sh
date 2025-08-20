#!/bin/bash

echo "=== SECURITY CONFIG MIGRATION SCRIPT ==="
echo

echo "🎯 ANALYZING SECURITYCONFIG FRAGMENTATION:"
echo "=========================================="
echo "SecurityConfig struct definitions:"
grep -r "struct SecurityConfig" crates/ --include="*.rs"

echo
echo "UnifiedSecurityConfig structure:"
echo "- beardog_integration: BearDogConfig"
echo "- encryption: EncryptionConfig" 
echo "- access_control: AccessControlConfig"

echo
echo "🔧 STARTING MIGRATION:"
echo "====================="

# Step 1: Replace imports
echo "Step 1: Replacing SecurityConfig imports..."
find crates/ -name "*.rs" -exec sed -i 's/use.*SecurityConfig[^;]*;/use songbird_config::UnifiedSecurityConfig;/g' {} \;

# Step 2: Update field access patterns
echo "Step 2: Updating field access patterns..."
find crates/ -name "*.rs" -exec sed -i 's/\.encryption_key_size/.encryption.algorithm/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.session_timeout/.access_control.session_timeout_minutes/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/\.beardog_endpoint/.beardog_integration.endpoint/g' {} \;

echo "Step 3: Checking progress..."
remaining_configs=$(grep -r "struct SecurityConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining SecurityConfig struct definitions: $remaining_configs"

