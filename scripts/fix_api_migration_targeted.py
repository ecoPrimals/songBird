#!/usr/bin/env python3
"""
Targeted API migration fixes based on compiler error patterns
"""

import re
import sys
from pathlib import Path

def fix_ok_unit_returns(content):
    """Fix Ok(()) to Ok(SongbirdResponse::success(()))"""
    # Only replace standalone Ok(()) statements at end of functions
    pattern = r'(\s+)Ok\(\(\)\)$'
    replacement = r'\1Ok(SongbirdResponse::success(()))'
    return re.sub(pattern, replacement, content, flags=re.MULTILINE)

def fix_field_assignment_unwrap(content):
    """Fix field = response_value to field = response_value.data"""
    # This is complex and needs context, so we'll be conservative
    # Pattern: somefield = function_call; where function returns SongbirdResponse
    # This is hard to do automatically, so skip for now
    return content

def process_file(filepath):
    """Process a single file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original = content
        content = fix_ok_unit_returns(content)
        
        if content != original:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
    except Exception as e:
        print(f"Error processing {filepath}: {e}", file=sys.stderr)
    return False

def main():
    # Focus on specific files mentioned in errors
    files = [
        "crates/songbird-network/src/http_server.rs",
        "crates/songbird-network/src/network/discovery/engine.rs",
        "crates/songbird-network/src/network/discovery/peer_registry.rs",
    ]
    
    fixed_count = 0
    for filepath in files:
        path = Path(filepath)
        if path.exists() and process_file(filepath):
            fixed_count += 1
            print(f"✓ Fixed: {filepath}")
    
    print(f"\n✅ Fixed {fixed_count} files")
    print("⚠️  Manual review still needed for:")
    print("   - Field assignment unwrapping (response.data)")
    print("   - SongbirdError::Network struct variant construction")

if __name__ == "__main__":
    main()

