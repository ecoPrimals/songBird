#!/usr/bin/env python3
"""
Fix Configuration Error Fields Script

This script fixes all SongbirdError::Configuration field structure issues
to match the canonical error type definition.
"""

import os
import re
import sys
from pathlib import Path

def fix_configuration_errors(file_path):
    """Fix Configuration error structures in a single file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix field from Option<String> to String
        content = re.sub(r'field: Some\("([^"]+)"\)', r'field: "\1"', content)
        
        # Fix incomplete Configuration error structures
        # Pattern: SongbirdError::Configuration { message: ..., field: ..., suggestion: ... }
        # Should be: SongbirdError::Configuration { field: ..., message: ..., current_value: None, expected_format: None, suggestion: ... }
        
        def fix_config_error(match):
            # Extract the content inside the braces
            content_inside = match.group(1)
            
            # Parse existing fields
            field_match = re.search(r'field:\s*"([^"]+)"', content_inside)
            message_match = re.search(r'message:\s*([^,}]+)', content_inside)
            suggestion_match = re.search(r'suggestion:\s*([^,}]+)', content_inside)
            
            field = field_match.group(1) if field_match else "unknown"
            message = message_match.group(1) if message_match else '"Unknown error"'.strip()
            suggestion = suggestion_match.group(1) if suggestion_match else "None"
            
            # Clean up message if it has trailing comma
            message = re.sub(r',\s*$', '', message.strip())
            
            # Build the corrected structure
            return f'''SongbirdError::Configuration {{
            field: "{field}".to_string(),
            message: {message},
            current_value: None,
            expected_format: None,
            suggestion: {suggestion},
        }}'''
        
        # Apply the fix to Configuration error patterns
        content = re.sub(r'SongbirdError::Configuration\s*\{([^}]+)\}', fix_config_error, content, flags=re.DOTALL)
        
        # Fix malformed format strings
        content = re.sub(r'format!\("([^"]*)\{([^,}]+),\s*$', r'format!("\1{\2}"', content, flags=re.MULTILINE)
        
        # Remove duplicate field definitions and trailing commas
        content = re.sub(r',\s*,', ',', content)
        content = re.sub(r'},\s*current_value:', '},', content)
        
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
    
    # Find all Rust files that might have Configuration errors
    rust_files = []
    for crate_dir in crates_dir.iterdir():
        if crate_dir.is_dir():
            for rust_file in crate_dir.rglob("*.rs"):
                try:
                    with open(rust_file, 'r', encoding='utf-8') as f:
                        content = f.read()
                        if 'SongbirdError::Configuration' in content:
                            rust_files.append(rust_file)
                except Exception:
                    continue
    
    print(f"Found {len(rust_files)} files with Configuration errors to fix")
    
    updated_count = 0
    for file_path in rust_files:
        if fix_configuration_errors(file_path):
            updated_count += 1
    
    print(f"\nFixed {updated_count} files")
    print("Configuration error fixes complete!")

if __name__ == "__main__":
    main() 