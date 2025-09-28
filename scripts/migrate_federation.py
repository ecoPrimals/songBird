#!/usr/bin/env python3
"""
Automated Federation Migration Tool

This script helps automatically migrate Rust code from the old songbird-federation
system to the new discovery-based approach with FederationAwareDiscovery and
SovereigntyAwareAdapter.

Usage:
    python3 scripts/migrate_federation.py --input src/ --output src_migrated/
    python3 scripts/migrate_federation.py --file src/main.rs --in-place
    python3 scripts/migrate_federation.py --check src/  # Check only, no changes
"""

import argparse
import os
import re
import shutil
import sys
from pathlib import Path
from typing import List, Tuple, Dict, Optional

class FederationMigrator:
    """Automated migration tool for federation code"""
    
    def __init__(self, dry_run: bool = False, verbose: bool = False):
        self.dry_run = dry_run
        self.verbose = verbose
        self.migrations_applied = 0
        self.files_processed = 0
        self.warnings = []
        
        # Migration patterns
        self.import_migrations = [
            # Old imports -> New imports
            (r'use songbird_federation::\{([^}]+)\};', self._migrate_federation_imports),
            (r'use songbird_federation::([^;]+);', self._migrate_single_import),
        ]
        
        self.code_migrations = [
            # FederationManager -> FederationAwareDiscovery
            (r'FederationManager::new\(([^)]+)\)', r'Self::create_federation_discovery(\1)'),
            (r'federation\.discover_peers\(\)', r'federation_discovery.discover_federation_aware_services().await'),
            (r'federation\.route_request\(([^)]+)\)', r'sovereignty_adapter.execute_with_sovereignty_routing(\1).await'),
            (r'federation\.join_network\(([^)]+)\)', r'federation_discovery.join_sovereign_network(\1).await'),
            (r'federation\.assess_sovereignty\(([^)]+)\)', r'// Sovereignty assessment now automatic in discovery results'),
            (r'federation\.get_network_effects\(\)', r'federation_discovery.calculate_network_effect_potential(&services)'),
        ]
        
        self.config_migrations = [
            # Configuration migrations
            (r'FederationConfig\s*\{', 'FederationDiscoveryConfig {'),
            (r'peer_discovery_enabled:\s*([^,]+),', r'enable_federation_patterns: \1,'),
            (r'sovereignty_level:\s*([^,]+),', r'enable_sovereignty_assessment: \1.is_some(),'),
            (r'enable_network_effects:\s*([^,]+),', r'enable_network_effects: \1,'),
        ]
    
    def _migrate_federation_imports(self, match) -> str:
        """Migrate federation imports to new discovery imports"""
        imports = match.group(1)
        new_imports = []
        
        # Map old imports to new ones
        import_mapping = {
            'FederationManager': 'songbird_discovery::federation_aware_discovery::FederationAwareDiscovery',
            'FederationConfig': 'songbird_discovery::federation_aware_discovery::FederationDiscoveryConfig',
            'ProductionFederation': 'songbird_discovery::federation_aware_discovery::FederationAwareDiscovery',
            'CanonicalProductionFederation': 'songbird_discovery::federation_aware_discovery::FederationAwareDiscovery',
        }
        
        for old_import, new_import in import_mapping.items():
            if old_import in imports:
                new_imports.append(f'use {new_import};')
        
        # Add migration helper import
        new_imports.append('use songbird_discovery::migration::FederationMigrationHelper;')
        
        # Add sovereignty adapter import if routing is used
        if any(routing_term in imports for routing_term in ['route', 'routing', 'request']):
            new_imports.append('use songbird_universal::sovereignty_aware_adapter::SovereigntyAwareAdapter;')
        
        return '\n'.join(new_imports)
    
    def _migrate_single_import(self, match) -> str:
        """Migrate single federation import"""
        import_path = match.group(1)
        
        if 'FederationManager' in import_path:
            return 'use songbird_discovery::federation_aware_discovery::FederationAwareDiscovery;'
        elif 'FederationConfig' in import_path:
            return 'use songbird_discovery::federation_aware_discovery::FederationDiscoveryConfig;'
        elif 'production' in import_path.lower():
            return 'use songbird_discovery::federation_aware_discovery::FederationAwareDiscovery;'
        else:
            return match.group(0)  # No change
    
    def migrate_file(self, file_path: Path) -> bool:
        """Migrate a single Rust file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            
            # Apply import migrations
            for pattern, replacement in self.import_migrations:
                if callable(replacement):
                    content = re.sub(pattern, replacement, content)
                else:
                    content = re.sub(pattern, replacement, content)
            
            # Apply code migrations
            for pattern, replacement in self.code_migrations:
                content = re.sub(pattern, replacement, content)
            
            # Apply config migrations
            for pattern, replacement in self.config_migrations:
                content = re.sub(pattern, replacement, content)
            
            # Add migration helper code if federation code detected
            if self._needs_migration_helper(original_content):
                content = self._add_migration_helper_code(content)
            
            # Check if changes were made
            if content != original_content:
                if not self.dry_run:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                
                self.migrations_applied += 1
                if self.verbose:
                    print(f"✅ Migrated: {file_path}")
                return True
            else:
                if self.verbose:
                    print(f"⏭️ No changes: {file_path}")
                return False
                
        except Exception as e:
            self.warnings.append(f"Error processing {file_path}: {e}")
            print(f"⚠️ Error processing {file_path}: {e}")
            return False
    
    def _needs_migration_helper(self, content: str) -> bool:
        """Check if file needs migration helper code"""
        federation_indicators = [
            'FederationConfig',
            'FederationManager',
            'ProductionFederation',
            'federation.discover_peers',
            'federation.route_request',
        ]
        return any(indicator in content for indicator in federation_indicators)
    
    def _add_migration_helper_code(self, content: str) -> str:
        """Add migration helper code to assist with transition"""
        helper_code = '''
// MIGRATION HELPER: Add this code to assist with federation migration
// Remove this comment block after migration is complete

/*
// Example migration pattern:
async fn migrate_federation_config(old_config: LegacyFederationConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Migrate configuration
    let mut migration_helper = FederationMigrationHelper::default();
    let migration_result = migration_helper.migrate_with_validation(old_config).await?;
    
    if migration_result.success {
        println!("🎉 Migration successful!");
        let new_config = migration_result.new_discovery_config;
        
        // 2. Create new federation-aware discovery
        let base_discovery = ServiceDiscoveryFactory::create(&new_config.base_config)?;
        let mut federation_discovery = FederationAwareDiscovery::new(base_discovery, new_config);
        
        // 3. Use new system
        let services = federation_discovery.discover_federation_aware_services().await?;
        println!("Discovered {} services with enhanced sovereignty", services.len());
        
    } else {
        println!("⚠️ Migration issues: {:?}", migration_result.errors);
    }
    
    Ok(())
}
*/
'''
        
        # Insert helper code after the last use statement
        use_pattern = r'((?:use [^;]+;[\s]*)+)'
        if re.search(use_pattern, content):
            content = re.sub(use_pattern, r'\1' + helper_code, content, count=1)
        else:
            # If no use statements, add at the beginning
            content = helper_code + '\n' + content
        
        return content
    
    def migrate_directory(self, input_dir: Path, output_dir: Optional[Path] = None) -> None:
        """Migrate all Rust files in a directory"""
        if output_dir and not self.dry_run:
            output_dir.mkdir(parents=True, exist_ok=True)
        
        rust_files = list(input_dir.rglob('*.rs'))
        
        print(f"🔍 Found {len(rust_files)} Rust files to process...")
        
        for rust_file in rust_files:
            self.files_processed += 1
            
            if output_dir:
                # Copy to output directory
                relative_path = rust_file.relative_to(input_dir)
                output_file = output_dir / relative_path
                output_file.parent.mkdir(parents=True, exist_ok=True)
                
                if not self.dry_run:
                    shutil.copy2(rust_file, output_file)
                
                # Migrate the copied file
                self.migrate_file(output_file)
            else:
                # Migrate in place
                self.migrate_file(rust_file)
    
    def generate_migration_report(self) -> str:
        """Generate a migration report"""
        report = f"""
