#!/usr/bin/env python3
"""
Script to clean up syntax errors from regex replacements
"""

import os
import re

def clean_syntax_errors(filepath):
    """Clean up syntax errors in a file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix broken struct fields with trailing ;,
        content = re.sub(r';\s*,\s*$', ',', content, flags=re.MULTILINE)
        
        # Fix broken comments with ;,
        content = re.sub(r'//[^;\n]*;\s*,\s*$', '', content, flags=re.MULTILINE)
        
        # Fix standalone commas on lines
        content = re.sub(r'^\s*,\s*$', '', content, flags=re.MULTILINE)
        
        # Fix format string issues with unmatched parentheses
        content = re.sub(r'format!\("([^"]*)\{([^}]*)\)([^"]*)"', r'format!("\1{}\3", \2)', content)
        
        # Fix broken #[must_use] attributes
        content = re.sub(r'#\[must_use[^\]]*\]\s*;\s*$', '', content, flags=re.MULTILINE)
        
        # Fix broken Default implementations with ::
        content = re.sub(r'Self:\s*:new\(\)', 'Self::new()', content)
        
        # Remove extra semicolons after closing braces
        content = re.sub(r'}\s*;\s*}', '}\n}', content)
        
        # Clean up empty lines
        content = re.sub(r'\n\s*\n\s*\n', '\n\n', content)
        
        # Write back if changed
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Cleaned syntax errors in: {filepath}")
            return True
        return False
        
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return False

def main():
    """Main function to clean syntax errors in security files"""
    
    # Focus on the security files that have issues
    security_files = [
        'crates/songbird-security/src/security/universal_security/authentication.rs',
        'crates/songbird-security/src/security/universal_security/types.rs',
        'crates/songbird-security/src/security/universal_security_provider.rs',
    ]
    
    fixed_count = 0
    for filepath in security_files:
        if os.path.exists(filepath):
            if clean_syntax_errors(filepath):
                fixed_count += 1
        else:
            print(f"File not found: {filepath}")
    
    print(f"Cleaned syntax errors in {fixed_count} files")

if __name__ == "__main__":
    main() 