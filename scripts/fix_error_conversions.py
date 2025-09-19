#!/usr/bin/env python3
"""
Songbird Network Error Conversion Fixer

This script fixes `?` operator errors where underlying errors can't be automatically
converted to SongbirdError by adding explicit `.map_err()` conversions.

Common patterns:
- tokio::net operations (io::Error)
- serde operations (serde::Error)
- std::fs operations (io::Error)
- reqwest operations (reqwest::Error)
"""

import os
import re
import sys
from pathlib import Path

def fix_tokio_net_errors(content):
    """Fix tokio::net operations that need error conversion"""
    patterns = [
        # UdpSocket::bind
        (r'(tokio::net::UdpSocket::bind\([^)]+\)\.await)\?', 
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to bind UDP socket: {e}"), None))?'),
        
        # TcpListener::bind
        (r'(TcpListener::bind\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to bind TCP listener: {e}"), None))?'),
        
        # socket.send_to
        (r'(socket\.send_to\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to send data: {e}"), None))?'),
        
        # socket.recv_from
        (r'(socket\.recv_from\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to receive data: {e}"), None))?'),
        
        # TcpStream::connect
        (r'(TcpStream::connect\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to connect: {e}"), None))?'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_serde_errors(content):
    """Fix serde operations that need error conversion"""
    patterns = [
        # serde_json::to_string
        (r'(serde_json::to_string\([^)]+\))\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("JSON serialization failed: {e}")))?'),
        
        # serde_json::from_str
        (r'(serde_json::from_str\([^)]+\))\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("JSON deserialization failed: {e}")))?'),
        
        # serde_json::from_slice
        (r'(serde_json::from_slice\([^)]+\))\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("JSON deserialization failed: {e}")))?'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_fs_errors(content):
    """Fix filesystem operations that need error conversion"""
    patterns = [
        # std::fs::read_to_string
        (r'(std::fs::read_to_string\([^)]+\))\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("Failed to read file: {e}")))?'),
        
        # std::fs::write
        (r'(std::fs::write\([^)]+\))\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("Failed to write file: {e}")))?'),
        
        # tokio::fs::read_to_string
        (r'(tokio::fs::read_to_string\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("Failed to read file: {e}")))?'),
        
        # tokio::fs::write
        (r'(tokio::fs::write\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::internal_error(format!("Failed to write file: {e}")))?'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_reqwest_errors(content):
    """Fix reqwest operations that need error conversion"""
    patterns = [
        # Client::get().send()
        (r'(\.send\(\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("HTTP request failed: {e}"), None))?'),
        
        # response.text()
        (r'(\.text\(\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to read response text: {e}"), None))?'),
        
        # response.json()
        (r'(\.json\(\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Failed to parse JSON response: {e}"), None))?'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_generic_await_errors(content):
    """Fix generic .await? patterns that need error conversion"""
    # This is more conservative - only fix obvious patterns
    patterns = [
        # Generic socket operations
        (r'(\.bind\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Bind operation failed: {e}"), None))?'),
        
        # Generic connect operations  
        (r'(\.connect\([^)]+\)\.await)\?',
         r'\1\n            .map_err(|e| SongbirdError::network_error(format!("Connect operation failed: {e}"), None))?'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    return content

def fix_file(file_path):
    """Fix all error conversion patterns in a single file"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Apply all fixes
        content = fix_tokio_net_errors(content)
        content = fix_serde_errors(content)
        content = fix_fs_errors(content)
        content = fix_reqwest_errors(content)
        content = fix_generic_await_errors(content)
        
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