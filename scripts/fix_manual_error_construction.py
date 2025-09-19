#!/usr/bin/env python3
"""
Songbird Network Manual Error Construction Fixer

This script fixes manual SongbirdError construction patterns that are missing required
fields by replacing them with proper constructor calls.

Patterns to fix:
- SongbirdError::Config { ... } -> SongbirdError::config_error(...)
- SongbirdError::Network { ... } -> SongbirdError::network_error(...)
- Other manual constructions missing required fields
"""

import os
import re
import sys
from pathlib import Path

def fix_config_error_constructions(content):
    """Fix manual Config error constructions"""
    # Pattern for Config errors with manual field construction
    pattern = r'SongbirdError::Config\s*\{[^}]*?message:\s*([^,}]+)[^}]*?field:\s*(Some\([^)]+\)|None)[^}]*?\}'
    
    def replacement_func(match):
        message = match.group(1).strip()
        field = match.group(2).strip()
        return f'SongbirdError::config_error({message}, {field})'
    
    content = re.sub(pattern, replacement_func, content, flags=re.DOTALL)
    
    # Also handle cases where field comes first
    pattern2 = r'SongbirdError::Config\s*\{[^}]*?field:\s*(Some\([^)]+\)|None)[^}]*?message:\s*([^,}]+)[^}]*?\}'
    
    def replacement_func2(match):
        field = match.group(1).strip()
        message = match.group(2).strip()
        return f'SongbirdError::config_error({message}, {field})'
    
    content = re.sub(pattern2, replacement_func2, content, flags=re.DOTALL)
    
    return content

def fix_network_error_constructions(content):
    """Fix manual Network error constructions"""
    # Pattern for Network errors with manual field construction
    pattern = r'SongbirdError::Network\s*\{[^}]*?message:\s*([^,}]+)[^}]*?endpoint:\s*(Some\([^)]+\)|None)[^}]*?\}'
    
    def replacement_func(match):
        message = match.group(1).strip()
        endpoint = match.group(2).strip()
        return f'SongbirdError::network_error({message}, {endpoint})'
    
    content = re.sub(pattern, replacement_func, content, flags=re.DOTALL)
    
    # Also handle cases where endpoint comes first
    pattern2 = r'SongbirdError::Network\s*\{[^}]*?endpoint:\s*(Some\([^)]+\)|None)[^}]*?message:\s*([^,}]+)[^}]*?\}'
    
    def replacement_func2(match):
        endpoint = match.group(1).strip()
        message = match.group(2).strip()
        return f'SongbirdError::network_error({message}, {endpoint})'
    
    content = re.sub(pattern2, replacement_func2, content, flags=re.DOTALL)
    
    return content

def fix_service_error_constructions(content):
    """Fix manual Service error constructions"""
    # Pattern for Service errors - these should use service_error constructor
    pattern = r'SongbirdError::Service\s*\{[^}]*?message:\s*([^,}]+)[^}]*?service:\s*([^,}]+)[^}]*?\}'
    
    def replacement_func(match):
        message = match.group(1).strip()
        service = match.group(2).strip()
        return f'SongbirdError::service_error({service}, {message})'
    
    content = re.sub(pattern, replacement_func, content, flags=re.DOTALL)
    
    return content

def fix_validation_error_constructions(content):
    """Fix manual Validation error constructions"""
    # Pattern for Validation errors
    pattern = r'SongbirdError::Validation\s*\{[^}]*?message:\s*([^,}]+)[^}]*?\}'
    
    def replacement_func(match):
        message = match.group(1).strip()
        return f'SongbirdError::validation_error({message})'
    
    content = re.sub(pattern, replacement_func, content, flags=re.DOTALL)
    
    return content

def fix_generic_missing_category(content):
    """Fix any remaining manual constructions missing category field"""
    # Find SongbirdError constructions that are missing category
    # Replace with internal_error as a safe fallback
    pattern = r'SongbirdError::(\w+)\s*\{[^}]*?message:\s*([^,}]+)[^}]*?\}'
    
    def replacement_func(match):
        variant = match.group(1)
        message = match.group(2).strip()
        
        # Skip if it's already a proper constructor call
        if variant in ['Config', 'Network', 'Service', 'Validation']:
            return match.group(0)  # Don't change, should be handled by specific functions
        
        # For unknown variants, use internal_error
        return f'SongbirdError::internal_error({message})'
    
    content = re.sub(pattern, replacement_func, content, flags=re.DOTALL)
    
    return content

def fix_file(file_path):
    """Fix all manual error construction patterns in a single file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_config_error_constructions(content)
        content = fix_network_error_constructions(content)
        content = fix_service_error_constructions(content)
        content = fix_validation_error_constructions(content)
        content = fix_generic_missing_category(content)
        
        # Only write if changes were made
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Fixed: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function to fix all Rust files in songbird-network"""
    network_dir = Path("crates/songbird-network/src")
    
    if not network_dir.exists():
        print(f"Directory {network_dir} not found!")
        sys.exit(1)
    
    fixed_files = 0
    total_files = 0
    
    # Find all Rust files
    for rust_file in network_dir.rglob("*.rs"):
        total_files += 1
        if fix_file(rust_file):
            fixed_files += 1
    
    print(f"\nProcessed {total_files} files, fixed {fixed_files} files")

if __name__ == "__main__":
    main() 