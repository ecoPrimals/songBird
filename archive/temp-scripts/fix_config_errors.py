#!/usr/bin/env python3
"""
Script to fix Configuration error structures to match the new unified format.
This fixes the field from Option<String> to String and adds missing fields.
"""

import os
import re
import glob

def fix_config_error(content):
    """Fix Configuration error structures in the content."""
    
    # Pattern to match old Configuration error format
    old_pattern = r'SongbirdError::Configuration\s*\{\s*field:\s*Some\(([^)]+)\),\s*message:\s*([^,]+),\s*suggestion:\s*([^}]+)\s*\}'
    
    def replace_config_error(match):
        field_expr = match.group(1)
        message_expr = match.group(2)
        suggestion_expr = match.group(3)
        
        # Remove quotes and .to_string() from field if present
        field_clean = field_expr.replace('"', '').replace('.to_string()', '')
        if field_clean.startswith('"') and field_clean.endswith('"'):
            field_clean = field_clean[1:-1]
        
        return f'''SongbirdError::Configuration {{
                field: {field_expr}.replace("\"", ""),
                message: {message_expr},
                current_value: None,
                expected_format: None,
                suggestion: {suggestion_expr},
            }}'''
    
    # Apply the replacement
    content = re.sub(old_pattern, replace_config_error, content, flags=re.MULTILINE | re.DOTALL)
    
    # Fix remaining field: Some(...) patterns more simply
    content = re.sub(r'field:\s*Some\(([^)]+)\)', r'field: \1.replace("\"", "")', content)
    
    return content

def process_file(filepath):
    """Process a single file to fix Configuration errors."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_config_error(content)
        
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