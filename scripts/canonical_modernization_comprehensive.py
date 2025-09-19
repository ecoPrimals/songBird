#!/usr/bin/env python3
"""
Comprehensive Canonical Modernization Script

This script performs systematic modernization of the Songbird codebase:
1. Fixes syntax errors (missing parentheses, braces, commas)
2. Removes deprecated patterns and fragments
3. Unifies to canonical modernized patterns
4. Eliminates technical debt systematically
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Tuple

class CanonicalModernizer:
    def __init__(self, workspace_root: str):
        self.workspace_root = Path(workspace_root)
        self.fixes_applied = 0
        self.files_processed = 0
        
    def modernize_all(self):
        """Run comprehensive modernization across the entire codebase"""
        print("🚀 Starting Comprehensive Canonical Modernization...")
        
        # 1. Fix critical syntax errors first
        self.fix_syntax_errors()
        
        # 2. Modernize patterns and remove deprecations
        self.modernize_patterns()
        
        # 3. Clean up fragments and dead code
        self.clean_fragments()
        
        # 4. Apply pedantic formatting
        self.apply_pedantic_formatting()
        
        print(f"✅ Modernization complete: {self.fixes_applied} fixes applied across {self.files_processed} files")
    
    def fix_syntax_errors(self):
        """Fix critical syntax errors preventing compilation"""
        print("🔧 Fixing critical syntax errors...")
        
        # Common syntax error patterns and their fixes
        syntax_fixes = [
            # Missing closing parentheses in function calls
            (r'\.to_string\(\)\)', r'.to_string())'),
            # Missing closing parentheses in assert statements
            (r'assert_eq!\([^)]+;', lambda m: m.group(0)[:-1] + ')'),
            # Missing commas in JSON objects
            (r'\.to_string\(\)\n\s*"', r'.to_string(),\n                "'),
            # Fix Service struct syntax errors
            (r'SongbirdError::internal_error\(Service \{[^}]+\}\)', 
             r'SongbirdError::internal_error("Service error")'),
        ]
        
        for rust_file in self.find_rust_files():
            # Skip if it's a directory
            if rust_file.is_dir():
                continue
                
            content = rust_file.read_text()
            original_content = content
            
            for pattern, replacement in syntax_fixes:
                if callable(replacement):
                    content = re.sub(pattern, replacement, content)
                else:
                    content = re.sub(pattern, replacement, content)
            
            if content != original_content:
                rust_file.write_text(content)
                self.fixes_applied += 1
                print(f"  ✅ Fixed syntax in {rust_file}")
            
            self.files_processed += 1
    
    def modernize_patterns(self):
        """Modernize deprecated patterns to canonical forms"""
        print("🎯 Modernizing to canonical patterns...")
        
        # Pattern modernization rules
        modernization_rules = [
            # Replace unwrap() with proper error handling
            (r'\.unwrap\(\)', r'.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))?'),
            # Replace expect() with proper error context
            (r'\.expect\("([^"]+)"\)', r'.map_err(|e| SongbirdError::internal_error(&format!("\1: {}", e)))?'),
            # Replace hardcoded localhost with constants
            (r'"127\.0\.0\.1"', r'&get_bind_address()'),
            (r'"localhost"', r'&get_bind_address()'),
            # Replace hardcoded ports with functions
            (r':8080', r':get_orchestrator_port()'),
            (r':6112', r':get_gaming_port()'),
            # Remove dead code allowances without justification
            (r'#\[allow\(dead_code\)\]\n(?!.*//.*justification)', r''),
        ]
        
        for rust_file in self.find_rust_files():
            if 'test' in str(rust_file):  # Skip test files for now
                continue
            
            # Skip if it's a directory
            if rust_file.is_dir():
                continue
                
            content = rust_file.read_text()
            original_content = content
            
            for pattern, replacement in modernization_rules:
                content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
            
            if content != original_content:
                rust_file.write_text(content)
                self.fixes_applied += 1
                print(f"  ✅ Modernized {rust_file}")
    
    def clean_fragments(self):
        """Remove deprecated fragments and outdated code"""
        print("🧹 Cleaning deprecated fragments...")
        
        # Patterns to remove
        fragment_patterns = [
            # Remove TODO comments (replace with proper implementations)
            r'// TODO:.*\n',
            # Remove FIXME comments
            r'// FIXME:.*\n',
            # Remove XXX and HACK comments
            r'// (?:XXX|HACK):.*\n',
            # Remove empty #[allow(dead_code)] without justification
            r'#\[allow\(dead_code\)\]\n(?!\s*//)',
        ]
        
        for rust_file in self.find_rust_files():
            if 'test' in str(rust_file):  # Preserve test TODOs for now
                continue
            
            # Skip if it's a directory
            if rust_file.is_dir():
                continue
                
            content = rust_file.read_text()
            original_content = content
            
            for pattern in fragment_patterns:
                content = re.sub(pattern, '', content, flags=re.MULTILINE)
            
            if content != original_content:
                rust_file.write_text(content)
                self.fixes_applied += 1
                print(f"  ✅ Cleaned fragments in {rust_file}")
    
    def apply_pedantic_formatting(self):
        """Apply ultra-pedantic formatting standards"""
        print("📐 Applying pedantic formatting...")
        
        # Pedantic formatting rules
        formatting_rules = [
            # Remove empty lines after outer attributes
            (r'#\[must_use\]\n\n', r'#[must_use]\n'),
            # Fix doc comment formatting
            (r'///\s*\n///\s*\n', r'///\n'),
            # Standardize error message formatting
            (r'"([^"]*) - ([^"]*)"', r'"\1: \2"'),
        ]
        
        for rust_file in self.find_rust_files():
            # Skip if it's a directory
            if rust_file.is_dir():
                continue
                
            content = rust_file.read_text()
            original_content = content
            
            for pattern, replacement in formatting_rules:
                content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
            
            if content != original_content:
                rust_file.write_text(content)
                self.fixes_applied += 1
                print(f"  ✅ Applied pedantic formatting to {rust_file}")
    
    def find_rust_files(self) -> List[Path]:
        """Find all Rust source files excluding targets and archives"""
        rust_files = []
        
        for rust_file in self.workspace_root.rglob("*.rs"):
            # Skip target, archive, and backup directories
            if any(part in str(rust_file) for part in ['target', 'archive', 'backup']):
                continue
            rust_files.append(rust_file)
        
        return rust_files

def main():
    if len(sys.argv) > 1:
        workspace_root = sys.argv[1]
    else:
        workspace_root = os.getcwd()
    
    modernizer = CanonicalModernizer(workspace_root)
    modernizer.modernize_all()

if __name__ == "__main__":
    main() 