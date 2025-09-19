#!/usr/bin/env python3
"""
Systematic compilation fix script for Songbird codebase.
Fixes missing semicolons, malformed statements, and other syntax issues.
"""

import os
import re
import glob
from pathlib import Path

def fix_missing_semicolons(content):
    """Fix missing semicolons after various statements."""
    
    # Fix missing semicolons after use statements
    content = re.sub(r'(use [^;]+)(\n)', r'\1;\2', content)
    
    # Fix missing semicolons after type aliases
    content = re.sub(r'(pub type \w+[^;]+)(\n)', r'\1;\2', content)
    
    # Fix missing semicolons after const declarations
    content = re.sub(r'(pub const \w+[^;]+)(\n)', r'\1;\2', content)
    
    # Fix missing semicolons after let statements
    content = re.sub(r'(let [^;]+)(\n\s+[a-zA-Z])', r'\1;\2', content)
    
    # Fix missing semicolons after expressions in functions
    content = re.sub(r'(\s+\w+\([^)]*\))(\n\s+\w)', r'\1;\2', content)
    
    # Fix missing semicolons after field assignments
    content = re.sub(r'(self\.\w+ = [^;]+)(\n\s+self)', r'\1;\2', content)
    
    return content

def fix_trait_definitions(content):
    """Fix malformed trait method definitions."""
    
    # Fix trait methods missing semicolons
    content = re.sub(
        r'(async fn \w+\([^)]*\) -> [^;{]+)(\n\s+///)',
        r'\1;\n\n\2',
        content
    )
    
    # Fix non-async trait methods missing semicolons
    content = re.sub(
        r'(fn \w+\([^)]*\) -> [^;{]+)(\n\s+///)',
        r'\1;\n\n\2',
        content
    )
    
    return content

def fix_impl_blocks(content):
    """Fix implementation blocks with missing semicolons."""
    
    # Fix const declarations in impl blocks
    content = re.sub(
        r'(pub const \w+: [^=]+ = [^;]+)(\n\s+///)',
        r'\1;\n\n\2',
        content
    )
    
    return content

def fix_struct_fields(content):
    """Fix struct field definitions."""
    
    # Ensure struct fields end with commas
    content = re.sub(
        r'(pub \w+: [^,\n]+)(\n\s*})',
        r'\1,\2',
        content
    )
    
    return content

def fix_function_returns(content):
    """Fix function return statements."""
    
    # Fix function bodies with missing semicolons before return
    content = re.sub(
        r'(\s+)([^;\n]+)(\n\s+Self\s*\{)',
        r'\1\2;\3',
        content
    )
    
    return content

def process_file(file_path):
    """Process a single Rust file to fix compilation issues."""
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_missing_semicolons(content)
        content = fix_trait_definitions(content)
        content = fix_impl_blocks(content)
        content = fix_struct_fields(content)
        content = fix_function_returns(content)
        
        # Only write if content changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function to process all Rust files."""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for pattern in ['crates/**/*.rs', 'src/**/*.rs', 'tests/**/*.rs']:
        rust_files.extend(glob.glob(pattern, recursive=True))
    
    # Filter out target directories and other build artifacts
    rust_files = [f for f in rust_files if '/target/' not in f and '/build/' not in f]
    
    print(f"Processing {len(rust_files)} Rust files...")
    
    fixed_count = 0
    for file_path in rust_files:
        if process_file(file_path):
            fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == "__main__":
    main() 