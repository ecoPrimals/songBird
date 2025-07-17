#!/usr/bin/env python3

import re

def fix_discovery_errors():
    file_path = "crates/songbird-discovery/src/discovery/mod.rs"
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # First, let's fix the basic pattern: SongbirdError::Discovery { -> SongbirdError::Discovery(Box::new(DiscoveryError {
    content = content.replace('SongbirdError::Discovery {', 'SongbirdError::Discovery(Box::new(DiscoveryError {')
    
    # Now fix the closing patterns more carefully
    # For map_err patterns: })?; -> })))?;
    content = re.sub(r'(\s+)}\)\?\;', r'\1})))?;', content)
    
    # For return Err patterns: }); -> })));
    content = re.sub(r'(\s+)}\)\;', r'\1})));', content)
    
    with open(file_path, 'w') as f:
        f.write(content)
    
    print("Fixed discovery errors")

if __name__ == "__main__":
    fix_discovery_errors() 