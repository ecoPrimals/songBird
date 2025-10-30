#!/usr/bin/env python3
"""
Targeted Types Crate Syntax Repair

This script fixes specific syntax issues in the songbird-types crate:
- Malformed struct field definitions
- Missing field names and types
- Incorrect enum syntax
- Broken impl blocks
"""

import os
import re
from pathlib import Path

def fix_struct_syntax(content: str) -> str:
    """Fix malformed struct definitions"""
    
    # Fix trailing commas without field names
    content = re.sub(r'(\w+:\s*[^,}]+)\s*,\s*,', r'\1,', content)
    
    # Fix empty field lines
    content = re.sub(r'^\s*,\s*$', '', content, flags=re.MULTILINE)
    
    # Fix semicolons in struct definitions
    content = re.sub(r'(\s+);\s*$', r'', content, flags=re.MULTILINE)
    
    # Fix Default impl blocks
    content = re.sub(r'(impl Default for \w+\s*\{\s*fn default\(\) -> Self\s*\{\s*Self\s*\{[^}]*)\s*;\s*', 
                     r'\1', content)
    
    return content

def fix_enum_syntax(content: str) -> str:
    """Fix malformed enum definitions"""
    
    # Fix enum variants with semicolons
    content = re.sub(r'(pub enum \w+\s*\{[^}]*)\s*;\s*\}', r'\1\n}', content)
    
    return content

def fix_impl_blocks(content: str) -> str:
    """Fix malformed impl blocks"""
    
    # Fix impl blocks with semicolons instead of proper structure
    content = re.sub(r'(impl \w+\s*\{[^}]*)\s*;\s*([^}]*)\s*}', r'\1\n    \2\n}', content)
    
    return content

def fix_import_syntax(content: str) -> str:
    """Fix malformed import statements"""
    
    # Fix :: syntax in imports
    content = re.sub(r':\s*:', '::', content)
    
    return content

def fix_function_syntax(content: str) -> str:
    """Fix malformed function definitions"""
    
    # Fix function parameters and return types
    content = re.sub(r'fn from\(\) -> Self\s*\{', r'fn from(error: serde_json::Error) -> Self {', content)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix syntax errors in a single file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return False
        
    original_content = content
    
    # Apply all fixes
    content = fix_struct_syntax(content)
    content = fix_enum_syntax(content)
    content = fix_impl_blocks(content)
    content = fix_import_syntax(content)
    content = fix_function_syntax(content)
    
    # Write back if changes were made
    if content != original_content:
        try:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        except Exception as e:
            print(f"Error writing {file_path}: {e}")
            return False
            
    return False

def main():
    """Fix all syntax errors in songbird-types crate"""
    types_dir = Path('crates/songbird-types')
    
    if not types_dir.exists():
        print("Error: songbird-types directory not found")
        return
    
    rust_files = list(types_dir.rglob('*.rs'))
    fixed_count = 0
    
    print(f"🔧 Fixing {len(rust_files)} Rust files in songbird-types")
    
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"✅ Fixed: {rust_file.relative_to(types_dir)}")
    
    print(f"\n📊 Fixed {fixed_count} files")

if __name__ == '__main__':
    main() 