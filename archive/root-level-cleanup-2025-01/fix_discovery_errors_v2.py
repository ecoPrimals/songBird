#!/usr/bin/env python3

import re

def fix_discovery_errors():
    file_path = "crates/songbird-discovery/src/discovery/mod.rs"
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to match SongbirdError::Discovery { ... } constructions
    pattern = r'SongbirdError::Discovery\s*\{([^}]+)\}'
    
    def replacement(match):
        inner_content = match.group(1)
        return f'SongbirdError::Discovery(Box::new(DiscoveryError {{{inner_content}}})'
    
    # Replace all matches
    new_content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    
    # Fix any remaining closing issues
    new_content = new_content.replace('})?;', '})))?;')
    new_content = new_content.replace('});', '})));')
    
    with open(file_path, 'w') as f:
        f.write(new_content)
    
    print("Fixed discovery errors")

if __name__ == "__main__":
    fix_discovery_errors() 