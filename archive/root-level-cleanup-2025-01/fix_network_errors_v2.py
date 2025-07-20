#!/usr/bin/env python3

import re
import os
import glob

def fix_network_errors():
    # Find all .rs files in the network crate
    files = glob.glob("crates/songbird-network/src/**/*.rs", recursive=True)
    
    for file_path in files:
        print(f"Processing {file_path}")
        
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Fix Network errors - all patterns
        # Pattern 1: .map_err(|e| SongbirdError::Network { ... })?
        pattern1 = r'\.map_err\(\|e\| SongbirdError::Network \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)\?'
        def replacement1(match):
            inner = match.group(1)
            return f'.map_err(|e| SongbirdError::Network(Box::new(NetworkError {{{inner}}})))?'
        content = re.sub(pattern1, replacement1, content, flags=re.DOTALL)
        
        # Pattern 2: return Err(SongbirdError::Network { ... });
        pattern2 = r'return Err\(SongbirdError::Network \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)\;'
        def replacement2(match):
            inner = match.group(1)
            return f'return Err(SongbirdError::Network(Box::new(NetworkError {{{inner}}})));'
        content = re.sub(pattern2, replacement2, content, flags=re.DOTALL)
        
        # Pattern 3: .ok_or_else(|| SongbirdError::Network { ... })?
        pattern3 = r'\.ok_or_else\(\|\| SongbirdError::Network \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)\?'
        def replacement3(match):
            inner = match.group(1)
            return f'.ok_or_else(|| SongbirdError::Network(Box::new(NetworkError {{{inner}}})))?'
        content = re.sub(pattern3, replacement3, content, flags=re.DOTALL)
        
        # Pattern 4: SongbirdError::Network { ... } (standalone)
        pattern4 = r'SongbirdError::Network \{([^}]*(?:\{[^}]*\}[^}]*)*)\}'
        def replacement4(match):
            inner = match.group(1)
            return f'SongbirdError::Network(Box::new(NetworkError {{{inner}}}))'
        content = re.sub(pattern4, replacement4, content, flags=re.DOTALL)
        
        # Fix Protocol errors - all patterns
        # Pattern 5: .ok_or_else(|| SongbirdError::Protocol { ... })?
        pattern5 = r'\.ok_or_else\(\|\| SongbirdError::Protocol \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)\?'
        def replacement5(match):
            inner = match.group(1)
            return f'.ok_or_else(|| SongbirdError::Protocol(Box::new(ProtocolError {{{inner}}})))?'
        content = re.sub(pattern5, replacement5, content, flags=re.DOTALL)
        
        # Pattern 6: Err(SongbirdError::Protocol { ... }),
        pattern6 = r'Err\(SongbirdError::Protocol \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\),'
        def replacement6(match):
            inner = match.group(1)
            return f'Err(SongbirdError::Protocol(Box::new(ProtocolError {{{inner}}}))),'
        content = re.sub(pattern6, replacement6, content, flags=re.DOTALL)
        
        # Pattern 7: Err(SongbirdError::Protocol { ... })
        pattern7 = r'Err\(SongbirdError::Protocol \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)'
        def replacement7(match):
            inner = match.group(1)
            return f'Err(SongbirdError::Protocol(Box::new(ProtocolError {{{inner}}})))'
        content = re.sub(pattern7, replacement7, content, flags=re.DOTALL)
        
        # Pattern 8: return Err(SongbirdError::Protocol { ... });
        pattern8 = r'return Err\(SongbirdError::Protocol \{([^}]*(?:\{[^}]*\}[^}]*)*)\}\)\;'
        def replacement8(match):
            inner = match.group(1)
            return f'return Err(SongbirdError::Protocol(Box::new(ProtocolError {{{inner}}})));'
        content = re.sub(pattern8, replacement8, content, flags=re.DOTALL)
        
        # Add imports if needed
        if 'ProtocolError' in content and 'use songbird_errors::' in content:
            if 'ProtocolError' not in content.split('use songbird_errors::')[1].split(';')[0]:
                content = content.replace('use songbird_errors::{', 'use songbird_errors::{ProtocolError, ')
        if 'NetworkError' in content and 'use songbird_errors::' in content:
            if 'NetworkError' not in content.split('use songbird_errors::')[1].split(';')[0]:
                content = content.replace('use songbird_errors::{', 'use songbird_errors::{NetworkError, ')
        
        with open(file_path, 'w') as f:
            f.write(content)
    
    print("Fixed network errors")

if __name__ == "__main__":
    fix_network_errors() 