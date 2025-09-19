#!/usr/bin/env python3
"""
Fix error signature issues in Songbird universal adapters
"""

import re
import os
from pathlib import Path

def fix_network_errors(content):
    """Fix network_error calls with malformed None arguments"""
    # Pattern: network_error(format!("...", ..., None))
    pattern = r'SongbirdError::network_error\(format!\("([^"]*)", ([^,]*), None\)\)'
    replacement = r'SongbirdError::network_error(format!("\1", \2), None::<String>)'
    return re.sub(pattern, replacement, content)

def fix_service_errors(content):
    """Fix service_error calls with malformed arguments"""
    # Pattern: service_error(&provider.name, format!("...", e, vec![...]))
    pattern = r'SongbirdError::service_error\(([^,]*), format!\("([^"]*)", ([^,]*), (vec!\[[^\]]*\])\)\)'
    replacement = r'SongbirdError::service_error(\1, format!("\2", \3), \4)'
    return re.sub(pattern, replacement, content)

def fix_operation_errors(content):
    """Fix operation_error calls that should be service_error"""
    # Replace operation_error with service_error
    content = re.sub(r'SongbirdError::operation_error', 'SongbirdError::service_error', content)
    return content

def fix_validation_errors(content):
    """Fix validation_error calls with missing arguments"""
    # Pattern: validation_error("message")
    pattern = r'SongbirdError::validation_error\(\s*"([^"]*)",?\s*\)'
    replacement = r'SongbirdError::config_error("\1", None::<String>)'
    return re.sub(pattern, replacement, content)

def fix_error_with_data_calls(content):
    """Fix error_with_data calls that don't exist"""
    # Replace SongbirdResponse::error_with_data with SongbirdResponse::error
    content = re.sub(r'SongbirdResponse::error_with_data', 'SongbirdResponse::error', content)
    return content

def fix_missing_request_fields(content):
    """Fix UniversalRequest struct initialization"""
    # Add missing operation and payload fields
    pattern = r'UniversalRequest\s*\{\s*request_id:\s*([^,]*),\s*source_primal_id:\s*([^,]*),\s*target_capability:\s*([^,]*),\s*timeout_ms:\s*([^,]*),\s*requires_response:\s*([^,]*),\s*\}'
    replacement = r'''UniversalRequest {
            request_id: \1,
            source_primal_id: \2,
            target_capability: \3,
            timeout_ms: \4,
            requires_response: \5,
            operation: "generic".to_string(),
            payload: serde_json::json!({}),
        }'''
    return re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)

def process_file(file_path):
    """Process a single file to fix error signatures"""
    print(f"Processing {file_path}")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Apply all fixes
    content = fix_network_errors(content)
    content = fix_service_errors(content)
    content = fix_operation_errors(content)
    content = fix_validation_errors(content)
    content = fix_error_with_data_calls(content)
    content = fix_missing_request_fields(content)
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"  ✅ Fixed error signatures in {file_path}")
        return True
    else:
        print(f"  ⏭️  No changes needed in {file_path}")
        return False

def main():
    """Main function to fix error signatures across the codebase"""
    print("🔧 Fixing error signatures in universal adapters...")
    
    # Files to process
    target_files = [
        "crates/songbird-universal/src/adapters/security.rs",
        "crates/songbird-universal/src/adapters/types/enums.rs", 
        "crates/songbird-universal/src/adapters/types/requests.rs",
        "crates/songbird-universal/src/adapters/compute.rs",
        "crates/songbird-universal/src/adapters/primal_integration.rs",
        "crates/songbird-universal/src/agnostic_adapter.rs",
    ]
    
    fixed_count = 0
    total_count = 0
    
    for file_path in target_files:
        if os.path.exists(file_path):
            total_count += 1
            if process_file(file_path):
                fixed_count += 1
        else:
            print(f"⚠️  File not found: {file_path}")
    
    print(f"\n📊 Summary:")
    print(f"  Files processed: {total_count}")
    print(f"  Files fixed: {fixed_count}")
    print(f"  Success rate: {(fixed_count/total_count*100):.1f}%")
    
    print("\n🎯 Error signature fixes complete!")

if __name__ == "__main__":
    main() 