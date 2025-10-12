#!/usr/bin/env python3
"""
Fix common syntax error patterns from previous bad search/replace operations
"""

import re
import sys
from pathlib import Path

def fix_trailing_quote_semicolon(content):
    """Fix ); at end of lines that should be ;;"""
    # Pattern: ");" at end of line when it should be ";"
    content = re.sub(r'"\);$', '";', content, flags=re.MULTILINE)
    # Pattern: "); at end of line (with extra space)
    content = re.sub(r'"\)\;$', '";', content, flags=re.MULTILINE)
    return content

def fix_missing_closing_parens(content):
    """Fix common missing closing parentheses"""
    # Pattern: .to_string( without closing paren before next token
    content = re.sub(r'\.to_string\(\s*$', '.to_string()', content, flags=re.MULTILINE)
    # Pattern: Some(...to_string( before comma
    content = re.sub(r'\.to_string\(,', '.to_string()),', content)
    return content

def fix_match_arm_parens(content):
    """Fix match arms with ) instead of comma"""
    # Pattern: {field) instead of {field,
    content = re.sub(r'\{(\w+)\)$', r'{\1,', content, flags=re.MULTILINE)
    return content

def fix_vec_closing(content):
    """Fix vec![...] with ) instead of ,"""
    # Pattern: ExperimentType::Something) instead of ExperimentType::Something,
    content = re.sub(r'(::\w+)\)$', r'\1,', content, flags=re.MULTILINE)
    return content

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        content = fix_trailing_quote_semicolon(content)
        content = fix_missing_closing_parens(content)
        content = fix_match_arm_parens(content)
        content = fix_vec_closing(content)
        
        if content != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
    except Exception as e:
        print(f"Error processing {filepath}: {e}", file=sys.stderr)
    return False

def main():
    # Target test files that commonly have these issues
    patterns = [
        "crates/**/tests/**/*.rs",
        "crates/**/benches/**/*.rs",
    ]
    
    files = set()
    for pattern in patterns:
        for path in Path(".").glob(pattern):
            if path.is_file():
                files.add(str(path))
    
    fixed_count = 0
    for filepath in sorted(files):
        if process_file(filepath):
            fixed_count += 1
            print(f"✓ Fixed: {filepath}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

