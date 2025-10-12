#!/usr/bin/env python3
"""
Systematic Syntax Error Fixer for Songbird
Fixes common patterns: missing closing parentheses, unclosed delimiters
"""

import re
import sys
from pathlib import Path

def fix_missing_closing_paren(content):
    """Fix common pattern: Err(format!("...").into() -> Err(format!("...").into())"""
    # Pattern: .into() without closing paren before it
    pattern = r'(Err\([^)]+\.into\()(?!\))'
    
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        # Check if line has .into() without closing paren
        if '.into()' in line and 'Err(' in line:
            # Count opening and closing parens
            open_count = line.count('(')
            close_count = line.count(')')
            
            # If unbalanced and ends with .into(), add closing paren
            if open_count > close_count and line.strip().endswith('.into()'):
                line = line.replace('.into()', '.into())')
        
        fixed_lines.append(line)
    
    return '\n'.join(fixed_lines)

def fix_unwrap_or_else(content):
    """Fix: .unwrap_or_else(|| "...".to_string(); -> .unwrap_or_else(|| "...".to_string());"""
    pattern = r'\.unwrap_or_else\(\|\| ([^)]+)\.to_string\(\);'
    replacement = r'.unwrap_or_else(|| \1.to_string());'
    return re.sub(pattern, replacement, content)

def fix_hashmap_new(content):
    """Fix: HashMap::new(); -> HashMap::new());"""
    content = content.replace('HashMap::new();', 'HashMap::new());')
    return content

def fix_arc_new(content):
    """Fix: Arc::new(...; -> Arc::new(...);"""
    pattern = r'Arc::new\(([^)]+);'
    
    lines = content.split('\n')
    fixed_lines = []
    
    for line in lines:
        if 'Arc::new(' in line:
            # Count parentheses
            open_count = line.count('(')
            close_count = line.count(')')
            
            # If ends with semicolon but missing closing paren
            if open_count > close_count and ';' in line:
                line = line.replace(';', ');')
        
        fixed_lines.append(line)
    
    return '\n'.join(fixed_lines)

def fix_ok_return(content):
    """Fix: Ok(...( -> Ok(...))"""
    pattern = r'Ok\(([^)]+)\('
    
    lines = content.split('\n')
    fixed_lines = []
    
    for line in lines:
        if 'Ok(' in line and line.strip().endswith('('):
            # Missing closing parens
            line = line.rstrip('(') + '))'
        
        fixed_lines.append(line)
    
    return '\n'.join(fixed_lines)

def fix_missing_insert_paren(content):
    """Fix: .insert(...)); -> .insert(...);"""
    # This fixes extra closing parens
    pattern = r'\.insert\(([^)]+)\)\);'
    replacement = r'.insert(\1);'
    return re.sub(pattern, replacement, content)

def fix_clone_semicolon(content):
    """Fix: .clone(); in wrong context -> .clone())"""
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        if '.clone();' in line:
            # Check if previous line or context suggests this should be .clone())
            if i > 0:
                prev = lines[i-1].strip()
                if prev.endswith('(') or 'push(' in prev or 'insert(' in prev:
                    line = line.replace('.clone();', '.clone())')
        
        fixed_lines.append(line)
    
    return '\n'.join(fixed_lines)

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        
        # Apply all fixes
        content = fix_missing_closing_paren(content)
        content = fix_unwrap_or_else(content)
        content = fix_hashmap_new(content)
        content = fix_arc_new(content)
        content = fix_ok_return(content)
        content = fix_clone_semicolon(content)
        
        if content != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        
        return False
    
    except Exception as e:
        print(f"Error processing {filepath}: {e}", file=sys.stderr)
        return False

def main():
    """Main entry point"""
    project_root = Path(__file__).parent
    crates_dir = project_root / 'crates'
    
    if not crates_dir.exists():
        print(f"Error: {crates_dir} not found", file=sys.stderr)
        return 1
    
    # Find all .rs files
    rs_files = list(crates_dir.rglob('*.rs'))
    
    print(f"Found {len(rs_files)} Rust files")
    
    fixed_count = 0
    for filepath in rs_files:
        if process_file(filepath):
            fixed_count += 1
            print(f"Fixed: {filepath.relative_to(project_root)}")
    
    print(f"\nProcessed {len(rs_files)} files, fixed {fixed_count} files")
    return 0

if __name__ == '__main__':
    sys.exit(main())

