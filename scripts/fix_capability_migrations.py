#!/usr/bin/env python3
"""
Capability Migration Fixer

Automatically fixes common capability field name issues after type consolidation.
This handles the systematic patterns identified in compilation errors.
"""

import os
import re
import sys
from pathlib import Path

def fix_capability_fields(content: str) -> str:
    """Fix common capability field name issues"""
    
    # Fix Storage capabilities - add missing persistence_levels
    storage_pattern = r'PrimalCapability::Storage\s*{\s*types:\s*([^}]+)\s*}'
    def storage_replacement(match):
        types_content = match.group(1)
        return f'PrimalCapability::Storage {{ types: {types_content}, persistence_levels: vec!["persistent".to_string()] }}'
    content = re.sub(storage_pattern, storage_replacement, content)
    
    # Fix Compute capabilities - replace 'types' with 'runtimes' and add resource_limits
    compute_pattern = r'PrimalCapability::Compute\s*{\s*types:\s*([^}]+)\s*}'
    def compute_replacement(match):
        types_content = match.group(1)
        return f'PrimalCapability::Compute {{ runtimes: {types_content}, resource_limits: HashMap::new() }}'
    content = re.sub(compute_pattern, compute_replacement, content)
    
    # Fix AI capabilities - add missing inference_types
    ai_pattern = r'PrimalCapability::AI\s*{\s*models:\s*([^}]+)\s*}'
    def ai_replacement(match):
        models_content = match.group(1)
        return f'PrimalCapability::AI {{ models: {models_content}, inference_types: vec!["standard".to_string()] }}'
    content = re.sub(ai_pattern, ai_replacement, content)
    
    # Fix AI capabilities without models field
    ai_empty_pattern = r'PrimalCapability::AI\s*{\s*}'
    content = re.sub(ai_empty_pattern, 'PrimalCapability::AI { models: vec![], inference_types: vec!["standard".to_string()] }', content)
    
    # Fix Authentication capabilities - replace 'methods' with 'providers'
    auth_pattern = r'PrimalCapability::Authentication\s*{\s*methods:\s*([^}]+)\s*}'
    def auth_replacement(match):
        methods_content = match.group(1)
        return f'PrimalCapability::Authentication {{ providers: {methods_content} }}'
    content = re.sub(auth_pattern, auth_replacement, content)
    
    # Fix ServiceDiscovery capabilities - add missing features
    service_discovery_pattern = r'PrimalCapability::ServiceDiscovery\s*{\s*protocols:\s*([^}]+)\s*}'
    def service_discovery_replacement(match):
        protocols_content = match.group(1)
        return f'PrimalCapability::ServiceDiscovery {{ protocols: {protocols_content}, features: vec!["discovery".to_string()] }}'
    content = re.sub(service_discovery_pattern, service_discovery_replacement, content)
    
    # Fix ServiceDiscovery without protocols
    service_discovery_empty_pattern = r'PrimalCapability::ServiceDiscovery\s*{\s*}'
    content = re.sub(service_discovery_empty_pattern, 'PrimalCapability::ServiceDiscovery { protocols: vec!["http".to_string()], features: vec!["discovery".to_string()] }', content)
    
    # Fix ThreatDetection capabilities - replace ml_enabled with detection_types
    threat_pattern = r'PrimalCapability::ThreatDetection\s*{\s*ml_enabled:\s*([^}]+)\s*}'
    def threat_replacement(match):
        ml_enabled = match.group(1).strip()
        detection_types = '["ml_detection".to_string()]' if 'true' in ml_enabled else '["basic_detection".to_string()]'
        return f'PrimalCapability::ThreatDetection {{ detection_types: vec!{detection_types} }}'
    content = re.sub(threat_pattern, threat_replacement, content)
    
    # Fix KeyManagement capabilities - replace hsm_support with key_types
    key_mgmt_pattern = r'PrimalCapability::KeyManagement\s*{\s*hsm_support:\s*([^}]+)\s*}'
    def key_mgmt_replacement(match):
        hsm_support = match.group(1).strip()
        key_types = '["hsm".to_string(), "software".to_string()]' if 'true' in hsm_support else '["software".to_string()]'
        return f'PrimalCapability::KeyManagement {{ key_types: vec!{key_types} }}'
    content = re.sub(key_mgmt_pattern, key_mgmt_replacement, content)
    
    # Fix GpuAcceleration capabilities - replace cuda_support with gpu_types
    gpu_pattern = r'PrimalCapability::GpuAcceleration\s*{\s*cuda_support:\s*([^}]+)\s*}'
    def gpu_replacement(match):
        cuda_support = match.group(1).strip()
        gpu_types = '["cuda".to_string(), "opencl".to_string()]' if 'true' in cuda_support else '["opencl".to_string()]'
        return f'PrimalCapability::GpuAcceleration {{ gpu_types: vec!{gpu_types} }}'
    content = re.sub(gpu_pattern, gpu_replacement, content)
    
    # Fix NetworkRouting capabilities - add missing routing_types
    network_routing_pattern = r'PrimalCapability::NetworkRouting\s*{\s*protocols:\s*([^}]+)\s*}'
    def network_routing_replacement(match):
        protocols_content = match.group(1)
        return f'PrimalCapability::NetworkRouting {{ protocols: {protocols_content}, routing_types: vec!["static".to_string()] }}'
    content = re.sub(network_routing_pattern, network_routing_replacement, content)
    
    # Fix MachineLearning capabilities - replace training_support with frameworks
    ml_pattern = r'PrimalCapability::MachineLearning\s*{\s*training_support:\s*([^}]+)\s*}'
    def ml_replacement(match):
        training_support = match.group(1).strip()
        frameworks = '["tensorflow".to_string(), "pytorch".to_string()]' if 'true' in training_support else '["inference_only".to_string()]'
        return f'PrimalCapability::MachineLearning {{ frameworks: vec!{frameworks} }}'
    content = re.sub(ml_pattern, ml_replacement, content)
    
    return content

