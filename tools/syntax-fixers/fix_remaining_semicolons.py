#!/usr/bin/env python3
"""Fix remaining tracing macro semicolons."""

import re
from pathlib import Path

def fix_tracing_macros(content):
    """Fix tracing::*!(...) patterns."""
    # Match tracing::macro_name!(...) (with or without trailing text in parens)
    pattern = r'(tracing::(?:info|debug|warn|error|trace)!\([^)]*\))\s*$'
    
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        # Check if line ends with tracing macro call without semicolon
        if re.search(pattern, line):
            # Check next line to see if it starts code (not closing braces)
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if next_line and not next_line.startswith('}') and not next_line.startswith(')'):
                    # Add semicolon
                    line = line.rstrip() + ';'
        fixed_lines.append(line)
    
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
    files_to_check = [
        "crates/songbird-canonical/src/performance.rs",
        "crates/songbird-config/src/canonical_network.rs",
    ]
    
    fixed_count = 0
    for file_path in files_to_check:
        path = Path(file_path)
        if path.exists() and fix_file(path):
            fixed_count += 1
            print(f"Fixed: {file_path}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

