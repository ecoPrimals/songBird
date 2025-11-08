#!/bin/bash
# Batch fix test compilation issues - Sprint 1
# Fixes missing imports and error handling patterns

set -e

echo "🔧 Fixing test compilation issues..."

# List of test files that need SongbirdResult/SongbirdError imports
TEST_FILES=(
    "crates/songbird-types/tests/service_info_comprehensive_tests.rs"
    "crates/songbird-types/tests/config_module_enhanced_tests.rs"
    "crates/songbird-types/tests/performance_tests.rs"
    "crates/songbird-types/tests/health_tests.rs"
    "crates/songbird-types/tests/service_module_comprehensive_tests.rs"
    "crates/songbird-types/tests/config_unified_tests.rs"
    "crates/songbird-types/tests/primal_and_health_tests.rs"
    "crates/songbird-types/tests/service_types_comprehensive_tests.rs"
    "crates/songbird-types/tests/health_comprehensive_tests.rs"
    "crates/songbird-types/tests/health_module_comprehensive_tests.rs"
    "crates/songbird-types/tests/additional_tests.rs"
    "crates/songbird-types/tests/response_tests.rs"
    "crates/songbird-types/tests/response_module_comprehensive_tests.rs"
    "crates/songbird-types/tests/type_conversion_tests.rs"
    "crates/songbird-types/tests/error_handling_comprehensive_tests.rs"
    "crates/songbird-types/tests/traits_comprehensive_tests.rs"
    "crates/songbird-types/tests/canonical_adapter_tests.rs"
    "crates/songbird-types/tests/gaming_config_tests.rs"
    "crates/songbird-types/tests/core_types_tests.rs"
)

for file in "${TEST_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "Processing: $file"
        
        # Check if imports are already present
        if ! grep -q "use songbird_types::{SongbirdResult, SongbirdError}" "$file" 2>/dev/null; then
            # Check if file uses SongbirdResult
            if grep -q "SongbirdResult" "$file" 2>/dev/null; then
                echo "  ✓ Adding imports to $file"
                
                # Find the last use statement and add our imports after it
                if grep -q "^use " "$file"; then
                    # Add after last use statement
                    sed -i '/^use /a use songbird_types::{SongbirdResult, SongbirdError};' "$file" 2>/dev/null || true
                    
                    # Remove duplicates (keep only first occurrence)
                    awk '!seen[$0]++' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
                fi
            fi
        fi
        
        # Fix error handling patterns: |_| with reference to e
        sed -i 's/\.map_err(|_| SongbirdError::configuration(format!("\([^"]*\)", e)))/.map_err(|e| SongbirdError::configuration(format!("\1", e)))/g' "$file" 2>/dev/null || true
        
        # Fix .or_else to .ok_or_else for Option -> Result
        sed -i 's/\.as_ref()\.or_else(/\.as_ref().ok_or_else(/g' "$file" 2>/dev/null || true
        
    fi
done

echo "✅ Batch fixes applied!"
echo "🧪 Testing compilation..."

# Try to compile a few key test files
cargo test -p songbird-types --test service_info_comprehensive_tests --no-run 2>&1 | tail -3
cargo test -p songbird-types --test config_module_enhanced_tests --no-run 2>&1 | tail -3

echo "🎯 Done! Check compilation results above."

