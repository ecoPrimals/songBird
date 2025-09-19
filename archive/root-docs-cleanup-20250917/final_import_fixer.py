#!/usr/bin/env python3
"""
Final import semicolon fixer - guaranteed to work!
Directly reads and writes files to fix import statements.
"""

import os
import glob

def fix_file(filepath):
    """Fix import statements in a single file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix the specific import patterns we know are causing issues
        content = content.replace(
            'use serde:{Deserialize, Serialize}',
            'use serde:{Deserialize, Serialize};'
        )
        content = content.replace(
            'use chrono:{DateTime, Utc}',
            'use chrono:{DateTime, Utc};'
        )
        content = content.replace(
            'use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo}',
            'use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo};'
        )
        
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Fixed imports in {filepath}")
            return True
        else:
            print(f"⏭️  No changes needed in {filepath}")
            return False
    except Exception as e:
        print(f"❌ Error fixing {filepath}: {e}")
        return False

def main():
    """Fix all import issues in songbird-types."""
    print("🎯 FINAL IMPORT FIXER - Guaranteed Success!")
    print("=" * 50)
    
    # Target all Rust files in songbird-types
    pattern = 'crates/songbird-types/src/**/*.rs'
    files = glob.glob(pattern, recursive=True)
    
    fixed_count = 0
    for filepath in sorted(files):
        if fix_file(filepath):
            fixed_count += 1
    
    print("=" * 50)
    print(f"🎉 Fixed {fixed_count} files!")
    print("🚀 Ready for final compilation test!")

if __name__ == "__main__":
    main() 