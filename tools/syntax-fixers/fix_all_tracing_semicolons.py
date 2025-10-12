#!/usr/bin/env python3
"""Fix all tracing macro semicolons across the codebase."""

import re
from pathlib import Path

def fix_tracing_macros(content):
    """Fix tracing::*!(...) patterns that need semicolons."""
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        stripped = line.rstrip()
        # Check if line ends with tracing macro call
        if re.search(r'(info|debug|warn|error|trace)!\([^)]*\)\s*$', stripped):
            # Check if it needs a semicolon
            needs_semicolon = False
            
            # Look at next line
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                # Needs semicolon if next line starts code (not closing braces/parens)
                if next_line and not next_line.startswith('}') and not next_line.startswith(')') and not next_line.startswith(','):
                    # Also check it doesn't already have a semicolon
                    if not stripped.endswith(';'):
                        needs_semicolon = True
            
            if needs_semicolon:
                stripped += ';'
        
        fixed_lines.append(stripped if stripped else line)
    
    return '\n'.join(fixed_lines)

def fix_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_tracing_macros(content)
        
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

