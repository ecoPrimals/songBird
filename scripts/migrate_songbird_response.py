#!/usr/bin/env python3
"""
Systematic SongbirdResponse Migration Script

Migrates code from:
  Ok(()) → Ok(SongbirdResponse::unit())
  Ok(data) → Ok(SongbirdResponse::success(data))

For use with the new SongbirdResult<T> = Result<SongbirdResponse<T>, SongbirdError> type.
"""

import re
import sys
from pathlib import Path

def migrate_file(file_path):
    """Migrate a single file's response patterns"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    changes = []
    
    # Pattern 1: Ok(()) at end of line with potential comment
    # This is the most common pattern for unit returns
    pattern1 = re.compile(r'\bOk\(\(\)\)(.*?)$', re.MULTILINE)
    content, count1 = pattern1.subn(r'Ok(SongbirdResponse::unit())\1', content)
    if count1 > 0:
        changes.append(f"Migrated {count1} Ok(()) patterns")
    
    # Pattern 2: Ok with simple identifiers (be conservative)
    # Only match simple variable names, not complex expressions
    # Format: Ok(simple_var) where simple_var is just letters, numbers, underscore
    pattern2 = re.compile(r'\bOk\(([a-z_][a-z0-9_]*)\)(?!\()', re.MULTILINE)
    
    def check_and_replace(match):
        var_name = match.group(1)
        # Don't replace if it's already wrapped or looks like a constructor
        if 'Response' in var_name or var_name[0].isupper():
            return match.group(0)
        return f'Ok(SongbirdResponse::success({var_name}))'
    
    content = pattern2.sub(check_and_replace, content)
    
    # Pattern 3: Ok with tuple/struct construction - be more conservative
    # We'll handle these manually as they're more complex
    
    # Only write if changes were made
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True, changes
    
    return False, []

def find_network_files(base_path):
    """Find all .rs files in the network crate"""
    network_path = Path(base_path) / 'crates' / 'songbird-network' / 'src'
    return list(network_path.rglob('*.rs'))

def main():
    if len(sys.argv) > 1:
        base_path = sys.argv[1]
    else:
        # Assume we're in the project root
        base_path = Path(__file__).parent.parent
    
    print("🔧 Starting SongbirdResponse migration...")
    print(f"📁 Base path: {base_path}")
    
    files = find_network_files(base_path)
    print(f"📄 Found {len(files)} Rust files in songbird-network")
    
    migrated_count = 0
    total_changes = 0
    
    for file_path in files:
        was_modified, changes = migrate_file(file_path)
        if was_modified:
            migrated_count += 1
            rel_path = file_path.relative_to(base_path)
            print(f"✅ {rel_path}")
            for change in changes:
                print(f"   - {change}")
                total_changes += 1
    
    print(f"\n🎉 Migration complete!")
    print(f"   Files modified: {migrated_count}")
    print(f"   Total migrations: {total_changes}")
    
    if migrated_count > 0:
        print(f"\n💡 Next step: cargo build -p songbird-network")

if __name__ == '__main__':
    main() 