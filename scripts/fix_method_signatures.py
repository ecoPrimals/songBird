#!/usr/bin/env python3
"""
Method Signature Repair Script

This script fixes method signatures that are missing self parameters,
which is causing many compilation errors in the types crate.
"""

import re
from pathlib import Path

def fix_method_signatures(content: str) -> str:
    """Fix method signatures that need self parameters"""
    
    # Common patterns that need &self parameter
    fixes = [
        # Methods that clearly need &self (accessing self fields/methods)
        (r'pub fn (with_\w+)\(\) -> Self\s*\{', r'pub fn \1(&mut self) -> Self {'),
        (r'pub fn (as_\w+)\(\) -> ([^{]+)\{', r'pub fn \1(&self) -> \2{'),
        (r'pub fn (is_\w+)\(\) -> bool\s*\{', r'pub fn \1(&self) -> bool {'),
        (r'pub fn (url)\(\) -> String\s*\{', r'pub fn \1(&self) -> String {'),
        (r'pub fn (get_\w+)\(\) -> ([^{]+)\{', r'pub fn \1(&self) -> \2{'),
        
        # Methods that need mutable self
        (r'pub fn (clone_arc)\(\) -> ([^{]+)\{', r'pub fn \1(&self) -> \2{'),
        (r'pub fn (get_mut)\(\) -> ([^{]+)\{', r'pub fn \1(&mut self) -> \2{'),
        
        # Trait methods that need self
        (r'fn (into_shared)\(\) -> ([^{]+)\{', r'fn \1(self) -> \2{'),
        
        # Constructor-style methods that need parameters
        (r'pub fn (new)\(\) -> Self\s*\{', r'pub fn \1(host: impl Into<String>, port: u16, protocol: impl Into<String>) -> Self {'),
        (r'pub fn (success)\(\) -> Self\s*\{', r'pub fn \1(data: T) -> Self {'),
        (r'pub fn (error)\(\) -> Self\s*\{', r'pub fn \1(request_id: impl Into<String>, error: impl Into<String>) -> Self {'),
        
        # Methods with parameters that are missing them
        (r'pub fn (with_type)\(\) -> Self\s*\{', r'pub fn \1(&mut self, addr_type: impl Into<String>) -> Self {'),
        (r'pub fn (with_city)\(\) -> Self\s*\{', r'pub fn \1(&mut self, city: impl Into<String>) -> Self {'),
        (r'pub fn (with_country)\(\) -> Self\s*\{', r'pub fn \1(&mut self, country: impl Into<String>) -> Self {'),
        (r'pub fn (with_path)\(\) -> Self\s*\{', r'pub fn \1(&mut self, path: impl Into<String>) -> Self {'),
        (r'pub fn (with_metadata)\(\) -> Self\s*\{', r'pub fn \1(&mut self, key: impl Into<String>, value: impl Into<String>) -> Self {'),
        (r'pub fn (with_capability)\(\) -> Self\s*\{', r'pub fn \1(&mut self, capability: impl Into<String>) -> Self {'),
        (r'pub fn (with_dependency)\(\) -> Self\s*\{', r'pub fn \1(&mut self, dependency: impl Into<String>) -> Self {'),
        (r'pub fn (with_description)\(\) -> Self\s*\{', r'pub fn \1(&mut self, description: impl Into<String>) -> Self {'),
        
        # Associated functions that need parameters  
        (r'fn (is_known_field)\(\) -> bool\s*\{', r'fn \1(_key: &str) -> bool {'),
        (r'pub fn (validate)\(\) -> Result<\(\), String>\s*\{', r'pub fn \1(&self) -> Result<(), String> {'),
    ]
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_empty_function_bodies(content: str) -> str:
    """Fix empty function bodies that have trailing semicolons"""
    
    fixes = [
        # Remove trailing semicolons from const functions
        (r'pub const fn (\w+)\(\) -> ([^{]+)\{;', r'pub const fn \1() -> \2 { unimplemented!() }'),
        
        # Fix return statements in functions
        (r'(\s+);  }', r'\1 }'),
        (r'(\s+);;}', r'\1 }'),
    ]
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix method signatures in a file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return False
        
    original_content = content
    
    # Apply fixes
    content = fix_method_signatures(content)
    content = fix_empty_function_bodies(content)
    
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
    """Fix method signatures in songbird-types crate"""
    types_dir = Path('crates/songbird-types')
    
    if not types_dir.exists():
        print("Error: songbird-types directory not found")
        return
    
    rust_files = list(types_dir.rglob('*.rs'))
    fixed_count = 0
    
    print(f"🔧 Fixing method signatures in {len(rust_files)} Rust files")
    
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"✅ Fixed: {rust_file.relative_to(types_dir)}")
    
    print(f"\n📊 Fixed {fixed_count} files")

if __name__ == '__main__':
    main() 