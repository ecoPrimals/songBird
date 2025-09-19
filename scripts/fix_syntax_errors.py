#!/usr/bin/env python3
"""
Comprehensive Syntax Error Repair Script

This script systematically fixes common syntax errors in the Songbird codebase:
- Mismatched delimiters (parentheses, braces, brackets)
- Missing semicolons
- Unclosed function/struct definitions
- Malformed string literals
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict

class SyntaxFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.fixed_files = []
        self.errors_found = []
        
    def fix_all_rust_files(self):
        """Fix syntax errors in all Rust files"""
        rust_files = list(self.root_path.rglob("*.rs"))
        
        print(f"🔧 Found {len(rust_files)} Rust files to check")
        
        for rust_file in rust_files:
            try:
                if self.fix_file(rust_file):
                    self.fixed_files.append(rust_file)
                    print(f"✅ Fixed: {rust_file.relative_to(self.root_path)}")
            except Exception as e:
                self.errors_found.append((rust_file, str(e)))
                print(f"❌ Error fixing {rust_file}: {e}")
        
        print(f"\n📊 Summary:")
        print(f"   Fixed files: {len(self.fixed_files)}")
        print(f"   Errors: {len(self.errors_found)}")
        
    def fix_file(self, file_path: Path) -> bool:
        """Fix syntax errors in a single file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except UnicodeDecodeError:
            print(f"⚠️  Skipping binary file: {file_path}")
            return False
            
        original_content = content
        
        # Apply all fixes
        content = self.fix_mismatched_delimiters(content)
        content = self.fix_missing_semicolons(content)
        content = self.fix_malformed_strings(content)
        content = self.fix_function_definitions(content)
        content = self.fix_struct_definitions(content)
        content = self.fix_impl_blocks(content)
        content = self.fix_match_expressions(content)
        content = self.fix_closure_syntax(content)
        content = self.fix_format_strings(content)
        
        # Write back if changes were made
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
            
        return False
    
    def fix_mismatched_delimiters(self, content: str) -> str:
        """Fix common delimiter mismatches"""
        fixes = [
            # Fix common function call patterns
            (r'(\w+)\(([^)]*)\)([^;{}]*)}', r'\1(\2)\3;}'),
            
            # Fix struct initialization
            (r'(\w+)\s*\{\s*([^}]*)\s*}([^;}])', r'\1 { \2 }\3'),
            
            # Fix function definitions
            (r'fn\s+(\w+)\s*\([^)]*\)\s*\{([^}]*)}([^;}])', r'fn \1() { \2 }\3'),
            
            # Fix impl blocks
            (r'impl\s+(\w+)\s*\{([^}]*)}([^;}])', r'impl \1 { \2 }\3'),
            
            # Fix match expressions
            (r'match\s+([^{]+)\s*\{([^}]*)}([^;}])', r'match \1 { \2 }\3'),
            
            # Fix closure syntax
            (r'\|\|\s*\{([^}]*)}([^;}])', r'|| { \1 }\2'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
            
        return content
    
    def fix_missing_semicolons(self, content: str) -> str:
        """Fix missing semicolons"""
        fixes = [
            # Add semicolons after simple statements
            (r'(\w+\([^)]*\))(\s*\n\s*let)', r'\1;\2'),
            (r'(\w+\([^)]*\))(\s*\n\s*assert)', r'\1;\2'),
            (r'(\w+\([^)]*\))(\s*\n\s*return)', r'\1;\2'),
            
            # Add semicolons after struct field assignments
            (r'(\w+:\s*[^,}]+)(\s*})', r'\1;\2'),
            
            # Add semicolons after variable declarations
            (r'(let\s+\w+\s*=\s*[^;]+)(\s*\n\s*let)', r'\1;\2'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
            
        return content
    
    def fix_malformed_strings(self, content: str) -> str:
        """Fix malformed string literals"""
        fixes = [
            # Fix URLs with spaces
            (r'"http:\s*//([^"]+)"', r'"http://\1"'),
            (r'"https:\s*//([^"]+)"', r'"https://\1"'),
            
            # Fix format strings
            (r'format!\s*\(\s*"([^"]*)",([^)]*)\)', r'format!("\1", \2)'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content)
            
        return content
    
    def fix_function_definitions(self, content: str) -> str:
        """Fix function definition syntax"""
        fixes = [
            # Fix function signatures with missing braces
            (r'fn\s+(\w+)\s*\([^)]*\)\s*([^{;]+)([^{]*)$', r'fn \1() {\n    \2\n}'),
            
            # Fix function returns
            (r'fn\s+(\w+)\([^)]*\)\s*->\s*([^{]+)\s*\{([^}]*)}([^;}])', r'fn \1() -> \2 {\n    \3\n}\4'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
            
        return content
    
    def fix_struct_definitions(self, content: str) -> str:
        """Fix struct definition syntax"""
        fixes = [
            # Fix struct field definitions
            (r'struct\s+(\w+)\s*\{\s*([^}]+)\s*}([^;}])', r'struct \1 {\n    \2,\n}\3'),
            
            # Fix struct field types
            (r'(\w+):\s*([^,}]+)([,}])', r'\1: \2\3'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
            
        return content
    
    def fix_impl_blocks(self, content: str) -> str:
        """Fix impl block syntax"""
        fixes = [
            # Fix impl block structure
            (r'impl\s+(\w+)\s*\{([^}]*)}([^;}])', r'impl \1 {\n\2\n}\3'),
            
            # Fix method definitions within impl blocks
            (r'(pub\s+)?fn\s+(\w+)\s*\([^)]*\)\s*\{([^}]*)}([^;}])', r'\1fn \2() {\n        \3\n    }\4'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
            
        return content
    
    def fix_match_expressions(self, content: str) -> str:
        """Fix match expression syntax"""
        fixes = [
            # Fix match arms
            (r'(\w+)\s*=>\s*([^,}]+)([,}])', r'\1 => \2\3'),
            
            # Fix match expression structure
            (r'match\s+([^{]+)\s*\{([^}]*)}([^;}])', r'match \1 {\n        \2\n    }\3'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
            
        return content
    
    def fix_closure_syntax(self, content: str) -> str:
        """Fix closure syntax"""
        fixes = [
            # Fix closure definitions
            (r'\|([^|]*)\|\s*\{([^}]*)}([^;}])', r'|\1| {\n        \2\n    }\3'),
            
            # Fix closure calls
            (r'(\w+)\(([^)]*)\|\s*\{([^}]*)}([^)]*)\)', r'\1(\2|| {\n        \3\n    }\4)'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
            
        return content
    
    def fix_format_strings(self, content: str) -> str:
        """Fix format string issues"""
        fixes = [
            # Fix format! macro calls
            (r'format!\s*\(\s*"([^"]*)",\s*([^)]*)\)', r'format!("\1", \2)'),
            
            # Fix println! macro calls
            (r'println!\s*\(\s*"([^"]*)",\s*([^)]*)\)', r'println!("\1", \2)'),
            
            # Fix debug! macro calls
            (r'debug!\s*\(\s*"([^"]*)",\s*([^)]*)\)', r'debug!("\1", \2)'),
        ]
        
        for pattern, replacement in fixes:
            content = re.sub(pattern, replacement, content)
            
        return content

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 fix_syntax_errors.py <root_directory>")
        sys.exit(1)
    
    root_dir = sys.argv[1]
    if not os.path.exists(root_dir):
        print(f"Error: Directory {root_dir} does not exist")
        sys.exit(1)
    
    print("🚀 Starting comprehensive syntax error repair...")
    
    fixer = SyntaxFixer(root_dir)
    fixer.fix_all_rust_files()
    
    print("✅ Syntax repair complete!")

if __name__ == "__main__":
    main() 