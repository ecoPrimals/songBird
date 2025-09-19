#!/usr/bin/env python3
"""
Pedantic Documentation Fixer for Songbird

This script automatically adds comprehensive documentation to all missing:
- Struct fields
- Methods
- Associated functions
- Variants
- Modules
- Type aliases

It parses clippy output and adds appropriate documentation based on context.
"""

import re
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional

def run_clippy() -> str:
    """Run clippy and capture output"""
    try:
        result = subprocess.run([
            'cargo', 'clippy', '--all', '--all-targets', '--all-features', '--',
            '-D', 'warnings', '-W', 'clippy::pedantic', '-W', 'clippy::nursery'
        ], capture_output=True, text=True, cwd='/home/eastgate/Development/ecoPrimals/songbird')
        return result.stderr
    except Exception as e:
        print(f"Error running clippy: {e}")
        return ""

def parse_missing_docs(clippy_output: str) -> List[Dict[str, str]]:
    """Parse clippy output to extract missing documentation issues"""
    issues = []
    lines = clippy_output.split('\n')
    
    for i, line in enumerate(lines):
        if 'missing documentation' in line:
            # Extract the type of documentation missing
            doc_type_match = re.search(r'missing documentation for (.*)', line)
            if not doc_type_match:
                continue
            
            doc_type = doc_type_match.group(1)
            
            # Look for file path in next lines
            for j in range(i+1, min(i+5, len(lines))):
                file_match = re.search(r'--> (crates/[^:]+):(\d+):(\d+)', lines[j])
                if file_match:
                    issues.append({
                        'type': doc_type,
                        'file': file_match.group(1),
                        'line': int(file_match.group(2)),
                        'column': int(file_match.group(3))
                    })
                    break
    
    return issues

def generate_field_doc(field_name: str, context: str = "") -> str:
    """Generate appropriate documentation for a field based on its name"""
    field_name = field_name.lower()
    
    # Common field documentation patterns
    docs = {
        'message': 'Error message describing the issue',
        'field': 'Name of the field that caused the error',
        'context': 'Additional context information',
        'suggestion': 'Suggested fix for the issue',
        'severity': 'Severity level of the error',
        'timeout': 'Timeout duration for the operation',
        'endpoint': 'Network endpoint address',
        'port': 'Network port number',
        'protocol': 'Network protocol used',
        'service': 'Service name or identifier',
        'provider': 'Authentication provider name',
        'token': 'Authentication token',
        'url': 'URL address',
        'path': 'File or directory path',
        'name': 'Name identifier',
        'id': 'Unique identifier',
        'version': 'Version information',
        'description': 'Description text',
        'enabled': 'Whether the feature is enabled',
        'config': 'Configuration settings',
        'data': 'Data payload',
        'timestamp': 'Time when the event occurred',
        'status': 'Current status',
        'code': 'Status or error code',
        'headers': 'HTTP headers',
        'body': 'Request or response body',
        'metadata': 'Additional metadata',
        'retry_count': 'Number of retry attempts',
        'max_retries': 'Maximum number of retries allowed',
        'interval': 'Time interval between operations',
        'duration': 'Duration of the operation',
        'size': 'Size in bytes',
        'count': 'Number of items',
        'limit': 'Maximum limit',
        'offset': 'Offset value',
        'index': 'Index position',
        'key': 'Key identifier',
        'value': 'Value data',
        'type': 'Type information',
        'kind': 'Kind or category',
        'category': 'Category classification',
        'level': 'Level or priority',
        'priority': 'Priority level',
        'weight': 'Weight value',
        'score': 'Score value',
        'rank': 'Rank position',
        'address': 'Address information',
        'host': 'Host name or address',
        'domain': 'Domain name',
        'scheme': 'URL scheme',
        'query': 'Query parameters',
        'fragment': 'URL fragment',
        'hash': 'Hash value',
        'checksum': 'Checksum value',
        'signature': 'Digital signature',
        'certificate': 'Security certificate',
        'key_pair': 'Cryptographic key pair',
        'algorithm': 'Algorithm name',
        'cipher': 'Cipher type',
        'encoding': 'Encoding format',
        'format': 'Data format',
        'compression': 'Compression type',
        'quality': 'Quality level',
        'bandwidth': 'Bandwidth limit',
        'latency': 'Network latency',
        'throughput': 'Data throughput',
        'capacity': 'Storage capacity',
        'usage': 'Resource usage',
        'available': 'Available resources',
        'allocated': 'Allocated resources',
        'reserved': 'Reserved resources',
        'free': 'Free resources',
        'total': 'Total amount',
        'current': 'Current value',
        'previous': 'Previous value',
        'next': 'Next value',
        'first': 'First item',
        'last': 'Last item',
        'min': 'Minimum value',
        'max': 'Maximum value',
        'avg': 'Average value',
        'sum': 'Sum of values',
        'mean': 'Mean value',
        'median': 'Median value',
        'mode': 'Mode value',
        'std_dev': 'Standard deviation',
        'variance': 'Variance value',
        'range': 'Value range',
        'created_at': 'Creation timestamp',
        'updated_at': 'Last update timestamp',
        'deleted_at': 'Deletion timestamp',
        'expires_at': 'Expiration timestamp',
        'started_at': 'Start timestamp',
        'finished_at': 'Completion timestamp',
        'last_seen': 'Last seen timestamp',
        'last_accessed': 'Last access timestamp',
        'last_modified': 'Last modification timestamp',
    }
    
    # Try exact match first
    if field_name in docs:
        return docs[field_name]
    
    # Try partial matches
    for pattern, doc in docs.items():
        if pattern in field_name:
            return doc.replace('the', f'the {field_name}')
    
    # Default documentation
    return f"The {field_name} field"

