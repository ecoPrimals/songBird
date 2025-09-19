#!/usr/bin/env python3
"""
🌌 Fractal Federation Migration Script

This script helps migrate from legacy federation system to the new fractal federation.
It performs the following tasks:

1. Identifies legacy federation usage patterns
2. Suggests fractal federation replacements
3. Updates imports and usage patterns
4. Validates migration completeness

Usage:
    python3 scripts/migrate_to_fractal_federation.py
    python3 scripts/migrate_to_fractal_federation.py --dry-run
    python3 scripts/migrate_to_fractal_federation.py --validate-only
"""

import os
import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass


@dataclass
class MigrationPattern:
    """Represents a migration pattern from legacy to fractal federation"""
    legacy_pattern: str
    fractal_replacement: str
    description: str
    file_types: List[str]


# Migration patterns for legacy to fractal federation
MIGRATION_PATTERNS = [
    MigrationPattern(
        legacy_pattern=r"use songbird_federation::McpFederation",
        fractal_replacement="use songbird_federation::FractalFederationManager",
        description="Replace legacy MCP federation with fractal federation manager",
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"use songbird_federation::CanonicalFederationManager",
        fractal_replacement="use songbird_federation::FractalFederationManager",
        description="Replace canonical federation manager with fractal federation manager", 
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"McpFederation::new\(",
        fractal_replacement="FractalFederationManager::new(",
        description="Replace MCP federation instantiation",
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"CanonicalFederationManager::new\(",
        fractal_replacement="FractalFederationManager::new(",
        description="Replace canonical federation manager instantiation",
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"use songbird_federation::mcp_handler",
        fractal_replacement="use songbird_federation::{FractalFederationManager, ZeroCostFederationBuilder}",
        description="Replace MCP handler imports with fractal federation",
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"use songbird_federation::canonical_federation",
        fractal_replacement="use songbird_federation::{FractalFederationManager, ZeroCostFederationBuilder}",
        description="Replace canonical federation imports",
        file_types=["rs"]
    ),
    MigrationPattern(
        legacy_pattern=r"FederationConfig\s*\{",
        fractal_replacement="// TODO: Migrate to FractalNodeId configuration\n    FractalNodeId {",
        description="Replace legacy federation config with fractal node ID",
        file_types=["rs"]
    ),
]

# Deprecated modules that should trigger warnings
DEPRECATED_MODULES = [
    "mcp_handler",
    "canonical_federation", 
    "canonical_production_federation",
    "canonical_unified_federation",
]

# Files to exclude from migration (already migrated or not relevant)
EXCLUDE_PATTERNS = [
    r"target/",
    r"\.git/",
    r"archive/",
    r"fractal_federation\.rs$",
    r"zero_cost_federation\.rs$",
    r"migrate_to_fractal_federation\.py$",
]


