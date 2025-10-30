#!/usr/bin/env python3
"""
Fix malformed error constructor calls in songbird-network crate.
This script fixes patterns like:
- .to_string(, -> .to_string(),
- SongbirdError::network_error(...) -> SongbirdError::Network { ... }
- SongbirdError::config_error(...) -> SongbirdError::Config { ... }
"""

import os
import re
import sys

def fix_to_string_calls(content):
    """Fix malformed to_string(, calls"""
    # Fix .to_string(, patterns
    content = re.sub(r'\.to_string\(\s*,', '.to_string()', content)
    return content

def fix_network_error_calls(content):
    """Convert old network_error calls to new Network variant"""
    def replace_network_error(match):
        message = match.group(1)
        return f'''SongbirdError::Network {{
                message: {message},
                endpoint: None,
                operation: None,
                suggestion: None,
                interface: None,
            }}'''
    
    # Pattern for SongbirdError::network_error("message", ...)
    pattern = r'SongbirdError::network_error\(([^,]+),\s*[^)]*\)'
    content = re.sub(pattern, replace_network_error, content)
    
    # Pattern for songbird_types::SongbirdError::network_error
    pattern = r'songbird_types::SongbirdError::network_error\(([^,]+),\s*[^)]*\)'
    content = re.sub(pattern, replace_network_error, content)
    
    return content

def fix_config_error_calls(content):
    """Convert old config_error calls to new Config variant"""
    def replace_config_error(match):
        message = match.group(1)
        field = match.group(2) if len(match.groups()) > 1 else 'None'
        return f'''SongbirdError::Config {{
                message: {message},
                field: {field},
                context: None,
                suggestion: None,
                category: None,
            }}'''
    
    # Pattern for SongbirdError::config_error("message", Some("field"))
    pattern = r'SongbirdError::config_error\(([^,]+),\s*([^)]+)\)'
    content = re.sub(pattern, replace_config_error, content)
    
    # Pattern for SongbirdError::config_error("message")
    pattern = r'SongbirdError::config_error\(([^)]+)\)'
    content = re.sub(pattern, lambda m: replace_config_error(type('Match', (), {'group': lambda i: m.group(1) if i == 1 else 'None', 'groups': lambda: [m.group(1)]})()), content)
    
    return content

def fix_file(file_path):
    """Fix a single Rust file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply fixes
        content = fix_to_string_calls(content)
        content = fix_network_error_calls(content)
        content = fix_config_error_calls(content)
        
        # Only write if content changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        else:
            return False
    except Exception as e:
        print(f"Error fixing {file_path}: {e}")
        return False

def main():
    """Fix all Rust files in songbird-network crate"""
    network_crate_path = "crates/songbird-network/src"
    
    if not os.path.exists(network_crate_path):
        print(f"Error: {network_crate_path} does not exist")
        sys.exit(1)
    
    fixed_count = 0
    total_count = 0
    
    # Walk through all .rs files
    for root, dirs, files in os.walk(network_crate_path):
        for file in files:
            if file.endswith('.rs'):
                file_path = os.path.join(root, file)
                total_count += 1
                if fix_file(file_path):
                    fixed_count += 1
    
    print(f"\nSummary: Fixed {fixed_count} out of {total_count} files")

if __name__ == "__main__":
    main() 