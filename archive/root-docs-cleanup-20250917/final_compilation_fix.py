#!/usr/bin/env python3
"""
Final comprehensive compilation fix for songbird-types.
Resolves all remaining import semicolons and function return expressions.
"""

import os
import re
import glob

def fix_imports_directly():
    """Fix import semicolons by directly editing each file."""
    files = glob.glob('crates/songbird-types/src/**/*.rs', recursive=True)
    
    for file_path in files:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix all import patterns
        content = re.sub(r'^use serde:\{([^}]*)\}$', r'use serde:{\1};', content, flags=re.MULTILINE)
        content = re.sub(r'^use chrono:\{([^}]*)\}$', r'use chrono:{\1};', content, flags=re.MULTILINE)
        content = re.sub(r'^use crate::service:\{([^}]*)\}$', r'use crate::service:{\1};', content, flags=re.MULTILINE)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed imports in {file_path}")

def fix_function_returns():
    """Fix function return expressions that end with semicolons."""
    
    # Files with function return issues
    files = [
        'crates/songbird-types/src/errors.rs',
        'crates/songbird-types/src/response.rs',
    ]
    
    for file_path in files:
        if not os.path.exists(file_path):
            continue
            
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix function returns - remove semicolons from struct constructors at end of functions
        content = re.sub(
            r'(\s+)(Self::\w+\s*\{[^}]+\});(\s*})',
            r'\1\2\3',
            content,
            flags=re.DOTALL
        )
        
        # Fix function returns - remove semicolons from Self { ... } constructors
        content = re.sub(
            r'(\s+)(Self\s*\{[^}]+\});(\s*})',
            r'\1\2\3',
            content,
            flags=re.DOTALL
        )
        
        # Fix specific error constructors
        content = re.sub(
            r'(\s+)(SongbirdError::\w+\s*\{[^}]+\});(\s*})',
            r'\1\2\3',
            content,
            flags=re.DOTALL
        )
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed function returns in {file_path}")

def fix_match_expressions():
    """Fix match expressions in test functions."""
    
    file_path = 'crates/songbird-types/src/errors.rs'
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Fix test function endings - remove semicolons after test function bodies
    content = re.sub(r'(    #\[test\][^}]+});\s*$', r'\1', content, flags=re.MULTILINE | re.DOTALL)
    
    # Fix specific test patterns
    patterns = [
        (r'(\s+assert_eq!\([^)]+\);)\s*}\s*;', r'\1\n    }'),
        (r'(\s+assert!\([^)]+\);)\s*}\s*;', r'\1\n    }'),
        (r'(\s+let [^;]+;)\s*}\s*;', r'\1\n    }'),
        (r'(\s+match [^{]+\{[^}]+\})\s*}\s*;', r'\1\n    }'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"Fixed match expressions in {file_path}")

def fix_specific_issues():
    """Fix specific remaining issues."""
    
    # Fix response.rs specific issue
    response_path = 'crates/songbird-types/src/response.rs'
    with open(response_path, 'r') as f:
        content = f.read()
    
    # Fix the specific Self::error pattern
    content = re.sub(
        r'(\s+)Self::error\(ai_error\);(\s*})',
        r'\1Self::error(ai_error)\2',
        content
    )
    
    with open(response_path, 'w') as f:
        f.write(content)
    print(f"Fixed specific issues in {response_path}")

def main():
    """Apply all fixes."""
    print("Applying final comprehensive compilation fixes...")
    
    fix_imports_directly()
    fix_function_returns()
    fix_match_expressions()
    fix_specific_issues()
    
    print("All fixes applied! Testing compilation...")

if __name__ == "__main__":
    main() 