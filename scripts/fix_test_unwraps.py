#!/usr/bin/env python3
"""
Deep debt solution: Evolve test unwraps to modern idiomatic Rust with expect()
Following principle: descriptive error messages for test failures
"""

import re
import sys
from pathlib import Path

def fix_unwraps_in_file(file_path: Path) -> tuple[int, int]:
    """
    Replace .unwrap() and .unwrap_err() with .expect() in test files.
    Returns (unwraps_fixed, unwrap_errs_fixed)
    """
    content = file_path.read_text()
    original = content
    
    unwraps_fixed = 0
    unwrap_errs_fixed = 0
    
    # Pattern 1: .unwrap() -> .expect("descriptive message")
    # Look for context to generate meaningful messages
    lines = content.split('\n')
    new_lines = []
    
    for line in lines:
        original_line = line
        
        # Fix .unwrap() with context-aware messages
        if '.unwrap()' in line and not line.strip().startswith('//'):
            # Determine context from variable names or function calls
            if 'from_str' in line or 'parse' in line:
                line = line.replace('.unwrap()', '.expect("should parse valid input")')
            elif 'push' in line:
                line = line.replace('.unwrap()', '.expect("should have capacity")')
            elif 'get' in line or 'find' in line:
                line = line.replace('.unwrap()', '.expect("should find expected value")')
            elif 'lock' in line:
                line = line.replace('.unwrap()', '.expect("should acquire lock")')
            else:
                line = line.replace('.unwrap()', '.expect("test precondition")')
            
            if line != original_line:
                unwraps_fixed += line.count('.expect(')
        
        # Fix .unwrap_err() - these are testing error cases
        if '.unwrap_err()' in line and not line.strip().startswith('//'):
            line = line.replace('.unwrap_err()', '.expect_err("testing error case")')
            if line != original_line:
                unwrap_errs_fixed += line.count('.expect_err(')
        
        new_lines.append(line)
    
    new_content = '\n'.join(new_lines)
    
    if new_content != original:
        file_path.write_text(new_content)
        print(f"✅ Fixed {file_path.name}: {unwraps_fixed} unwraps, {unwrap_errs_fixed} unwrap_errs")
    
    return unwraps_fixed, unwrap_errs_fixed

def main():
    crate_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("crates/songbird-types")
    tests_dir = crate_path / "tests"
    
    if not tests_dir.exists():
        print(f"❌ Tests directory not found: {tests_dir}")
        return 1
    
    total_unwraps = 0
    total_unwrap_errs = 0
    files_fixed = 0
    
    for test_file in tests_dir.glob("*.rs"):
        unwraps, unwrap_errs = fix_unwraps_in_file(test_file)
        if unwraps > 0 or unwrap_errs > 0:
            files_fixed += 1
            total_unwraps += unwraps
            total_unwrap_errs += unwrap_errs
    
    print(f"\n🎉 Summary:")
    print(f"   Files fixed: {files_fixed}")
    print(f"   Total .unwrap() → .expect(): {total_unwraps}")
    print(f"   Total .unwrap_err() → .expect_err(): {total_unwrap_errs}")
    
    return 0

if __name__ == "__main__":
    sys.exit(main())

