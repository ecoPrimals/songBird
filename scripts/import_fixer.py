#!/usr/bin/env python3
"""
Import Fixer for Songbird Error System
=====================================

Fixes missing imports for SongbirdResult, SongbirdResponse, and DiscoveryResult
across the remaining crates to achieve 100% compilation success.
"""

import re
import sys
from pathlib import Path

class ImportFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        self.fixes_applied = 0

    def fix_imports_in_file(self, file_path: Path) -> int:
        """Fix missing imports in a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            fixes_made = 0
            
            # Check if we need SongbirdResult import
            if 'SongbirdResult' in content and 'use songbird_errors::SongbirdResult' not in content:
                # Add import at the top after existing songbird_errors imports
                if 'use songbird_errors::' in content:
                    # Find existing songbird_errors import and extend it
                    content = re.sub(
                        r'(use songbird_errors::\{[^}]*)\}',
                        r'\1, SongbirdResult}',
                        content
                    )
                    if '::SongbirdResult' not in original_content:
                        fixes_made += 1
                else:
                    # Add new import line
                    # Find the first use statement and add before it
                    lines = content.split('\n')
                    for i, line in enumerate(lines):
                        if line.strip().startswith('use ') and not line.strip().startswith('use std::'):
                            lines.insert(i, 'use songbird_errors::SongbirdResult;')
                            content = '\n'.join(lines)
                            fixes_made += 1
                            break
            
            # Check if we need SongbirdResponse import
            if 'SongbirdResponse::' in content and 'use songbird_errors::SongbirdResponse' not in content:
                if 'use songbird_errors::' in content and 'SongbirdResult' in content:
                    # Extend existing import
                    content = re.sub(
                        r'(use songbird_errors::\{[^}]*)\}',
                        r'\1, SongbirdResponse}',
                        content
                    )
                    if '::SongbirdResponse' not in original_content:
                        fixes_made += 1
                else:
                    # Add new import
                    lines = content.split('\n')
                    for i, line in enumerate(lines):
                        if line.strip().startswith('use ') and not line.strip().startswith('use std::'):
                            lines.insert(i, 'use songbird_errors::SongbirdResponse;')
                            content = '\n'.join(lines)
                            fixes_made += 1
                            break
            
            # Check if we need DiscoveryResult import
            if 'DiscoveryResult' in content and 'use songbird_errors::DiscoveryResult' not in content:
                if 'use songbird_errors::' in content:
                    # Extend existing import
                    content = re.sub(
                        r'(use songbird_errors::\{[^}]*)\}',
                        r'\1, DiscoveryResult}',
                        content
                    )
                    if '::DiscoveryResult' not in original_content:
                        fixes_made += 1
                else:
                    # Add new import
                    lines = content.split('\n')
                    for i, line in enumerate(lines):
                        if line.strip().startswith('use ') and not line.strip().startswith('use std::'):
                            lines.insert(i, 'use songbird_errors::DiscoveryResult;')
                            content = '\n'.join(lines)
                            fixes_made += 1
                            break
            
            # Fix missing await keywords
            content = re.sub(
                r'self\.get_(\w+)_metrics\(\)\?\.',
                r'self.get_\1_metrics().await?.',
                content
            )
            if '.await?' not in original_content and 'get_' in content and '_metrics()?' in content:
                fixes_made += 1
            
            # Clean up duplicate imports
            content = re.sub(
                r'(use songbird_errors::\{[^}]*), ([^}]*), \2([^}]*)\}',
                r'use songbird_errors::{\1, \2\3}',
                content
            )
            
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                self.fixes_applied += fixes_made
                return fixes_made
                
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
            
        return 0

    def fix_crate(self, crate_name: str) -> dict:
        """Fix all Rust files in a specific crate"""
        crate_path = self.crates_path / crate_name
        if not crate_path.exists():
            print(f"Crate {crate_name} not found at {crate_path}")
            return {"fixes": 0, "files": 0}
        
        rust_files = list(crate_path.rglob("*.rs"))
        total_fixes = 0
        files_processed = 0
        
        for rust_file in rust_files:
            fixes = self.fix_imports_in_file(rust_file)
            if fixes > 0:
                files_processed += 1
                total_fixes += fixes
                print(f"  ✅ Fixed {fixes} import issues in {rust_file.relative_to(self.root_path)}")
        
        return {"fixes": total_fixes, "files": files_processed}

    def fix_all_imports(self):
        """Fix imports in all problematic crates"""
        problem_crates = [
            "songbird-observability",
            "songbird-universal", 
            "songbird-discovery",
            "songbird-test-utils"
        ]
        
        print("🔧 Fixing import issues for perfect compilation...")
        
        total_fixes = 0
        total_files = 0
        
        for crate_name in problem_crates:
            print(f"\n📦 Processing imports in: {crate_name}")
            results = self.fix_crate(crate_name)
            total_fixes += results["fixes"]
            total_files += results["files"]
        
        print(f"\n🎉 Import fixes completed! Applied {total_fixes} fixes across {total_files} files")

def main():
    """Main entry point"""
    root_path = "."
    fixer = ImportFixer(root_path)
    
    if len(sys.argv) > 1 and sys.argv[1] == "--crate":
        crate_name = sys.argv[2] if len(sys.argv) > 2 else "songbird-observability"
        results = fixer.fix_crate(crate_name)
        print(f"Fixed {results['fixes']} import issues in {results['files']} files")
    else:
        fixer.fix_all_imports()

if __name__ == "__main__":
    main() 