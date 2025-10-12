#!/usr/bin/env python3
"""Fix HashMap::new()) with extra closing paren"""

import re
from pathlib import Path

def fix_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    
    # Fix: HashMap::new()); -> HashMap::new();
    content = re.sub(r'HashMap::new\(\)\)\s*;', 'HashMap::new();', content)
    
    # Fix: HashMap::new()),  (in struct/enum) -> HashMap::new(),
    content = re.sub(r'HashMap::new\(\)\)\s*,', 'HashMap::new(),', content)
    
    # Fix: Vec::new()); -> Vec::new();
    content = re.sub(r'Vec::new\(\)\)\s*;', 'Vec::new();', content)
    
    # Fix: Vec::new()),  -> Vec::new(),
    content = re.sub(r'Vec::new\(\)\)\s*,', 'Vec::new(),', content)
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

# Fix specific files mentioned in errors
files = [
    "crates/songbird-test-utils/src/canonical_test_framework.rs",
    "crates/songbird-types/src/health.rs",
]

for file_path in files:
    filepath = Path(file_path)
    if filepath.exists():
        if fix_file(filepath):
            print(f"✓ Fixed {filepath}")
    else:
        print(f"✗ Not found: {filepath}")

