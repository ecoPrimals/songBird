#!/usr/bin/env python3
"""
Modernize Network Error API Usage

Systematically converts old-style error constructions to unified canonical patterns:
- NetworkError(Box::new(...)) → Network { message, operation, suggestion }
- Protocol(Box::new(...)) → Communication(message)
- Remove deprecated NetworkError imports
"""

import re
import sys
from pathlib import Path

def modernize_network_error_construction(content: str) -> tuple[str, int]:
    """Convert NetworkError box patterns to modern structure"""
    changes = 0
    
    # Pattern 1: SongbirdError::Network(Box::new(NetworkError { ... }))
    pattern1 = r'(songbird_errors::)?SongbirdError::Network\(Box::new\((songbird_errors::)?NetworkError\s*\{\s*message:\s*([^,]+),\s*endpoint:\s*[^,]+,\s*port:\s*[^,]+,\s*protocol:\s*[^}]+\}\)\)'
    
    def replace_network(match):
        nonlocal changes
        changes += 1
        message = match.group(3)
        return f'''songbird_errors::SongbirdError::Network {{
                message: {message},
                operation: Some("network_operation".to_string()),
                suggestion: Some("Check network configuration".to_string()),
            }}'''
    
    content = re.sub(pattern1, replace_network, content, flags=re.DOTALL)
    
    # Pattern 2: Simpler Network error patterns
    pattern2 = r'(songbird_errors::)?SongbirdError::Network\(Box::new\(\s*NetworkError\s*\{[^}]+\}\s*\)\)'
    
    def replace_simple_network(match):
        nonlocal changes
        changes += 1
        return '''songbird_errors::SongbirdError::Network {
                message: "Network error".to_string(),
                operation: None,
                suggestion: None,
            }'''
    
    content = re.sub(pattern2, replace_simple_network, content, flags=re.DOTALL)
    
    return content, changes

def modernize_protocol_error_construction(content: str) -> tuple[str, int]:
    """Convert Protocol box patterns to Communication variant"""
    changes = 0
    
    # Pattern: SongbirdError::Protocol(Box::new(ProtocolError { ... }))
    pattern = r'(songbird_errors::)?SongbirdError::Protocol\(Box::new\((songbird_errors::)?ProtocolError\s*\{[^}]+\}\)\)'
    
    def replace_protocol(match):
        nonlocal changes
        changes += 1
        return 'songbird_errors::SongbirdError::Communication("Protocol error".to_string())'
    
    content = re.sub(pattern, replace_protocol, content, flags=re.DOTALL)
    
    return content, changes

def modernize_imports(content: str) -> tuple[str, int]:
    """Remove deprecated NetworkError and ProtocolError imports"""
    changes = 0
    
    # Pattern: use songbird_errors::{NetworkError, ...};
    # Replace with: use songbird_errors::{Result, SongbirdError};
    
    patterns = [
        (r'use songbird_errors::\{NetworkError,\s*Result,\s*SongbirdError\};', 
         'use songbird_errors::{Result, SongbirdError};'),
        (r'use songbird_errors::\{NetworkError,\s*ProtocolError,\s*Result,\s*SongbirdError\};',
         'use songbird_errors::{Result, SongbirdError};'),
        (r'use songbird_errors::\{NetworkError,\s*Result\};',
         'use songbird_errors::{Result, SongbirdError};'),
        (r'use songbird_errors::\{NetworkError,\s*SongbirdError\};',
         'use songbird_errors::{Result, SongbirdError};'),
        (r'use songbird_errors::\{ProtocolError,\s*Result,\s*SongbirdError\};',
         'use songbird_errors::{Result, SongbirdError};'),
    ]
    
    for pattern, replacement in patterns:
        if re.search(pattern, content):
            content = re.sub(pattern, replacement, content)
            changes += 1
    
    return content, changes

def modernize_file(file_path: Path) -> tuple[int, str]:
    """Modernize a single Rust file"""
    try:
        content = file_path.read_text()
        original_content = content
        total_changes = 0
        
        # Apply modernization transformations
        content, changes = modernize_imports(content)
        total_changes += changes
        
        content, changes = modernize_network_error_construction(content)
        total_changes += changes
        
        content, changes = modernize_protocol_error_construction(content)
        total_changes += changes
        
        # Only write if changes were made
        if total_changes > 0:
            file_path.write_text(content)
            return total_changes, f"✅ {file_path.relative_to(Path.cwd())}: {total_changes} changes"
        else:
            return 0, f"⏭️  {file_path.relative_to(Path.cwd())}: No changes needed"
            
    except Exception as e:
        return 0, f"❌ {file_path}: Error - {e}"

def main():
    """Main modernization process"""
    print("🔧 Network Error API Modernization")
    print("=" * 60)
    
    # Target directory
    network_crate = Path("crates/songbird-network/src")
    
    if not network_crate.exists():
        print(f"❌ Directory not found: {network_crate}")
        return 1
    
    # Find all Rust files
    rust_files = list(network_crate.rglob("*.rs"))
    print(f"📁 Found {len(rust_files)} Rust files in songbird-network\n")
    
    total_changes = 0
    results = []
    
    # Process each file
    for file_path in rust_files:
        changes, message = modernize_file(file_path)
        total_changes += changes
        results.append(message)
    
    # Print results
    print("\n📊 Modernization Results:")
    print("-" * 60)
    for result in results:
        if result.startswith("✅"):
            print(result)
    
    print("\n" + "=" * 60)
    print(f"✨ Total changes: {total_changes}")
    print(f"📝 Files processed: {len(rust_files)}")
    print(f"✅ Files modified: {sum(1 for r in results if r.startswith('✅'))}")
    
    return 0

if __name__ == "__main__":
    sys.exit(main()) 