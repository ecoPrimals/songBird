#!/usr/bin/env python3
"""
Comprehensive Error API Modernization Script for songbird-core
Fixes all outdated error API usage to use modern SongbirdError constructors.
"""

import os
import re
import glob

def fix_error_api_in_file(file_path):
    """Fix all error API issues in a single file"""
    print(f"Processing {file_path}...")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Fix 1: service_error -> service
    content = re.sub(r'SongbirdError::service_error\(', 'SongbirdError::service(', content)
    
    # Fix 2: io_error -> network (since there's no io_error)
    content = re.sub(r'SongbirdError::io_error\(', 'SongbirdError::network(', content)
    
    # Fix 3: rate_limit_error -> configuration
    content = re.sub(r'SongbirdError::rate_limit_error\(', 'SongbirdError::configuration(', content)
    
    # Fix 4: resource_exhausted_error -> configuration
    content = re.sub(r'SongbirdError::resource_exhausted_error\(', 'SongbirdError::configuration(', content)
    
    # Fix 5: circuit_breaker_error -> configuration
    content = re.sub(r'SongbirdError::circuit_breaker_error\(', 'SongbirdError::configuration(', content)
    
    # Fix 6: Old enum variant constructors to new function calls
    # SongbirdError::Service(Box::new(...)) -> SongbirdError::service(...)
    content = re.sub(
        r'SongbirdError::Service\(Box::new\(\s*ServiceError\s*\{[^}]*service:\s*([^,}]+)[^}]*\}\s*\)\)',
        lambda m: f'SongbirdError::service({m.group(1)}, "Service error")',
        content,
        flags=re.DOTALL
    )
    
    # Fix 7: SongbirdError::Network(Box::new(...)) -> SongbirdError::network(...)
    content = re.sub(
        r'SongbirdError::Network\(Box::new\(\s*NetworkError::new\(([^)]+)\)\s*\)\)',
        r'SongbirdError::network(\1)',
        content,
        flags=re.DOTALL
    )
    
    # Fix 8: configuration() calls with two arguments -> single argument
    # This handles: SongbirdError::configuration("field", "message") -> SongbirdError::configuration("message")
    content = re.sub(
        r'SongbirdError::configuration\(\s*"[^"]*"\s*\.to_string\(\)\s*,\s*([^)]+)\)',
        r'SongbirdError::configuration(\1)',
        content
    )
    
    # Fix 9: Fix PrimalHealth enum usage
    content = re.sub(r'PrimalHealth::Healthy', 'PrimalHealth::healthy()', content)
    content = re.sub(r'PrimalHealth::Degraded\s*\{\s*\.\.\s*\}', 'PrimalHealth::degraded("Unknown")', content)
    content = re.sub(r'PrimalHealth::Unhealthy\s*\{\s*\.\.\s*\}', 'PrimalHealth::unhealthy("Unknown")', content)
    content = re.sub(r'PrimalHealth::Unknown', 'PrimalHealth::degraded("Unknown")', content)
    
    # Fix 10: Method name corrections
    content = re.sub(r'\.health_check\(\)', '.health()', content)
    content = re.sub(r'\.handle_primal_request\(', '.handle_request(', content)
    
    # Fix 11: Fix return type issues for benchmark functions
    content = re.sub(
        r'let \(fast_ops_per_second, selection_times\) =\s*self\.benchmark_fast_algorithm\([^)]+\)\.await\?;',
        'let (fast_ops_per_second, selection_times) = self.benchmark_fast_algorithm(&instances).await?;\n        let selection_times: Vec<u64> = selection_times;',
        content
    )
    
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
        if fix_error_api_in_file(file_path):
            updated_files += 1
    
    print(f"\n🎉 Modernization complete!")
    print(f"📊 Updated {updated_files} out of {len(rust_files)} files")

if __name__ == "__main__":
    main() 