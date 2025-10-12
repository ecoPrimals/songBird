#!/usr/bin/env python3
"""
Revert pattern match conversions - they can't have function calls
"""

import re
from pathlib import Path

def revert_patterns(content):
    """Revert SongbirdResponse in pattern matches"""
    # Pattern: Ok(SongbirdResponse::success(var)) => in patterns
    # Should be: Ok(var) =>
    pattern1 = re.compile(r'\bOk\(SongbirdResponse::success\(([^)]+)\)\)\s*=>', re.MULTILINE)
    content = pattern1.sub(r'Ok(\1) =>', content)
    
    # Pattern: Ok(SongbirdResponse::unit()) => in patterns  
    pattern2 = re.compile(r'\bOk\(SongbirdResponse::unit\(\)\)\s*=>', re.MULTILINE)
    content = pattern2.sub(r'Ok(()) =>', content)
    
    return content

def process_file(file_path):
    """Process one file"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    new_content = revert_patterns(content)
    
    if new_content != content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

def main():
    base_path = Path(__file__).parent.parent
    network_path = base_path / 'crates' / 'songbird-network' / 'src'
    
    print("🔄 Reverting pattern match conversions...")
    
    files = list(network_path.rglob('*.rs'))
    modified = 0
    
    for file_path in files:
        if process_file(file_path):
            modified += 1
            rel_path = file_path.relative_to(base_path)
            print(f"✅ {rel_path}")
    
    print(f"\n🎉 Reverted patterns in {modified} files")

if __name__ == '__main__':
    main() 