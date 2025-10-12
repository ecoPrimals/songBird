#!/usr/bin/env python3
"""Fix ALL remaining syntax errors comprehensively."""

import re
from pathlib import Path

def fix_line_patterns(content):
    """Fix patterns that need semicolons."""
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        fixed = line.rstrip()
        
        # Pattern: lines ending with ) that should have semicolon
        # Check patterns: info!(...) debug!(...) warn!(...) error!(...) println!(...) format!(...)
        if re.search(r'(info|debug|warn|error|trace|println|format)!\([^)]*\)\s*$', fixed):
            # Check next line to determine if semicolon needed
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                # Needs semicolon if next line is code (not closing delimiters)
                if next_line and not next_line.startswith('}') and not next_line.startswith(')') and not next_line.startswith(','):
                    if not fixed.endswith(';'):
                        # Add semicolon
                        fixed += ';'
        
        # Pattern: format!(...) with comment but no semicolon
        elif re.search(r'format!\([^)]*\)\s*//.*$', fixed):
            if not re.search(r'format!\([^)]*\);', fixed):
                # Insert semicolon before comment
                fixed = re.sub(r'(\))\s*(//.*)', r'\1;\2', fixed)
        
        fixed_lines.append(fixed if fixed or not line.strip() else line)
    
    return '\n'.join(fixed_lines)

def fix_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_line_patterns(content)
        
        if content != fixed_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    # Target specific files that still have errors
    problem_files = [
        "crates/songbird-cli/src/bin/gaming_demo.rs",
        "crates/songbird-universal/src/capabilities.rs",
        "crates/songbird-universal/src/discovery.rs",
    ]
    
    # Also scan all Rust files
    crates_dir = Path("crates")
    all_files = list(crates_dir.rglob("*.rs"))
    
    fixed_count = 0
    for rust_file in all_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"Fixed: {rust_file}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

