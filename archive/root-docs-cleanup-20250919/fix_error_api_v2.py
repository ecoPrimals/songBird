#!/usr/bin/env python3
"""
Fix remaining SongbirdError API issues with improved pattern matching.
"""

import os
import re

def fix_remaining_errors():
    """Fix remaining error patterns that the first script missed."""
    
    # Find all Rust files in the crates directory
    rust_files = []
    for root, dirs, files in os.walk("crates"):
        if "target" in root:
            continue
        for file in files:
            if file.endswith(".rs"):
                rust_files.append(os.path.join(root, file))
    
    fixes_made = 0
    
    for file_path in rust_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            
            # Fix multiline Config struct patterns
            content = re.sub(
                r'SongbirdError::configuration\s*\(\s*([^,\n]+)\s*,\s*endpoint:\s*[^,\n]+,\s*port:\s*[^,\n]+,\s*protocol:\s*[^}]+\s*\)\s*\)',
                r'SongbirdError::network(\1)',
                content,
                flags=re.MULTILINE | re.DOTALL
            )
            
            # Fix broken format strings
            content = re.sub(
                r'format!\s*\(\s*"([^"]*)\{([^}]*)\}([^"]*)"[^)]*\)\.to_string\(\)',
                r'format!("\1{}\3", \2)',
                content
            )
            
            # Fix NetworkDetection with message field
            content = re.sub(
                r'SongbirdError::network\s*\(\s*([^,\n]+)\s*,\s*endpoint:\s*[^,\n]+,\s*port:\s*[^,\n]+,\s*protocol:\s*[^}]+\s*\)',
                r'SongbirdError::network(\1)',
                content,
                flags=re.MULTILINE | re.DOTALL
            )
            
            # Fix any remaining struct-like error patterns
            content = re.sub(
                r'SongbirdError::(\w+)\s*\{[^}]*message:\s*([^,}]+)[^}]*\}',
                lambda m: f'SongbirdError::{m.group(1).lower()}({m.group(2)})',
                content
            )
            
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                fixes_made += 1
                print(f"Fixed: {file_path}")
        
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
    
    print(f"Additional files fixed: {fixes_made}")

if __name__ == "__main__":
    fix_remaining_errors() 