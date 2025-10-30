#!/usr/bin/env python3
"""
Error System Unification Script

This script updates all imports from the fragmented error systems to use
the canonical error system from songbird-types.
"""

import os
import re
import sys
from pathlib import Path

def update_error_imports(file_path):
    """Update error imports in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Replace import statements
        replacements = [
            # Replace songbird-security-errors imports
            (r'use songbird_security_errors::', 'use songbird_types::'),
            (r'use songbird_security_errors\s*;', 'use songbird_types::*;'),
            
            # Replace specific error type imports
            (r'use.*?songbird_security_errors::errors::(.*?);', r'use songbird_types::\1;'),
            (r'use.*?songbird_security_errors::(.*?);', r'use songbird_types::\1;'),
            
            # Replace error type usage in code
            (r'songbird_security_errors::', 'songbird_types::'),
            
            # Update specific error types that might be used directly
            (r'\bPrimalError\b', 'SongbirdError'),
            (r'\bPrimalResult\b', 'SongbirdResult'),
            
            # Update error constructor calls
            (r'PrimalError::', 'SongbirdError::'),
            
            # Update From implementations
            (r'impl From<.*?> for PrimalError', 'impl From<_> for SongbirdError'),
        ]
        
        for pattern, replacement in replacements:
            content = re.sub(pattern, replacement, content)
        
        # Write back if changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Updated: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error updating {file_path}: {e}")
        return False

def main():
    """Main function to update all files."""
    crates_dir = Path("crates")
    
    if not crates_dir.exists():
        print("Error: crates/ directory not found. Run from project root.")
        sys.exit(1)
    
    # Find all Rust files that import from songbird-security-errors
    rust_files = []
    for crate_dir in crates_dir.iterdir():
        if crate_dir.is_dir():
            for rust_file in crate_dir.rglob("*.rs"):
                try:
                    with open(rust_file, 'r', encoding='utf-8') as f:
                        content = f.read()
                        if 'songbird_security_errors' in content:
                            rust_files.append(rust_file)
                except Exception:
                    continue
    
    print(f"Found {len(rust_files)} files to update")
    
    updated_count = 0
    for file_path in rust_files:
        if update_error_imports(file_path):
            updated_count += 1
    
    print(f"\nUpdated {updated_count} files")
    print("Error system unification complete!")

if __name__ == "__main__":
    main() 