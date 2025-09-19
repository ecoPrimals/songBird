#!/usr/bin/env python3
"""
Add #[must_use] attributes to all methods returning Self
"""

import re
import sys

def add_must_use_to_file(file_path):
    """Add #[must_use] attributes to methods returning Self"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to match methods that return Self but don't have #[must_use]
    pattern = r'(\n    )(pub fn .+mut self.+ -> Self \{)'
    
    def replace_func(match):
        indent = match.group(1)
        method = match.group(2)
        return f"{indent}#[must_use]\n    {method}"
    
    # Replace all matches
    new_content = re.sub(pattern, replace_func, content)
    
    with open(file_path, 'w') as f:
        f.write(new_content)
    
    # Count how many were added
    added = len(re.findall(pattern, content))
    print(f"Added {added} #[must_use] attributes to {file_path}")

if __name__ == '__main__':
    file_path = 'crates/songbird-errors/src/songbird_errors/conversions.rs'
    add_must_use_to_file(file_path) 