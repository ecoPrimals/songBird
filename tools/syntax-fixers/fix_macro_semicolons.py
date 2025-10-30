#!/usr/bin/env python3
"""
Fix macro semicolon errors across the codebase.
Pattern: println!(...); → println!(...)
Same for debug!, info!, warn!, error!, assert!, etc.
"""

import re
import sys
from pathlib import Path

def fix_macro_semicolons(content):
    """Fix semicolons after macro invocations."""
    # Pattern matches: macro_name!(...);
    # But NOT: macro_name!(...), or macro_name!(...) at end of line
    pattern = r'((?:println|debug|info|warn|error|trace|assert|assert_eq|assert_ne|panic|todo|unimplemented|unreachable|format|vec|write|writeln)!\([^)]*\));'
    
    fixed = re.sub(pattern, r'\1', content)
    return fixed

def fix_file(file_path):
    """Fix a single Rust file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_macro_semicolons(content)
        
        if content != fixed_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return False

def main():
    """Process all Rust files in crates/."""
    crates_dir = Path("crates")
    if not crates_dir.exists():
        print("Error: crates/ directory not found", file=sys.stderr)
        sys.exit(1)
    
    rust_files = list(crates_dir.rglob("*.rs"))
    fixed_count = 0
    
    print(f"Processing {len(rust_files)} Rust files...")
    
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"Fixed: {rust_file}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

