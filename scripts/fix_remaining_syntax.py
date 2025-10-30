#!/usr/bin/env python3
"""
Fix remaining syntax errors - second pass
"""

import re
from pathlib import Path

def fix_file(filepath: Path) -> int:
    """Fix remaining syntax errors in a file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except:
        return 0
    
    original = content
    fixes = 0
    
    # Fix: Some(value; -> Some(value);
    pattern1 = r'(Some\([^)]+);'
    matches = list(re.finditer(pattern1, content))
    for match in reversed(matches):  # reverse to maintain positions
        # Check if there's actually a missing paren
        inner = match.group(1)[5:]  # Remove "Some("
        if inner.count('(') > inner.count(')'):
            # Balanced, skip
            continue
        # Add missing paren
        new_text = match.group(1) + ');'
        content = content[:match.start()] + new_text + content[match.end():]
        fixes += 1
    
    # Fix: assert!(value)); -> assert!(value);
    pattern2 = r'assert!\(([^)]+)\)\s*\)\s*;'
    matches = list(re.finditer(pattern2, content))
    for match in reversed(matches):
        new_text = f'assert!({match.group(1)});'
        content = content[:match.start()] + new_text + content[match.end():]
        fixes += 1
    
    # Fix: assert_eq!(a, b)); -> assert_eq!(a, b);
    pattern3 = r'assert_eq!\(([^)]+,[^)]+)\)\s*\)\s*;'
    matches = list(re.finditer(pattern3, content))
    for match in reversed(matches):
        new_text = f'assert_eq!({match.group(1)});'
        content = content[:match.start()] + new_text + content[match.end():]
        fixes += 1
    
    # Fix: function_call("value", Some(arg); -> function_call("value", Some(arg));
    pattern4 = r'([a-zA-Z_][a-zA-Z0-9_]*\([^;]+, Some\([^)]+\));'
    matches = list(re.finditer(pattern4, content))
    for match in reversed(matches):
        if match.group(0).count('(') != match.group(0).count(')'):
            new_text = match.group(0)[:-2] + '));'
            content = content[:match.start()] + new_text + content[match.end():]
            fixes += 1
    
    # Fix: banner("text", Some("text"); -> banner("text", Some("text"));
    pattern5 = r'([a-zA-Z_][a-zA-Z0-9_]*)\(([^;]+), Some\(([^)]+)\);'
    matches = list(re.finditer(pattern5, content))
    for match in reversed(matches):
        # Count parens to check if missing
        text = match.group(0)
        if text.count('(') != text.count(')'):
            new_text = f'{match.group(1)}({match.group(2)}, Some({match.group(3)}));'
            content = content[:match.start()] + new_text + content[match.end():]
            fixes += 1
    
    # Fix: self.field = Some(value; -> self.field = Some(value);
    pattern6 = r'= Some\(([^)]+);'
    matches = list(re.finditer(pattern6, content))
    for match in reversed(matches):
        inner = match.group(1)
        if inner.count('(') <= inner.count(')'):
            # Likely missing closing paren
            new_text = f'= Some({inner});'
            content = content[:match.start()] + new_text + content[match.end():]
            fixes += 1
    
    # Fix: method(arg1, arg2; -> method(arg1, arg2);
    pattern7 = r'([a-zA-Z_][a-zA-Z0-9_]*)\(([^;()]+,[^;()]+);'
    matches = list(re.finditer(pattern7, content))
    for match in reversed(matches):
        # Simple case: no nested parens in args
        if '(' not in match.group(2):
            new_text = f'{match.group(1)}({match.group(2)});'
            content = content[:match.start()] + new_text + content[match.end():]
            fixes += 1
    
    if content != original:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✓ {filepath.name}: {fixes} fixes")
        return fixes
    
    return 0

def main():
    # Target the specific files with errors
    error_files = [
        "crates/songbird-canonical/src/metadata.rs",
        "crates/songbird-cli/src/cli/commands/status.rs",
        "crates/songbird-cli/tests/cli_comprehensive_tests.rs",
        "crates/songbird-config/src/config/mod.rs",
        "crates/songbird-config/tests/comprehensive_config_tests.rs",
        "crates/songbird-core/src/api/ai_optimized/cache.rs",
        "crates/songbird-discovery/src/discovery/backends/consul.rs",
        "crates/songbird-discovery/tests/discovery_basic_tests.rs",
    ]
    
    total_fixes = 0
    for file_path in error_files:
        filepath = Path(file_path)
        if filepath.exists():
            fixes = fix_file(filepath)
            total_fixes += fixes
        else:
            print(f"✗ {file_path}: not found")
    
    print(f"\nTotal fixes: {total_fixes}")

if __name__ == "__main__":
    main()

