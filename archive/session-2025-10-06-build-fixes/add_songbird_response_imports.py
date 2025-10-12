#!/usr/bin/env python3
"""
Add SongbirdResponse imports to files that use it
"""

import re
from pathlib import Path

def needs_songbird_response_import(content):
    """Check if file uses SongbirdResponse but doesn't import it"""
    has_usage = 'SongbirdResponse::success' in content
    has_import = 'use songbird_errors::SongbirdResponse' in content or 'use songbird_errors::{' in content and 'SongbirdResponse' in content
    return has_usage and not has_import

def add_import(content):
    """Add SongbirdResponse import to file"""
    # Find the use statements section
    use_pattern = r'(use\s+[^;]+;)'
    uses = list(re.finditer(use_pattern, content))
    
    if not uses:
        # No use statements, add after mod declarations or at top
        mod_pattern = r'((?:pub\s+)?mod\s+[^;]+;)'
        mods = list(re.finditer(mod_pattern, content))
        if mods:
            # Add after last mod
            last_mod = mods[-1]
            insert_pos = last_mod.end()
            return content[:insert_pos] + '\nuse songbird_errors::SongbirdResponse;' + content[insert_pos:]
        else:
            # Add at start after comments
            lines = content.split('\n')
            insert_line = 0
            for i, line in enumerate(lines):
                if not line.strip().startswith('//') and not line.strip().startswith('/*') and line.strip():
                    insert_line = i
                    break
            lines.insert(insert_line, 'use songbird_errors::SongbirdResponse;')
            return '\n'.join(lines)
    else:
        # Add after last use statement
        last_use = uses[-1]
        insert_pos = last_use.end()
        return content[:insert_pos] + '\nuse songbird_errors::SongbirdResponse;' + content[insert_pos:]

def fix_file(filepath):
    """Add SongbirdResponse import if needed"""
    with open(filepath, 'r') as f:
        content = f.read()
    
    if needs_songbird_response_import(content):
        content = add_import(content)
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    """Fix all files in songbird-network/src"""
    base_path = Path('crates/songbird-network/src')
    
    files = list(base_path.rglob('*.rs'))
    print(f"🔍 Checking {len(files)} Rust files for missing imports")
    print("=" * 60)
    
    fixed_count = 0
    for filepath in files:
        if fix_file(filepath):
            fixed_count += 1
            print(f"✓ Added import: {filepath}")
    
    print("=" * 60)
    print(f"✅ Added imports to {fixed_count} files")
    return 0

if __name__ == '__main__':
    main()