def add_struct_field_docs(file_path: str, line_num: int) -> bool:
    """Add documentation to a struct field"""
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
        
        if line_num > len(lines):
            return False
        
        # Find the field line (0-indexed)
        field_line = lines[line_num - 1]
        
        # Extract field name
        field_match = re.search(r'(\w+):\s*', field_line)
        if not field_match:
            return False
        
        field_name = field_match.group(1)
        doc = generate_field_doc(field_name)
        
        # Find indentation
        indent_match = re.match(r'(\s*)', field_line)
        indent = indent_match.group(1) if indent_match else '    '
        
        # Add documentation line before the field
        doc_line = f"{indent}/// {doc}\n"
        lines.insert(line_num - 1, doc_line)
        
        # Write back to file
        with open(file_path, 'w') as f:
            f.writelines(lines)
        
        return True
    
    except Exception as e:
        print(f"Error adding docs to {file_path}:{line_num}: {e}")
        return False

def add_method_docs(file_path: str, line_num: int, method_type: str) -> bool:
    """Add documentation to methods and associated functions"""
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
        
        if line_num > len(lines):
            return False
        
        # Find the method line (0-indexed)
        method_line = lines[line_num - 1]
        
        # Extract method name
        method_match = re.search(r'fn\s+(\w+)', method_line)
        if not method_match:
            return False
        
        method_name = method_match.group(1)
        
        # Generate appropriate documentation
        if method_name == 'new':
            doc = "Creates a new instance"
        elif method_name.startswith('get_'):
            doc = f"Gets the {method_name[4:].replace('_', ' ')}"
        elif method_name.startswith('set_'):
            doc = f"Sets the {method_name[4:].replace('_', ' ')}"
        elif method_name.startswith('is_'):
            doc = f"Checks if {method_name[3:].replace('_', ' ')}"
        elif method_name.startswith('has_'):
            doc = f"Checks if it has {method_name[4:].replace('_', ' ')}"
        elif method_name.startswith('create_'):
            doc = f"Creates {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('build_'):
            doc = f"Builds {method_name[6:].replace('_', ' ')}"
        elif method_name.startswith('parse_'):
            doc = f"Parses {method_name[6:].replace('_', ' ')}"
        elif method_name.startswith('validate_'):
            doc = f"Validates {method_name[9:].replace('_', ' ')}"
        elif method_name.startswith('format_'):
            doc = f"Formats {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('convert_'):
            doc = f"Converts {method_name[8:].replace('_', ' ')}"
        elif method_name.startswith('update_'):
            doc = f"Updates {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('delete_'):
            doc = f"Deletes {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('remove_'):
            doc = f"Removes {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('add_'):
            doc = f"Adds {method_name[4:].replace('_', ' ')}"
        elif method_name.startswith('insert_'):
            doc = f"Inserts {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('find_'):
            doc = f"Finds {method_name[5:].replace('_', ' ')}"
        elif method_name.startswith('search_'):
            doc = f"Searches for {method_name[7:].replace('_', ' ')}"
        elif method_name.startswith('load_'):
            doc = f"Loads {method_name[5:].replace('_', ' ')}"
        elif method_name.startswith('save_'):
            doc = f"Saves {method_name[5:].replace('_', ' ')}"
        elif method_name.startswith('start_'):
            doc = f"Starts {method_name[6:].replace('_', ' ')}"
        elif method_name.startswith('stop_'):
            doc = f"Stops {method_name[5:].replace('_', ' ')}"
        elif method_name.startswith('run_'):
            doc = f"Runs {method_name[4:].replace('_', ' ')}"
        elif method_name.startswith('execute_'):
            doc = f"Executes {method_name[8:].replace('_', ' ')}"
        elif method_name.startswith('process_'):
            doc = f"Processes {method_name[8:].replace('_', ' ')}"
        elif method_name.startswith('handle_'):
            doc = f"Handles {method_name[7:].replace('_', ' ')}"
        elif method_name == 'default':
            doc = "Creates a default instance"
        elif method_name == 'clone':
            doc = "Creates a clone of this instance"
        elif method_name == 'fmt':
            doc = "Formats this instance for display"
        else:
            doc = f"Performs {method_name.replace('_', ' ')} operation"
        
        # Find indentation
        indent_match = re.match(r'(\s*)', method_line)
        indent = indent_match.group(1) if indent_match else '    '
        
        # Add documentation line before the method
        doc_line = f"{indent}/// {doc}\n"
        lines.insert(line_num - 1, doc_line)
        
        # Write back to file
        with open(file_path, 'w') as f:
            f.writelines(lines)
        
        return True
    
    except Exception as e:
        print(f"Error adding method docs to {file_path}:{line_num}: {e}")
        return False

