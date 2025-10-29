#!/usr/bin/env python3
"""Find unwraps in production code (excluding tests)."""

import os
import re
from pathlib import Path

def is_in_test_context(lines, line_idx):
    """Check if the line is within a test context."""
    # Look backwards up to 15 lines
    start = max(0, line_idx - 15)
    context = '\n'.join(lines[start:line_idx + 1])
    
    # Test markers
    test_markers = [
        r'#\[test\]',
        r'#\[tokio::test\]',
        r'#\[cfg\(test\)\]',
        r'fn test_',
        r'mod tests\s*\{',
    ]
    
    for marker in test_markers:
        if re.search(marker, context):
            return True
    
    return False

def find_production_unwraps():
    """Find unwraps in production code."""
    crates_dir = Path('crates')
    production_unwraps = []
    test_unwraps = 0
    
    for rs_file in crates_dir.rglob('*.rs'):
        # Skip test files and test directories
        if 'tests' in rs_file.parts or 'test' in rs_file.name.lower():
            continue
        
        # Skip if not in src directory
        if 'src' not in rs_file.parts:
            continue
        
        try:
            with open(rs_file, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            for idx, line in enumerate(lines):
                if '.unwrap()' in line or '.expect(' in line:
                    if is_in_test_context(lines, idx):
                        test_unwraps += 1
                    else:
                        production_unwraps.append({
                            'file': str(rs_file),
                            'line': idx + 1,
                            'content': line.strip()
                        })
        except Exception as e:
            print(f"Error reading {rs_file}: {e}")
    
    return production_unwraps, test_unwraps

def main():
    print("🔍 Finding unwraps in PRODUCTION code only (excluding tests)...")
    print()
    
    production, tests = find_production_unwraps()
    
    if production:
        print(f"⚠️  Found {len(production)} unwraps in PRODUCTION code:")
        print()
        
        # Group by file
        by_file = {}
        for item in production:
            file = item['file']
            if file not in by_file:
                by_file[file] = []
            by_file[file].append(item)
        
        for file, items in sorted(by_file.items()):
            print(f"📁 {file} ({len(items)} unwraps)")
            for item in items[:5]:  # Show first 5
                print(f"   Line {item['line']}: {item['content']}")
            if len(items) > 5:
                print(f"   ... and {len(items) - 5} more")
            print()
    else:
        print("✅ No unwraps found in production code!")
        print()
    
    print(f"📊 Summary:")
    print(f"   Production unwraps: {len(production)}")
    print(f"   Test unwraps (excluded): {tests}")
    print(f"   Total: {len(production) + tests}")
    print()
    
    if production:
        print("🎯 Priority files to review:")
        sorted_files = sorted(by_file.items(), key=lambda x: len(x[1]), reverse=True)
        for file, items in sorted_files[:10]:
            print(f"   {len(items):3d}  {file}")

if __name__ == '__main__':
    main()