class FractalFederationMigrator:
    """Handles migration from legacy federation to fractal federation"""
    
    def __init__(self, project_root: Path, dry_run: bool = False):
        self.project_root = project_root
        self.dry_run = dry_run
        self.migration_stats = {
            'files_processed': 0,
            'files_modified': 0,
            'patterns_replaced': 0,
            'deprecated_usage_found': 0,
        }
        
    def should_exclude_file(self, file_path: Path) -> bool:
        """Check if file should be excluded from migration"""
        relative_path = file_path.relative_to(self.project_root)
        for pattern in EXCLUDE_PATTERNS:
            if re.search(pattern, str(relative_path)):
                return True
        return False
        
    def find_rust_files(self) -> List[Path]:
        """Find all Rust files in the project"""
        rust_files = []
        for root, dirs, files in os.walk(self.project_root):
            # Skip excluded directories
            dirs[:] = [d for d in dirs if not any(re.search(pattern, d) for pattern in EXCLUDE_PATTERNS)]
            
            for file in files:
                if file.endswith('.rs'):
                    file_path = Path(root) / file
                    if not self.should_exclude_file(file_path):
                        rust_files.append(file_path)
        return rust_files
        
    def analyze_file(self, file_path: Path) -> Tuple[List[str], List[str]]:
        """Analyze file for legacy federation patterns"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"❌ Error reading {file_path}: {e}")
            return [], []
            
        issues = []
        suggestions = []
        
        # Check for deprecated module usage
        for module in DEPRECATED_MODULES:
            if f"use songbird_federation::{module}" in content or f"use songbird_federation::{{{module}" in content:
                issues.append(f"Uses deprecated module: {module}")
                suggestions.append(f"Replace {module} with fractal_federation or zero_cost_federation")
                self.migration_stats['deprecated_usage_found'] += 1
                
        # Check for legacy patterns
        for pattern in MIGRATION_PATTERNS:
            if re.search(pattern.legacy_pattern, content):
                issues.append(f"Legacy pattern found: {pattern.description}")
                suggestions.append(f"Replace with: {pattern.fractal_replacement}")
                
        return issues, suggestions
        
    def migrate_file(self, file_path: Path) -> bool:
        """Migrate a single file from legacy to fractal federation"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                original_content = f.read()
        except Exception as e:
            print(f"❌ Error reading {file_path}: {e}")
            return False
            
        modified_content = original_content
        modifications_made = 0
        
        # Apply migration patterns
        for pattern in MIGRATION_PATTERNS:
            if pattern.file_types and file_path.suffix.lstrip('.') not in pattern.file_types:
                continue
                
            new_content, count = re.subn(pattern.legacy_pattern, pattern.fractal_replacement, modified_content)
            if count > 0:
                print(f"  📝 {pattern.description}: {count} replacements")
                modified_content = new_content
                modifications_made += count
                self.migration_stats['patterns_replaced'] += count
                
        # Write back if modifications were made
        if modifications_made > 0:
            if not self.dry_run:
                try:
                    with open(file_path, 'w', encoding='utf-8') as f:
                        f.write(modified_content)
                    print(f"  ✅ Modified {file_path} ({modifications_made} changes)")
                except Exception as e:
                    print(f"  ❌ Error writing {file_path}: {e}")
                    return False
            else:
                print(f"  🔍 Would modify {file_path} ({modifications_made} changes)")
                
            self.migration_stats['files_modified'] += 1
            return True
            
        return False
        
    def validate_migration(self) -> bool:
        """Validate that migration is complete"""
        print("\n🔍 Validating migration completeness...")
        
        rust_files = self.find_rust_files()
        validation_passed = True
        
        for file_path in rust_files:
            issues, suggestions = self.analyze_file(file_path)
            if issues:
                print(f"\n⚠️  Issues found in {file_path}:")
                for issue in issues:
                    print(f"   • {issue}")
                for suggestion in suggestions:
                    print(f"   💡 {suggestion}")
                validation_passed = False
                
        if validation_passed:
            print("✅ Migration validation passed - no legacy federation usage found!")
        else:
            print("❌ Migration validation failed - legacy federation usage still present")
            
        return validation_passed
        
    def run_migration(self) -> bool:
        """Run the complete migration process"""
        print("🌌 Starting Fractal Federation Migration")
        print("=" * 50)
        
        if self.dry_run:
            print("🔍 DRY RUN MODE - No files will be modified")
            
        # Find all Rust files
        rust_files = self.find_rust_files()
        print(f"📁 Found {len(rust_files)} Rust files to process")
        
        # Process each file
        for file_path in rust_files:
            self.migration_stats['files_processed'] += 1
            
            # Analyze file first
            issues, suggestions = self.analyze_file(file_path)
            
            if issues:
                print(f"\n🔧 Processing {file_path}:")
                for issue in issues:
                    print(f"   ⚠️  {issue}")
                    
                # Attempt migration
                self.migrate_file(file_path)
            else:
                # File is already clean
                continue
                
        # Print summary
        self.print_summary()
        
        # Validate migration if not dry run
        if not self.dry_run:
            return self.validate_migration()
        else:
            print("\n🔍 Run without --dry-run to perform actual migration")
            return True
            
    def print_summary(self):
        """Print migration summary"""
        print("\n📊 Migration Summary")
        print("=" * 30)
        print(f"Files processed:      {self.migration_stats['files_processed']}")
        print(f"Files modified:       {self.migration_stats['files_modified']}")
        print(f"Patterns replaced:    {self.migration_stats['patterns_replaced']}")
        print(f"Deprecated usage:     {self.migration_stats['deprecated_usage_found']}")


def main():
    parser = argparse.ArgumentParser(description="Migrate from legacy federation to fractal federation")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be changed without modifying files")
    parser.add_argument("--validate-only", action="store_true", help="Only validate migration, don't perform it")
    parser.add_argument("--project-root", type=str, default=".", help="Root directory of the project")
    
    args = parser.parse_args()
    
    project_root = Path(args.project_root).resolve()
    if not project_root.exists():
        print(f"❌ Project root does not exist: {project_root}")
        sys.exit(1)
        
    migrator = FractalFederationMigrator(project_root, dry_run=args.dry_run)
    
    if args.validate_only:
        success = migrator.validate_migration()
    else:
        success = migrator.run_migration()
        
    if success:
        print("\n🎉 Fractal Federation Migration Completed Successfully!")
        print("\n📋 Next Steps:")
        print("1. Run `cargo check` to verify compilation")
        print("2. Run `cargo test` to ensure tests pass")
        print("3. Update any remaining manual configurations")
        print("4. Deploy fractal federation to production")
        sys.exit(0)
    else:
        print("\n❌ Migration encountered issues - please review and fix manually")
        sys.exit(1)


if __name__ == "__main__":
    main() 