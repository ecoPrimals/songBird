#!/usr/bin/env python3
"""
Automated fixer for SongbirdResponse API migration
Converts Result<T, E> returns to Result<SongbirdResponse<T>, E>
"""

import re
import sys
from pathlib import Path

def fix_file(filepath):
    """Fix SongbirdResponse patterns in a single file"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original = content
    fixes = 0
    
    # Pattern 1: Ok(()) → Ok(SongbirdResponse::success(()))
    pattern1 = r'\bOk\(\(\)\)(?!\s*\.)'
    if re.search(pattern1, content):
        content = re.sub(pattern1, 'Ok(SongbirdResponse::success(()))', content)
        fixes += content.count('Ok(SongbirdResponse::success(()))') - original.count('Ok(SongbirdResponse::success(()))')
    
    # Pattern 2: Ok(simple_var) → Ok(SongbirdResponse::success(simple_var))
    # Match Ok(identifier) but not Ok(SongbirdResponse::...)
    pattern2 = r'\bOk\(([a-z_][a-z0-9_]*)\)(?!\s*\.)'
    matches = re.findall(pattern2, content)
    for match in set(matches):
        if 'songbird' not in match.lower():
            old = f'Ok({match})'
            new = f'Ok(SongbirdResponse::success({match}))'
            # Only replace if it's at end of line or before comment
            content = re.sub(
                rf'\bOk\({match}\)(\s*(?://|$))',
                rf'Ok(SongbirdResponse::success({match}))\1',
                content
            )
   
    # Pattern 3: Ok(Vec::new()) → Ok(SongbirdResponse::success(Vec::new()))
    pattern3 = r'\bOk\(Vec::new\(\)\)'
    if 'Ok(Vec::new())' in content and 'Ok(SongbirdResponse::success(Vec::new()))' not in content:
        content = content.replace('Ok(Vec::new())', 'Ok(SongbirdResponse::success(Vec::new()))')
        fixes += 1
    
    # Pattern 4: Ok(None) → Ok(SongbirdResponse::success(None))
    pattern4 = r'\bOk\(None\)(?!\s*\.)'
    if 'Ok(None)' in content and 'SongbirdResponse' not in content[max(0, content.find('Ok(None)')-50):content.find('Ok(None)')+50]:
        content = content.replace('Ok(None)', 'Ok(SongbirdResponse::success(None))')
        fixes += 1
        
    # Pattern 5: Fix .into() calls to use SongbirdResponse
    # Ok(value.into()) where compiler suggests it
    
    # Pattern 6: Fix struct construction in Ok()
    # Ok(SomeStruct { ... }) → Ok(SongbirdResponse::success(SomeStruct { ... }))
    
    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    """Fix all files in songbird-network/src"""
    base_path = Path('crates/songbird-network/src')
    
    if not base_path.exists():
        print(f"❌ Path not found: {base_path}")
        return 1
    
    files = list(base_path.rglob('*.rs'))
    print(f"🔍 Found {len(files)} Rust files")
    print("=" * 60)
    
    fixed_count = 0
    for filepath in files:
        if fix_file(filepath):
            fixed_count += 1
            print(f"✓ Fixed: {filepath}")
    
    print("=" * 60)
    print(f"✅ Fixed {fixed_count} files")
    return 0

if __name__ == '__main__':
    sys.exit(main())

