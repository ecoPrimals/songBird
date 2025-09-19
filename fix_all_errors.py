#!/usr/bin/env python3
"""
Comprehensive script to fix all SongbirdError API usage patterns
"""

import os
import re

def fix_all_errors_in_file(filepath):
    """Fix all error patterns in a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix security_error -> security
        content = re.sub(r'SongbirdError::security_error\(', 'SongbirdError::security(', content)
        content = re.sub(r'songbird_errors::SongbirdError::security_error\(', 'songbird_errors::SongbirdError::security(', content)
        
        # Fix remaining Auth patterns that weren't caught before
        auth_pattern = r'SongbirdError::Auth\(Box::new\(AuthError\s*\{\s*message:\s*([^,}]+)[^}]*\}\)\)'
        def replace_auth_error(match):
            message = match.group(1).strip()
            if message.startswith('"') and message.endswith('"'):
                return f'SongbirdError::security({message})'
            else:
                return f'SongbirdError::security({message})'
        
        content = re.sub(auth_pattern, replace_auth_error, content, flags=re.DOTALL)
        
        # Also handle with songbird_errors prefix
        auth_pattern2 = r'songbird_errors::SongbirdError::Auth\(Box::new\(AuthError\s*\{\s*message:\s*([^,}]+)[^}]*\}\)\)'
        content = re.sub(auth_pattern2, lambda m: f'songbird_errors::SongbirdError::security({m.group(1).strip()})', content, flags=re.DOTALL)
        
        # Fix Security variant field usage - more comprehensive pattern
        security_field_pattern = r'SongbirdError::Security\s*\{\s*message:\s*([^,}]+)[^}]*\}'
        def replace_security_fields(match):
            message = match.group(1).strip()
            return f'SongbirdError::security({message})'
        
        content = re.sub(security_field_pattern, replace_security_fields, content, flags=re.DOTALL)
        
        # Write back if changed
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed error patterns in: {filepath}")
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to fix all error patterns in Rust files"""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk('crates'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files to check")
    
    fixed_count = 0
    for filepath in rust_files:
        if fix_all_errors_in_file(filepath):
            fixed_count += 1
    
    print(f"Fixed error patterns in {fixed_count} files")

if __name__ == "__main__":
    main() 