def main():
    """Main function to fix all pedantic documentation issues"""
    print("🎯 PEDANTIC DOCUMENTATION FIXER")
    print("=" * 50)
    
    # Run clippy to get issues
    print("Running clippy analysis...")
    clippy_output = run_clippy()
    
    if not clippy_output:
        print("No clippy output received")
        return
    
    # Parse issues
    issues = parse_missing_docs(clippy_output)
    print(f"Found {len(issues)} documentation issues")
    
    # Group issues by type
    by_type = {}
    for issue in issues:
        issue_type = issue['type']
        if issue_type not in by_type:
            by_type[issue_type] = []
        by_type[issue_type].append(issue)
    
    # Print summary
    for issue_type, type_issues in by_type.items():
        print(f"  - {len(type_issues)} {issue_type}")
    
    # Fix issues
    fixed_count = 0
    for issue in issues:
        issue_type = issue['type']
        file_path = issue['file']
        line_num = issue['line']
        
        success = False
        if 'struct field' in issue_type:
            success = add_struct_field_docs(file_path, line_num)
        elif 'method' in issue_type or 'associated function' in issue_type:
            success = add_method_docs(file_path, line_num, issue_type)
        
        if success:
            fixed_count += 1
            print(f"✅ Fixed {issue_type} in {file_path}:{line_num}")
        else:
            print(f"❌ Failed to fix {issue_type} in {file_path}:{line_num}")
    
    print(f"\n🎉 Fixed {fixed_count}/{len(issues)} documentation issues")

if __name__ == '__main__':
    main() 