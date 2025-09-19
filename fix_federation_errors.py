#!/usr/bin/env python3
"""
Script to fix federation crate compilation errors
"""

import os
import re

def fix_federation_errors(filepath):
    """Fix all federation errors in the protocol.rs file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Fix malformed format strings with missing closing braces
        content = re.sub(r'format!\("([^"]*\{[^}]*)\)',  r'format!("\1}")', content)
        content = re.sub(r'format!\("([^"]*\{[^}]*\{[^}]*)\)', r'format!("\1}}")', content)
        
        # Fix specific malformed format strings
        content = re.sub(r'format!\("([^"]*\{e)\)', r'format!("\1}", e)', content)
        content = re.sub(r'format!\("([^"]*\{)\)', r'format!("\1}")', content)
        
        # Remove extra struct fields that don't belong to the new error API
        content = re.sub(r',\s*endpoint:\s*Some\([^)]+\)\s*,\s*port:\s*None\s*,\s*protocol:\s*None\s*,?\s*}', '', content, flags=re.MULTILINE | re.DOTALL)
        
        # Fix Rust 2021 prefix errors by adding spaces
        content = re.sub(r'"([^"]*)/([a-z]+)"', r'"\1/\2 "', content)
        content = re.sub(r'"([^"]*[a-z]+)"\.to_string\(\)', lambda m: f'"{m.group(1)} ".to_string()', content)
        
        # Fix specific prefix issues
        content = content.replace('"federation/register"', '"federation/register "')
        content = content.replace('"federation/request"', '"federation/request "') 
        content = content.replace('"Request failed with status"', '"Request failed with status "')
        content = content.replace('"Request timestamp is too old"', '"Request timestamp is too old "')
        content = content.replace('"Use a more recent timestamp for the request"', '"Use a more recent timestamp for the request "')
        content = content.replace('"Service provider registered successfully"', '"Service provider registered successfully "')
        content = content.replace('capability-based metrics"', 'capability-based metrics "')
        content = content.replace('"/health"', '"/health "')
        content = content.replace('"Status update received"', '"Status update received "')
        
        # Fix remaining format string issues
        content = re.sub(r'SongbirdError::Communication\(format!\("([^"]+):\s*\{e\}"\)\)', 
                        r'songbird_errors::SongbirdError::network(format!("\1: {}", e))', content)
        
        if content != original_content:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed federation errors in {filepath}")
        else:
            print(f"No changes needed for {filepath}")
            
    except Exception as e:
        print(f"Error processing {filepath}: {e}")

if __name__ == "__main__":
    fix_federation_errors("crates/songbird-federation/src/mcp_handler/protocol.rs") 