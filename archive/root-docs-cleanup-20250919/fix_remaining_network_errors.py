#!/usr/bin/env python3
"""
Fix the remaining 19 network module compilation errors.
"""

import os
import re

def fix_remaining_errors():
    """Fix the remaining error patterns."""
    
    rust_files = []
    for root, dirs, files in os.walk("crates/songbird-network"):
        if "target" in root:
            continue
        for file in files:
            if file.endswith(".rs"):
                rust_files.append(os.path.join(root, file))
    
    fixes_made = 0
    
    for file_path in rust_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            
            # Fix Result import - use SongbirdResult instead
            content = re.sub(
                r'use songbird_errors::\{Result, SongbirdError\};',
                r'use songbird_errors::{SongbirdResult as Result, SongbirdError};',
                content
            )
            content = re.sub(
                r'use songbird_errors::Result;',
                r'use songbird_errors::SongbirdResult as Result;',
                content
            )
            
            # Fix NetworkError import
            content = re.sub(
                r'use songbird_errors::\{NetworkError, Result, SongbirdError\};',
                r'use songbird_errors::{SongbirdResult as Result, SongbirdError};',
                content
            )
            
            # Add DiscoveredPeer import where needed
            if 'DiscoveredPeer' in content and 'use crate::network::discovery::DiscoveredPeer;' not in content:
                # Find the imports section and add the import
                import_pattern = r'(use [^;]+;[\s\n]*)*'
                if re.search(r'use.*discovery.*', content):
                    content = re.sub(
                        r'(use.*discovery[^;]*;)',
                        r'\1\nuse crate::network::discovery::DiscoveredPeer;',
                        content,
                        count=1
                    )
            
            # Fix remaining configuration errors with two parameters
            content = re.sub(
                r'SongbirdError::configuration\("([^"]+)",\s*"([^"]+)"\)',
                r'SongbirdError::configuration(format!("{}: {}", "\1", "\2"))',
                content
            )
            
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                fixes_made += 1
                print(f"✅ Fixed {file_path}")
        
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
    
    print(f"\n🎯 Fixed {fixes_made} files")
    return fixes_made

if __name__ == "__main__":
    fixes_made = fix_remaining_errors()
    print(f"Remaining error fixing complete. Fixed {fixes_made} files.")
