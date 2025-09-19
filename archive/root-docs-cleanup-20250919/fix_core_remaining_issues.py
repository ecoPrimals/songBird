#!/usr/bin/env python3
"""
Fix remaining issues in songbird-core after the initial error API modernization.
"""

import os
import re

def fix_remaining_issues_in_file(file_path):
    """Fix remaining issues in a single file"""
    print(f"Processing {file_path}...")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Fix 1: Replace songbird_errors::Result with songbird_errors::SongbirdResult
    content = re.sub(r'use songbird_errors::Result;', 'use songbird_errors::SongbirdResult;', content)
    content = re.sub(r'use songbird_errors::\{Result\};', 'use songbird_errors::{SongbirdResult};', content)
    
    # Fix 2: Replace remaining old enum constructors with new function calls
    # SongbirdError::Service(Box::new(ServiceError::new(...))) -> SongbirdError::service(...)
    content = re.sub(
        r'SongbirdError::Service\(Box::new\(\s*ServiceError::new\(\s*"([^"]+)",\s*([^)]+)\)\s*\)\)',
        r'SongbirdError::service("\1", \2)',
        content,
        flags=re.DOTALL
    )
    
    # Fix 3: SongbirdError::Network(Box::new(NetworkError::new(...))) -> SongbirdError::network(...)
    content = re.sub(
        r'SongbirdError::Network\(Box::new\(\s*NetworkError::new\(([^)]+)\)\s*\)\)',
        r'SongbirdError::network(\1)',
        content,
        flags=re.DOTALL
    )
    
    # Fix 4: Fix remaining ServiceError struct literals
    content = re.sub(
        r'SongbirdError::Service\(Box::new\(\s*ServiceError\s*\{[^}]*service:\s*"([^"]+)"[^}]*message:\s*"([^"]+)"[^}]*\}\s*\)\)',
        r'SongbirdError::service("\1", "\2")',
        content,
        flags=re.DOTALL
    )
    
    # Fix 5: Fix PrimalHealth pattern matching - can't use function calls in patterns
    # Replace function calls with wildcard patterns and use if guards
    content = re.sub(
        r'songbird_universal_primals::traits::PrimalHealth::healthy\(\)',
        'ref health if health.is_healthy()',
        content
    )
    
    content = re.sub(
        r'songbird_universal_primals::traits::PrimalHealth::degraded\("[^"]*"\)',
        'ref health if health.is_degraded()',
        content
    )
    
    content = re.sub(
        r'songbird_universal_primals::traits::PrimalHealth::unhealthy\("[^"]*"\)',
        'ref health if health.is_unhealthy()',
        content
    )
    
    # Fix 6: Fix configuration calls with two arguments
    content = re.sub(
        r'SongbirdError::configuration\(\s*([^,)]+),\s*([^)]+)\)',
        r'SongbirdError::configuration(\2)',
        content
    )
    
    # Fix 7: Fix imports for missing types
    if 'ServiceError' in content and 'use songbird_universal::ServiceError' not in content:
        # Add the import after other imports
        import_pattern = r'(use songbird_[^;]+;)'
        if re.search(import_pattern, content):
            content = re.sub(
                import_pattern,
                r'\1\nuse songbird_universal::ServiceError;',
                content,
                count=1
            )
    
    # Fix 8: Fix core module imports
    content = re.sub(
        r'pub use core::\{types as core_types, ResourceUsage as CoreResourceUsage\};',
        'pub use crate::api::core::{CoreApiConfig, CoreApiHandler, CoreApiRequest, CoreApiResponse};',
        content
    )
    
    # Fix 9: Fix method name mismatches
    content = re.sub(r'\.health\(\)', '.health_check()', content)
    
    # Fix 10: Fix CanonicalResponse field access
    content = re.sub(r'primal_response\.success', 'primal_response.status == "success"', content)
    content = re.sub(r'primal_response\.payload', 'primal_response.data', content)
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"  ✅ Updated {file_path}")
        return True
    else:
        print(f"  ℹ️  No changes needed in {file_path}")
        return False

def main():
    """Main function to process all Rust files in songbird-core"""
    core_dir = "crates/songbird-core/src"
    
    if not os.path.exists(core_dir):
        print(f"❌ Directory {core_dir} not found")
        return
    
    # Find all Rust files
    rust_files = []
    for root, dirs, files in os.walk(core_dir):
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(os.path.join(root, file))
    
    print(f"🔍 Found {len(rust_files)} Rust files to process")
    
    updated_files = 0
    for file_path in rust_files:
        if fix_remaining_issues_in_file(file_path):
            updated_files += 1
    
    print(f"\n🎉 Remaining issues fixed!")
    print(f"📊 Updated {updated_files} out of {len(rust_files)} files")

if __name__ == "__main__":
    main() 