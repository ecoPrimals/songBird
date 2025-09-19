#!/usr/bin/env python3
"""
Config Dependencies Fixer

This script adds songbird-config dependencies to all crates that reference
songbird_config constants but don't have the dependency declared.
"""

import os
import re
import sys
from pathlib import Path
import toml

class ConfigDependencyFixer:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.crates_dir = repo_root / "crates"
        self.fixes_applied = 0
        self.files_processed = 0
    
    def find_crates_needing_config(self):
        """Find all crates that use songbird_config but don't have the dependency"""
        needy_crates = set()
        
        # Find all Rust files that import songbird_config
        rust_files = list(self.crates_dir.glob("**/*.rs"))
        
        for rust_file in rust_files:
            try:
                with open(rust_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                if 'use songbird_config' in content or 'songbird_config::' in content:
                    # Find the crate this file belongs to
                    crate_path = rust_file
                    while crate_path.parent != self.crates_dir:
                        crate_path = crate_path.parent
                        if crate_path == self.crates_dir:
                            break
                    
                    if crate_path != self.crates_dir:
                        needy_crates.add(crate_path)
                        
            except Exception as e:
                print(f"❌ Error reading {rust_file}: {e}")
        
        return needy_crates
    
    def has_config_dependency(self, cargo_toml_path: Path) -> bool:
        """Check if Cargo.toml already has songbird-config dependency"""
        try:
            with open(cargo_toml_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            return 'songbird-config' in content and not content.count('# songbird-config')
            
        except Exception:
            return False
    
    def add_config_dependency(self, cargo_toml_path: Path) -> bool:
        """Add songbird-config dependency to Cargo.toml"""
        try:
            with open(cargo_toml_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Check if there's a commented out dependency
            if '# songbird-config = { path = "../songbird-config" }' in content:
                # Uncomment it
                new_content = content.replace(
                    '# songbird-config = { path = "../songbird-config" }',
                    'songbird-config = { path = "../songbird-config" }'
                )
            else:
                # Add new dependency
                lines = content.split('\n')
                
                # Find [dependencies] section
                deps_idx = -1
                for i, line in enumerate(lines):
                    if line.strip() == '[dependencies]':
                        deps_idx = i
                        break
                
                if deps_idx >= 0:
                    # Insert after [dependencies]
                    insert_idx = deps_idx + 1
                    
                    # Skip any comments
                    while insert_idx < len(lines) and lines[insert_idx].strip().startswith('#'):
                        insert_idx += 1
                    
                    lines.insert(insert_idx, 'songbird-config = { path = "../songbird-config" }')
                    new_content = '\n'.join(lines)
                else:
                    # Add [dependencies] section
                    new_content = content + '\n\n[dependencies]\nsongbird-config = { path = "../songbird-config" }\n'
            
            with open(cargo_toml_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            
            return True
            
        except Exception as e:
            print(f"❌ Error updating {cargo_toml_path}: {e}")
            return False
    
    def fix_all_dependencies(self):
        """Fix config dependencies for all crates that need them"""
        print("🔧 Finding crates that need songbird-config dependency...")
        
        needy_crates = self.find_crates_needing_config()
        print(f"📊 Found {len(needy_crates)} crates that may need config dependency")
        
        for crate_path in needy_crates:
            cargo_toml = crate_path / "Cargo.toml"
            
            if not cargo_toml.exists():
                continue
            
            if not self.has_config_dependency(cargo_toml):
                if self.add_config_dependency(cargo_toml):
                    print(f"✅ Added songbird-config dependency to {crate_path.name}")
                    self.fixes_applied += 1
                else:
                    print(f"❌ Failed to add dependency to {crate_path.name}")
            else:
                print(f"⚠️ {crate_path.name} already has songbird-config dependency")
            
            self.files_processed += 1
        
        print(f"\n📊 Dependency Fix Summary:")
        print(f"   • Crates processed: {self.files_processed}")
        print(f"   • Dependencies added: {self.fixes_applied}")
        print(f"   • Status: {'✅ Complete' if self.fixes_applied > 0 else '⚠️ No changes needed'}")

def main():
    repo_root = Path(__file__).parent.parent
    fixer = ConfigDependencyFixer(repo_root)
    fixer.fix_all_dependencies()

if __name__ == "__main__":
    main() 