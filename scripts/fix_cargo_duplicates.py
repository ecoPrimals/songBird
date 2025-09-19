#!/usr/bin/env python3
"""
Fix duplicate dependencies in Cargo.toml files
"""

import os
import re
from pathlib import Path

def fix_cargo_toml(file_path: Path):
    """Remove duplicate songbird-types dependencies"""
    content = file_path.read_text()
    
    # Find all songbird-types dependencies
    pattern = r'songbird-types = \{ path = "\.\./songbird-types" \}'
    matches = list(re.finditer(pattern, content))
    
    if len(matches) <= 1:
        return False  # No duplicates
    
    # Keep only the first occurrence
    lines = content.split('\n')
    new_lines = []
    found_first = False
    
    for line in lines:
        if re.search(pattern, line):
            if not found_first:
                new_lines.append(line)
                found_first = True
            # Skip subsequent duplicates
        else:
            new_lines.append(line)
    
    new_content = '\n'.join(new_lines)
    if new_content != content:
        file_path.write_text(new_content)
        print(f"Fixed duplicates in: {file_path}")
        return True
    
    return False

def main():
    """Fix all Cargo.toml files"""
    crates_dir = Path('crates')
    fixed_count = 0
    
    for cargo_toml in crates_dir.rglob('Cargo.toml'):
        if fix_cargo_toml(cargo_toml):
            fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == '__main__':
    main() 