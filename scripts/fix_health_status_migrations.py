#!/usr/bin/env python3
"""
HealthStatus Migration Script

This script systematically migrates deprecated HealthStatus usage to CanonicalHealthStatus
throughout the codebase, including imports, type references, and variant usage.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

# HealthStatus migration patterns
HEALTH_STATUS_PATTERNS = [
    # Import statements
    {
        'pattern': r'use\s+([^:]+::)?types::HealthStatus;',
        'replacement': r'use songbird_types::CanonicalHealthStatus;',
        'description': 'Import statement migration'
    },
    {
        'pattern': r'use\s+([^:]+::)?HealthStatus;',
        'replacement': r'use songbird_types::CanonicalHealthStatus;',
        'description': 'Direct import migration'
    },
    # Type references
    {
        'pattern': r'\btypes::HealthStatus\b',
        'replacement': r'CanonicalHealthStatus',
        'description': 'Type reference migration'
    },
    {
        'pattern': r'\bHealthStatus\b(?!::)',
        'replacement': r'CanonicalHealthStatus',
        'description': 'Direct type reference migration'
    },
    # Variant references
    {
        'pattern': r'\btypes::HealthStatus::(\w+)\b',
        'replacement': r'CanonicalHealthStatus::\1',
        'description': 'Variant reference migration'
    },
    {
        'pattern': r'\bHealthStatus::(\w+)\b',
        'replacement': r'CanonicalHealthStatus::\1',
        'description': 'Direct variant migration'
    },
]

def get_files_with_health_status_usage():
    """Get files that use deprecated HealthStatus."""
    try:
        result = subprocess.run(['grep', '-r', '--include=*.rs', 'HealthStatus', 'crates/'], 
                              capture_output=True, text=True, cwd=os.getcwd())
        
        files_with_usage = set()
        if result.stdout:
            for line in result.stdout.split('\n'):
                if ':' in line:
                    file_path = line.split(':')[0]
                    if file_path.endswith('.rs'):
                        files_with_usage.add(file_path)
        
        return list(files_with_usage)
    except Exception as e:
        print(f"Error finding files with HealthStatus usage: {e}")
        return []

def migrate_health_status_in_file(file_path):
    """Migrate HealthStatus usage in a single file."""
    try:
        path = Path(file_path)
        if not path.exists():
            return False, []
        
        content = path.read_text()
        original_content = content
        changes_made = []
        
        # Add import if needed and not already present
        needs_import = any(pattern['pattern'] in content for pattern in HEALTH_STATUS_PATTERNS[2:])  # Skip import patterns
        has_import = 'use songbird_types::CanonicalHealthStatus;' in content
        
        if needs_import and not has_import:
            # Find a good place to add the import
            lines = content.split('\n')
            import_line_idx = -1
            
            # Look for existing songbird_types imports
            for i, line in enumerate(lines):
                if line.strip().startswith('use songbird_types::'):
                    import_line_idx = i
                    break
            
            # If no songbird_types imports, look for other use statements
            if import_line_idx == -1:
                for i, line in enumerate(lines):
                    if line.strip().startswith('use ') and '::' in line:
                        import_line_idx = i
                        break
            
            # Add the import
            if import_line_idx != -1:
                lines.insert(import_line_idx + 1, 'use songbird_types::CanonicalHealthStatus;')
                content = '\n'.join(lines)
                changes_made.append('Added CanonicalHealthStatus import')
        
        # Apply migration patterns
        for pattern_info in HEALTH_STATUS_PATTERNS:
            pattern = pattern_info['pattern']
            replacement = pattern_info['replacement']
            description = pattern_info['description']
            
            if re.search(pattern, content):
                new_content = re.sub(pattern, replacement, content)
                if new_content != content:
                    content = new_content
                    changes_made.append(description)
        
        # Write back if changes were made
        if content != original_content:
            path.write_text(content)
            return True, changes_made
        
        return False, []
    except Exception as e:
        print(f"Error migrating HealthStatus in {file_path}: {e}")
        return False, []

def clean_up_duplicate_imports(file_path):
    """Remove duplicate imports after migration."""
    try:
        path = Path(file_path)
        if not path.exists():
            return False
        
        content = path.read_text()
        lines = content.split('\n')
        
        # Find and remove duplicate CanonicalHealthStatus imports
        canonical_import_lines = []
        for i, line in enumerate(lines):
            if 'use songbird_types::CanonicalHealthStatus;' in line:
                canonical_import_lines.append(i)
        
        # Remove duplicates (keep the first one)
        if len(canonical_import_lines) > 1:
            for line_idx in reversed(canonical_import_lines[1:]):
                lines.pop(line_idx)
            
            path.write_text('\n'.join(lines))
            return True
        
        return False
    except Exception as e:
        print(f"Error cleaning up imports in {file_path}: {e}")
        return False

def main():
    """Main migration function."""
    if '--help' in sys.argv:
        print("Usage: python3 fix_health_status_migrations.py [--dry-run]")
        print("Systematically migrates deprecated HealthStatus to CanonicalHealthStatus")
        return 0
    
    dry_run = '--dry-run' in sys.argv
    if dry_run:
        print("🔍 DRY RUN MODE - No files will be modified")
    
    print("🚀 Starting HealthStatus migration...")
    print("=" * 60)
    
    # Get files with HealthStatus usage
    print("📊 Finding files with HealthStatus usage...")
    files_with_usage = get_files_with_health_status_usage()
    
    if not files_with_usage:
        print("✅ No files with HealthStatus usage found!")
        return 0
    
    print(f"Found {len(files_with_usage)} files with HealthStatus usage")
    print()
    
    # Process each file
    total_files_migrated = 0
    total_changes = 0
    
    for file_path in files_with_usage:
        print(f"Processing: {file_path}")
        
        if not dry_run:
            # Create backup
            backup_path = Path(file_path).with_suffix(Path(file_path).suffix + '.health-status-bak')
            try:
                backup_path.write_text(Path(file_path).read_text())
            except Exception as e:
                print(f"  ❌ Failed to create backup: {e}")
                continue
        
        if dry_run:
            # Just analyze what would be changed
            try:
                content = Path(file_path).read_text()
                changes_found = []
                
                for pattern_info in HEALTH_STATUS_PATTERNS:
                    if re.search(pattern_info['pattern'], content):
                        changes_found.append(pattern_info['description'])
                
                if changes_found:
                    print(f"  🔍 Would apply: {', '.join(changes_found)}")
                    total_files_migrated += 1
                    total_changes += len(changes_found)
                else:
                    print(f"  ℹ️  No applicable changes found")
            except Exception as e:
                print(f"  ❌ Error analyzing file: {e}")
        else:
            # Apply migration
            success, changes_made = migrate_health_status_in_file(file_path)
            
            if success:
                print(f"  ✅ Applied: {', '.join(changes_made)}")
                total_files_migrated += 1
                total_changes += len(changes_made)
                
                # Clean up duplicate imports
                if clean_up_duplicate_imports(file_path):
                    print(f"  🧹 Cleaned up duplicate imports")
            else:
                print(f"  ℹ️  No changes needed")
        
        print()
    
    print("=" * 60)
    print(f"🎉 Migration Summary:")
    print(f"  📁 Files processed: {len(files_with_usage)}")
    print(f"  ✅ Files migrated: {total_files_migrated}")
    print(f"  🔧 Total changes: {total_changes}")
    
    if not dry_run:
        print()
        print("🎯 Next steps:")
        print("  1. Run `cargo check` to verify migration")
        print("  2. Review any remaining warnings")
        print("  3. Remove .health-status-bak files if everything works")
        print("  4. Commit the changes")
    else:
        print()
        print("🎯 Dry run complete. Use without --dry-run to apply changes.")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 