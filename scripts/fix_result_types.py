#!/usr/bin/env python3
"""
Result Type Fixer for Songbird
==============================

Fixes Result type mismatches by replacing success(()) with simple_success()
where functions expect Result<(), E> instead of Result<EvolvedResponse<()>, E>.
"""

import re
import sys
from pathlib import Path

class ResultTypeFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        self.fixes_applied = 0

    def fix_file(self, file_path: Path) -> int:
        """Fix Result type mismatches in a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            
            # Fix success(()) in Ok() contexts where () is expected
            # Pattern: Ok(success(())) -> Ok(())
            content = re.sub(
                r'Ok\(success\(\(\)\)\)',
                'Ok(())',
                content
            )
            
            # Fix success imports to include simple_success
            if 'use crate::success;' in content:
                content = content.replace(
                    'use crate::success;',
                    'use crate::{success, simple_success};'
                )
            
            # Fix config function calls
            content = re.sub(
                r'SongbirdError::config\(',
                'SongbirdError::configuration_error(',
                content
            )
            
            fixes_made = len(re.findall(r'Ok\(\(\)\)', content)) - len(re.findall(r'Ok\(\(\)\)', original_content))
            
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
            fixes = self.fix_file(rust_file)
            if fixes > 0:
                files_processed += 1
                total_fixes += fixes
                print(f"  ✅ Fixed {fixes} issues in {rust_file.relative_to(self.root_path)}")
        
        return {"fixes": total_fixes, "files": files_processed}

    def fix_all(self):
        """Fix all crates"""
        if not self.crates_path.exists():
            print(f"Crates directory not found: {self.crates_path}")
            return
        
        print("🔧 Fixing Result type mismatches across all crates...")
        
        total_fixes = 0
        total_files = 0
        
        for crate_dir in self.crates_path.iterdir():
            if crate_dir.is_dir() and crate_dir.name.startswith("songbird-"):
                print(f"\n📦 Processing crate: {crate_dir.name}")
                results = self.fix_crate(crate_dir.name)
                total_fixes += results["fixes"]
                total_files += results["files"]
        
        print(f"\n🎉 Completed! Fixed {total_fixes} issues across {total_files} files")

def main():
    """Main entry point"""
    root_path = "."
    fixer = ResultTypeFixer(root_path)
    
    if len(sys.argv) > 1 and sys.argv[1] == "--crate":
        crate_name = sys.argv[2] if len(sys.argv) > 2 else "songbird-errors"
        results = fixer.fix_crate(crate_name)
        print(f"Fixed {results['fixes']} issues in {results['files']} files")
    else:
        fixer.fix_all()

if __name__ == "__main__":
    main() 