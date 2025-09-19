#!/usr/bin/env python3
"""
🔧 Fix Error Constructor Parameters Script

This script fixes all SongbirdError constructor calls to match the correct signatures.
"""

import os
import re
import sys
from pathlib import Path

class ErrorConstructorFixer:
    """Fixes SongbirdError constructor calls"""
    
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        
        # Error constructor fixes
        self.fixes = [
            # network_error needs endpoint parameter
            (
                r'SongbirdError::network_error\(([^)]+)\)\)',
                r'SongbirdError::network_error(\1, None))'
            ),
            # service_error needs recovery_actions parameter
            (
                r'SongbirdError::service_error\(\s*([^,]+),\s*([^)]+)\)',
                r'SongbirdError::service_error(\1, \2, vec!["retry_operation".to_string()])'
            ),
            # Fix specific authentication error cases
            (
                r'\.map_err\(\|_\| Ok\(false\)\)\?',
                r'.map_err(|e| SongbirdError::service_error("security", format!("Parse error: {:?}", e), vec!["check_response_format".to_string()]))?'
            ),
        ]
        
    def fix_file(self, file_path: Path) -> bool:
        """Fix a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            
            # Apply all fixes
            for pattern, replacement in self.fixes:
                content = re.sub(pattern, replacement, content)
            
            # Special handling for specific patterns
            content = self.fix_special_cases(content)
            
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                print(f"✅ Fixed: {file_path.relative_to(self.repo_root)}")
                return True
            
            return False
            
        except Exception as e:
            print(f"❌ Error fixing {file_path}: {e}")
            return False
    
    def fix_special_cases(self, content: str) -> str:
        """Handle special error constructor cases"""
        
        # Fix network_error calls that already have one parameter
        content = re.sub(
            r'SongbirdError::network_error\(([^,)]+), None\), None\)',
            r'SongbirdError::network_error(\1, None)',
            content
        )
        
        # Fix service_error calls that already have recovery actions
        content = re.sub(
            r'SongbirdError::service_error\(([^,]+), ([^,]+), vec!\[.*?\], vec!\[.*?\]\)',
            r'SongbirdError::service_error(\1, \2, vec!["retry_operation".to_string()])',
            content
        )
        
        # Fix method signature issues
        content = re.sub(
            r'async fn send_capability_request\(&self\) -> SongbirdResult<Value>',
            r'async fn send_capability_request(&self, _capability: &str, _operation: &str, _payload: serde_json::Value) -> SongbirdResult<Value>',
            content
        )
        
        # Fix method calls to match new signature
        content = re.sub(
            r'self\.send_capability_request\(\)',
            r'self.send_capability_request("", "", serde_json::json!({}))',
            content
        )
        
        return content
    
    def fix_all_files(self) -> tuple[int, int]:
        """Fix all Rust files in the repository"""
        fixed_count = 0
        total_count = 0
        
        for file_path in self.repo_root.rglob("*.rs"):
            if self.should_fix_file(file_path):
                total_count += 1
                if self.fix_file(file_path):
                    fixed_count += 1
        
        return fixed_count, total_count
    
    def should_fix_file(self, file_path: Path) -> bool:
        """Check if file should be fixed"""
        try:
            content = file_path.read_text(encoding='utf-8')
            return 'SongbirdError::' in content
        except Exception:
            return False

def main():
    """Main function"""
    if len(sys.argv) > 1:
        repo_root = Path(sys.argv[1])
    else:
        repo_root = Path.cwd()
    
    print(f"🔧 Fixing error constructors in: {repo_root}")
    
    fixer = ErrorConstructorFixer(repo_root)
    fixed_count, total_count = fixer.fix_all_files()
    
    print(f"\n📊 Fix Summary:")
    print(f"   📁 Files processed: {total_count}")
    print(f"   ✅ Files fixed: {fixed_count}")
    print(f"   📈 Success rate: {(fixed_count/total_count*100):.1f}%" if total_count > 0 else "   📈 Success rate: N/A")
    
    if fixed_count > 0:
        print(f"\n🎉 Error constructor fixes completed!")
    else:
        print(f"\n✅ No fixes needed - all constructors are correct!")

if __name__ == "__main__":
    main() 