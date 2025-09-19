#!/usr/bin/env python3
"""
Fix all remaining error API issues in the songbird-network module.
"""

import os
import re

def fix_network_error_patterns():
    """Fix all error patterns in the network module."""
    
    # Find all Rust files in the network crate
    rust_files = []
    for root, dirs, files in os.walk("crates/songbird-network"):
        if "target" in root:
            continue
        for file in files:
            if file.endswith(".rs"):
                rust_files.append(os.path.join(root, file))
    
    print(f"Found {len(rust_files)} Rust files to process")
    
    fixes_made = 0
    
    for file_path in rust_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            
            # Fix error variant names
            content = re.sub(r'SongbirdError::Communication\(', r'SongbirdError::network(', content)
            content = re.sub(r'SongbirdError::io_error\(', r'SongbirdError::network(', content)
            content = re.sub(r'SongbirdError::execution_error\(', r'SongbirdError::network(', content)
            
            # Fix configuration error calls with two parameters
            content = re.sub(
                r'SongbirdError::configuration\("([^"]+)",\s*"([^"]+)"\)',
                r'SongbirdError::configuration(format!("{}: {}", "\1", "\2"))',
                content
            )
            
            # Fix field type mismatches
            content = re.sub(
                r'field:\s*"([^"]+)"\.to_string\(\)',
                r'field: Some("\1".to_string())',
                content
            )
            
            # Fix error pattern matching
            content = re.sub(
                r'SongbirdError::Communication\s*\{\s*\.\.\s*\}',
                r'SongbirdError::Network { .. }',
                content
            )
            
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                fixes_made += 1
                print(f"✅ Fixed {file_path}")
        
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
    
    print(f"\n🎯 Fixed {fixes_made} files")
    return fixes_made

if __name__ == "__main__":
    fixes_made = fix_network_error_patterns()
    print(f"Network error fixing complete. Fixed {fixes_made} files.")
