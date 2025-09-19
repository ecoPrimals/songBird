#!/usr/bin/env python3
"""
Restore proper function return expressions that were incorrectly modified.
"""

import os
import re

def restore_function_returns():
    """Restore function return expressions by removing semicolons from return statements."""
    
    files = [
        'crates/songbird-types/src/config/unified.rs',
        'crates/songbird-types/src/errors.rs',
        'crates/songbird-types/src/health.rs',
        'crates/songbird-types/src/memory_optimized.rs',
        'crates/songbird-types/src/primal.rs',
        'crates/songbird-types/src/response.rs',
        'crates/songbird-types/src/service.rs',
        'crates/songbird-types/src/types.rs',
    ]
    
    for file_path in files:
        if not os.path.exists(file_path):
            continue
            
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern 1: Function constructors - remove semicolons from struct returns
        patterns = [
            # Self { field: value, };  ->  Self { field: value, }
            (r'(Self \{[^}]+\});', r'\1'),
            # SomeType { field: value, };  ->  SomeType { field: value, }
            (r'([A-Z][a-zA-Z0-9_]+ \{[^}]+\});', r'\1'),
            # Ok(value);  ->  Ok(value)
            (r'(Ok\([^)]+\));', r'\1'),
            # Err(value);  ->  Err(value)  
            (r'(Err\([^)]+\));', r'\1'),
            # Some(value);  ->  Some(value)
            (r'(Some\([^)]+\));', r'\1'),
            # match expressions at end of functions
            (r'(\s+)(match [^{]+\{[^}]+\});(\s*\})', r'\1\2\3'),
            # Simple return values at end of functions
            (r'(\s+)([a-zA-Z_][a-zA-Z0-9_]*\([^)]*\));(\s*\})', r'\1\2\3'),
        ]
        
        for pattern, replacement in patterns:
            content = re.sub(pattern, replacement, content, flags=re.DOTALL)
        
        # Specific fixes for common return patterns
        # Fix write! macro returns
        content = re.sub(r'(write!\([^)]+\));', r'\1', content)
        
        # Fix format! returns  
        content = re.sub(r'(format!\([^)]+\));', r'\1', content)
        
        # Fix PathBuf::from returns
        content = re.sub(r'(PathBuf::from\([^)]+\));', r'\1', content)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Restored returns in {file_path}")

def fix_specific_issues():
    """Fix specific known issues."""
    
    # Fix unified.rs closure returns
    with open('crates/songbird-types/src/config/unified.rs', 'r') as f:
        content = f.read()
    
    # Fix the closure return issues
    content = re.sub(r'            \};(\s*\}\);)', r'            }\1', content)
    
    with open('crates/songbird-types/src/config/unified.rs', 'w') as f:
        f.write(content)
    print("Fixed unified.rs closures")
    
    # Fix match arm returns in errors.rs
    with open('crates/songbird-types/src/errors.rs', 'r') as f:
        content = f.read()
    
    # Fix match arms that shouldn't have semicolons
    content = re.sub(r'(\*ctx = Some\([^)]+\));', r'\1', content)
    content = re.sub(r'(ctx\.insert\([^)]+\));', r'\1', content)
    
    with open('crates/songbird-types/src/errors.rs', 'w') as f:
        f.write(content)
    print("Fixed errors.rs match arms")

def add_missing_default_derive():
    """Add Default derive for CommonCapabilityFlags."""
    
    with open('crates/songbird-types/src/memory_optimized.rs', 'r') as f:
        content = f.read()
    
    # Add Default to the bitflags
    content = re.sub(
        r'bitflags! \{\n    /// Common capability flags for memory-optimized storage\n    #\[derive\(Debug, Clone, Copy, PartialEq, Eq, Hash\)\]',
        r'bitflags! {\n    /// Common capability flags for memory-optimized storage\n    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]',
        content
    )
    
    # Add empty() method for Default implementation
    content = re.sub(
        r'(    }\n}\n)',
        r'    }\n}\n\nimpl Default for CommonCapabilityFlags {\n    fn default() -> Self {\n        CommonCapabilityFlags::empty()\n    }\n}\n',
        content
    )
    
    with open('crates/songbird-types/src/memory_optimized.rs', 'w') as f:
        f.write(content)
    print("Added Default impl for CommonCapabilityFlags")

def main():
    """Apply all fixes."""
    print("Restoring proper function returns...")
    
    restore_function_returns()
    fix_specific_issues() 
    add_missing_default_derive()
    
    print("All return fixes applied!")

if __name__ == "__main__":
    main() 