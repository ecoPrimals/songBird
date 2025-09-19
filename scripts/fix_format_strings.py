#!/usr/bin/env python3
"""
Format String Reference Fixer

This script fixes format!() string reference issues in error handling calls.
"""

import re
import sys
from pathlib import Path

def fix_format_strings_in_content(content: str) -> str:
    """Fix format string reference issues"""
    
    # Patterns to fix format! usage in error calls
    patterns = [
        # Fix bare format!() calls that should be &format!()
        (r'SongbirdError::(\w+)\(([^,]+),\s*format!\(([^)]+)\)\)', 
         r'SongbirdError::\1(\2, &format!(\3))'),
        
        # Fix single argument error calls with format!
        (r'SongbirdError::(\w+)\(format!\(([^)]+)\)\)', 
         r'SongbirdError::\1(&format!(\2))'),
        
        # Fix nested format! issues
        (r'&format!\(format!\(([^)]+)\)\)', 
         r'&format!(\1)'),
        
        # Fix double references
        (r'&&format!\(([^)]+)\)', 
         r'&format!(\1)'),
    ]
    
    for old_pattern, new_pattern in patterns:
        content = re.sub(old_pattern, new_pattern, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix format strings in a single file"""
    try:
        content = file_path.read_text(encoding='utf-8')
        original_content = content
        
        # Apply format string fixes
        content = fix_format_strings_in_content(content)
        
        # Write back if changed
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            print(f"Fixed format strings in: {file_path}")
            return True
            
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        
    return False

def main():
    """Main function"""
    repo_root = Path(__file__).parent.parent
    crates_dir = repo_root / "crates"
    
    if not crates_dir.exists():
        print(f"Crates directory not found: {crates_dir}")
        sys.exit(1)
    
    fixed_files = 0
    total_files = 0
    
    print("🚀 Fixing format string references...")
    
    # Process all Rust files in crates
    for rust_file in crates_dir.rglob("*.rs"):
        if rust_file.is_file():
            total_files += 1
            if fix_file(rust_file):
                fixed_files += 1
    
    print(f"\n✅ Format string fixing complete!")
    print(f"📊 Files processed: {total_files}")
    print(f"🔧 Files fixed: {fixed_files}")

if __name__ == "__main__":
    main() 