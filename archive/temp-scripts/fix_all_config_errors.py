#!/usr/bin/env python3
"""
Comprehensive script to fix all Configuration error structure issues.
"""

import os
import re

def fix_all_config_errors(content):
    """Fix all Configuration error structure issues."""
    
    # 1. Fix field: Some("...") -> field: "..."
    content = re.sub(r'field:\s*Some\(([^)]+)\),', r'field: \1,', content)
    
    # 2. Add missing fields to Configuration errors that don't have them
    # This is more comprehensive - matches any Configuration error missing the fields
    
    # Pattern for Configuration errors with just field, message, suggestion
    pattern1 = r'SongbirdError::Configuration\s*\{\s*field:\s*([^,]+),\s*message:\s*([^,]+),\s*suggestion:\s*([^}]+)\s*\}'
    replacement1 = r'''SongbirdError::Configuration {
            field: \1,
            message: \2,
            current_value: None,
            expected_format: None,
            suggestion: \3
        }'''
    content = re.sub(pattern1, replacement1, content, flags=re.MULTILINE)
    
    # Pattern for Configuration errors with field and message only
    pattern2 = r'SongbirdError::Configuration\s*\{\s*field:\s*([^,]+),\s*message:\s*([^}]+)\s*\}'
    replacement2 = r'''SongbirdError::Configuration {
            field: \1,
            message: \2,
            current_value: None,
            expected_format: None,
            suggestion: None,
        }'''
    content = re.sub(pattern2, replacement2, content, flags=re.MULTILINE)
    
    return content

def process_file(filepath):
    """Process a single file to fix all Configuration errors."""
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        original_content = content
        content = fix_all_config_errors(content)
        
        if content != original_content:
            with open(filepath, 'w') as f:
                f.write(content)
            print(f"✅ Fixed Configuration errors in: {filepath}")
            
            # Show what changed
            lines_orig = original_content.split('\n')
            lines_new = content.split('\n')
            changes = 0
            for i, (old, new) in enumerate(zip(lines_orig, lines_new)):
                if old != new and 'Configuration' in old:
                    changes += 1
            print(f"   📝 {changes} Configuration errors fixed")
        else:
            print(f"⏭️  No changes needed in: {filepath}")
            
    except Exception as e:
        print(f"❌ Error processing {filepath}: {e}")

def main():
    """Fix all Configuration error structures in the config crate."""
    config_dir = "crates/songbird-config/src"
    
    if not os.path.exists(config_dir):
        print(f"❌ Config directory not found: {config_dir}")
        return
    
    print("🔧 Fixing Configuration error structures...")
    
    # Find all Rust files in the config crate
    for root, dirs, files in os.walk(config_dir):
        for file in files:
            if file.endswith('.rs'):
                filepath = os.path.join(root, file)
                process_file(filepath)
    
    print("✅ All Configuration error structure fixes complete!")

if __name__ == "__main__":
    main() 