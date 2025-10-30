#!/usr/bin/env python3
"""
Final Cleanup Script for Songbird Modernization
===============================================

Fixes remaining syntax errors and compilation issues after the main modernization.
"""

import re
import sys
from pathlib import Path

class FinalCleanup:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        self.fixes_applied = 0

    def fix_import_syntax(self, content: str) -> str:
        """Fix malformed import statements"""
        # Fix leading comma in imports
        content = re.sub(r'use\s+([^:]+)::\{\s*,\s*', r'use \1::{', content)
        
        # Fix duplicate imports in the same statement
        def fix_duplicate_imports(match):
            use_stmt = match.group(0)
            # Extract the imports between braces
            imports_match = re.search(r'\{([^}]+)\}', use_stmt)
            if imports_match:
                imports = imports_match.group(1)
                # Split, clean, deduplicate, and sort
                import_list = [imp.strip() for imp in imports.split(',') if imp.strip()]
                unique_imports = sorted(list(set(import_list)))
                return use_stmt.replace(imports_match.group(1), ', '.join(unique_imports))
            return use_stmt
        
        content = re.sub(r'use\s+[^;]+::\{[^}]+\};', fix_duplicate_imports, content)
        
        # Remove duplicate use statements
        lines = content.split('\n')
        seen_uses = set()
        cleaned_lines = []
        
        for line in lines:
            if line.strip().startswith('use '):
                if line.strip() not in seen_uses:
                    seen_uses.add(line.strip())
                    cleaned_lines.append(line)
            else:
                cleaned_lines.append(line)
        
        return '\n'.join(cleaned_lines)

    def fix_async_syntax(self, content: str) -> str:
        """Fix async function syntax issues"""
        # Fix functions that should be async but aren't
        content = re.sub(
            r'pub\s+fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*->\s*SongbirdResult<([^>]+)>\s*\{',
            r'pub async fn \1(\2) -> SongbirdResult<\3> {',
            content
        )
        
        # Fix trait methods that should be async
        content = re.sub(
            r'(\s+)fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*->\s*SongbirdResult<([^>]+)>\s*;',
            r'\1async fn \2(\3) -> SongbirdResult<\4>;',
            content
        )
        
        return content

    def fix_result_patterns(self, content: str) -> str:
        """Fix remaining Result type patterns"""
        # Fix std::result::Result patterns that were missed
        content = re.sub(
            r'std::result::Result<([^,>]+),\s*([^>]+)>',
            r'Result<\1, \2>',
            content
        )
        
        # Fix Result<(), SongbirdError> to SongbirdResult<()>
        content = re.sub(
            r'Result<\(\),\s*SongbirdError>',
            r'SongbirdResult<()>',
            content
        )
        
        return content

    def fix_file(self, file_path: Path) -> int:
        """Fix a single file"""
        if not file_path.suffix == '.rs':
            return 0
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            
            # Apply fixes
            content = self.fix_import_syntax(content)
            content = self.fix_async_syntax(content)
            content = self.fix_result_patterns(content)
            
            # Write back if changes were made
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"✅ Fixed syntax in {file_path.relative_to(self.root_path)}")
                return 1
                
            return 0
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            return 0

    def cleanup_all(self) -> None:
        """Clean up all files in the workspace"""
        print("🧹 Starting Final Cleanup")
        print("=" * 40)
        
        total_fixes = 0
        
        # Process all Rust files
        for rust_file in self.crates_path.rglob("*.rs"):
            fixes = self.fix_file(rust_file)
            total_fixes += fixes
        
        print("\n" + "=" * 40)
        print(f"🎉 Final Cleanup Complete!")
        print(f"🔧 Files fixed: {total_fixes}")

def main():
    """Main entry point"""
    root_path = "."
    cleanup = FinalCleanup(root_path)
    cleanup.cleanup_all()

if __name__ == "__main__":
    main() 