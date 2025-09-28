#!/usr/bin/env python3
"""
Fix CLI Import Issues
Systematically fix all import issues in the songbird-cli crate
"""

import os
import re
from pathlib import Path

def fix_imports(content):
    """Fix import statements in CLI files"""
    
    # Fix songbird_errors::Result -> songbird_types::SongbirdResult
    content = re.sub(
        r'use songbird_errors::Result;',
        'use songbird_types::SongbirdResult;',
        content
    )
    
    # Fix CLI imports - use the correct paths
    content = re.sub(
        r'use crate::cli::\{([^}]*)\};',
        lambda m: fix_cli_import_block(m.group(1)),
        content
    )
    
    # Fix individual CLI imports
    content = re.sub(
        r'use crate::cli::CliError;',
        'use crate::errors::CliError;',
        content
    )
    
    content = re.sub(
        r'use crate::cli::CliResult;',
        'use crate::errors::CliResult;',
        content
    )
    
    content = re.sub(
        r'use crate::cli::OutputFormat;',
        'use crate::types::OutputFormat;',
        content
    )
    
    content = re.sub(
        r'crate::cli::CliError',
        'crate::errors::CliError',
        content
    )
    
    content = re.sub(
        r'crate::cli::CliResult',
        'crate::errors::CliResult',
        content
    )
    
    content = re.sub(
        r'crate::cli::OutputFormat',
        'crate::types::OutputFormat',
        content
    )
    
    # Fix error API calls to match current SongbirdError
    content = re.sub(
        r'SongbirdError::command_error\(',
        'SongbirdError::service("cli", ',
        content
    )
    
    content = re.sub(
        r'SongbirdError::config_error\(',
        'SongbirdError::configuration(',
        content
    )
    
    content = re.sub(
        r'SongbirdError::internal_error\(',
        'SongbirdError::configuration(',
        content
    )
    
    # Fix gaming error variants that don't exist
    content = re.sub(
        r'SongbirdError::Gaming\([^)]+\)',
        'SongbirdError::service("gaming", "Gaming operation failed")',
        content
    )
    
    # Fix Io error variants that don't exist
    content = re.sub(
        r'SongbirdError::Io\([^)]+\)',
        'SongbirdError::configuration("IO operation failed")',
        content
    )
    
    # Fix Config error variants
    content = re.sub(
        r'SongbirdError::Config \{ message, field, \.\. \}',
        'SongbirdError::Configuration { message, field: None, suggestion: None }',
        content
    )
    
    # Fix Deployment error variants that don't exist
    content = re.sub(
        r'SongbirdError::Deployment\([^)]+\)',
        'SongbirdError::service("deployment", "Deployment operation failed")',
        content
    )
    
    # Fix error matching patterns
    content = re.sub(
        r'SongbirdError::Network\(network_error\)',
        'SongbirdError::Network { message, interface: _, suggestion: _ }',
        content
    )
    
    content = re.sub(
        r'SongbirdError::Service\(service_error\)',
        'SongbirdError::Service { service: _, message, suggested_alternatives: _, recovery_actions: _ }',
        content
    )
    
    return content

def fix_cli_import_block(import_content):
    """Fix a CLI import block like {CliError, CliResult}"""
    imports = [item.strip() for item in import_content.split(',')]
    fixed_imports = []
    
    for imp in imports:
        if 'CliError' in imp:
            fixed_imports.append('use crate::errors::CliError;')
        elif 'CliResult' in imp:
            fixed_imports.append('use crate::errors::CliResult;')
        elif 'OutputFormat' in imp:
            fixed_imports.append('use crate::types::OutputFormat;')
        elif 'DeploymentType' in imp:
            fixed_imports.append('use crate::types::DeploymentType;')
    
    return '\n'.join(fixed_imports)

def process_cli_files():
    """Process all Rust files in the songbird-cli crate"""
    cli_crate_path = Path("crates/songbird-cli/src")
    
    if not cli_crate_path.exists():
        print(f"Error: {cli_crate_path} does not exist.")
        return
        
    print(f"Processing files in {cli_crate_path}...")
    
    for file_path in cli_crate_path.rglob("*.rs"):
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            content = fix_imports(content)
            
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"Updated: {file_path}")
            else:
                print(f"No changes to: {file_path}")
                
        except Exception as e:
            print(f"Error processing {file_path}: {e}")

if __name__ == "__main__":
    process_cli_files() 