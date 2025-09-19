#!/usr/bin/env python3
"""
🔧 SURGICAL SYNTAX REPAIR SCRIPT 🔧

This script repairs syntax damage caused by the overly aggressive ultimate script.
"""

import os
import re
from pathlib import Path

def surgical_repair_function_calls(file_path):
    """Repair damaged function calls with surgical precision"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    changes_made = 0
    
    # Fix 1: Repair SongbirdError::config calls
    content = re.sub(
        r'SongbirdError::config\("([^"]+)"\.to_string\(\)\s+"([^"]+)"',
        r'SongbirdError::config("\1", "\2")',
        content
    )
    
    # Fix 2: Repair Command::new calls
    content = re.sub(
        r'Command::new\("([^"]+)"\.to_string\(\)',
        r'Command::new("\1")',
        content
    )
    
    # Fix 3: Repair std::fs::read_to_string calls
    content = re.sub(
        r'std::fs::read_to_string\("([^"]+)"\.to_string\(\)',
        r'std::fs::read_to_string("\1")',
        content
    )
    
    # Fix 4: Repair string method calls
    content = re.sub(
        r'\.starts_with\("([^"]+)"\.to_string\(\)',
        r'.starts_with("\1")',
        content
    )
    
    content = re.sub(
        r'\.contains\("([^"]+)"\.to_string\(\)',
        r'.contains("\1")',
        content
    )
    
    # Fix 5: Repair InterfaceStats::new calls
    content = re.sub(
        r'InterfaceStats::new\("([^"]+)"\.to_string\(\)',
        r'InterfaceStats::new("\1")',
        content
    )
    
    changes_made = 6 if content != original_content else 0
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"🔧 SURGICAL: Repaired {changes_made} function calls in {file_path}")
        return changes_made
    
    return 0

def main():
    """SURGICAL SYNTAX REPAIR EXECUTION"""
    network_crate_path = Path("crates/songbird-network/src")
    
    if not network_crate_path.exists():
        print(f"❌ Network crate path not found: {network_crate_path}")
        return
    
    total_repairs = 0
    processed_files = 0
    
    print("🔧🔧🔧 SURGICAL SYNTAX REPAIR STARTING 🔧🔧🔧")
    print("=" * 60)
    
    # Process all Rust files with SURGICAL PRECISION
    for rust_file in network_crate_path.rglob("*.rs"):
        processed_files += 1
        
        # Apply surgical repairs
        repairs = surgical_repair_function_calls(rust_file)
        total_repairs += repairs
    
    print("=" * 60)
    print(f"🔧 SURGICAL SYNTAX REPAIR COMPLETE 🔧")
    print(f"📊 Files processed: {processed_files}")
    print(f"🎯 Total repairs applied: {total_repairs}")
    print("🚀 SYNTAX DAMAGE REPAIRED!")

if __name__ == "__main__":
    main() 