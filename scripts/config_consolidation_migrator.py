#!/usr/bin/env python3
"""
Configuration Consolidation Migration Script

This script migrates fragmented configuration patterns to use the new canonical
configuration types from songbird-types, eliminating the 418+ configuration
structure duplications across the codebase.

Usage:
    python3 scripts/config_consolidation_migrator.py [--dry-run] [--verbose]
"""

import os
import re
import argparse
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Set

class ConfigConsolidationMigrator:
    """Migrates fragmented configurations to canonical types."""
    
    def __init__(self, dry_run: bool = False, verbose: bool = False):
        self.dry_run = dry_run
        self.verbose = verbose
        self.changes_made = 0
        self.files_processed = 0
        
        # Mapping of old config patterns to canonical ones
        self.security_config_patterns = {
            # SecurityConfig variations
            r'use\s+crate::security::types::SecurityConfig': 'use songbird_types::CanonicalSecurityConfig as SecurityConfig',
            r'use\s+songbird_config::unified::security::UnifiedSecurityConfig': 'use songbird_types::CanonicalSecurityConfig as UnifiedSecurityConfig',
            r'use\s+songbird_security::security::types::SecurityConfig': 'use songbird_types::CanonicalSecurityConfig as SecurityConfig',
            r'use\s+songbird_network::network::security_integration::SecurityConfig': 'use songbird_types::CanonicalSecurityConfig as SecurityConfig',
            
            # Struct definitions to remove (will be replaced by canonical)
            r'pub\s+struct\s+SecurityConfig\s*\{[^}]*\}': '// SecurityConfig moved to songbird_types::CanonicalSecurityConfig',
            r'pub\s+struct\s+UnifiedSecurityConfig\s*\{[^}]*\}': '// UnifiedSecurityConfig moved to songbird_types::CanonicalSecurityConfig',
        }
        
        self.network_config_patterns = {
            # NetworkConfig variations
            r'use\s+crate::config::network::NetworkConfig': 'use songbird_types::CanonicalNetworkConfig as NetworkConfig',
            r'use\s+songbird_config::canonical_network::CanonicalNetworkConfig': 'use songbird_types::CanonicalNetworkConfig',
            r'use\s+songbird_network::unified_types::UnifiedNetworkConfig': 'use songbird_types::CanonicalNetworkConfig as UnifiedNetworkConfig',
            r'use\s+songbird_network::management::config::NetworkConfig': 'use songbird_types::CanonicalNetworkConfig as NetworkConfig',
            
            # Struct definitions to remove
            r'pub\s+struct\s+NetworkConfig\s*\{[^}]*\}': '// NetworkConfig moved to songbird_types::CanonicalNetworkConfig',
            r'pub\s+struct\s+UnifiedNetworkConfig\s*\{[^}]*\}': '// UnifiedNetworkConfig moved to songbird_types::CanonicalNetworkConfig',
        }
        
        self.health_config_patterns = {
            # HealthConfig variations
            r'use\s+crate::traits::health::HealthCheckConfig': 'use songbird_types::CanonicalHealthConfig as HealthCheckConfig',
            r'use\s+songbird_core::traits::health::HealthCheckConfig': 'use songbird_types::CanonicalHealthConfig as HealthCheckConfig',
            r'use\s+songbird_network::management::health::HealthCheckConfig': 'use songbird_types::CanonicalHealthConfig as HealthCheckConfig',
            r'use\s+songbird_federation::canonical::health::HealthConfig': 'use songbird_types::CanonicalHealthConfig as HealthConfig',
            
            # Struct definitions to remove
            r'pub\s+struct\s+HealthCheckConfig\s*\{[^}]*\}': '// HealthCheckConfig moved to songbird_types::CanonicalHealthConfig',
            r'pub\s+struct\s+HealthConfig\s*\{[^}]*\}': '// HealthConfig moved to songbird_types::CanonicalHealthConfig',
        }
        
        # Files to skip (already canonical or special cases)
        self.skip_files = {
            'crates/songbird-types/src/config.rs',  # The canonical source
            'crates/songbird-canonical/src/config/',  # Already refactored
        }
        
    def should_skip_file(self, file_path: str) -> bool:
        """Check if a file should be skipped."""
        for skip_pattern in self.skip_files:
            if skip_pattern in file_path:
                return True
        return False
        
    def migrate_file(self, file_path: str) -> bool:
        """Migrate a single file to use canonical configurations."""
        if self.should_skip_file(file_path):
            if self.verbose:
                print(f"Skipping {file_path} (canonical source or already migrated)")
            return False
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            file_changed = False
            
            # Apply security config migrations
            for pattern, replacement in self.security_config_patterns.items():
                new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
                if new_content != content:
                    content = new_content
                    file_changed = True
                    if self.verbose:
                        print(f"  Applied security config pattern: {pattern[:50]}...")
                        
            # Apply network config migrations  
            for pattern, replacement in self.network_config_patterns.items():
                new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
                if new_content != content:
                    content = new_content
                    file_changed = True
                    if self.verbose:
                        print(f"  Applied network config pattern: {pattern[:50]}...")
                        
            # Apply health config migrations
            for pattern, replacement in self.health_config_patterns.items():
                new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
                if new_content != content:
                    content = new_content
                    file_changed = True
                    if self.verbose:
                        print(f"  Applied health config pattern: {pattern[:50]}...")
            
            # Write changes if any were made
            if file_changed:
                if not self.dry_run:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                        
                self.changes_made += 1
                print(f"{'[DRY RUN] ' if self.dry_run else ''}Migrated: {file_path}")
                return True
                
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
            return False
            
        return False
        
    def find_rust_files(self, directory: str) -> List[str]:
        """Find all Rust files in the directory."""
        rust_files = []
        for root, dirs, files in os.walk(directory):
            # Skip target directory and other build artifacts
            if 'target' in dirs:
                dirs.remove('target')
            if '.git' in dirs:
                dirs.remove('.git')
                
            for file in files:
                if file.endswith('.rs'):
                    rust_files.append(os.path.join(root, file))
                    
        return rust_files
        
    def analyze_fragmentation(self, directory: str) -> Dict[str, int]:
        """Analyze configuration fragmentation across the codebase."""
        fragmentation_stats = {
            'SecurityConfig': 0,
            'NetworkConfig': 0, 
            'HealthConfig': 0,
            'HealthCheckConfig': 0,
            'UnifiedSecurityConfig': 0,
            'UnifiedNetworkConfig': 0,
            'TotalConfigStructs': 0,
        }
        
        rust_files = self.find_rust_files(directory)
        
        for file_path in rust_files:
            if self.should_skip_file(file_path):
                continue
                
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                # Count configuration struct definitions
                for config_name in fragmentation_stats.keys():
                    if config_name == 'TotalConfigStructs':
                        continue
                        
                    pattern = rf'struct\s+{config_name}\s*{{'
                    matches = re.findall(pattern, content)
                    fragmentation_stats[config_name] += len(matches)
                    
                # Count total config structs
                total_configs = re.findall(r'struct\s+\w*Config\s*{', content)
                fragmentation_stats['TotalConfigStructs'] += len(total_configs)
                
            except Exception as e:
                if self.verbose:
                    print(f"Error analyzing {file_path}: {e}")
                    
        return fragmentation_stats
        
    def run_migration(self, directory: str = "crates") -> None:
        """Run the full migration process."""
        print("🎯 Configuration Consolidation Migration")
        print("=" * 50)
        
        # Analyze current fragmentation
        print("📊 Analyzing configuration fragmentation...")
        stats = self.analyze_fragmentation(directory)
        
        print("\n📈 Current Fragmentation:")
        for config_type, count in stats.items():
            if count > 0:
                print(f"  {config_type}: {count} definitions")
                
        print(f"\n🔍 Total config structures found: {stats['TotalConfigStructs']}")
        
        # Find and process files
        rust_files = self.find_rust_files(directory)
        print(f"\n🔧 Processing {len(rust_files)} Rust files...")
        
        if self.dry_run:
            print("⚠️  DRY RUN MODE - No files will be modified")
            
        for file_path in rust_files:
            self.files_processed += 1
            if self.migrate_file(file_path):
                pass  # File was changed, already logged
                
        # Summary
        print(f"\n✅ Migration Complete!")
        print(f"   Files processed: {self.files_processed}")
        print(f"   Files modified: {self.changes_made}")
        
        if self.changes_made > 0:
            print(f"\n🎉 Successfully consolidated {self.changes_made} configuration files!")
            print("   Next steps:")
            print("   1. Run 'cargo check --workspace' to verify compilation")
            print("   2. Run tests to ensure functionality is preserved")
            print("   3. Remove deprecated configuration modules")
        else:
            print("   No configuration files needed migration.")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Migrate fragmented configurations to canonical types"
    )
    parser.add_argument(
        '--dry-run', 
        action='store_true',
        help='Show what would be changed without making modifications'
    )
    parser.add_argument(
        '--verbose', '-v',
        action='store_true', 
        help='Show detailed migration progress'
    )
    parser.add_argument(
        '--directory', '-d',
        default='crates',
        help='Directory to process (default: crates)'
    )
    
    args = parser.parse_args()
    
    # Check if we're in the right directory
    if not os.path.exists('Cargo.toml'):
        print("❌ Error: Please run this script from the project root directory")
        sys.exit(1)
        
    migrator = ConfigConsolidationMigrator(
        dry_run=args.dry_run,
        verbose=args.verbose
    )
    
    try:
        migrator.run_migration(args.directory)
    except KeyboardInterrupt:
        print("\n⚠️  Migration interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Migration failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main() 