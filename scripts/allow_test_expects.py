#!/usr/bin/env python3
"""
Idiomatic solution: Allow expect/unwrap in test code where it's appropriate.
This is the Rust community standard - tests can use unwrap/expect for clarity.
"""

from pathlib import Path
import sys

def add_test_allows(file_path: Path) -> bool:
    """Add #![allow(clippy::unwrap_used)] to test files."""
    content = file_path.read_text()
    
    # Check if already has the allow
    if '#![allow(clippy::unwrap_used)]' in content or '#![allow(clippy::expect_used)]' in content:
        return False
    
    # Add allows at the top after any existing attributes
    lines = content.split('\n')
    
    # Find where to insert (after any #! attributes at top)
    insert_pos = 0
    for i, line in enumerate(lines):
        if line.strip().startswith('#!['):
            insert_pos = i + 1
        elif line.strip() and not line.strip().startswith('//'):
            break
    
    # Insert the allows
    lines.insert(insert_pos, '// Allow unwrap/expect in tests - idiomatic for test code')
    lines.insert(insert_pos + 1, '#![allow(clippy::unwrap_used, clippy::expect_used)]')
    lines.insert(insert_pos + 2, '')
    
    file_path.write_text('\n'.join(lines))
    return True

def main():
    crate_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("crates")
    
    total_fixed = 0
    for tests_dir in crate_path.glob("*/tests"):
        for test_file in tests_dir.glob("*.rs"):
            if add_test_allows(test_file):
                print(f"✅ Added test allows to {test_file.relative_to(crate_path)}")
                total_fixed += 1
    
    print(f"\n🎉 Summary: Added test allows to {total_fixed} test files")
    return 0

if __name__ == "__main__":
    sys.exit(main())

