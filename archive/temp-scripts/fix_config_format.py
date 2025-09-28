#!/usr/bin/env python3
"""
Script to convert Configuration errors from songbird-security-errors format 
to songbird-types format (which uses Option<String> for field).
"""

import os
import re

def fix_config_format(content):
    """Convert Configuration errors to songbird-types format."""
    
    # Pattern to match Configuration errors with current_value and expected_format
    pattern = r'(SongbirdError::Configuration\s*\{\s*field:\s*)([^,]+)(,\s*message:\s*[^,]+,)\s*current_value:\s*[^,]+,\s*expected_format:\s*[^,]+,(.*?\})'
    
    def replace_config(match):
        prefix = match.group(1)
        field = match.group(2)
        message_part = match.group(3)
        suffix = match.group(4)
        
        # Wrap field in Some() if it's not already
        if not field.strip().startswith('Some('):
            field = f'Some({field})'
        
        return f'{prefix}{field}{message_part}{suffix}'
    
    content = re.sub(pattern, replace_config, content, flags=re.MULTILINE | re.DOTALL)
    
    # Fix remaining field values that need Some() wrapper
    content = re.sub(r'field:\s*"([^"]+)"\.to_string\(\)', r'field: Some("\1".to_string())', content)
    
    return content

def process_file(filepath):
    """Process a single file to fix Configuration format."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_config_format(content)
        
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
    """Main function to process config crate files."""
    # Find all Rust files in the songbird-config crate
    rust_files = []
    for root, dirs, files in os.walk('crates/songbird-config'):
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