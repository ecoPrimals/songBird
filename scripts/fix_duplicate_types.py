#!/usr/bin/env python3
"""
🔧 Fix Duplicate Types Script

This script fixes all duplicate type annotations created by the emergency syntax fix.
"""

import os
import re
import sys
from pathlib import Path

class DuplicateTypeFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.fixes_applied = 0
        
    def fix_all_duplicates(self):
        """Fix all duplicate type issues"""
        print("🔧 Starting Duplicate Type Fix...")
        
        # Find all Rust files
        rust_files = list(self.root_path.rglob("*.rs"))
        
        for file_path in rust_files:
            try:
                self._fix_file(file_path)
            except Exception as e:
                print(f"❌ Error processing {file_path}: {e}")
        
        print(f"✅ Duplicate Type Fix Complete!")
        print(f"   🔧 Fixes applied: {self.fixes_applied}")
    
    def _fix_file(self, file_path: Path):
        """Fix a single file"""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix all the duplicate type patterns
        content = self._fix_struct_field_duplicates(content)
        content = self._fix_enum_variant_duplicates(content)
        content = self._fix_use_statement_issues(content)
        content = self._fix_function_call_issues(content)
        content = self._fix_malformed_docs(content)
        
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"🔧 Fixed: {file_path}")
            self.fixes_applied += 1
    
    def _fix_struct_field_duplicates(self, content: str) -> str:
        """Fix duplicate type annotations in struct fields"""
        
        # Pattern: field: Type\n Type,
        pattern = r'(\w+):\s*(\w+)\n\s*\2,\s*'
        replacement = r'\1: \2,\n    '
        content = re.sub(pattern, replacement, content)
        
        # Pattern: field: Type\n Type<...>,
        pattern = r'(\w+):\s*([\w::<>]+)\n\s*\2,\s*'
        replacement = r'\1: \2,\n    '
        content = re.sub(pattern, replacement, content)
        
        return content
    
    def _fix_enum_variant_duplicates(self, content: str) -> str:
        """Fix duplicate type annotations in enum variants"""
        
        # Pattern: command: Type\n Type,
        pattern = r'(command):\s*(\w+)\n\s*\2,\s*'
        replacement = r'\1: \2,\n        '
        content = re.sub(pattern, replacement, content)
        
        return content
    
    def _fix_use_statement_issues(self, content: str) -> str:
        """Fix use statement syntax issues"""
        
        # Fix missing commas in use statements
        pattern = r'(\w+)\n\s*(\w+),\s*'
        replacement = r'\1, \2,\n    '
        content = re.sub(pattern, replacement, content)
        
        # Fix use statements with trailing documentation
        pattern = r'(\w+), /// `(\w+)` variant'
        replacement = r'\1, \2'
        content = re.sub(pattern, replacement, content)
        
        return content
    
    def _fix_function_call_issues(self, content: str) -> str:
        """Fix function call and assignment issues"""
        
        # Fix patterns like: field: Some\n Some(value),
        pattern = r'(\w+):\s*Some\n\s*Some\(([^)]+)\),\s*'
        replacement = r'\1: Some(\2),\n            '
        content = re.sub(pattern, replacement, content)
        
        # Fix patterns like: field: None\n None,
        pattern = r'(\w+):\s*None\n\s*None,\s*'
        replacement = r'\1: None,\n            '
        content = re.sub(pattern, replacement, content)
        
        return content
    
    def _fix_malformed_docs(self, content: str) -> str:
        """Fix malformed documentation"""
        
        # Fix crate-level docs with orphaned words
        pattern = r'//! ([^\n]*)\n ([A-Z]\w+)\n([^/])'
        replacement = r'//! \1 \2\n\3'
        content = re.sub(pattern, replacement, content)
        
        # Fix orphaned words before pub use
        pattern = r'^([A-Z]\w+)\n(pub use )'
        replacement = r'//! \1\n\n\2'
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        
        # Fix orphaned words before pub struct/enum
        pattern = r'^([A-Z]\w+)\n(\s+pub (struct|enum) )'
        replacement = r'// \1\n\2'
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        
        return content

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 fix_duplicate_types.py <root_path>")
        sys.exit(1)
    
    root_path = sys.argv[1]
    fixer = DuplicateTypeFixer(root_path)
    fixer.fix_all_duplicates()

if __name__ == "__main__":
    main() 