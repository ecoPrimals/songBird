#!/usr/bin/env python3
"""
Fix function return expressions that incorrectly end with semicolons.
This targets the specific pattern causing the remaining 45 compilation errors.
"""

import os
import re
import glob

def fix_function_returns():
    """Fix function return expressions by removing trailing semicolons."""
    
    files = glob.glob('crates/songbird-types/src/**/*.rs', recursive=True)
    
    for file_path in files:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern 1: Fix struct constructors - Self { ... };
        content = re.sub(r'(\s+)(Self \{[^}]+\});(\s*})', r'\1\2\3', content, flags=re.DOTALL)
        
        # Pattern 2: Fix typed constructors - TypeName { ... };
        content = re.sub(r'(\s+)([A-Z][a-zA-Z0-9_]+ \{[^}]+\});(\s*})', r'\1\2\3', content, flags=re.DOTALL)
        
        # Pattern 3: Fix Result returns - Ok(...); and Err(...);
        content = re.sub(r'(\s+)(Ok\([^)]+\));(\s*})', r'\1\2\3', content)
        content = re.sub(r'(\s+)(Err\([^)]+\));(\s*})', r'\1\2\3', content)
        
        # Pattern 4: Fix Option returns - Some(...);
        content = re.sub(r'(\s+)(Some\([^)]+\));(\s*})', r'\1\2\3', content)
        
        # Pattern 5: Fix string literals and simple expressions
        content = re.sub(r'(\s+)("([^"\\]|\\.)*");(\s*})', r'\1\2\3', content)
        content = re.sub(r'(\s+)(\'([^\'\\]|\\.)*\');(\s*})', r'\1\2\3', content)
        
        # Pattern 6: Fix function calls at end of functions
        content = re.sub(r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*::[a-zA-Z_][a-zA-Z0-9_]*\([^)]*\));(\s*})', r'\1\2\3', content)
        content = re.sub(r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*\([^)]*\));(\s*})', r'\1\2\3', content)
        
        # Pattern 7: Fix match expressions at end of functions
        content = re.sub(r'(\s+)(match [^{]+\{[^}]+\});(\s*})', r'\1\2\3', content, flags=re.DOTALL)
        
        # Pattern 8: Fix if expressions at end of functions
        content = re.sub(r'(\s+)(if [^{]+\{[^}]+\} else \{[^}]+\});(\s*})', r'\1\2\3', content, flags=re.DOTALL)
        
        # Pattern 9: Fix variable references at end of functions
        content = re.sub(r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*);(\s*})', r'\1\2\3', content)
        
        # Pattern 10: Fix field access at end of functions  
        content = re.sub(r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z_][a-zA-Z0-9_]*);(\s*})', r'\1\2\3', content)
        
        # Pattern 11: Fix method chains at end of functions
        content = re.sub(r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*\([^)]*\))+);(\s*})', r'\1\2\3', content)
        
        # Pattern 12: Fix macro calls - format!, write!, etc.
        content = re.sub(r'(\s+)((?:format|write|println|print)!\([^)]+\));(\s*})', r'\1\2\3', content)
        
        # Pattern 13: Fix PathBuf::from and similar constructors
        content = re.sub(r'(\s+)(PathBuf::from\([^)]+\));(\s*})', r'\1\2\3', content)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed return expressions in {file_path}")

def fix_closure_returns():
    """Fix closure return expressions in unwrap_or_else patterns."""
    
    files = ['crates/songbird-types/src/config/unified.rs']
    
    for file_path in files:
        if not os.path.exists(file_path):
            continue
            
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix closure returns in unwrap_or_else
        content = re.sub(r'(\s+)(\w+(?:\([^)]+\))?);(\s*}\);)', r'\1\2\3', content)
        
        # Fix match expressions in closures
        content = re.sub(r'(\s+)(match [^{]+\{[^}]+\});(\s*}\);)', r'\1\2\3', content, flags=re.DOTALL)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed closure returns in {file_path}")

def fix_specific_patterns():
    """Fix remaining specific patterns that need special handling."""
    
    # Fix response.rs match arm issue
    file_path = 'crates/songbird-types/src/response.rs'
    if os.path.exists(file_path):
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Fix the match expression in into_result
        content = re.sub(
            r'(match self\.error \{[^}]+\});(\s*}\s*;)',
            r'\1\2',
            content,
            flags=re.DOTALL
        )
        
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"Fixed specific patterns in {file_path}")

def main():
    """Apply all fixes."""
    print("Fixing function return expressions...")
    
    fix_function_returns()
    fix_closure_returns()
    fix_specific_patterns()
    
    print("All return expression fixes applied!")
    print("Testing compilation...")

if __name__ == "__main__":
    main() 