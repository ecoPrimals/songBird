#!/usr/bin/env python3
"""
Targeted import semicolon fixer for the exact pattern found.
"""

import os
import glob
import re

def fix_file(filepath):
    """Fix import statements in a single file."""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            lines = f.readlines()
        
        modified = False
        for i, line in enumerate(lines):
            # Fix the exact patterns we see in the errors
            if line.strip() == 'use serde:{Deserialize, Serialize}':
                lines[i] = 'use serde:{Deserialize, Serialize};\n'
                modified = True
                print(f"  → Fixed serde import on line {i+1}")
            elif line.strip() == 'use chrono:{DateTime, Utc}':
                lines[i] = 'use chrono:{DateTime, Utc};\n'
                modified = True
                print(f"  → Fixed chrono import on line {i+1}")
            elif line.strip() == 'use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo}':
                lines[i] = 'use crate::service:{CanonicalServiceEndpoint, CanonicalServiceInfo};\n'
                modified = True
                print(f"  → Fixed crate::service import on line {i+1}")
        
        if modified:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.writelines(lines)
            print(f"✅ Fixed imports in {filepath}")
            return True
        else:
            return False
    except Exception as e:
        print(f"❌ Error fixing {filepath}: {e}")
        return False

def main():
    """Fix all import issues in songbird-types."""
    print("🎯 TARGETED IMPORT FIXER")
    print("=" * 40)
    
    # Target specific files that had errors
    files = [
        'crates/songbird-types/src/config/api.rs',
        'crates/songbird-types/src/config/communication.rs',
        'crates/songbird-types/src/config/environment.rs',
        'crates/songbird-types/src/config/federation.rs',
        'crates/songbird-types/src/config/gaming.rs',
        'crates/songbird-types/src/config/health.rs',
        'crates/songbird-types/src/config/migration.rs',
        'crates/songbird-types/src/config/network.rs',
        'crates/songbird-types/src/config/orchestration.rs',
        'crates/songbird-types/src/config/security.rs',
        'crates/songbird-types/src/config/unified.rs',
        'crates/songbird-types/src/errors.rs',
        'crates/songbird-types/src/health.rs',
        'crates/songbird-types/src/primal.rs',
        'crates/songbird-types/src/response.rs',
        'crates/songbird-types/src/service.rs',
        'crates/songbird-types/src/traits.rs',
        'crates/songbird-types/src/types.rs',
    ]
    
    fixed_count = 0
    for filepath in files:
        if os.path.exists(filepath):
            print(f"\n📝 Processing {filepath}...")
            if fix_file(filepath):
                fixed_count += 1
        else:
            print(f"⚠️  File not found: {filepath}")
    
    print("=" * 40)
    print(f"🎉 Fixed {fixed_count} files!")
    print("🚀 Ready for compilation test!")

if __name__ == "__main__":
    main() 