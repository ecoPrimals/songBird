#!/usr/bin/env python3
"""
Fix complex SongbirdResponse patterns
Handles struct construction, function calls, and method chaining
"""

import re
from pathlib import Path

def fix_struct_construction(content):
    """Fix Ok(StructName { ... }) patterns"""
    # Pattern: Ok(Struct { field: value, ... })
    # Need to handle multi-line struct construction
    
    # Find Ok( followed by a struct with braces
    pattern = r'Ok\((\w+)\s*\{([^}]+)\}\)'
    
    def replace_struct(match):
        struct_name = match.group(1)
        fields = match.group(2)
        # Don't wrap if already SongbirdResponse
        if 'SongbirdResponse' in struct_name:
            return match.group(0)
        return f'Ok(SongbirdResponse::success({struct_name} {{{fields}}})'
    
    return re.sub(pattern, replace_struct, content)

def fix_function_calls(content):
    """Fix Ok(function_call()) patterns"""
    # Pattern: Ok(func(...)) where func is not SongbirdResponse
    pattern = r'\bOk\(([a-z_][a-z0-9_]*(?:::[a-z_][a-z0-9_]*)*)\(([^)]*)\)\)(?!\s*\.)'
    
    def replace_call(match):
        func = match.group(1)
        args = match.group(2)
        # Skip if already wrapped
        if 'songbird' in func.lower() or 'response' in func.lower():
            return match.group(0)
        return f'Ok(SongbirdResponse::success({func}({args})))'
    
    return re.sub(pattern, replace_call, content)

def fix_format_macro(content):
    """Fix Ok(format!(...)) patterns"""
    pattern = r'\bOk\(format!\(([^)]+)\)\)'
    
    def replace_format(match):
        args = match.group(1)
        return f'Ok(SongbirdResponse::success(format!({args})))'
    
    return re.sub(pattern, replace_format, content)

def fix_box_new(content):
    """Fix Ok(Box::new(...)) patterns"""
    pattern = r'\bOk\(Box::new\(([^)]+)\)\)(?!\s*\.)'
    
    def replace_box(match):
        inner = match.group(1)
        return f'Ok(SongbirdResponse::success(Box::new({inner})))'
    
    content = re.sub(pattern, replace_box, content)
    
    # Also handle: Ok(Box::new(...) as Box<dyn Trait>)
    pattern2 = r'\bOk\((Box::new\([^)]+\)\s+as\s+[^)]+)\)(?!\s*\.)'
    
    def replace_box_cast(match):
        expr = match.group(1)
        return f'Ok(SongbirdResponse::success({expr}))'
    
    return re.sub(pattern2, replace_box_cast, content)

def fix_method_chains(content):
    """Fix Ok(value.method()) patterns"""
    # This is tricky - need to detect method calls
    pattern = r'\bOk\(([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)\(\)\)'
    
    def replace_method(match):
        var = match.group(1)
        method = match.group(2)
        return f'Ok(SongbirdResponse::success({var}.{method}()))'
    
    return re.sub(pattern, replace_method, content)

def fix_collection_operations(content):
    """Fix Ok(vec.clone()), Ok(iter.collect()), etc."""
    # Ok(value.to_vec())
    pattern1 = r'\bOk\(([a-z_][a-z0-9_]*)\.to_vec\(\)\)'
    content = re.sub(pattern1, r'Ok(SongbirdResponse::success(\1.to_vec()))', content)
    
    # Ok(value.clone())
    pattern2 = r'\bOk\(([a-z_][a-z0-9_]*)\.clone\(\)\)'
    content = re.sub(pattern2, r'Ok(SongbirdResponse::success(\1.clone()))', content)
    
    return content

def fix_enum_construction(content):
    """Fix Ok(EnumVariant { ... }) patterns"""
    pattern = r'\bOk\(([A-Z]\w*)::([\w]+)\s*\{([^}]+)\}\)'
    
    def replace_enum(match):
        enum_name = match.group(1)
        variant = match.group(2)
        fields = match.group(3)
        if 'SongbirdResponse' in enum_name:
            return match.group(0)
        return f'Ok(SongbirdResponse::success({enum_name}::{variant} {{{fields}}})'
    
    return re.sub(pattern, replace_enum, content)

def needs_fixes(content):
    """Check if file has patterns that need fixing"""
    patterns = [
        r'\bOk\(\w+\s*\{',  # Struct construction
        r'\bOk\([a-z_][a-z0-9_]*\(',  # Function calls
        r'\bOk\(format!',  # format! macro
        r'\bOk\(Box::new',  # Box::new
        r'\bOk\([a-z_][a-z0-9_]*\.[a-z_]',  # Method calls
    ]
    
    for pattern in patterns:
        if re.search(pattern, content) and 'SongbirdResponse' not in re.search(pattern, content).group(0):
            return True
    return False

def fix_file(filepath):
    """Apply all fixes to a file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    if not needs_fixes(content):
        return False
    
    original = content
    
    # Apply fixes in order
    content = fix_struct_construction(content)
    content = fix_enum_construction(content)
    content = fix_function_calls(content)
    content = fix_format_macro(content)
    content = fix_box_new(content)
    content = fix_method_chains(content)
    content = fix_collection_operations(content)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    """Fix all files in songbird-network/src"""
    base_path = Path('crates/songbird-network/src')
    
    if not base_path.exists():
        print(f"❌ Path not found: {base_path}")
        return 1
    
    files = list(base_path.rglob('*.rs'))
    print(f"🔍 Processing {len(files)} Rust files for complex patterns")
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

