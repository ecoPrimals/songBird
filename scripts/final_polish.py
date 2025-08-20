#!/usr/bin/env python3
"""
Final Polish Script for Songbird Error System
=============================================

Fixes remaining syntax errors, duplicate imports, and Result type mismatches
to achieve 100% compilation success across the entire workspace.
"""

import re
import sys
from pathlib import Path

class FinalPolisher:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        self.fixes_applied = 0

    def polish_file(self, file_path: Path) -> int:
        """Polish a single file for final compilation success"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            fixes_made = 0
            
            # Fix duplicate SongbirdResponse imports
            content = re.sub(
                r'use songbird_errors::\{([^}]*), SongbirdResponse([^}]*), SongbirdResponse([^}]*)\}',
                r'use songbird_errors::{\1, SongbirdResponse\2\3}',
                content
            )
            if ', SongbirdResponse' in content and content.count('SongbirdResponse') > 1:
                fixes_made += 1
            
            # Fix malformed function signatures with extra >
            content = re.sub(r'-> SongbirdResult<([^>]+)>>', r'-> SongbirdResult<\1>', content)
            if '>>' in original_content and 'SongbirdResult' in content:
                fixes_made += 1
            
            # Fix unclosed delimiters in imports
            content = re.sub(
                r'use songbird_errors::\{use songbird_errors::\{([^}]+)\}',
                r'use songbird_errors::{\1}',
                content
            )
            if '{use songbird_errors::{' in original_content:
                fixes_made += 1
            
            # Fix struct literal syntax errors
            content = re.sub(
                r'if let Some\(context\) = ([A-Z][a-zA-Z]+) \{',
                r'if let Some(context) = Some(\1 {',
                content
            )
            if 'if let Some(context) =' in content and ' = Some(' not in original_content:
                fixes_made += 1
            
            # Fix Ok(()) to Ok(evolved_success(())) for SongbirdResult<()>
            if 'SongbirdResult<()>' in content:
                # Replace Ok(()) with Ok(evolved_success(()))
                content = re.sub(
                    r'(\s+)Ok\(\(\)\)(\s*[;}])',
                    r'\1Ok(evolved_success(()))\2',
                    content
                )
                if 'Ok(())' in original_content and 'SongbirdResult<()>' in content:
                    fixes_made += 1
                    # Add import for evolved_success if needed
                    if 'evolved_success' not in content:
                        content = re.sub(
                            r'(use songbird_errors::\{[^}]*)\}',
                            r'\1, evolved_success}',
                            content
                        )
                        fixes_made += 1
            
            # Fix unclosed delimiters in use statements
            lines = content.split('\n')
            for i, line in enumerate(lines):
                if 'use' in line and line.count('{') > line.count('}'):
                    # Find the next line that might close it
                    for j in range(i + 1, min(i + 5, len(lines))):
                        if '}' in lines[j] and '{' not in lines[j]:
                            # Merge the lines
                            lines[i] = line + ' ' + lines[j].strip()
                            lines[j] = ''
                            fixes_made += 1
                            break
            
            if fixes_made > 0:
                content = '\n'.join(lines)
            
            # Clean up empty lines from merging
            content = re.sub(r'\n\s*\n\s*\n', '\n\n', content)
            
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                self.fixes_applied += fixes_made
                return fixes_made
                
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
            
        return 0

    def polish_crate(self, crate_name: str) -> dict:
        """Polish all Rust files in a specific crate"""
        crate_path = self.crates_path / crate_name
        if not crate_path.exists():
            print(f"Crate {crate_name} not found at {crate_path}")
            return {"fixes": 0, "files": 0}
        
        rust_files = list(crate_path.rglob("*.rs"))
        total_fixes = 0
        files_processed = 0
        
        for rust_file in rust_files:
            fixes = self.polish_file(rust_file)
            if fixes > 0:
                files_processed += 1
                total_fixes += fixes
                print(f"  ✨ Polished {fixes} issues in {rust_file.relative_to(self.root_path)}")
        
        return {"fixes": total_fixes, "files": files_processed}

    def polish_all(self):
        """Polish all crates with remaining issues"""
        problem_crates = [
            "songbird-discovery",
            "songbird-test-utils", 
            "songbird-core"
        ]
        
        print("✨ Applying final polish for perfect compilation...")
        
        total_fixes = 0
        total_files = 0
        
        for crate_name in problem_crates:
            print(f"\n📦 Polishing crate: {crate_name}")
            results = self.polish_crate(crate_name)
            total_fixes += results["fixes"]
            total_files += results["files"]
        
        print(f"\n🎉 Final polish completed! Applied {total_fixes} fixes across {total_files} files")

def main():
    """Main entry point"""
    root_path = "."
    polisher = FinalPolisher(root_path)
    
    if len(sys.argv) > 1 and sys.argv[1] == "--crate":
        crate_name = sys.argv[2] if len(sys.argv) > 2 else "songbird-discovery"
        results = polisher.polish_crate(crate_name)
        print(f"Polished {results['fixes']} issues in {results['files']} files")
    else:
        polisher.polish_all()

if __name__ == "__main__":
    main() 