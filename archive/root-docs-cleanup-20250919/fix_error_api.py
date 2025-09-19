#!/usr/bin/env python3
"""
Fix outdated SongbirdError API usage throughout the Songbird codebase.
This script updates all the deprecated error constructors to use the modern API.
"""

import os
import re
import glob

def fix_error_patterns():
    """Fix all outdated error patterns in Rust files."""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk("crates"):
        # Skip target directories
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
            
            # Fix patterns systematically
            
            # 1. Fix config_field -> configuration
            content = re.sub(
                r'SongbirdError::config_field\s*\(\s*([^)]+)\s*\)',
                r'SongbirdError::configuration(\1)',
                content
            )
            
            # 2. Fix network_error -> network
            content = re.sub(
                r'SongbirdError::network_error\s*\(\s*([^)]+)\s*\)',
                r'SongbirdError::network(\1)',
                content
            )
            
            # 3. Fix Config variant usage (old struct-like usage)
            content = re.sub(
                r'SongbirdError::Config\s*\{[^}]*message:\s*([^,}]+)[^}]*\}',
                r'SongbirdError::configuration(\1)',
                content
            )
            
            # 4. Fix Protocol variant -> network (since Protocol doesn't exist)
            content = re.sub(
                r'SongbirdError::Protocol\s*\([^)]*ProtocolError\s*\{[^}]*message:\s*([^,}]+)[^}]*\}[^)]*\)',
                r'SongbirdError::network(\1)',
                content
            )
            
            # 5. Fix Network variant struct usage
            content = re.sub(
                r'SongbirdError::Network\s*\([^)]*NetworkError\s*\{[^}]*message:\s*([^,}]+)[^}]*\}[^)]*\)',
                r'SongbirdError::network(\1)',
                content
            )
            
            # 6. Fix NetworkDetection variant -> network
            content = re.sub(
                r'SongbirdError::NetworkDetection\s*\{[^}]*message:\s*([^,}]+)[^}]*\}',
                r'SongbirdError::network(\1)',
                content
            )
            
            # Write back if changes were made
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                fixes_made += 1
                print(f"Fixed: {file_path}")
        
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
    
    print(f"Total files fixed: {fixes_made}")

if __name__ == "__main__":
    fix_error_patterns() 