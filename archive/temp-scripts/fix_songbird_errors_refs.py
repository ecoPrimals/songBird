#!/usr/bin/env python3
"""
Script to replace songbird_errors:: references with direct usage.
"""

import os
import re

def fix_songbird_errors_refs(content):
    """Fix songbird_errors:: references."""
    
    # Replace songbird_errors::SongbirdError with SongbirdError
    content = re.sub(r'songbird_errors::SongbirdError', 'SongbirdError', content)
    
    # Remove other songbird_errors:: references that don't make sense
    content = re.sub(r'songbird_errors::evolved_success\(([^)]+)\)', r'\1', content)
    
    return content

def process_file(filepath):
    """Process a single file to fix references."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_songbird_errors_refs(content)
        
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