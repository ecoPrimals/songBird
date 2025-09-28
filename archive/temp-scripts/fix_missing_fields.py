#!/usr/bin/env python3
"""
Script to add missing current_value and expected_format fields to Configuration errors
that use songbird-security-errors format.
"""

import os
import re

def add_missing_fields(content):
    """Add missing fields to Configuration errors."""
    
    # Pattern to match Configuration errors missing the fields
    pattern = r'(SongbirdError::Configuration\s*\{\s*field:\s*[^,]+,\s*message:\s*[^,]+,)(\s*suggestion:\s*[^}]+\s*\})'
    
    def replace_config(match):
        prefix = match.group(1)
        suffix = match.group(2)
        return f'''{prefix}
            current_value: None,
            expected_format: None,{suffix}'''
    
    content = re.sub(pattern, replace_config, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def process_file(filepath):
    """Process a single file to add missing fields."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = add_missing_fields(content)
        
        if content != original_content:
            with open(filepath, 'w') as f:
                f.write(content)
            print(f"Fixed: {filepath}")
            return True
        return False
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to process all Rust files."""
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk('crates'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    fixed_count = 0
    for filepath in rust_files:
        if process_file(filepath):
            fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == "__main__":
    main() 