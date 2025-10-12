#!/usr/bin/env python3
"""
Add SongbirdResponse import to files that use it
"""

import re
from pathlib import Path

def needs_import(content):
    """Check if file uses SongbirdResponse but doesn't import it"""
    uses_response = 'SongbirdResponse::' in content
    has_import = re.search(r'use\s+(?:songbird_errors::)?SongbirdResponse', content)
    return uses_response and not has_import

def add_import(content):
    """Add SongbirdResponse import to the file"""
    # Find the last `use songbird_errors::` statement
    uses_pattern = re.compile(r'(use\s+songbird_errors::\{[^}]+\};)', re.MULTILINE)
    
    # Try to find existing songbird_errors imports with braces
    match = uses_pattern.search(content)
    if match:
        # Add SongbirdResponse to existing import list
        import_stmt = match.group(1)
        # Check if it ends with }; and insert before the }
        if import_stmt.endswith('};'):
            new_import = import_stmt[:-2] + ', SongbirdResponse};'
            content = content.replace(import_stmt, new_import, 1)
            return content
    
    # Look for simple songbird_errors import
    simple_pattern = re.compile(r'(use\s+songbird_errors::[^;]+;)', re.MULTILINE)
    matches = list(simple_pattern.finditer(content))
    if matches:
        # Add after the last songbird_errors import
        last_match = matches[-1]
        insert_pos = last_match.end()
        new_import = '\nuse songbird_errors::SongbirdResponse;'
        content = content[:insert_pos] + new_import + content[insert_pos:]
        return content
    
    # Look for any use statement
    any_use_pattern = re.compile(r'(use\s+[^;]+;)', re.MULTILINE)
    matches = list(any_use_pattern.finditer(content))
    if matches:
        # Add after the last use statement
        last_match = matches[-1]
        insert_pos = last_match.end()
        new_import = '\nuse songbird_errors::SongbirdResponse;'
        content = content[:insert_pos] + new_import + content[insert_pos:]
        return content
    
    return content

def migrate_file(file_path):
    """Add SongbirdResponse import if needed"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    if needs_import(content):
        new_content = add_import(content)
        if new_content != content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(new_content)
            return True
    
    return False

def main():
    base_path = Path(__file__).parent.parent
    network_path = base_path / 'crates' / 'songbird-network' / 'src'
    
    print("📦 Adding SongbirdResponse imports...")
    
    files = list(network_path.rglob('*.rs'))
    modified = 0
    
    for file_path in files:
        if migrate_file(file_path):
            modified += 1
            rel_path = file_path.relative_to(base_path)
            print(f"✅ {rel_path}")
    
    print(f"\n🎉 Added imports to {modified} files")
    if modified > 0:
        print(f"💡 Next: cargo build -p songbird-network")

if __name__ == '__main__':
    main() 