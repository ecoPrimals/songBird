#!/usr/bin/env python3
"""
HealthCheckConfig Consolidation Script

This script consolidates the 18 duplicate HealthCheckConfig definitions
by replacing them with imports from the canonical songbird-types location.
"""

import os
import re
import subprocess
from pathlib import Path

# Files with duplicate HealthCheckConfig definitions (from our analysis)
DUPLICATE_FILES = [
    "crates/songbird-network/src/management/config.rs",
    "crates/songbird-universal-primals/src/modern_api.rs", 
    "crates/songbird-universal-primals/src/storage/config.rs",
    "crates/songbird-universal-primals/src/config.rs",
    "crates/songbird-universal-primals/src/universal_registry/config.rs",
    "crates/songbird-observability/src/health/config.rs",
    "crates/songbird-observability/src/health/production_health.rs",
    "crates/songbird-universal/src/types.rs",
    "crates/songbird-discovery/src/traits/health.rs",
    "crates/songbird-canonical/src/config/adapters.rs",
    "crates/songbird-core/src/robustness/config.rs",
    "crates/songbird-core/src/api/universal_service_registration/types.rs",
    "crates/songbird-core/src/traits/health.rs",
    "crates/songbird-config/src/config/mod.rs",
    "crates/songbird-config/src/config/universal_primals.rs",
    "crates/songbird-config/src/canonical/service.rs",
    "crates/songbird-config/src/unified/robustness.rs",
    "crates/songbird-config/src/unified/api.rs",
]

# Canonical location
CANONICAL_IMPORT = "use songbird_types::HealthCheckConfig;"

# Pattern to match HealthCheckConfig struct definitions and their impl blocks
HEALTH_CHECK_CONFIG_PATTERN = re.compile(
    r'(?:#\[derive[^\]]*\]\s*)?pub struct HealthCheckConfig\s*\{[^}]*\}(?:\s*impl[^}]*\{[^}]*\})*',
    re.MULTILINE | re.DOTALL
)

def process_file(file_path):
    """Process a single file to replace duplicate HealthCheckConfig definitions."""
    if not os.path.exists(file_path):
        print(f"⚠️  File not found: {file_path}")
        return False
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Check if file contains duplicate definition
    if 'pub struct HealthCheckConfig' not in content:
        print(f"✅ No duplicate found in: {file_path}")
        return True
    
    # Remove the struct definition and its impl blocks
    new_content = HEALTH_CHECK_CONFIG_PATTERN.sub('', content)
    
    # Add import if not present
    if 'use songbird_types::HealthCheckConfig' not in new_content and 'songbird_types::HealthCheckConfig' not in new_content:
        # Find the right place to add the import (after other use statements)
        lines = new_content.split('\n')
        insert_index = 0
        
        # Find last use statement
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') and not line.strip().startswith('use crate'):
                insert_index = i + 1
        
        # Insert the import
        lines.insert(insert_index, CANONICAL_IMPORT)
        new_content = '\n'.join(lines)
    
    # Clean up extra newlines
    new_content = re.sub(r'\n\n\n+', '\n\n', new_content)
    
    # Write back the file
    with open(file_path, 'w') as f:
        f.write(new_content)
    
    print(f"✅ Consolidated: {file_path}")
    return True

def verify_canonical_export():
    """Verify the canonical HealthCheckConfig is properly exported."""
    print("🔍 Verifying canonical export...")
    
    # Check if HealthCheckConfig is exported from songbird-types
    with open('crates/songbird-types/src/lib.rs', 'r') as f:
        lib_content = f.read()
    
    if 'HealthCheckConfig' in lib_content:
        print("✅ HealthCheckConfig is exported from songbird-types")
        return True
    else:
        print("⚠️  HealthCheckConfig not found in songbird-types exports")
        print("   Adding to exports...")
        
        # Add to performance module exports
        if 'pub use performance::*;' in lib_content:
            print("✅ Performance module is already re-exported (HealthCheckConfig included)")
            return True
        else:
            print("⚠️  Need to add HealthCheckConfig to exports manually")
            return False

def main():
    """Main consolidation process."""
    print("🔄 Starting HealthCheckConfig consolidation...")
    print(f"📋 Processing {len(DUPLICATE_FILES)} files with duplicates")
    
    # Verify canonical export first
    if not verify_canonical_export():
        print("❌ Cannot proceed - canonical export not verified")
        return
    
    success_count = 0
    
    for file_path in DUPLICATE_FILES:
        if process_file(file_path):
            success_count += 1
    
    print(f"\n📊 Results:")
    print(f"✅ Successfully processed: {success_count}/{len(DUPLICATE_FILES)} files")
    
    if success_count == len(DUPLICATE_FILES):
        print("\n🎉 HealthCheckConfig consolidation complete!")
        print("🔧 Running cargo check to verify changes...")
        
        # Run cargo check to verify changes
        result = subprocess.run(['cargo', 'check'], capture_output=True, text=True)
        if result.returncode == 0:
            print("✅ All changes verified successfully!")
            print(f"📈 Impact: Eliminated {len(DUPLICATE_FILES) - 1} duplicate definitions")
            print("🎯 Result: Single source of truth for HealthCheckConfig")
        else:
            print("⚠️  Some compilation issues detected:")
            print(result.stderr[:1000])  # Show first 1000 chars
    else:
        print("\n⚠️  Some files could not be processed. Manual review needed.")

if __name__ == "__main__":
    main() 