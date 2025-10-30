#!/usr/bin/env python3
"""Find actual unsafe blocks in the codebase."""

import re
from pathlib import Path

def find_unsafe_blocks():
    """Find actual unsafe code blocks."""
    crates_dir = Path('crates')
    unsafe_blocks = []
    
    # Patterns for actual unsafe code
    unsafe_patterns = [
        r'unsafe\s*\{',           # unsafe blocks
        r'unsafe\s+fn\s+\w+',     # unsafe functions
        r'unsafe\s+impl',         # unsafe impl
        r'unsafe\s+trait',        # unsafe trait
    ]
    
    for rs_file in crates_dir.rglob('*.rs'):
        try:
            with open(rs_file, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            for idx, line in enumerate(lines):
                # Skip if it's a lint attribute
                if '#![forbid(unsafe_code)]' in line or '#![deny(unsafe_code)]' in line:
                    continue
                if '#![warn(unsafe_code)]' in line:
                    continue
                # Skip if it's just a comment or string
                stripped = line.strip()
                if stripped.startswith('//') or stripped.startswith('*'):
                    continue
                
                # Check for actual unsafe code
                for pattern in unsafe_patterns:
                    if re.search(pattern, line):
                        unsafe_blocks.append({
                            'file': str(rs_file),
                            'line': idx + 1,
                            'content': line.strip()
                        })
                        break
        except Exception as e:
            print(f"Error reading {rs_file}: {e}")
    
    return unsafe_blocks

def main():
    print("🔍 Finding ACTUAL unsafe code blocks...")
    print()
    
    blocks = find_unsafe_blocks()
    
    if blocks:
        print(f"⚠️  Found {len(blocks)} unsafe code instances:")
        print()
        
        # Group by file
        by_file = {}
        for item in blocks:
            file = item['file']
            if file not in by_file:
                by_file[file] = []
            by_file[file].append(item)
        
        for file, items in sorted(by_file.items()):
            print(f"📁 {file} ({len(items)} unsafe)")
            for item in items:
                print(f"   Line {item['line']}: {item['content']}")
            print()
    else:
        print("✅ No unsafe code blocks found!")
        print()
        print("🎉 Most crates have #![forbid(unsafe_code)] or #![deny(unsafe_code)]")
        print()
    
    print(f"📊 Total unsafe instances: {len(blocks)}")

if __name__ == '__main__':
    main()

