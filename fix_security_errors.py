#!/usr/bin/env python3
"""
Script to fix SongbirdError::Security field usage to use correct SecurityError construction
"""

import os
import re

def fix_security_errors_in_file(filepath):
    """Fix Security error field patterns in a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Pattern to match: SongbirdError::Security { message: "...", context: Some(...), ... }
        # Replace with: SongbirdError::security("...")
        
        # Complex pattern to match the entire Security error struct
        security_pattern = r'SongbirdError::Security\s*\{\s*message:\s*"([^"]+)"[^}]*\}'
        
        def replace_security_error(match):
            message = match.group(1)
            return f'SongbirdError::security("{message}")'
        
        # Apply the replacement
        new_content = re.sub(security_pattern, replace_security_error, content, flags=re.DOTALL)
        
        # Write back if changed
        if new_content != content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Fixed Security errors in: {filepath}")
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to fix all Security errors in Rust files"""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk('crates'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files to check")
    
    fixed_count = 0
    for filepath in rust_files:
        if fix_security_errors_in_file(filepath):
            fixed_count += 1
    
    print(f"Fixed Security errors in {fixed_count} files")

if __name__ == "__main__":
    main() 