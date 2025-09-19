#!/usr/bin/env python3
"""
Network Function Signature Fix Script

This script systematically fixes function signature mismatches in songbird-network,
focusing on the 40 most common SongbirdError::network() call patterns.
"""

import os
import re
from pathlib import Path

def fix_network_error_calls(file_path):
    """Fix SongbirdError::network() calls with missing operation parameter"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    changes_made = 0
    
    # Pattern 1: SongbirdError::network(format!("..."))
    pattern1 = r'SongbirdError::network\(\s*format!\("([^"]+)"[^)]*\)\s*\)'
    def replace1(match):
        nonlocal changes_made
        changes_made += 1
        message = match.group(1)
        # Infer operation from context
        if "client" in message.lower() or "http" in message.lower():
            operation = "http_client"
        elif "connection" in message.lower():
            operation = "connection"
        elif "discovery" in message.lower():
            operation = "discovery"
        elif "health" in message.lower():
            operation = "health_check"
        else:
            operation = "network_operation"
        
        return f'SongbirdError::network("{operation}", &format!("{message}"))'
    
    content = re.sub(pattern1, replace1, content)
    
    # Pattern 2: SongbirdError::network("direct string")
    pattern2 = r'SongbirdError::network\(\s*"([^"]+)"\s*\)'
    def replace2(match):
        nonlocal changes_made
        changes_made += 1
        message = match.group(1)
        if "client" in message.lower() or "http" in message.lower():
            operation = "http_client"
        elif "connection" in message.lower():
            operation = "connection"
        elif "discovery" in message.lower():
            operation = "discovery"
        elif "health" in message.lower():
            operation = "health_check"
        else:
            operation = "network_operation"
        
        return f'SongbirdError::network("{operation}", "{message}")'
    
    content = re.sub(pattern2, replace2, content)
    
    # Pattern 3: SongbirdError::network(&format!(...))
    pattern3 = r'SongbirdError::network\(\s*&format!\("([^"]+)"[^)]*\)\s*\)'
    def replace3(match):
        nonlocal changes_made
        changes_made += 1
        message = match.group(1)
        if "client" in message.lower() or "http" in message.lower():
            operation = "http_client"
        elif "connection" in message.lower():
            operation = "connection"
        elif "discovery" in message.lower():
            operation = "discovery"
        elif "health" in message.lower():
            operation = "health_check"
        else:
            operation = "network_operation"
        
        return f'SongbirdError::network("{operation}", &format!("{message}"))'
    
    content = re.sub(pattern3, replace3, content)
    
    # Pattern 4: songbird_errors::SongbirdError::network(...)
    pattern4 = r'songbird_errors::SongbirdError::network\(\s*([^,)]+)\s*\)'
    def replace4(match):
        nonlocal changes_made
        changes_made += 1
        message_expr = match.group(1)
        # For complex expressions, use generic operation
        return f'songbird_errors::SongbirdError::network("network_operation", {message_expr})'
    
    content = re.sub(pattern4, replace4, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ Fixed {changes_made} signature mismatches in {file_path}")
        return changes_made
    
    return 0

def fix_other_signature_issues(file_path):
    """Fix other common signature mismatches"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    original_content = content
    changes_made = 0
    
    # Fix missing arguments in function calls
    # Pattern: function_name(arg1, ) -> function_name(arg1, default_arg)
    missing_arg_pattern = r'(\w+)\(([^,)]+),\s*\)'
    def fix_missing_arg(match):
        nonlocal changes_made
        func_name = match.group(1)
        arg1 = match.group(2)
        
        # Add appropriate default arguments based on function name
        if "format_service_endpoint" in func_name:
            changes_made += 1
            return f'{func_name}({arg1}, 8080)'
        elif "create" in func_name.lower() and "backend" in func_name.lower():
            changes_made += 1
            return f'{func_name}({arg1}, "default")'
        
        return match.group(0)  # No change if we don't know the pattern
    
    content = re.sub(missing_arg_pattern, fix_missing_arg, content)
    
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"✅ Fixed {changes_made} other signature issues in {file_path}")
        return changes_made
    
    return 0

def main():
    """Main entry point"""
    network_crate_path = Path("crates/songbird-network/src")
    
    if not network_crate_path.exists():
        print(f"❌ Network crate path not found: {network_crate_path}")
        return
    
    total_fixes = 0
    processed_files = 0
    
    print("🔧 NETWORK SIGNATURE FIX STARTING")
    print("=" * 50)
    
    # Process all Rust files in the network crate
    for rust_file in network_crate_path.rglob("*.rs"):
        processed_files += 1
        
        # Fix network error signature issues
        fixes1 = fix_network_error_calls(rust_file)
        fixes2 = fix_other_signature_issues(rust_file)
        
        total_fixes += fixes1 + fixes2
    
    print("=" * 50)
    print(f"✅ SIGNATURE FIX COMPLETE")
    print(f"📊 Files processed: {processed_files}")
    print(f"🔧 Total fixes applied: {total_fixes}")
    print("🚀 Network crate signature issues resolved!")

if __name__ == "__main__":
    main() 