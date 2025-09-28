#!/usr/bin/env python3
"""
Fix Error Field Structure Script

This script fixes the field structure issues in SongbirdError::Configuration usage
that were caused by the automated import replacement.
"""

import os
import re
import sys
from pathlib import Path

def fix_error_fields(file_path):
    """Fix error field structures in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix field types from Option<String> to String in Configuration errors
        content = re.sub(r'field: Some\("([^"]+)"\)', r'field: "\1"', content)
        
        # Fix malformed format strings
        content = re.sub(r'format!\("([^"]*)\{([^,}]+),\s*$', r'format!("\1{\2}"', content, flags=re.MULTILINE)
        
        # Remove duplicate field definitions
        content = re.sub(r'(current_value: [^,]+,\s*expected_format: [^,]+,\s*suggestion: [^,}]+),\s*,\s*current_value: [^,]+,\s*expected_format: [^,]+,\s*suggestion: [^,}]+', r'\1', content, flags=re.DOTALL)
        
        # Fix trailing commas and malformed structures
        content = re.sub(r',\s*,\s*current_value:', r',\n            current_value:', content)
        content = re.sub(r'}\"\),\s*current_value:', r'}"),\n            current_value:', content)
        
        # Fix specific malformed patterns
        content = re.sub(r'suggestion: [^,}]+,\s*,\s*current_value: [^,]+,\s*expected_format: [^,]+,\s*suggestion: [^,}]+', 
                        lambda m: m.group(0).split(',')[0], content)
        
        # Remove orphaned field definitions
        content = re.sub(r',\s*\n\s*current_value: None,\s*expected_format: None,\s*$', '', content, flags=re.MULTILINE)
        
        # Write back if changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error fixing {file_path}: {e}")
        return False

def main():
    """Main function to fix all files."""
    crates_dir = Path("crates")
    
    if not crates_dir.exists():
        print("Error: crates/ directory not found. Run from project root.")
        sys.exit(1)
    
    # Find all Rust files in the config crate (most likely to have issues)
    config_files = list(Path("crates/songbird-config").rglob("*.rs"))
    
    print(f"Found {len(config_files)} config files to fix")
    
    updated_count = 0
    for file_path in config_files:
        if fix_error_fields(file_path):
            updated_count += 1
    
    print(f"\nFixed {updated_count} files")
    print("Error field structure fixes complete!")

if __name__ == "__main__":
    main() 