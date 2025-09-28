#!/usr/bin/env python3
"""
Script to fix Configuration error structure issues in the config crate.
The new unified structure expects:
- field: String (not Option<String>)
- current_value: Option<String> (new field)
- expected_format: Option<String> (new field)
"""

import os
import re

def fix_config_structures(content):
    """Fix Configuration error structures to match the new unified format."""
    
    # Pattern to match Configuration errors that need fixing
    # Look for field: Some("...") and replace with field: "..."
    content = re.sub(r'field:\s*Some\(([^)]+)\),', r'field: \1,', content)
    
    # Add missing fields to Configuration errors
    # Pattern: SongbirdError::Configuration { field: ..., message: ..., suggestion: ... }
    pattern = r'(SongbirdError::Configuration\s*\{\s*field:\s*[^,]+,\s*message:\s*[^,]+,)(.*?suggestion:\s*[^}]+)(.*?\})'
    
    def add_missing_fields(match):
        prefix = match.group(1)
        middle = match.group(2)
        suffix = match.group(3)
        
        # Add the missing fields
        return f"""{prefix}
            current_value: None,
            expected_format: None,
            {middle}{suffix}"""
    
    content = re.sub(pattern, add_missing_fields, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def process_file(filepath):
    """Process a single file to fix Configuration errors."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_config_structures(content)
        
        if content != original_content:
            with open(filepath, 'w') as f:
                f.write(content)
            print(f"✅ Fixed Configuration structures in: {filepath}")
        else:
            print(f"⏭️  No changes needed in: {filepath}")
            
    except Exception as e:
        print(f"❌ Error processing {filepath}: {e}")

def main():
    """Fix Configuration error structures in the config crate."""
    config_dir = "crates/songbird-config/src"
    
    if not os.path.exists(config_dir):
        print(f"❌ Config directory not found: {config_dir}")
        return
    
    # Find all Rust files in the config crate
    for root, dirs, files in os.walk(config_dir):
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                process_file(filepath)
    
    print("✅ Configuration error structure fixes complete!")

if __name__ == "__main__":
    main() 