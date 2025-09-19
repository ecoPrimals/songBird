#!/usr/bin/env python3
"""
Systematic Error Pattern Migration Script

This script fixes all deprecated error patterns in the Songbird codebase
to use the modern unified error handling API.
"""

import os
import re
import sys
from pathlib import Path

# Error pattern mappings
ERROR_PATTERNS = {
    # Old pattern -> New pattern
    r'SongbirdError::internal_error\(([^)]+)\)': r'SongbirdError::internal("component", \1)',
    r'SongbirdError::service_error\(([^,]+),\s*([^)]+)\)': r'SongbirdError::service(\1, \2)',
    r'SongbirdError::validation_error\(([^)]+)\)': r'SongbirdError::config_general(\1)',
    r'SongbirdError::operation_error\(([^)]+)\)': r'SongbirdError::internal("operation", \1)',
    r'SongbirdError::network_error\(([^,]+),\s*([^)]+)\)': r'SongbirdError::network(\1, \2)',
    r'SongbirdError::config_error\(([^,]+),\s*([^)]+)\)': r'SongbirdError::config(\1, \2)',
}

# Import pattern fixes
IMPORT_PATTERNS = {
    r'use songbird_config::config::': 'use songbird_config::',
    r'songbird_config::config::': 'songbird_config::',
}

def fix_file(file_path: Path) -> bool:
    """Fix error patterns in a single file"""
    try:
        content = file_path.read_text(encoding='utf-8')
        original_content = content
        
        # Apply error pattern fixes
        for old_pattern, new_pattern in ERROR_PATTERNS.items():
            content = re.sub(old_pattern, new_pattern, content)
        
        # Apply import pattern fixes
        for old_pattern, new_pattern in IMPORT_PATTERNS.items():
            content = re.sub(old_pattern, new_pattern, content)
        
        # Write back if changed
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            print(f"Fixed: {file_path}")
            return True
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        
    return False

def main():
    """Main migration function"""
    repo_root = Path(__file__).parent.parent
    crates_dir = repo_root / "crates"
    
    if not crates_dir.exists():
        print(f"Crates directory not found: {crates_dir}")
        sys.exit(1)
    
    fixed_files = 0
    total_files = 0
    
    # Process all Rust files in crates
    for rust_file in crates_dir.rglob("*.rs"):
        if rust_file.is_file():
            total_files += 1
            if fix_file(rust_file):
                fixed_files += 1
    
    print(f"\nMigration complete!")
    print(f"Processed: {total_files} files")
    print(f"Fixed: {fixed_files} files")
    
    if fixed_files > 0:
        print(f"\nRun 'cargo check' to verify the fixes.")

if __name__ == "__main__":
    main() 