#!/usr/bin/env python3
"""
Builder Pattern Repair Script

This script fixes builder pattern methods that incorrectly return Self instead of &mut Self,
and adds missing parameters to constructor methods.
"""

import re
from pathlib import Path

def fix_builder_methods(content: str) -> str:
    """Fix builder methods that should return &mut Self"""
    
    # Fix methods that modify self and should return &mut Self
    builder_patterns = [
        # with_* methods that modify self should return &mut Self
        (r'pub fn (with_\w+)\(&mut self[^)]*\) -> Self \{', r'pub fn \1(&mut self, \2) -> &mut Self {'),
    ]
    
    # Fix specific method signatures that need parameters
    parameter_fixes = [
        # Add missing parameters to with_* methods
        (r'pub fn (with_endpoint)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, name: impl Into<String>, url: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_metadata)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_capability)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, capability: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_dependency)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, dependency: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_description)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, description: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_path)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, path: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_type)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, addr_type: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_city)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, city: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_country)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, country: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_context)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, context: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_confidence)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, confidence: f64) -> &mut Self {'),
        (r'pub fn (with_action)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, action: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_security_level)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, level: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_config)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {'),
        (r'pub fn (with_metric)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, key: impl Into<String>, metric_value: f64) -> &mut Self {'),
        (r'pub fn (with_component)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self, name: impl Into<String>, status: bool) -> &mut Self {'),
        
        # Fix security/storage/compute methods
        (r'pub fn (with_security)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self) -> &mut Self {'),
        (r'pub fn (with_storage)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self) -> &mut Self {'),
        (r'pub fn (with_compute)\(&mut self\) -> &mut Self \{', r'pub fn \1(&mut self) -> &mut Self {'),
    ]
    
    # Apply builder pattern fixes first
    for pattern, replacement in builder_patterns:
        # Extract parameter part for methods that need it
        matches = re.finditer(pattern, content)
        for match in matches:
            method_name = match.group(1)
            # Skip methods that already have parameters
            if method_name not in ['with_security', 'with_storage', 'with_compute']:
                continue
        
    # Apply parameter fixes
    for pattern, replacement in parameter_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    # Fix return statements to return &mut self instead of self
    content = re.sub(r'(\s+)self(\s*})$', r'\1&mut self\2', content, flags=re.MULTILINE)
    
    return content

def fix_constructor_methods(content: str) -> str:
    """Fix constructor methods that need parameters"""
    
    constructor_fixes = [
        # Add parameters to constructors that reference them
        (r'pub fn new\(host: impl Into<String>, port: u16, protocol: impl Into<String>\) -> Self \{([^}]+)host,([^}]+)protocol,', 
         r'pub fn new(host: impl Into<String>, port: u16, protocol: impl Into<String>) -> Self {\1host: host.into(),\2protocol: protocol.into(),'),
        
        # Fix new methods for different types
        (r'pub fn new\(([^)]+)\) -> Self \{([^}]+)Self \{([^}]+)\}', 
         r'pub fn new(\1) -> Self {\2Self {\3}'),
    ]
    
    for pattern, replacement in constructor_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_generic_bounds(content: str) -> str:
    """Add missing generic type parameters"""
    
    generic_fixes = [
        # Add T parameter to success methods
        (r'pub fn success\(data: T\) -> Self \{', r'pub fn success<T>(data: T) -> Self {'),
        (r'pub fn into_result\(\) -> Result<T, String>', r'pub fn into_result<T>(&self) -> Result<T, String>'),
        
        # Fix From implementations for specific error types
        (r'impl From<std::net::AddrParseError> for SongbirdError \{ fn from\(error: serde_json::Error\)', 
         r'impl From<std::net::AddrParseError> for SongbirdError { fn from(error: std::net::AddrParseError)'),
        (r'impl From<regex::Error> for SongbirdError \{ fn from\(error: serde_json::Error\)', 
         r'impl From<regex::Error> for SongbirdError { fn from(error: regex::Error)'),
    ]
    
    for pattern, replacement in generic_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_closure_syntax(content: str) -> str:
    """Fix malformed closure syntax"""
    
    closure_fixes = [
        # Fix triple pipe closures
        (r'\.unwrap_or_else\(\|_\|\|\| \{', r'.unwrap_or_else(|_| {'),
    ]
    
    for pattern, replacement in closure_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix builder patterns and other semantic issues in a file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return False
        
    original_content = content
    
    # Apply fixes
    content = fix_builder_methods(content)
    content = fix_constructor_methods(content)
    content = fix_generic_bounds(content)
    content = fix_closure_syntax(content)
    
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
    """Fix builder patterns in songbird-types crate"""
    types_dir = Path('crates/songbird-types')
    
    if not types_dir.exists():
        print("Error: songbird-types directory not found")
        return
    
    rust_files = list(types_dir.rglob('*.rs'))
    fixed_count = 0
    
    print(f"🔧 Fixing builder patterns in {len(rust_files)} Rust files")
    
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"✅ Fixed: {rust_file.relative_to(types_dir)}")
    
    print(f"\n📊 Fixed {fixed_count} files")

if __name__ == '__main__':
    main() 