#!/usr/bin/env python3
"""
Deep debt solution: Evolve tests to idiomatic Rust with Result<()> returns and ? operator
This is the modern, truly idiomatic approach for Rust tests.
"""

import re
from pathlib import Path
import sys

def evolve_test_to_result(file_path: Path) -> int:
    """
    Evolve test functions to use Result<()> return type and ? operator.
    This is the idiomatic Rust way - more maintainable than expect().
    """
    content = file_path.read_text()
    lines = content.split('\n')
    new_lines = []
    fixes = 0
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Find test functions
        if '#[test]' in line:
            # Look ahead for function signature
            j = i + 1
            while j < len(lines) and not lines[j].strip().startswith('fn '):
                new_lines.append(lines[i])
                i += 1
                j += 1
            
            if j < len(lines):
                func_line = lines[j]
                # Check if function already returns Result
                if '-> Result<' not in func_line and '-> SongbirdResult' not in func_line:
                    # Add Result<()> return type
                    func_line = func_line.replace('{', '-> Result<(), Box<dyn std::error::Error>> {')
                    
                    # Need to add Ok(()) at the end
                    # Find the closing brace
                    brace_count = 0
                    func_start = j
                    func_end = j
                    for k in range(j, len(lines)):
                        if '{' in lines[k]:
                            brace_count += lines[k].count('{')
                        if '}' in lines[k]:
                            brace_count -= lines[k].count('}')
                            if brace_count == 0:
                                func_end = k
                                break
                    
                    # Collect function body
                    new_lines.append(lines[i])  # #[test]
                    new_lines.append(func_line)  # modified function signature
                    i = j + 1
                    
                    # Add function body with fixes
                    while i < func_end:
                        body_line = lines[i]
                        # Replace .expect() with ?
                        body_line = re.sub(r'\.expect\([^)]+\)', '?', body_line)
                        # Replace .expect_err() with special handling
                        if '.expect_err(' in body_line:
                            # Keep expect_err for error testing - it's needed
                            pass  # Don't modify error testing code
                        new_lines.append(body_line)
                        i += 1
                    
                    # Add Ok(()) before closing brace
                    closing_line = lines[func_end]
                    indent = len(closing_line) - len(closing_line.lstrip())
                    new_lines.append(' ' * indent + 'Ok(())')
                    new_lines.append(closing_line)
                    i = func_end + 1
                    fixes += 1
                    continue
        
        new_lines.append(line)
        i += 1
    
    new_content = '\n'.join(new_lines)
    if new_content != content:
        file_path.write_text(new_content)
        print(f"✅ Evolved {file_path.name}: {fixes} test functions")
        return fixes
    return 0

def main():
    crate_path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("crates/songbird-types")
    tests_dir = crate_path / "tests"
    
    if not tests_dir.exists():
        print(f"❌ Tests directory not found: {tests_dir}")
        return 1
    
    total_fixes = 0
    for test_file in tests_dir.glob("*.rs"):
        fixes = evolve_test_to_result(test_file)
        total_fixes += fixes
    
    print(f"\n🎉 Summary: Evolved {total_fixes} test functions to use Result<()>")
    return 0

if __name__ == "__main__":
    sys.exit(main())

