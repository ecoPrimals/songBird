#!/usr/bin/env python3
"""Final comprehensive fix for all syntax errors"""

import re
import subprocess
from pathlib import Path

def fix_all_files():
    # Run cargo check and parse errors
    result = subprocess.run(
        ['cargo', 'check', '--workspace'],
        capture_output=True,
        text=True,
        cwd='/home/eastgate/Development/ecoPrimals/songbird'
    )
    
    # Extract all files with errors
    error_files = set()
    for line in result.stderr.split('\n'):
        if '-->' in line:
            parts = line.split('-->')[1].strip().split(':')
            if parts:
                error_files.add(parts[0])
    
    print(f"Found {len(error_files)} files with errors")
    
    # Fix each file
    for filepath in error_files:
        filepath = Path('/home/eastgate/Development/ecoPrimals/songbird') / filepath
        if not filepath.exists():
            continue
            
        try:
            with open(filepath, 'r') as f:
                content = f.read()
            
            original = content
            
            # Fix all Arc::new(RwLock::new(X(), missing ))
            content = re.sub(
                r'Arc::new\(RwLock::new\(HashMap::new\(\)(?!\))',
                'Arc::new(RwLock::new(HashMap::new())',
                content
            )
            content = re.sub(
                r'Arc::new\(RwLock::new\(Vec::new\(\)(?!\))',
                'Arc::new(RwLock::new(Vec::new())',
                content
            )
            
            # Fix .method(args; -> .method(args);
            content = re.sub(
                r'= Some\(([^)]+\.into\(\))(?=;)',
                r'= Some(\1)',
                content
            )
            content = re.sub(
                r'= Some\(([^)]+\.to_string\(\))(?=;)',
                r'= Some(\1)',
                content
            )
            
            if content != original:
                with open(filepath, 'w') as f:
                    f.write(content)
                print(f"✓ Fixed {filepath.name}")
        except Exception as e:
            print(f"✗ Error fixing {filepath}: {e}")

if __name__ == "__main__":
    fix_all_files()

