#!/usr/bin/env python3
"""
Comprehensive Unwrap Elimination Script

This script systematically replaces unwrap() calls with proper error handling
using Songbird's unified error system.
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

class UnwrapEliminator:
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.crates_dir = repo_root / "crates"
        self.fixes_applied = 0
        self.files_processed = 0
        
    def get_unwrap_patterns(self) -> List[Tuple[str, str]]:
        """Get patterns for unwrap elimination"""
        return [
            # Basic unwrap patterns
            (r'\.unwrap\(\)', r'.map_err(|e| SongbirdError::internal("component", &format!("Operation failed: {}", e)))?'),
            
            # Environment variable unwraps
            (r'std::env::var\("([^"]+)"\)\.unwrap\(\)', 
             r'std::env::var("\1").map_err(|e| SongbirdError::config("\1", &format!("Environment variable not found: {}", e)))?'),
            
            # Parse unwraps
            (r'\.parse\(\)\.unwrap\(\)', 
             r'.parse().map_err(|e| SongbirdError::config("parse", &format!("Parse error: {}", e)))?'),
            
            # Network address unwraps
            (r'\.parse::<SocketAddr>\(\)\.unwrap\(\)', 
             r'.parse::<SocketAddr>().map_err(|e| SongbirdError::network("address_parse", &format!("Invalid socket address: {}", e)))?'),
            
            # File operations unwraps
            (r'\.read_to_string\([^)]*\)\.unwrap\(\)', 
             r'.read_to_string().map_err(|e| SongbirdError::internal("file_io", &format!("File read error: {}", e)))?'),
            
            # JSON unwraps
            (r'serde_json::from_str\([^)]*\)\.unwrap\(\)', 
             r'serde_json::from_str().map_err(|e| SongbirdError::internal("json_parse", &format!("JSON parse error: {}", e)))?'),
            
            # Lock unwraps
            (r'\.lock\(\)\.unwrap\(\)', 
             r'.lock().map_err(|e| SongbirdError::internal("lock", &format!("Lock acquisition failed: {}", e)))?'),
            
            # Channel unwraps
            (r'\.send\([^)]*\)\.unwrap\(\)', 
             r'.send().map_err(|e| SongbirdError::internal("channel", &format!("Channel send failed: {}", e)))?'),
        ]
    
    def get_expect_patterns(self) -> List[Tuple[str, str]]:
        """Get patterns for expect elimination"""
        return [
            # Basic expect patterns
            (r'\.expect\("([^"]+)"\)', 
             r'.map_err(|e| SongbirdError::internal("operation", &format!("\1: {}", e)))?'),
            
            # Test-specific expects (keep in test files)
            (r'\.expect\("Test ([^"]+)"\)', 
             r'.expect("Test \1")'),  # Keep test expects as-is
        ]
    
    def should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped"""
        skip_patterns = [
            "/tests/",
            "/benches/",
            "/examples/",
            "test_",
            "_test.rs",
            "_tests.rs",
        ]
        
        file_str = str(file_path)
        return any(pattern in file_str for pattern in skip_patterns)
    
    def fix_unwraps_in_content(self, content: str, is_test_file: bool) -> str:
        """Fix unwrap patterns in content"""
        original_content = content
        
        # Skip aggressive unwrap elimination in test files
        if is_test_file:
            # Only fix obvious production code patterns in tests
            patterns = [
                (r'std::env::var\("([^"]+)"\)\.unwrap\(\)', 
                 r'std::env::var("\1").unwrap_or_else(|_| "default".to_string())'),
            ]
        else:
            patterns = self.get_unwrap_patterns() + self.get_expect_patterns()
        
        for old_pattern, new_pattern in patterns:
            content = re.sub(old_pattern, new_pattern, content, flags=re.MULTILINE)
        
        return content
    
    def add_error_imports(self, content: str) -> str:
        """Add necessary error imports if not present"""
        if "use songbird_errors::" not in content and "SongbirdError::" in content:
            # Find the last use statement
            use_pattern = r'(use [^;]+;)'
            matches = list(re.finditer(use_pattern, content))
            if matches:
                last_use = matches[-1]
                insert_pos = last_use.end()
                import_line = "\nuse songbird_errors::SongbirdError;"
                content = content[:insert_pos] + import_line + content[insert_pos:]
        
        return content
    
    def fix_file(self, file_path: Path) -> bool:
        """Fix unwraps in a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            
            is_test_file = self.should_skip_file(file_path)
            
            # Apply unwrap fixes
            content = self.fix_unwraps_in_content(content, is_test_file)
            
            # Add imports if needed
            if not is_test_file:
                content = self.add_error_imports(content)
            
            # Write back if changed
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                print(f"Fixed unwraps in: {file_path}")
                self.fixes_applied += 1
                return True
                
        except Exception as e:
            print(f"Error processing {file_path}: {e}")
            
        return False
    
    def run(self) -> None:
        """Run the unwrap elimination process"""
        if not self.crates_dir.exists():
            print(f"Crates directory not found: {self.crates_dir}")
            sys.exit(1)
        
        print("🚀 Starting comprehensive unwrap elimination...")
        
        # Process all Rust files in crates
        for rust_file in self.crates_dir.rglob("*.rs"):
            if rust_file.is_file():
                self.files_processed += 1
                self.fix_file(rust_file)
        
        print(f"\n✅ Unwrap elimination complete!")
        print(f"📊 Files processed: {self.files_processed}")
        print(f"🔧 Files fixed: {self.fixes_applied}")
        
        if self.fixes_applied > 0:
            print(f"\n🧪 Run 'cargo check' to verify the fixes.")

def main():
    """Main function"""
    repo_root = Path(__file__).parent.parent
    eliminator = UnwrapEliminator(repo_root)
    eliminator.run()

if __name__ == "__main__":
    main() 