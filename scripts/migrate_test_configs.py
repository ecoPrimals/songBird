#!/usr/bin/env python3
"""
Test Configuration Migration Script

This script migrates test files from deprecated `SongbirdConfig` to the new 
`UnifiedSongbirdConfig` from `songbird-types`, and updates test-specific 
configurations to use the new `songbird-test-utils` config module.
"""

import os
import re
import sys
from pathlib import Path

def find_test_files(base_dir):
    """Find all test files that need migration."""
    test_files = []
    
    # Find files in tests/ directory
    tests_dir = Path(base_dir) / "tests"
    if tests_dir.exists():
        for file_path in tests_dir.glob("*.rs"):
            if file_path.is_file():
                test_files.append(file_path)
    
    # Find test files in crates/*/tests/ directories
    crates_dir = Path(base_dir) / "crates"
    if crates_dir.exists():
        for crate_dir in crates_dir.iterdir():
            if crate_dir.is_dir():
                crate_tests_dir = crate_dir / "tests"
                if crate_tests_dir.exists():
                    for file_path in crate_tests_dir.glob("*.rs"):
                        if file_path.is_file():
                            test_files.append(file_path)
    
    return test_files

def migrate_file_content(content):
    """Migrate the content of a single file."""
    # Replace imports
    content = re.sub(
        r'use songbird_config::SongbirdConfig;',
        'use songbird_types::UnifiedSongbirdConfig;',
        content
    )
    
    # Replace additional imports
    content = re.sub(
        r'use songbird_config::\{SongbirdConfig,([^}]*)\};',
        r'use songbird_types::{UnifiedSongbirdConfig,\1};',
        content
    )
    
    # Replace SongbirdConfig usage
    content = re.sub(
        r'\bSongbirdConfig\b',
        'UnifiedSongbirdConfig',
        content
    )
    
    # Add test-utils import if test-specific configs are used
    if any(config in content for config in [
        'TestExecutionConfig', 'IntegrationTestConfig', 'TestFederationConfig', 
        'ChaosTestConfig'
    ]):
        # Add import at the top
        lines = content.split('\n')
        import_added = False
        for i, line in enumerate(lines):
            if line.startswith('use ') and 'songbird_test_utils' not in line:
                lines.insert(i, 'use songbird_test_utils::{TestExecutionConfig, IntegrationTestConfig, TestFederationConfig, ChaosTestConfig};')
                import_added = True
                break
        
        if not import_added:
            # Add after existing imports
            for i, line in enumerate(lines):
                if not line.startswith('use ') and not line.startswith('//') and line.strip():
                    lines.insert(i, 'use songbird_test_utils::{TestExecutionConfig, IntegrationTestConfig, TestFederationConfig, ChaosTestConfig};')
                    lines.insert(i, '')
                    break
        
        content = '\n'.join(lines)
    
    return content

def backup_file(file_path):
    """Create a backup of the original file."""
    backup_path = file_path.with_suffix(file_path.suffix + '.bak')
    backup_path.write_text(file_path.read_text())
    return backup_path

def migrate_test_file(file_path):
    """Migrate a single test file."""
    print(f"Migrating: {file_path}")
    
    try:
        # Read original content
        original_content = file_path.read_text()
        
        # Check if migration is needed
        if 'SongbirdConfig' not in original_content:
            print(f"  Skipped: No SongbirdConfig usage found")
            return True
        
        # Create backup
        backup_path = backup_file(file_path)
        print(f"  Backup created: {backup_path}")
        
        # Migrate content
        migrated_content = migrate_file_content(original_content)
        
        # Write migrated content
        file_path.write_text(migrated_content)
        print(f"  ✅ Migration completed")
        
        return True
        
    except Exception as e:
        print(f"  ❌ Migration failed: {e}")
        return False

def main():
    """Main migration function."""
    if len(sys.argv) > 1:
        base_dir = sys.argv[1]
    else:
        base_dir = os.getcwd()
    
    print(f"🚀 Starting test configuration migration in: {base_dir}")
    print("=" * 60)
    
    # Find test files
    test_files = find_test_files(base_dir)
    print(f"Found {len(test_files)} test files to check")
    print()
    
    # Migrate each file
    successful_migrations = 0
    failed_migrations = 0
    
    for file_path in test_files:
        if migrate_test_file(file_path):
            successful_migrations += 1
        else:
            failed_migrations += 1
    
    print()
    print("=" * 60)
    print(f"🎉 Migration Summary:")
    print(f"  ✅ Successful: {successful_migrations}")
    print(f"  ❌ Failed: {failed_migrations}")
    print(f"  📁 Total files: {len(test_files)}")
    
    if failed_migrations > 0:
        print()
        print("⚠️  Some migrations failed. Please check the error messages above.")
        print("   Backup files (.bak) have been created for safety.")
        return 1
    
    print()
    print("🎯 Next steps:")
    print("  1. Run `cargo check` to verify migrations")
    print("  2. Run tests to ensure functionality")
    print("  3. Remove .bak files if everything works")
    print("  4. Commit the changes")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 