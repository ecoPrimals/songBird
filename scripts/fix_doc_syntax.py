#!/usr/bin/env python3
"""
🔧 Documentation Syntax Fix Script

This script fixes malformed documentation syntax created by the pedantic polish.
"""

import os
import re
import sys
from pathlib import Path

class DocSyntaxFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.fixes_applied = 0
        
    def fix_all_docs(self):
        """Fix all documentation syntax issues"""
        print("🔧 Starting Documentation Syntax Fix...")
        
        # Find all Rust files
        rust_files = list(self.root_path.rglob("*.rs"))
        
        for file_path in rust_files:
            try:
                self._fix_file(file_path)
            except Exception as e:
                print(f"❌ Error processing {file_path}: {e}")
        
        print(f"✅ Documentation Syntax Fix Complete!")
        print(f"   🔧 Fixes applied: {self.fixes_applied}")
    
    def _fix_file(self, file_path: Path):
        """Fix a single file"""
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix malformed crate-level documentation
        content = self._fix_crate_docs(content)
        
        # Fix malformed use statements in documentation
        content = self._fix_use_statements(content)
        
        # Fix malformed variant documentation
        content = self._fix_variant_docs(content)
        
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"🔧 Fixed: {file_path}")
            self.fixes_applied += 1
    
    def _fix_crate_docs(self, content: str) -> str:
        """Fix malformed crate-level documentation"""
        
        # Pattern: //! # Title /// Word variant\n Word\n//!
        pattern = r'//! # ([^\n]*?) /// (\w+) variant\n ([^\n]*?)\n//!'
        replacement = r'//! # \1 \2\n//!'
        content = re.sub(pattern, replacement, content)
        
        # Pattern: //! ## Title /// Word variant\n Word\n//!
        pattern = r'//! ## ([^\n]*?) /// (\w+) variant\n ([^\n]*?)\n//!'
        replacement = r'//! ## \1 \2\n//!'
        content = re.sub(pattern, replacement, content)
        
        # Fix standalone word lines after documentation
        pattern = r'//! ([^\n]*)\n ([A-Z]\w+)\n//!'
        replacement = r'//! \1 \2\n//!'
        content = re.sub(pattern, replacement, content)
        
        return content
    
    def _fix_use_statements(self, content: str) -> str:
        """Fix malformed use statements mixed with documentation"""
        
        # Pattern: word\npub use ...
        pattern = r'^([A-Z]\w+)\n(pub use )'
        replacement = r'//! \1\n\n\2'
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        
        return content
    
    def _fix_variant_docs(self, content: str) -> str:
        """Fix malformed variant documentation in use statements"""
        
        # Pattern: /// Word variant in use statements
        pattern = r', /// (\w+) variant'
        replacement = r', // \1'
        content = re.sub(pattern, replacement, content)
        
        return content

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 fix_doc_syntax.py <root_path>")
        sys.exit(1)
    
    root_path = sys.argv[1]
    fixer = DocSyntaxFixer(root_path)
    fixer.fix_all_docs()

if __name__ == "__main__":
    main() 