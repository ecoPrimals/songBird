#!/usr/bin/env python3
"""Fix remaining syntax errors comprehensively."""

import re
from pathlib import Path

def fix_all_patterns(content):
    """Fix all remaining syntax patterns."""
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        stripped = line.rstrip()
        original = line
        
        # Pattern 1: tracing::macro!(...) without semicolon
        if re.search(r'tracing::(info|debug|warn|error|trace)!\([^)]*\)\s*$', stripped):
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if next_line and not next_line.startswith('}') and not next_line.startswith(')'):
                    if not stripped.endswith(';'):
                        stripped += ';'
        
        # Pattern 2: warn!(...) or info!(...) without tracing:: prefix
        elif re.search(r'^\s*(info|debug|warn|error|trace)!\([^)]*\)\s*$', stripped):
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if next_line and not next_line.startswith('}') and not next_line.startswith(')'):
                    if not stripped.endswith(';') and not stripped.endswith(','):
                        stripped += ';'
        
        # Pattern 3: Comments ending lines that should have semicolons
        elif re.search(r'(info|debug|warn|error|trace)!\([^)]*\)\s*//.*$', stripped):
            # Check if there's already a semicolon before the comment
            if not re.search(r'(info|debug|warn|error|trace)!\([^)]*\);', stripped):
                # Insert semicolon before comment
                stripped = re.sub(r'(\))\s*(//.*)', r'\1;\2', stripped)
        
        fixed_lines.append(stripped if stripped else original)
    
    return '\n'.join(fixed_lines)

def fix_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_all_patterns(content)
        
        if content != fixed_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    crates_dir = Path("crates")
    rust_files = list(crates_dir.rglob("*.rs"))
    
    fixed_count = 0
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"Fixed: {rust_file}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

