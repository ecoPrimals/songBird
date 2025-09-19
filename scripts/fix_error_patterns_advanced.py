#!/usr/bin/env python3
"""
Advanced Error Pattern Migration Script

This script fixes all deprecated error patterns in the Songbird codebase
to use the modern unified error handling API with proper string handling.
"""

import os
import re
import sys
from pathlib import Path

def fix_error_patterns_in_content(content: str) -> str:
    """Fix error patterns in content with proper string handling"""
    
    # Fix format! strings in error calls
    patterns = [
        # internal_error with format!
        (r'SongbirdError::internal\("component",\s*format!\(([^)]+)\)\)', 
         r'SongbirdError::internal("component", &format!(\1))'),
        
        # service with format!
        (r'SongbirdError::service\(([^,]+),\s*format!\(([^)]+)\)\)', 
         r'SongbirdError::service(\1, &format!(\2))'),
        
        # config_general with format!
        (r'SongbirdError::config_general\(format!\(([^)]+)\)\)', 
         r'SongbirdError::config_general(&format!(\1))'),
        
        # network with format!
        (r'SongbirdError::network\(([^,]+),\s*format!\(([^)]+)\)\)', 
         r'SongbirdError::network(\1, &format!(\2))'),
        
        # config with format!
        (r'SongbirdError::config\(([^,]+),\s*format!\(([^)]+)\)\)', 
         r'SongbirdError::config(\1, &format!(\2))'),
    ]
    
    for old_pattern, new_pattern in patterns:
        content = re.sub(old_pattern, new_pattern, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_import_patterns_in_content(content: str) -> str:
    """Fix import patterns in content"""
    
    patterns = [
        (r'use songbird_config::config::', 'use songbird_config::'),
        (r'songbird_config::config::', 'songbird_config::'),
    ]
    
    for old_pattern, new_pattern in patterns:
        content = re.sub(old_pattern, new_pattern, content)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix error patterns in a single file"""
    try:
        content = file_path.read_text(encoding='utf-8')
        original_content = content
        
        # Apply error pattern fixes
        content = fix_error_patterns_in_content(content)
        
        # Apply import pattern fixes  
        content = fix_import_patterns_in_content(content)
        
        # Write back if changed
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            print(f"Fixed: {file_path}")
            return True
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        
    return False

def main():
    """Main migration function"""
    repo_root = Path(__file__).parent.parent
    crates_dir = repo_root / "crates"
    
    if not crates_dir.exists():
        print(f"Crates directory not found: {crates_dir}")
        sys.exit(1)
    
    fixed_files = 0
    total_files = 0
    
    # Process all Rust files in crates
    for rust_file in crates_dir.rglob("*.rs"):
        if rust_file.is_file():
            total_files += 1
            if fix_file(rust_file):
                fixed_files += 1
    
    print(f"\nAdvanced migration complete!")
    print(f"Processed: {total_files} files")
    print(f"Fixed: {fixed_files} files")

if __name__ == "__main__":
    main() 