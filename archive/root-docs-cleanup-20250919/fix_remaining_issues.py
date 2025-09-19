#!/usr/bin/env python3
"""
Fix remaining compilation issues in songbird-types crate.
"""

import re
import os

def fix_songbird_types():
    """Fix specific issues in songbird-types crate."""
    
    # Fix missing semicolons in specific patterns
    files_to_fix = [
        'crates/songbird-types/src/memory_optimized.rs',
        'crates/songbird-types/src/service.rs',
        'crates/songbird-types/src/zero_copy.rs',
        'crates/songbird-types/src/traits.rs',
        'crates/songbird-types/src/errors.rs',
        'crates/songbird-types/src/response.rs'
    ]
    
    for file_path in files_to_fix:
        if not os.path.exists(file_path):
            continue
            
        with open(file_path, 'r') as f:
            content = f.read()
            
        original_content = content
        
        # Fix specific patterns
        content = re.sub(r'(self\.[a-zA-Z_]+ = [^;]+)\n(\s+self)', r'\1;\n\2', content)
        content = re.sub(r'(let mut [^=]+ = [^;]+)\n(\s+if)', r'\1;\n\2', content)
        content = re.sub(r'(type \w+ = [^;]+)\n(\s+pub)', r'\1;\n\2', content)
        content = re.sub(r'(pub type \w+[^=]+ = [^;]+)\n(\s*///)', r'\1;\n\n\2', content)
        
        # Fix trait method definitions
        content = re.sub(r'(async fn [^{;]+)\n(\s+///)', r'\1;\n\n\2', content)
        content = re.sub(r'(fn [^{;]+)\n(\s+///)', r'\1;\n\n\2', content)
        
        # Remove trailing semicolons after function braces
        content = re.sub(r' {;\n', ' {\n', content)
        
        # Fix attribute declarations
        content = re.sub(r'(#\[[^\]]+\]);', r'\1', content)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed: {file_path}")

if __name__ == "__main__":
    fix_songbird_types() 