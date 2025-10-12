#!/usr/bin/env python3
"""
Fix missing closing parentheses from Ok(SongbirdResponse::success(Struct {...}))
The pattern should be: Ok(SongbirdResponse::success(Struct {...})) with 2 closing parens
"""

import re
from pathlib import Path

def fix_missing_parens(content):
    """Find and fix patterns where Ok(SongbirdResponse::success(X {...}) is missing a closing paren"""
    
    # Pattern: Ok(SongbirdResponse::success(THING {...})) where one ) is missing
    # We need to find lines with `})` followed by newline but should have `}))`
    
    # Look for: Ok(SongbirdResponse::success( ... {field: value, ...})
    # Should be: Ok(SongbirdResponse::success( ... {field: value, ...}))
    
    lines = content.split('\n')
    fixed = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this line contains Ok(SongbirdResponse::success(
        if 'Ok(SongbirdResponse::success(' in line and '{' in line:
            # This is the start of a pattern
            # Find the matching closing brace
            open_braces = line.count('{') - line.count('}')
            open_parens = line.count('(') - line.count(')')
            
            # Look ahead to find where the struct ends
            j = i + 1
            struct_lines = [line]
            
            while j < len(lines) and open_braces > 0:
                next_line = lines[j]
                struct_lines.append(next_line)
                open_braces += next_line.count('{') - next_line.count('}')
                j += 1
            
            # Now check if we need an extra closing paren
            full_pattern = '\n'.join(struct_lines)
            
            # Count parens in the full pattern
            total_open = full_pattern.count('(')
            total_close = full_pattern.count(')')
            
            if total_open > total_close:
                # Need to add closing parens
                # Find the last line with `})` and change to `}))`
                for k in range(len(struct_lines) - 1, -1, -1):
                    if '})' in struct_lines[k] and ')' not in struct_lines[k].split('})')[-1].strip():
                        struct_lines[k] = struct_lines[k].replace('})', '}))', 1)
                        break
                
                fixed.extend(struct_lines)
                i = j
                continue
        
        fixed.append(line)
        i += 1
    
    return '\n'.join(fixed)

def simpler_fix(content):
    """Simpler approach: find lines ending with }) that should be })"""
    # Pattern: any line with ONLY whitespace and }) that comes after Ok(SongbirdResponse::success(
    # and should be }))
    
    lines = content.split('\n')
    fixed = []
    in_songbird_response = False
    brace_depth = 0
    paren_depth = 0
    
    for i, line in enumerate(lines):
        stripped = line.strip()
        
        # Check if we're starting a SongbirdResponse::success pattern
        if 'Ok(SongbirdResponse::success(' in line:
            in_songbird_response = True
            brace_depth = line.count('{') - line.count('}')
            paren_depth = line.count('(') - line.count(')')
        elif in_songbird_response:
            brace_depth += line.count('{') - line.count('}')
            paren_depth += line.count('(') - line.count(')')
            
            # Check if this is the end of the struct (closing brace)
            if brace_depth == 0 and stripped in [')', '},)', '})']:
                # We've closed all braces, check if we need more close parens
                if paren_depth > 0 and stripped == '})':
                    line = line.replace('})', '}))')
                    paren_depth -= 1
                elif paren_depth > 0 and stripped == '}':
                    line = line.replace('}', '})')
                    paren_depth -= 1
                    
                if paren_depth <= 0:
                    in_songbird_response = False
        
        fixed.append(line)
    
    return '\n'.join(fixed)

def fix_file(filepath):
    """Fix missing parentheses in a file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    content = simpler_fix(content)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    """Fix all files in songbird-network/src"""
    base_path = Path('crates/songbird-network/src')
    
    files = list(base_path.rglob('*.rs'))
    print(f"🔧 Fixing missing parentheses in {len(files)} files")
    print("=" * 60)
    
    fixed_count = 0
    for filepath in files:
        if fix_file(filepath):
            fixed_count += 1
            print(f"✓ Fixed: {filepath}")
    
    print("=" * 60)
    print(f"✅ Fixed {fixed_count} files")
    return 0

if __name__ == '__main__':
    import sys
    sys.exit(main())

