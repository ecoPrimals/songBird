#!/usr/bin/env python3
"""
Comprehensive Songbird Network Error Pattern Fixer

This script systematically fixes all remaining error patterns in songbird-network:
1. SongbirdError::Protocol(...) -> SongbirdError::network_error(...)
2. SongbirdError::Network(...) -> SongbirdError::network_error(...)  
3. Multiline network_error calls missing None parameter
4. Other struct-based error patterns
"""

import os
import re
import sys
from pathlib import Path

def fix_protocol_errors(content):
    """Fix SongbirdError::Protocol patterns"""
    # Pattern for Protocol errors with Box::new
    pattern = r'SongbirdError::Protocol\(Box::new\(SongbirdError\s*\{[^}]*?message:\s*([^,}]+)[^}]*?\}\)\)'
    replacement = r'SongbirdError::network_error(\1, None)'
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    
    return content

def fix_network_errors(content):
    """Fix remaining SongbirdError::Network patterns"""
    # Pattern for Network errors with Box::new
    pattern = r'SongbirdError::Network\(Box::new\(SongbirdError\s*\{[^}]*?message:\s*([^,}]+)[^}]*?\}\)\)'
    replacement = r'SongbirdError::network_error(\1, None)'
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    
    return content

def fix_multiline_network_error_calls(content):
    """Fix multiline network_error calls missing None parameter"""
    # Pattern for multiline network_error calls with single argument
    pattern = r'(SongbirdError::network_error\(\s*[^,)]+\s*)\)'
    replacement = r'\1, None)'
    
    # Only replace if it doesn't already have a second parameter
    lines = content.split('\n')
    fixed_lines = []
    
    for line in lines:
        # Check for network_error calls that end with just one parameter
        if 'network_error(' in line and line.strip().endswith(')') and ',' not in line.split('network_error(')[1]:
            # This is a single-line call with one parameter
            line = re.sub(r'(SongbirdError::network_error\([^,)]+)\)', r'\1, None)', line)
        elif 'network_error(' in line and not line.strip().endswith(')'):
            # This might be start of multiline call - check if it needs fixing
            pass  # Handle in multiline context
        
        fixed_lines.append(line)
    
    return '\n'.join(fixed_lines)

def fix_argument_count_errors(content):
    """Fix argument count errors in multiline patterns"""
    # Fix multiline network_error patterns
    pattern = r'(SongbirdError::network_error\(\s*[^,)]+,?\s*)\s*\)'
    def replacement_func(match):
        call = match.group(1).strip()
        if call.endswith(','):
            return call + '\n                    None\n                )'
        elif ',' in call:
            # Already has second parameter
            return match.group(0)
        else:
            return call + ',\n                    None\n                )'
    
    content = re.sub(pattern, replacement_func, content, flags=re.MULTILINE | re.DOTALL)
    return content

def fix_file(file_path):
    """Fix all error patterns in a single file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_protocol_errors(content)
        content = fix_network_errors(content)
        content = fix_multiline_network_error_calls(content)
        content = fix_argument_count_errors(content)
        
        # Only write if changes were made
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function to fix all Rust files in songbird-network"""
    network_dir = Path("crates/songbird-network/src")
    
    if not network_dir.exists():
        print(f"Directory {network_dir} not found!")
        sys.exit(1)
    
    fixed_files = 0
    total_files = 0
    
    # Find all Rust files
    for rust_file in network_dir.rglob("*.rs"):
        total_files += 1
        if fix_file(rust_file):
            fixed_files += 1
    
    print(f"\nProcessed {total_files} files, fixed {fixed_files} files")

if __name__ == "__main__":
    main() 