def fix_pattern_matches(content: str) -> str:
    """Fix pattern matching issues"""
    
    # Fix Storage pattern matches to ignore extra fields
    storage_match_pattern = r'PrimalCapability::Storage\s*{\s*types:\s*vec!\[[^\]]+\]\s*}'
    content = re.sub(storage_match_pattern, 'PrimalCapability::Storage { .. }', content)
    
    # Fix vec![] in match patterns (not allowed)
    vec_in_pattern = r'vec!\[[^\]]+\](?=\s*[,}])'
    content = re.sub(vec_in_pattern, '_', content)
    
    return content

def fix_property_types(content: str) -> str:
    """Fix Vec vs HashMap type mismatches"""
    
    # Fix properties: vec![] to HashMap::new()
    properties_vec_pattern = r'properties:\s*vec!\[\]'
    content = re.sub(properties_vec_pattern, 'properties: HashMap::new()', content)
    
    # Fix properties: vec![(...)] to HashMap construction
    properties_vec_tuples_pattern = r'properties:\s*vec!\[([^\]]+)\]'
    def properties_replacement(match):
        tuples_content = match.group(1)
        # Convert vec of tuples to HashMap construction
        return f'properties: HashMap::from([{tuples_content}])'
    content = re.sub(properties_vec_tuples_pattern, properties_replacement, content)
    
    return content

def fix_missing_variants(content: str) -> str:
    """Fix missing capability variants"""
    
    # Replace Manifests with Configuration
    content = re.sub(r'PrimalCapability::Manifests', 'PrimalCapability::Configuration', content)
    
    # Replace ProxyServices with Network
    content = re.sub(r'PrimalCapability::ProxyServices', 'PrimalCapability::Network', content)
    
    # Replace VpnServices with Network  
    content = re.sub(r'PrimalCapability::VpnServices', 'PrimalCapability::Network', content)
    
    return content

def add_required_imports(content: str) -> str:
    """Add required imports for HashMap"""
    
    if 'HashMap::' in content and 'use std::collections::HashMap;' not in content:
        # Add HashMap import after existing use statements
        use_pattern = r'(use [^;]+;(?:\n|$))'
        matches = list(re.finditer(use_pattern, content))
        if matches:
            # Insert after the last use statement
            last_use = matches[-1]
            insert_pos = last_use.end()
            content = content[:insert_pos] + 'use std::collections::HashMap;\n' + content[insert_pos:]
        else:
            # Add at the beginning if no use statements found
            content = 'use std::collections::HashMap;\n' + content
    
    return content

def process_file(file_path: Path) -> bool:
    """Process a single Rust file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_capability_fields(content)
        content = fix_pattern_matches(content)
        content = fix_property_types(content)
        content = fix_missing_variants(content)
        content = add_required_imports(content)
        
        # Only write if content changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Fixed: {file_path}")
            return True
        else:
            print(f"⏭️  Skipped: {file_path} (no changes needed)")
            return False
            
    except Exception as e:
        print(f"❌ Error processing {file_path}: {e}")
        return False

def main():
    """Main migration function"""
    print("🔧 Starting capability field migration...")
    
    # Get the current directory (should be songbird-universal-primals)
    current_dir = Path.cwd()
    if not current_dir.name.endswith('songbird-universal-primals'):
        # Try to find the correct directory
        songbird_root = current_dir
        while songbird_root.name != 'songbird' and songbird_root.parent != songbird_root:
            songbird_root = songbird_root.parent
        
        if songbird_root.name == 'songbird':
            current_dir = songbird_root / 'crates' / 'songbird-universal-primals'
        else:
            print("❌ Could not find songbird-universal-primals directory")
            return
    
    if not current_dir.exists():
        print(f"❌ Directory does not exist: {current_dir}")
        return
    
    # Find all Rust files
    rust_files = list(current_dir.rglob('*.rs'))
    if not rust_files:
        print("❌ No Rust files found")
        return
    
    print(f"📁 Found {len(rust_files)} Rust files to process")
    
    # Process each file
    changed_files = 0
    for file_path in rust_files:
        if process_file(file_path):
            changed_files += 1
    
    print(f"\n🎉 Migration complete!")
    print(f"📊 Files processed: {len(rust_files)}")
    print(f"✏️  Files changed: {changed_files}")
    print(f"⏭️  Files skipped: {len(rust_files) - changed_files}")

if __name__ == "__main__":
    main() 