#!/usr/bin/env python3
"""
Health Status Consolidation Script

This script systematically replaces duplicate CanonicalHealthStatus definitions
across the codebase with imports from the canonical songbird-types location.
"""

import os
import re
import subprocess
from pathlib import Path

# Files with duplicate CanonicalHealthStatus definitions that should be replaced
DUPLICATE_FILES = [
    "crates/songbird-config/src/zero_touch/deployment.rs",
    "crates/songbird-core/src/traits/service.rs", 
    "crates/songbird-core/src/traits/discovery.rs",
    "crates/songbird-core/src/traits/mod.rs",
    "crates/songbird-core/src/biome/byob_coordinator/integration.rs",
    "crates/songbird-discovery/src/traits/health.rs",
    "crates/songbird-discovery/src/traits/discovery.rs",
    "crates/songbird-core/src/zero_touch/deployment.rs",
    "crates/songbird-discovery/src/traits/service.rs",
    "crates/songbird-network/src/management/monitoring.rs",
    "crates/songbird-network/src/communication/tarpc_client.rs",
    "crates/songbird-registry/src/production/persistent_registry.rs",
    "crates/songbird-orchestrator/src/server/mod.rs",
    "crates/songbird-observability/src/advanced_observability.rs",
    "crates/songbird-observability/src/health/production_health.rs",
    "crates/songbird-observability/src/observability/mod.rs",
    "crates/songbird-security/src/security/canonical/authentication.rs",
    "crates/songbird-core/src/robustness/error_types.rs",
    "crates/songbird-security/src/security/zero_cost_security_provider.rs",
]

# Pattern to match CanonicalHealthStatus enum definitions
HEALTH_STATUS_PATTERN = re.compile(
    r'pub enum CanonicalHealthStatus\s*\{[^}]+\}(?:\s*impl[^}]+\{[^}]+\})*',
    re.MULTILINE | re.DOTALL
)

def process_file(file_path):
    """Process a single file to replace duplicate CanonicalHealthStatus definitions."""
    if not os.path.exists(file_path):
        print(f"⚠️  File not found: {file_path}")
        return False
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Check if file contains duplicate definition
    if 'pub enum CanonicalHealthStatus' not in content:
        print(f"✅ No duplicate found in: {file_path}")
        return True
    
    # Remove the enum definition and its impl blocks
    new_content = HEALTH_STATUS_PATTERN.sub('', content)
    
    # Add import if not present
    if 'use songbird_types::CanonicalHealthStatus' not in new_content:
        # Find the right place to add the import (after other use statements)
        lines = new_content.split('\n')
        insert_index = 0
        
        # Find last use statement
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') and not line.strip().startswith('use crate'):
                insert_index = i + 1
        
        # Insert the import
        lines.insert(insert_index, 'use songbird_types::CanonicalHealthStatus;')
        new_content = '\n'.join(lines)
    
    # Clean up extra newlines
    new_content = re.sub(r'\n\n\n+', '\n\n', new_content)
    
    # Write back the file
    with open(file_path, 'w') as f:
        f.write(new_content)
    
    print(f"✅ Consolidated: {file_path}")
    return True

def main():
    """Main consolidation process."""
    print("🔄 Starting CanonicalHealthStatus consolidation...")
    print(f"📋 Processing {len(DUPLICATE_FILES)} files with duplicates")
    
    success_count = 0
    
    for file_path in DUPLICATE_FILES:
        if process_file(file_path):
            success_count += 1
    
    print(f"\n📊 Results:")
    print(f"✅ Successfully processed: {success_count}/{len(DUPLICATE_FILES)} files")
    
    if success_count == len(DUPLICATE_FILES):
        print("\n🎉 Health status consolidation complete!")
        print("🔧 Running cargo check to verify changes...")
        
        # Run cargo check to verify changes
        result = subprocess.run(['cargo', 'check'], capture_output=True, text=True)
        if result.returncode == 0:
            print("✅ All changes verified successfully!")
        else:
            print("⚠️  Some compilation issues detected:")
            print(result.stderr)
    else:
        print("\n⚠️  Some files could not be processed. Manual review needed.")

if __name__ == "__main__":
    main() 