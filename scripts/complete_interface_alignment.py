#!/usr/bin/env python3
"""
Complete interface alignment for vendor-agnostic architecture
Final polish to achieve full production readiness
"""

import re
import os
from pathlib import Path

def fix_discovery_config_fields(content):
    """Fix DiscoveryConfig field name mismatches"""
    # Fix discovery_timeout_ms -> discovery_timeout
    content = re.sub(r'discovery_timeout_ms:', 'discovery_timeout:', content)
    
    # Remove unsupported fields and use correct ones
    patterns = [
        (r'max_concurrent_discoveries:\s*\d+,?\s*\n', ''),
        (r'retry_attempts:\s*\d+,?\s*\n', ''),
        (r'health_check_interval_ms:\s*\d+,?\s*\n', ''),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_service_error_arguments(content):
    """Fix remaining service_error calls with incorrect argument counts"""
    # Pattern: service_error(format!("...", e)) -> service_error("service", format!("...", e), vec![])
    pattern = r'SongbirdError::service_error\(format!\("([^"]*)", ([^)]*)\)\)'
    replacement = r'SongbirdError::service_error("universal", format!("\1", \2), vec![])'
    content = re.sub(pattern, replacement, content)
    
    # Fix service_error calls missing the third argument
    pattern = r'SongbirdError::service_error\(([^,]+),\s*([^,]+)\)(?!\s*,)'
    replacement = r'SongbirdError::service_error(\1, \2, vec![])'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_network_error_arguments(content):
    """Fix network_error calls with malformed arguments"""
    # Fix network_error calls with None inside format
    pattern = r'SongbirdError::network_error\(format!\("([^"]*)", ([^,]*), None\)\)'
    replacement = r'SongbirdError::network_error(format!("\1", \2), None::<String>)'
    content = re.sub(pattern, replacement, content)
    
    # Fix network_error calls missing second argument
    pattern = r'SongbirdError::network_error\(([^,)]+)\)(?!\s*,)'
    replacement = r'SongbirdError::network_error(\1, None::<String>)'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_validation_error_calls(content):
    """Fix validation_error calls with incorrect signature"""
    # Replace validation_error with config_error (which exists)
    pattern = r'SongbirdError::validation_error\(\s*"([^"]*)"[^)]*\)'
    replacement = r'SongbirdError::config_error("\1", None::<String>)'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_config_error_method_calls(content):
    """Fix or_config_error method calls with too many arguments"""
    # Pattern: .or_config_error("field", "message") -> .or_config_error("message")
    pattern = r'\.or_config_error\(\s*"[^"]*",\s*("([^"]*)"\s*)\)'
    replacement = r'.or_config_error(\1)'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_field_access_issues(content):
    """Fix field access issues (.data on wrong types)"""
    # Remove .data access on strings and simple types
    content = re.sub(r'([a-zA-Z_][a-zA-Z0-9_]*)\.data(?=\s*[;})\]])', r'\1', content)
    
    # Fix capability_registry access (make it public or use getter)
    content = re.sub(r'\.capability_registry\.read\(\)\.await', '.get_registry().await', content)
    
    return content

def fix_response_error_calls(content):
    """Fix SongbirdResponse::error calls with wrong arguments"""
    # Pattern: SongbirdResponse::error(data, error) -> SongbirdResponse::error(error)
    pattern = r'SongbirdResponse::error\(\s*[^,]+,\s*([^)]+)\)'
    replacement = r'SongbirdResponse::error(\1)'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_universal_request_fields(content):
    """Fix UniversalRequest struct initialization with missing fields"""
    # Add missing operation and payload fields
    pattern = r'(UniversalRequest\s*\{\s*request_id:\s*[^,]+,\s*source_primal_id:\s*[^,]+,\s*target_capability:\s*[^,]+,\s*timeout_ms:\s*[^,]+,\s*requires_response:\s*[^,]+,)\s*\}'
    replacement = r'\1\n            operation: "generic".to_string(),\n            payload: serde_json::json!({}),\n        }'
    content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_iterator_issues(content):
    """Fix iterator and type issues"""
    # Fix for capability in capabilities.data -> for capability in capabilities
    content = re.sub(r'for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+([a-zA-Z_][a-zA-Z0-9_]*)\.data', r'for \1 in \2.iter()', content)
    
    # Fix string to String conversion in HashMap inserts
    pattern = r'\.insert\(\s*([a-zA-Z_][a-zA-Z0-9_]*),\s*([^)]+)\)'
    replacement = r'.insert(\1.to_string(), \2)'
    content = re.sub(pattern, replacement, content)
    
    return content

def fix_map_err_extra_arguments(content):
    """Fix map_err calls with extra arguments"""
    # Pattern: .map_err(|e| error, extra_arg) -> .map_err(|e| error)
    pattern = r'\.map_err\(([^)]+)\),\s*[^)]+\)\?'
    replacement = r'.map_err(\1)?'
    content = re.sub(pattern, replacement, content)
    
    return content

def process_file(file_path):
    """Process a single file to fix interface alignment issues"""
    print(f"Processing {file_path}")
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"  ❌ Error reading {file_path}: {e}")
        return False
    
    original_content = content
    
    # Apply all fixes
    content = fix_discovery_config_fields(content)
    content = fix_service_error_arguments(content)
    content = fix_network_error_arguments(content)
    content = fix_validation_error_calls(content)
    content = fix_config_error_method_calls(content)
    content = fix_field_access_issues(content)
    content = fix_response_error_calls(content)
    content = fix_universal_request_fields(content)
    content = fix_iterator_issues(content)
    content = fix_map_err_extra_arguments(content)
    
    if content != original_content:
        try:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"  ✅ Fixed interface alignment issues in {file_path}")
            return True
        except Exception as e:
            print(f"  ❌ Error writing {file_path}: {e}")
            return False
    else:
        print(f"  ⏭️  No changes needed in {file_path}")
        return False

def main():
    """Main function to complete interface alignment"""
    print("🔧 Completing interface alignment for production readiness...")
    
    # Files to process for interface alignment
    target_files = [
        "crates/songbird-universal/src/adapters/security.rs",
        "crates/songbird-universal/src/adapters/types/enums.rs", 
        "crates/songbird-universal/src/adapters/types/requests.rs",
        "crates/songbird-universal/src/adapters/compute.rs",
        "crates/songbird-universal/src/adapters/primal_integration.rs",
        "crates/songbird-universal/src/agnostic_adapter.rs",
        "crates/songbird-universal/src/adapters/storage.rs",
        "crates/songbird-universal/src/capabilities.rs",
        "crates/songbird-universal/src/self_discovery.rs",
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
    
    print(f"\n📊 Interface Alignment Summary:")
    print(f"  Files processed: {total_count}")
    print(f"  Files improved: {fixed_count}")
    print(f"  Success rate: {(fixed_count/total_count*100):.1f}%")
    
    print("\n🎯 Interface alignment improvements complete!")
    print("🚀 Ready for final compilation validation...")

if __name__ == "__main__":
    main() 