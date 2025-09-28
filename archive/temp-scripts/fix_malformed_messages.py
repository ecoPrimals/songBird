#!/usr/bin/env python3
"""
Script to fix malformed message fields in Configuration errors.
"""

import os
import re

def fix_malformed_messages(content):
    """Fix malformed message fields."""
    
    # Pattern to match malformed message fields with extra content
    patterns = [
        # Fix patterns like: message: format!("..."), extra_stuff)".to_string()),
        (r'message:\s*format!\("([^"]+)"\),\s*[^}]+\)"\s*\.to_string\(\)\),', r'message: format!("\1"),'),
        
        # Fix patterns like: message: "...".to_string(), extra_stuff)".to_string()),
        (r'message:\s*"([^"]+)"\.to_string\(\),\s*[^}]+\)"\s*\.to_string\(\)\),', r'message: "\1".to_string(),'),
        
        # Fix patterns like: suggestion: Some("... prefix".to_string()),
        (r'suggestion:\s*Some\("([^"]+)\s+(\w+)"\.to_string\(\)\),', r'suggestion: Some("\1 \2".to_string()),'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def process_file(filepath):
    """Process a single file to fix malformed messages."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_malformed_messages(content)
        
        if content != original_content:
            with open(filepath, 'w') as f:
                f.write(content)
            print(f"Fixed: {filepath}")
            return True
        return False
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to process config crate files."""
    # Find all Rust files in the songbird-config crate
    rust_files = []
    for root, dirs, files in os.walk('crates/songbird-config'):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    fixed_count = 0
    for filepath in rust_files:
        if process_file(filepath):
            fixed_count += 1
    
    print(f"Fixed {fixed_count} files")

if __name__ == "__main__":
    main() 