#!/usr/bin/env python3
"""
Automated Test Syntax Fixer for Songbird
Fixes common syntax errors in test files: missing braces, incomplete functions, etc.
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple

def fix_test_function_syntax(content: str) -> str:
    """Fix common syntax errors in test functions."""
    lines = content.split('\n')
    fixed_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this is a function definition
        fn_match = re.match(r'^(fn test_\w+)\((.*?)\)\s*(->.*?)?(\{)?$', line)
        
        if fn_match:
            func_name = fn_match.group(1)
            params = fn_match.group(2)
            return_type = fn_match.group(3) or ''
            has_opening_brace = fn_match.group(4)
            
            # Add #[test] if missing
            if i == 0 or not fixed_lines[-1].strip().startswith('#[test]'):
                fixed_lines.append('#[test]')
            
            # Add function signature with opening brace
            if has_opening_brace:
                fixed_lines.append(line)
            else:
                fixed_lines.append(f"{func_name}({params}){return_type} {{")
            
            # Look ahead to find where function should end
            i += 1
            brace_count = 1
            func_body = []
            
            while i < len(lines) and brace_count > 0:
                current = lines[i]
                
                # Count braces
                brace_count += current.count('{') - current.count('}')
                
                # Check if next line is a new function or end of file
                if i + 1 < len(lines):
                    next_line = lines[i + 1]
                    is_new_fn = re.match(r'^(#\[test\]|fn test_)', next_line)
                    is_comment = next_line.strip().startswith('//')
                    
                    if brace_count == 1 and (is_new_fn or is_comment):
                        # End of function - add closing brace if needed
                        func_body.append(current)
                        
                        # Add Ok(()) if function returns SongbirdResult
                        if 'SongbirdResult' in return_type and not any('Ok(())' in l for l in func_body[-5:]):
                            if func_body and not func_body[-1].strip().endswith('}'):
                                func_body.append('    Ok(())')
                        
                        # Add closing brace
                        func_body.append('}')
                        brace_count = 0
                        break
                
                func_body.append(current)
                i += 1
            
            fixed_lines.extend(func_body)
        else:
            fixed_lines.append(line)
        
        i += 1
    
    return '\n'.join(fixed_lines)

def add_missing_variables(content: str) -> str:
    """Add missing variable declarations."""
    fixes = [
        # Common patterns where config is used but not declared
        (r'^(\s+)(assert.*?config\.)', r'\1let config = CanonicalAdapterConfig::default();\n\1\2'),
        (r'^(\s+)(let debug_str = format.*?config)', r'\1let config = CanonicalAdapterConfig::default();\n\1\2'),
        
        # Similar for other common variables
        (r'^(\s+)(assert.*?perf\.)', r'\1let perf = CanonicalServicePerformance::default();\n\1\2'),
        (r'^(\s+)(assert.*?metrics\.)', r'\1let metrics = CanonicalAdapterMetrics::default();\n\1\2'),
    ]
    
    lines = content.split('\n')
    result = []
    declared_in_func = set()
    
    for i, line in enumerate(lines):
        # Track function boundaries
        if re.match(r'^fn test_', line):
            declared_in_func.clear()
        
        # Check if we're using a variable without declaring it
        for var_name in ['config', 'perf', 'metrics']:
            if var_name in line and f'let {var_name}' not in line and var_name not in declared_in_func:
                # Look back to see if it was declared recently
                recent_lines = result[-10:] if len(result) > 10 else result
                if not any(f'let {var_name}' in l for l in recent_lines):
                    # Add declaration
                    indent = re.match(r'^(\s*)', line).group(1)
                    if var_name == 'config':
                        result.append(f"{indent}let config = CanonicalAdapterConfig::default();")
                    elif var_name == 'perf':
                        result.append(f"{indent}let perf = CanonicalServicePerformance::default();")
                    elif var_name == 'metrics':
                        result.append(f"{indent}let metrics = CanonicalAdapterMetrics::default();")
                    declared_in_func.add(var_name)
        
        result.append(line)
    
    return '\n'.join(result)

def fix_incomplete_statements(content: str) -> str:
    """Fix incomplete statements and missing syntax."""
    lines = content.split('\n')
    fixed = []
    
    for i, line in enumerate(lines):
        # Fix incomplete assert_eq! statements
        if 'assert_eq!(' in line and i + 1 < len(lines):
            if not ')' in line and not lines[i + 1].strip().startswith(')'):
                # Missing closing paren
                line = line.rstrip() + ');'
        
        # Fix lines that should end with Ok(()) but don't
        if i + 1 < len(lines) and re.match(r'^fn test_.*SongbirdResult', lines[i]):
            # Check if function body ends without Ok(())
            next_line = lines[i + 1] if i + 1 < len(lines) else ''
            if next_line.strip() == '}' and i > 0:
                prev = fixed[-1] if fixed else ''
                if not 'Ok(())' in prev:
                    fixed.append('    Ok(())')
        
        fixed.append(line)
    
    return '\n'.join(fixed)

def fix_missing_semicolons(content: str) -> str:
    """Add missing semicolons."""
    lines = content.split('\n')
    fixed = []
    
    for line in lines:
        stripped = line.strip()
        # Statements that should end with semicolon but don't
        if (stripped and 
            not stripped.endswith(('{', '}', ';', ',')) and
            not stripped.startswith(('//','#','use','fn')) and
            ('let ' in stripped or 'assert' in stripped)):
            line = line.rstrip() + ';'
        
        fixed.append(line)
    
    return '\n'.join(fixed)

def process_file(filepath: Path) -> Tuple[bool, str]:
    """Process a single test file."""
    try:
        content = filepath.read_text()
        original_content = content
        
        # Apply fixes in order
        content = fix_test_function_syntax(content)
        content = add_missing_variables(content)
        content = fix_incomplete_statements(content)
        content = fix_missing_semicolons(content)
        
        # Only write if changed
        if content != original_content:
            filepath.write_text(content)
            return True, "Fixed"
        return True, "No changes needed"
        
    except Exception as e:
        return False, f"Error: {e}"

def main():
    # Test files with known syntax errors
    test_files = [
        "crates/songbird-types/tests/canonical_adapter_tests.rs",
        "crates/songbird-types/tests/config_module_enhanced_tests.rs",
        "crates/songbird-types/tests/config_unified_tests.rs",
        "crates/songbird-types/tests/core_types_tests.rs",
        "crates/songbird-types/tests/error_handling_comprehensive_tests.rs",
        "crates/songbird-types/tests/gaming_config_tests.rs",
        "crates/songbird-types/tests/health_comprehensive_tests.rs",
        "crates/songbird-types/tests/health_module_comprehensive_tests.rs",
        "crates/songbird-types/tests/health_tests.rs",
        "crates/songbird-types/tests/performance_tests.rs",
        "crates/songbird-types/tests/primal_and_health_tests.rs",
        "crates/songbird-types/tests/response_module_comprehensive_tests.rs",
        "crates/songbird-types/tests/response_tests.rs",
        "crates/songbird-types/tests/service_info_comprehensive_tests.rs",
        "crates/songbird-types/tests/service_module_comprehensive_tests.rs",
        "crates/songbird-types/tests/service_types_comprehensive_tests.rs",
        "crates/songbird-types/tests/traits_comprehensive_tests.rs",
        "crates/songbird-types/tests/type_conversion_tests.rs",
    ]
    
    project_root = Path(__file__).parent.parent
    
    print("🔧 Fixing test file syntax errors...")
    print(f"Project root: {project_root}\n")
    
    success_count = 0
    for test_file in test_files:
        filepath = project_root / test_file
        if not filepath.exists():
            print(f"❌ {test_file}: File not found")
            continue
        
        success, message = process_file(filepath)
        status = "✅" if success else "❌"
        print(f"{status} {test_file}: {message}")
        
        if success:
            success_count += 1
    
    print(f"\n📊 Results: {success_count}/{len(test_files)} files processed successfully")
    
    if success_count == len(test_files):
        print("\n✨ All files fixed! Run 'cargo test --package songbird-types' to verify.")
        return 0
    else:
        print("\n⚠️  Some files had errors. Manual review needed.")
        return 1

if __name__ == "__main__":
    sys.exit(main())

