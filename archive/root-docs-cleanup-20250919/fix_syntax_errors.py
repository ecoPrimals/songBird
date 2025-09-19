#!/usr/bin/env python3
"""
Fix specific syntax errors in the gaming module.
"""

import re

def fix_gaming_module():
    """Fix syntax errors in gaming/mod.rs"""
    
    file_path = "crates/songbird-network/src/network/gaming/mod.rs"
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix malformed string literals
        content = re.sub(r'=true"\.to_string\(\)', r'=true".to_string()', content)
        content = re.sub(r'default"\);', r'default");', content)
        content = re.sub(r"'{}' is invalid", r'"{}" is invalid', content)
        content = re.sub(r'"Age of Empires"', r'"Age of Empires"', content)
        content = re.sub(r'sessions",', r'sessions",', content)
        
        # Fix unterminated strings
        content = re.sub(r'#\[cfg\(feature = "beardog"\)\]', r'#[cfg(feature = "beardog")]', content)
        
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed syntax errors in {file_path}")
        else:
            print(f"No changes needed in {file_path}")
    
    except Exception as e:
        print(f"Error processing {file_path}: {e}")

if __name__ == "__main__":
    fix_gaming_module() 