🔄 **FEDERATION MIGRATION REPORT**

📊 **Statistics:**
- Files processed: {self.files_processed}
- Files migrated: {self.migrations_applied}
- Migration success rate: {(self.migrations_applied/self.files_processed*100):.1f}%

⚠️ **Warnings:** {len(self.warnings)}
"""
        
        if self.warnings:
            report += "\n📋 **Warning Details:**\n"
            for warning in self.warnings:
                report += f"  - {warning}\n"
        
        if self.migrations_applied > 0:
            report += f"""
✅ **Next Steps:**
1. Review migrated code for correctness
2. Update your Cargo.toml dependencies:
   ```toml
   [dependencies]
   songbird-discovery = {{ version = "0.8", features = ["federation-aware"] }}
   songbird-universal = {{ version = "0.8", features = ["sovereignty-aware"] }}
   # Remove: songbird-federation = "0.7"  # DEPRECATED
   ```
3. Test your application with the new federation system
4. Run: `cargo check` to verify compilation
5. Run: `cargo test` to verify functionality
6. See FEDERATION_MIGRATION_GUIDE.md for detailed instructions

🎉 **Migration completed successfully!**
"""
        else:
            report += "\n✨ **No federation code found** - your code is already up to date!"
        
        return report

def main():
    parser = argparse.ArgumentParser(
        description='Automated Federation Migration Tool',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Migrate entire directory to new location
  python3 scripts/migrate_federation.py --input src/ --output src_migrated/
  
  # Migrate single file in place
  python3 scripts/migrate_federation.py --file src/main.rs --in-place
  
  # Check what would be migrated (dry run)
  python3 scripts/migrate_federation.py --input src/ --dry-run --verbose
  
  # Migrate current directory in place
  python3 scripts/migrate_federation.py --input . --in-place
        """
    )
    
    parser.add_argument('--input', '-i', type=Path, help='Input directory to migrate')
    parser.add_argument('--output', '-o', type=Path, help='Output directory (creates copy)')
    parser.add_argument('--file', '-f', type=Path, help='Single file to migrate')
    parser.add_argument('--in-place', action='store_true', help='Migrate files in place')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be changed without making changes')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--check', action='store_true', help='Check mode (same as --dry-run --verbose)')
    
    args = parser.parse_args()
    
    if args.check:
        args.dry_run = True
        args.verbose = True
    
    # Validate arguments
    if not args.input and not args.file:
        parser.error("Must specify either --input directory or --file")
    
    if args.file and args.output:
        parser.error("Cannot use --output with --file (use --in-place instead)")
    
    if args.input and not args.in_place and not args.output and not args.dry_run:
        parser.error("Must specify --output directory or --in-place for directory migration")
    
    # Create migrator
    migrator = FederationMigrator(dry_run=args.dry_run, verbose=args.verbose)
    
    print("🚀 **Federation Migration Tool**")
    print("=" * 50)
    
    if args.dry_run:
        print("🔍 **DRY RUN MODE** - No files will be changed")
    
    try:
        if args.file:
            # Migrate single file
            print(f"📁 Processing single file: {args.file}")
            migrator.files_processed = 1
            if migrator.migrate_file(args.file):
                print(f"✅ Successfully migrated: {args.file}")
            else:
                print(f"⏭️ No changes needed: {args.file}")
        
        elif args.input:
            # Migrate directory
            print(f"📁 Processing directory: {args.input}")
            if args.output:
                print(f"📁 Output directory: {args.output}")
            elif args.in_place:
                print("📁 Migrating in place")
            
            migrator.migrate_directory(args.input, args.output)
        
        # Print report
        print("\n" + "=" * 50)
        print(migrator.generate_migration_report())
        
        if migrator.migrations_applied > 0 and not args.dry_run:
            print("\n🎯 **IMPORTANT**: Review the migrated code and test thoroughly!")
            print("📖 See FEDERATION_MIGRATION_GUIDE.md for detailed migration instructions")
        
    except KeyboardInterrupt:
        print("\n❌ Migration cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Migration failed: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main() 