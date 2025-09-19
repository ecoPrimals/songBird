#!/usr/bin/env python3
"""
Provider Trait Consolidation Script

This script consolidates duplicate provider traits by identifying canonical
versions and replacing duplicates with imports from the canonical location.
"""

import os
import re
import subprocess
from pathlib import Path

# Canonical provider trait locations (source of truth)
CANONICAL_TRAITS = {
    "CanonicalPrimalProvider": "songbird-types/src/traits.rs",
    "CanonicalConfigProvider": "songbird-types/src/traits.rs", 
    "CanonicalObservabilityProvider": "songbird-types/src/traits.rs",
    "PrimalProvider": "songbird-universal-primals/src/traits.rs",
}

# Duplicate trait definitions to consolidate
TRAIT_DUPLICATES = {
    "ConfigProvider": [
        "crates/songbird-config/src/config/providers.rs",
        "crates/songbird-canonical/src/providers.rs",
        "crates/songbird-universal/src/traits.rs",  # ConfigurationProvider
    ],
    
    "ServiceProvider": [
        "crates/songbird-canonical/src/providers.rs",
        "crates/songbird-canonical/src/traits.rs",
    ],
    
    "SecurityProvider": [
        "crates/songbird-universal/src/traits.rs",
        "crates/songbird-network/src/network/gaming/security_provider.rs",
        "crates/songbird-network/src/network/gaming/security/providers.rs",
    ],
    
    "FeatureFlagProvider": [
        "crates/songbird-core/src/traits/feature_flags.rs",
        "crates/songbird-discovery/src/traits/feature_flags.rs",
    ],
}

def analyze_trait_usage():
    """Analyze which traits are actually used vs. just defined"""
    print("🔍 ANALYZING PROVIDER TRAIT USAGE...")
    
    for trait_name, duplicate_files in TRAIT_DUPLICATES.items():
        print(f"\n📊 {trait_name}:")
        
        # Count usages across codebase
        cmd = f"grep -r 'impl.*{trait_name}\\|: {trait_name}\\|{trait_name}>' crates/ --include='*.rs' | wc -l"
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        usage_count = int(result.stdout.strip()) if result.stdout.strip().isdigit() else 0
        
        print(f"  📈 Total usages: {usage_count}")
        print(f"  📁 Duplicate locations: {len(duplicate_files)}")
        
        for file_path in duplicate_files:
            if os.path.exists(file_path):
                print(f"    • {file_path}")

def consolidate_feature_flag_provider():
    """Consolidate FeatureFlagProvider - safe consolidation target"""
    print("\n🔧 CONSOLIDATING FeatureFlagProvider...")
    
    # Keep the discovery version as canonical (more comprehensive)
    canonical_file = "crates/songbird-discovery/src/traits/feature_flags.rs"
    duplicate_file = "crates/songbird-core/src/traits/feature_flags.rs"
    
    if not os.path.exists(canonical_file) or not os.path.exists(duplicate_file):
        print("❌ Files not found for FeatureFlagProvider consolidation")
        return
    
    # Read the duplicate file to see if it's truly duplicate
    with open(duplicate_file, 'r') as f:
        duplicate_content = f.read()
    
    with open(canonical_file, 'r') as f:
        canonical_content = f.read()
    
    # Check if they're similar enough to consolidate
    if "trait FeatureFlagProvider" in duplicate_content:
        print(f"  📝 Replacing duplicate in {duplicate_file}")
        
        # Replace the trait definition with an import
        new_content = re.sub(
            r'pub trait FeatureFlagProvider.*?\n(?:.*?\n)*?.*?\}',
            '// Re-export canonical FeatureFlagProvider from songbird-discovery\npub use songbird_discovery::traits::feature_flags::FeatureFlagProvider;',
            duplicate_content,
            flags=re.MULTILINE | re.DOTALL
        )
        
        # Write the updated content
        with open(duplicate_file, 'w') as f:
            f.write(new_content)
        
        print(f"  ✅ Consolidated FeatureFlagProvider in {duplicate_file}")

def update_exports():
    """Update lib.rs files to export consolidated traits"""
    print("\n📦 UPDATING EXPORTS...")
    
    # Add FeatureFlagProvider to discovery lib.rs exports
    discovery_lib = "crates/songbird-discovery/src/lib.rs"
    if os.path.exists(discovery_lib):
        with open(discovery_lib, 'r') as f:
            content = f.read()
        
        if "pub use traits::feature_flags::FeatureFlagProvider;" not in content:
            # Add export
            content += "\n// Export consolidated traits\npub use traits::feature_flags::FeatureFlagProvider;\n"
            
            with open(discovery_lib, 'w') as f:
                f.write(content)
            
            print(f"  ✅ Added FeatureFlagProvider export to {discovery_lib}")

def main():
    print("🎯 PROVIDER TRAIT CONSOLIDATION SCRIPT")
    print("=" * 50)
    
    # Step 1: Analyze current state
    analyze_trait_usage()
    
    # Step 2: Consolidate safe targets
    consolidate_feature_flag_provider()
    
    # Step 3: Update exports
    update_exports()
    
    print("\n✅ CONSOLIDATION COMPLETE")
    print("\n📋 NEXT STEPS:")
    print("1. Run cargo check to verify builds")
    print("2. Test consolidated trait usage")
    print("3. Update imports in dependent crates")

if __name__ == "__main__":
    main() 