#!/usr/bin/env python3
"""
Fix unclosed delimiters in Rust test files.
Adds missing closing braces and #[test] attributes.
"""

import re
import sys
from pathlib import Path

def fix_test_file(filepath):
    """Fix common test file issues."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    lines = content.split('\n')
    fixed_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this is a function definition without #[test]
        if re.match(r'^fn test_\w+\(', line) and (i == 0 or not lines[i-1].strip().startswith('#[test]')):
            # Add #[test] attribute
            fixed_lines.append('#[test]')
        
        fixed_lines.append(line)
        
        # Check for incomplete blocks
        if re.match(r'^fn test_\w+\(', line):
            # Count braces in the function
            open_braces = 0
            func_lines = [line]
            j = i + 1
            
            while j < len(lines):
                func_lines.append(lines[j])
                open_braces += lines[j].count('{')
                open_braces -= lines[j].count('}')
                
                # If we've closed all braces or hit another function, stop
                if open_braces == 0 or (j > i and re.match(r'^fn test_\w+\(', lines[j])):
                    break
                j += 1
            
            # If braces aren't balanced, add closing brace
            if open_braces > 0:
                fixed_lines.append('}' * open_braces)
        
        i += 1
    
    return '\n'.join(fixed_lines)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: fix_tests.py <file>")
        sys.exit(1)
    
    filepath = Path(sys.argv[1])
    if not filepath.exists():
        print(f"File not found: {filepath}")
        sys.exit(1)
    
    print(f"Fixing {filepath}...")
    fixed_content = fix_test_file(filepath)
    
    # Write to temp file first
    temp_path = filepath.with_suffix('.rs.fixed')
    with open(temp_path, 'w') as f:
        f.write(fixed_content)
    
    print(f"Fixed content written to {temp_path}")
    print("Review the file, then rename to replace original if correct.")

