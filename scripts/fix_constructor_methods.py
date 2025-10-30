#!/usr/bin/env python3
"""
Constructor Method Parameter Fix Script

This script adds missing parameters to constructor and builder methods
that reference variables not in scope.
"""

import re
from pathlib import Path

def fix_constructor_signatures(content: str) -> str:
    """Fix constructor method signatures that need parameters"""
    
    # Fix new methods that reference parameters they don't have
    fixes = [
        # CanonicalPrimalId::new needs parameters
        (r'impl CanonicalPrimalId \{[^}]*pub fn new\([^)]*\) -> Self \{[^}]*primal_type,[^}]*instance_id: instance_id\.into\(\),[^}]*version: version\.into\(\),',
         lambda m: m.group(0).replace('pub fn new(', 'pub fn new(primal_type: CanonicalPrimalType, instance_id: impl Into<String>, version: impl Into<String>') if 'pub fn new(' in m.group(0) else m.group(0)),
        
        # CanonicalServiceInfo::new needs parameters  
        (r'impl CanonicalServiceInfo \{[^}]*pub fn new\([^)]*\) -> Self \{[^}]*name: name\.into\(\),[^}]*version: version\.into\(\),',
         lambda m: m.group(0).replace('pub fn new(', 'pub fn new(name: impl Into<String>, version: impl Into<String>') if 'pub fn new(' in m.group(0) else m.group(0)),
        
        # CanonicalRequest::new needs parameters
        (r'impl CanonicalRequest \{[^}]*pub fn new\([^)]*\) -> Self \{[^}]*operation: operation\.into\(\),[^}]*payload,',
         lambda m: m.group(0).replace('pub fn new(', 'pub fn new(operation: impl Into<String>, payload: serde_json::Value') if 'pub fn new(' in m.group(0) else m.group(0)),
        
        # CanonicalResponse::success needs parameters
        (r'impl CanonicalResponse \{[^}]*pub fn success<T>\(data: T\) -> Self \{[^}]*request_id: request_id\.into\(\),',
         lambda m: m.group(0).replace('pub fn success<T>(data: T)', 'pub fn success<T>(request_id: impl Into<String>, data: T') if 'pub fn success<T>(data: T)' in m.group(0) else m.group(0)),
        
        # AIFirstResponse::new needs parameters  
        (r'impl<T> AIFirstResponse<T> \{[^}]*pub fn new\([^)]*\) -> Self \{[^}]*data,',
         lambda m: m.group(0).replace('pub fn new(', 'pub fn new(data: T') if 'pub fn new(' in m.group(0) else m.group(0)),
        
        # PaginatedResponse::new needs parameters
        (r'impl<T> PaginatedResponse<T> \{[^}]*pub fn new\([^)]*\) -> Self \{[^}]*items,[^}]*page,[^}]*per_page,[^}]*total,',
         lambda m: m.group(0).replace('pub fn new(', 'pub fn new(items: Vec<T>, page: usize, per_page: usize, total: usize') if 'pub fn new(' in m.group(0) else m.group(0)),
    ]
    
    for pattern, replacement in fixes:
        if callable(replacement):
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
        else:
            content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_builder_method_signatures(content: str) -> str:
    """Fix builder method signatures that need parameters"""
    
    # Add parameters to with_* methods that reference them
    builder_fixes = [
        (r'pub fn (with_endpoint)\(&mut self\) -> &mut Self \{[^}]*self\.endpoints\.insert\(name\.into\(\), url\.into\(\)\);',
         r'pub fn \1(&mut self, name: impl Into<String>, url: impl Into<String>) -> &mut Self {\n        self.endpoints.insert(name.into(), url.into());'),
        
        (r'pub fn (with_metadata)\(&mut self\) -> &mut Self \{[^}]*self\.\w+\.insert\(key\.into\(\), value\.into\(\)\);',
         r'pub fn \1(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {\n        self.metadata.insert(key.into(), value.into());'),
         
        (r'pub fn (with_capability)\(&mut self\) -> &mut Self \{[^}]*self\.capabilities\.push\(capability\.into\(\)\);',
         r'pub fn \1(&mut self, capability: impl Into<String>) -> &mut Self {\n        self.capabilities.push(capability.into());'),
         
        (r'pub fn (with_dependency)\(&mut self\) -> &mut Self \{[^}]*self\.dependencies\.push\(dependency\.into\(\)\);',
         r'pub fn \1(&mut self, dependency: impl Into<String>) -> &mut Self {\n        self.dependencies.push(dependency.into());'),
         
        (r'pub fn (with_description)\(&mut self\) -> &mut Self \{[^}]*self\.description = Some\(description\.into\(\)\);',
         r'pub fn \1(&mut self, description: impl Into<String>) -> &mut Self {\n        self.description = Some(description.into());'),
         
        (r'pub fn (with_path)\(&mut self\) -> &mut Self \{[^}]*self\.path = Some\(path\.into\(\)\);',
         r'pub fn \1(&mut self, path: impl Into<String>) -> &mut Self {\n        self.path = Some(path.into());'),
         
        (r'pub fn (with_type)\(&mut self\) -> &mut Self \{[^}]*self\.addr_type = Some\(addr_type\.into\(\)\);',
         r'pub fn \1(&mut self, addr_type: impl Into<String>) -> &mut Self {\n        self.addr_type = Some(addr_type.into());'),
         
        (r'pub fn (with_city)\(&mut self\) -> &mut Self \{[^}]*self\.city = Some\(city\.into\(\)\);',
         r'pub fn \1(&mut self, city: impl Into<String>) -> &mut Self {\n        self.city = Some(city.into());'),
         
        (r'pub fn (with_country)\(&mut self\) -> &mut Self \{[^}]*self\.country = Some\(country\.into\(\)\);',
         r'pub fn \1(&mut self, country: impl Into<String>) -> &mut Self {\n        self.country = Some(country.into());'),
         
        (r'pub fn (with_context)\(&mut self\) -> &mut Self \{[^}]*self\.context = Some\(context\.into\(\)\);',
         r'pub fn \1(&mut self, context: impl Into<String>) -> &mut Self {\n        self.context = Some(context.into());'),
         
        (r'pub fn (with_confidence)\(&mut self\) -> &mut Self \{[^}]*self\.confidence = Some\(confidence\.clamp\(0\.0, 1\.0\)\);',
         r'pub fn \1(&mut self, confidence: f64) -> &mut Self {\n        self.confidence = Some(confidence.clamp(0.0, 1.0));'),
         
        (r'pub fn (with_action)\(&mut self\) -> &mut Self \{[^}]*self\.suggested_actions\.push\(action\.into\(\)\);',
         r'pub fn \1(&mut self, action: impl Into<String>) -> &mut Self {\n        self.suggested_actions.push(action.into());'),
         
        (r'pub fn (with_security_level)\(&mut self\) -> &mut Self \{[^}]*self\.security_level = Some\(level\.into\(\)\);',
         r'pub fn \1(&mut self, level: impl Into<String>) -> &mut Self {\n        self.security_level = Some(level.into());'),
         
        (r'pub fn (with_config)\(&mut self\) -> &mut Self \{[^}]*self\.config\.insert\(key\.into\(\), value\.into\(\)\);',
         r'pub fn \1(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {\n        self.config.insert(key.into(), value.into());'),
    ]
    
    for pattern, replacement in builder_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_error_method_signatures(content: str) -> str:
    """Fix error method signatures that need parameters"""
    
    error_fixes = [
        # Fix error methods that reference parameters
        (r'pub fn error\(request_id: impl Into<String>, error: impl Into<String>\) -> Self \{[^}]*code: code\.into\(\),[^}]*message: message\.into\(\),',
         r'pub fn error(request_id: impl Into<String>, error: impl Into<String>) -> Self {\n            success: false,\n            data: None,\n            error: Some(ResponseError {\n                code: "ERROR".to_string(),\n                message: error.into(),'),
    ]
    
    for pattern, replacement in error_fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    return content

def fix_file(file_path: Path) -> bool:
    """Fix constructor and builder methods in a file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return False
        
    original_content = content
    
    # Apply fixes
    content = fix_constructor_signatures(content)
    content = fix_builder_method_signatures(content)
    content = fix_error_method_signatures(content)
    
    # Write back if changes were made
    if content != original_content:
        try:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        except Exception as e:
            print(f"Error writing {file_path}: {e}")
            return False
            
    return False

def main():
    """Fix constructor methods in songbird-types crate"""
    types_dir = Path('crates/songbird-types')
    
    if not types_dir.exists():
        print("Error: songbird-types directory not found")
        return
    
    rust_files = list(types_dir.rglob('*.rs'))
    fixed_count = 0
    
    print(f"🔧 Fixing constructor methods in {len(rust_files)} Rust files")
    
    for rust_file in rust_files:
        if fix_file(rust_file):
            fixed_count += 1
            print(f"✅ Fixed: {rust_file.relative_to(types_dir)}")
    
    print(f"\n📊 Fixed {fixed_count} files")

if __name__ == '__main__':
    main() 