#!/usr/bin/env python3
"""
Script to fix SongbirdError::Auth usage to use the new SongbirdError::security API
"""

import os
import re
import glob

def fix_auth_errors_in_file(filepath):
    """Fix Auth error patterns in a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Pattern to match: SongbirdError::Auth(Box::new(AuthError { message: "...", ... }))
        # Replace with: SongbirdError::security("...")
        
        # First, extract the message from AuthError patterns
        auth_pattern = r'SongbirdError::Auth\(Box::new\(AuthError\s*\{\s*message:\s*"([^"]+)"[^}]*\}\)\)'
        
        def replace_auth_error(match):
            message = match.group(1)
            return f'SongbirdError::security("{message}")'
        
        # Apply the replacement
        new_content = re.sub(auth_pattern, replace_auth_error, content)
        
        # Also handle cases without songbird_errors:: prefix
        auth_pattern2 = r'songbird_errors::SongbirdError::Auth\(Box::new\(AuthError\s*\{\s*message:\s*"([^"]+)"[^}]*\}\)\)'
        new_content = re.sub(auth_pattern2, lambda m: f'songbird_errors::SongbirdError::security("{m.group(1)}")', new_content)
        
        # Write back if changed
        if new_content != content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(new_content)
            print(f"Fixed Auth errors in: {filepath}")
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to fix all Auth errors in Rust files"""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk('crates'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    # Also check tests directory
    for root, dirs, files in os.walk('tests'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files to check")
    
    fixed_count = 0
    for filepath in rust_files:
        if fix_auth_errors_in_file(filepath):
            fixed_count += 1
    
    print(f"Fixed Auth errors in {fixed_count} files")

if __name__ == "__main__":
    main() 