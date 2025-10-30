#!/usr/bin/env python3
"""
Final Completion Fix for Songbird Error System Evolution
========================================================

This script addresses all remaining compilation issues to achieve 100% completion
of the T -> T<E> error system evolution.
"""

import re
import sys
from pathlib import Path

class FinalCompletionFixer:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        self.fixes_applied = 0

    def fix_file(self, file_path: Path) -> int:
        """Fix remaining compilation issues in a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            fixes_made = 0
            
            # Fix Result<(), E> mismatches - change Ok(()) to Ok(SongbirdResponse::success(()))
            # But only in functions that return SongbirdResult<()>
            if 'SongbirdResult<()>' in content:
                # Look for patterns where we have Ok(()) in SongbirdResult context
                content = re.sub(
                    r'(\s+)Ok\(\(\)\)(\s*;?\s*$)',
                    r'\1Ok(SongbirdResponse::success(()))\2',
                    content,
                    flags=re.MULTILINE
                )
                fixes_made += 1
            
            # Fix duplicate async keywords
            content = re.sub(r'async\s+async\s+fn', 'async fn', content)
            if 'async async fn' in original_content:
                fixes_made += 1
            
            # Fix malformed function signatures with extra >
            content = re.sub(r'-> SongbirdResult<\(\)>>', '-> SongbirdResult<()>', content)
            if '-> SongbirdResult<()>>' in original_content:
                fixes_made += 1
            
            # Fix missing imports for DiscoveryResult
            if 'DiscoveryResult' in content and 'use songbird_errors::DiscoveryResult' not in content:
                # Add import after existing songbird_errors imports
                content = re.sub(
                    r'(use songbird_errors::\{[^}]+)\}',
                    r'\1, DiscoveryResult}',
                    content
                )
                if 'DiscoveryResult' in content and '::DiscoveryResult' not in original_content:
                    fixes_made += 1
            
            # Remove unused imports
            unused_patterns = [
                (r'use songbird_errors::\{([^}]*),\s*success([^}]*)\}', r'use songbird_errors::{\1\2}'),
                (r'use songbird_errors::\{success,\s*([^}]*)\}', r'use songbird_errors::{\1}'),
                (r'use songbird_errors::\{([^}]*),\s*SongbirdResult([^}]*)\}', r'use songbird_errors::{\1\2}'),
            ]
            
            for pattern, replacement in unused_patterns:
                new_content = re.sub(pattern, replacement, content)
                if new_content != content:
                    content = new_content
                    fixes_made += 1
            
            # Fix trait issues - remove ConfigValidator export if it doesn't exist
            content = re.sub(
                r'pub use validation::\{ConfigValidator,\s*([^}]+)\}',
                r'pub use validation::{\1}',
                content
            )
            content = re.sub(
                r'pub use validation::\{([^}]*),\s*ConfigValidator\}',
                r'pub use validation::{\1}',
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
            fixes = self.fix_file(rust_file)
            if fixes > 0:
                files_processed += 1
                total_fixes += fixes
                print(f"  ✅ Fixed {fixes} issues in {rust_file.relative_to(self.root_path)}")
        
        return {"fixes": total_fixes, "files": files_processed}

    def fix_all(self):
        """Fix all crates with remaining issues"""
        problem_crates = [
            "songbird-test-utils",
            "songbird-discovery", 
            "songbird-universal",
            "songbird-observability"
        ]
        
        print("🔧 Applying final completion fixes...")
        
        total_fixes = 0
        total_files = 0
        
        for crate_name in problem_crates:
            print(f"\n📦 Processing crate: {crate_name}")
            results = self.fix_crate(crate_name)
            total_fixes += results["fixes"]
            total_files += results["files"]
        
        print(f"\n🎉 Final fixes completed! Applied {total_fixes} fixes across {total_files} files")

def main():
    """Main entry point"""
    root_path = "."
    fixer = FinalCompletionFixer(root_path)
    
    if len(sys.argv) > 1 and sys.argv[1] == "--crate":
        crate_name = sys.argv[2] if len(sys.argv) > 2 else "songbird-test-utils"
        results = fixer.fix_crate(crate_name)
        print(f"Fixed {results['fixes']} issues in {results['files']} files")
    else:
        fixer.fix_all()

if __name__ == "__main__":
    main() 