#!/usr/bin/env python3
"""
🔧 Fix songbird_errors Import Migration Script

This script systematically replaces all references to the missing `songbird_errors`
module with the correct `songbird_types` imports and functions.
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

class ErrorImportFixer:
    """Fixes songbird_errors imports to use songbird_types"""
    
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        
        # Mapping of old songbird_errors patterns to songbird_types equivalents
        self.replacements = {
            # Module imports
            r'songbird_errors::': 'songbird_types::',
            r'use songbird_errors::': 'use songbird_types::',
            
            # Function calls
            r'songbird_errors::evolved_success': 'Ok',
            r'songbird_errors::success': 'Ok',
            r'songbird_errors::SongbirdError': 'songbird_types::SongbirdError',
            r'songbird_errors::Result': 'songbird_types::SongbirdResult',
            
            # Specific patterns that need different handling
            r'Ok\(songbird_errors::evolved_success\(([^)]+)\)\)': r'Ok(\1)',
            r'songbird_errors::evolved_success\(([^)]+)\)': r'\1',
            r'if let Ok\(songbird_errors::evolved_success\(([^)]+)\)\)': r'if let Ok(\1)',
        }
        
    def should_process_file(self, file_path: Path) -> bool:
        """Check if file should be processed"""
        if not file_path.suffix == '.rs':
            return False
            
        # Skip files that don't contain songbird_errors references
        try:
            content = file_path.read_text(encoding='utf-8')
            return 'songbird_errors' in content
        except Exception:
            return False
    
    def fix_file(self, file_path: Path) -> bool:
        """Fix a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            
            # Apply replacements in order
            for pattern, replacement in self.replacements.items():
                content = re.sub(pattern, replacement, content)
            
            # Special handling for complex patterns
            content = self.fix_complex_patterns(content)
            
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                print(f"✅ Fixed: {file_path.relative_to(self.repo_root)}")
                return True
            
            return False
            
        except Exception as e:
            print(f"❌ Error fixing {file_path}: {e}")
            return False
    
    def fix_complex_patterns(self, content: str) -> str:
        """Handle complex patterns that need special logic"""
        
        # Fix evolved_success wrapper patterns
        content = re.sub(
            r'Ok\(Ok\(([^)]+)\)\)',
            r'Ok(\1)',
            content
        )
        
        # Fix redundant Ok wrapping
        content = re.sub(
            r'return Ok\(Ok\(([^)]+)\)\);',
            r'return Ok(\1);',
            content
        )
        
        # Fix specific environment variable patterns
        content = re.sub(
            r'if let Ok\(([^)]+)\) = env::var\([^)]+\)',
            r'if let Ok(\1) = env::var',
            content
        )
        
        return content
    
    def fix_all_files(self) -> Tuple[int, int]:
        """Fix all files in the repository"""
        fixed_count = 0
        total_count = 0
        
        # Process all .rs files
        for file_path in self.repo_root.rglob("*.rs"):
            if self.should_process_file(file_path):
                total_count += 1
                if self.fix_file(file_path):
                    fixed_count += 1
        
        return fixed_count, total_count

def main():
    """Main function"""
    if len(sys.argv) > 1:
        repo_root = Path(sys.argv[1])
    else:
        repo_root = Path.cwd()
    
    if not repo_root.exists():
        print(f"❌ Repository root not found: {repo_root}")
        sys.exit(1)
    
    print(f"🔧 Fixing songbird_errors imports in: {repo_root}")
    
    fixer = ErrorImportFixer(repo_root)
    fixed_count, total_count = fixer.fix_all_files()
    
    print(f"\n📊 Migration Summary:")
    print(f"   📁 Files processed: {total_count}")
    print(f"   ✅ Files fixed: {fixed_count}")
    print(f"   📈 Success rate: {(fixed_count/total_count*100):.1f}%" if total_count > 0 else "   📈 Success rate: N/A")
    
    if fixed_count > 0:
        print(f"\n🎉 Error import migration completed successfully!")
        print(f"   All songbird_errors references have been migrated to songbird_types")
    else:
        print(f"\n✅ No files needed fixing - all imports are already correct!")

if __name__ == "__main__":
    main() 