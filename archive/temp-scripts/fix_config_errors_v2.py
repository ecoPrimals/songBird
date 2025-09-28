#!/usr/bin/env python3
"""
Script to fix malformed Configuration error structures.
"""

import os
import re

def fix_malformed_config_errors(content):
    """Fix malformed Configuration error structures."""
    
    # Fix the malformed .to_string(.replace("\"", "")) patterns
    content = re.sub(r'\.to_string\(\.replace\("\\\"", ""\)\)', '.to_string()', content)
    
    # Fix field: Some(...) patterns that were broken by the first script
    content = re.sub(r'field: ([^,\n]+)\.replace\("\\\"", ""\)', r'field: Some(\1)', content)
    
    return content

def process_file(filepath):
    """Process a single file to fix malformed errors."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_malformed_config_errors(content)
        
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