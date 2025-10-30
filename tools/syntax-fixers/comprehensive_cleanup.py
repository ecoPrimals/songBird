#!/usr/bin/env python3
"""
Comprehensive cleanup script for remaining Songbird syntax errors.
Handles all remaining semicolon and formatting issues.
"""

import os
import re
from pathlib import Path

def fix_comprehensive(file_path):
    """Apply all comprehensive fixes to a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        
        # Fix format! calls that need semicolons before next statement
        content = re.sub(
            r'(\s+)(format!\([^)]+\))\n(\s+)(assert_eq!|get_bind_address|bind_str|registry|Ok\(|let\s)',
            r'\1\2;\n\3\4',
            content
        )
        
        # Fix assert_eq!/assert! before let or other statements
        content = re.sub(
            r'(\s+)(assert(?:_eq)?!\([^)]+\))\n(\s+)(let\s|Ok\()',
            r'\1\2;\n\3\4',
            content
        )
        
        # Fix other macros before keywords
        content = re.sub(
            r'(\s+)(println!|info!|debug!|warn!|error!)\(([^)]+)\)\n(\s+)(let\s|match\s|if\s|for\s|while\s)',
            r'\1\2(\3);\n\4\5',
            content
        )
        
        # Fix string literals that need semicolons
        content = re.sub(
            r'(\s+)"([^"]+)"\n(\s+)(get_bind_address|assert_eq!)',
            r'\1"\2";\n\3\4',
            content
        )
        
        if content != original:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Process all Rust files in the crates directory."""
    crates_dir = Path('/home/eastgate/Development/ecoPrimals/songbird/crates')
    
    fixed_count = 0
    for rust_file in crates_dir.rglob('*.rs'):
        if fix_comprehensive(rust_file):
            fixed_count += 1
            print(f"Fixed: {rust_file}")
    
    print(f"\nTotal files fixed: {fixed_count}")

if __name__ == '__main__':
    main()

