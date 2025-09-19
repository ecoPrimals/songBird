#!/usr/bin/env python3
"""
Hardcoded Value Migration Script

This script systematically migrates hardcoded localhost and IP addresses
to use centralized configuration constants.
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

class HardcodedValueMigrator:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.crates_dir = repo_root / "crates"
        self.fixes_applied = 0
        self.files_processed = 0
        
    def get_hardcoded_patterns(self) -> List[Tuple[str, str]]:
        """Get patterns for hardcoded value migration"""
        return [
            # Localhost patterns
            (r'"localhost"', r'&songbird_config::constants::network::DEFAULT_HOST'),
            (r"'localhost'", r'&songbird_config::constants::network::DEFAULT_HOST'),
            
            # 127.0.0.1 patterns
            (r'"127\.0\.0\.1"', r'&songbird_config::constants::network::DEFAULT_HOST'),
            (r"'127\.0\.0\.1'", r'&songbird_config::constants::network::DEFAULT_HOST'),
            
            # Common port patterns
            (r'"8080"', r'&songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT.to_string()'),
            (r'"3000"', r'&songbird_config::constants::network::DEFAULT_DEV_PORT.to_string()'),
            (r'"9090"', r'&songbird_config::constants::network::DEFAULT_METRICS_PORT.to_string()'),
            
            # URL patterns
            (r'"http://localhost:8080"', r'&format!("http://{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT)'),
            (r'"http://127\.0\.0\.1:8080"', r'&format!("http://{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT)'),
            
            # Socket address patterns
            (r'"127\.0\.0\.1:8080"', r'&format!("{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT)'),
            (r'"localhost:8080"', r'&format!("{}:{}", songbird_config::constants::network::DEFAULT_HOST, songbird_config::constants::network::DEFAULT_ORCHESTRATOR_PORT)'),
            
            # Test-specific patterns
            (r'127\.0\.0\.1', r'songbird_config::constants::network::DEFAULT_HOST'),
            (r'localhost', r'songbird_config::constants::network::DEFAULT_HOST'),
        ]
    
    def needs_config_import(self, content: str) -> bool:
        """Check if file needs songbird_config import"""
        return 'songbird_config::constants' in content and 'use songbird_config' not in content
    
    def add_config_import(self, content: str) -> str:
        """Add songbird_config import to file"""
        lines = content.split('\n')
        
        # Find the last use statement
        last_use_idx = -1
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') and not line.strip().startswith('use super'):
                last_use_idx = i
        
        if last_use_idx >= 0:
            # Insert after last use statement
            lines.insert(last_use_idx + 1, 'use songbird_config;')
        else:
            # Insert after any existing imports or at the top
            insert_idx = 0
            for i, line in enumerate(lines):
                if line.strip().startswith('//!') or line.strip().startswith('/*!'):
                    continue
                if line.strip() == '' or line.strip().startswith('#['):
                    continue
                insert_idx = i
                break
            lines.insert(insert_idx, 'use songbird_config;')
        
        return '\n'.join(lines)
    
    def migrate_file(self, file_path: Path) -> bool:
        """Migrate hardcoded values in a single file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                original_content = f.read()
            
            content = original_content
            modified = False
            
            # Apply hardcoded value patterns
            for old_pattern, new_pattern in self.get_hardcoded_patterns():
                new_content = re.sub(old_pattern, new_pattern, content)
                if new_content != content:
                    content = new_content
                    modified = True
                    self.fixes_applied += 1
            
            # Add config import if needed
            if modified and self.needs_config_import(content):
                content = self.add_config_import(content)
            
            if modified:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"✅ Migrated hardcoded values in {file_path.relative_to(self.repo_root)}")
                return True
                
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            
        return False
    
    def migrate_all(self):
        """Migrate hardcoded values in all Rust files"""
        print("🔧 Starting hardcoded value migration...")
        
        rust_files = list(self.crates_dir.glob("**/*.rs"))
        
        for file_path in rust_files:
            if self.migrate_file(file_path):
                self.files_processed += 1
        
        print(f"\n📊 Migration Summary:")
        print(f"   • Files processed: {self.files_processed}")
        print(f"   • Fixes applied: {self.fixes_applied}")
        print(f"   • Status: {'✅ Complete' if self.fixes_applied > 0 else '⚠️ No changes needed'}")

def main():
    repo_root = Path(__file__).parent.parent
    migrator = HardcodedValueMigrator(repo_root)
    migrator.migrate_all()

if __name__ == "__main__":
    main() 