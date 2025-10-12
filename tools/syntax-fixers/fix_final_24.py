#!/usr/bin/env python3
"""Fix the final remaining errors to achieve full compilation."""

import re
from pathlib import Path

def fix_file_content(content, filename):
    """Fix remaining patterns in file content."""
    lines = content.split('\n')
    fixed_lines = []
    
    for i, line in enumerate(lines):
        fixed = line.rstrip()
        
        # Fix println! patterns without semicolons
        if 'println!' in fixed and fixed.endswith(')'):
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                # Add semicolon if next line is code
                if next_line and not next_line.startswith('}') and not next_line.startswith(')'):
                    if not fixed.endswith(';'):
                        fixed += ';'
        
        # Fix string literals with trailing identifiers (Rust 2021 prefix issue)
        # Pattern: "text identifier" -> "text identifier "
        if 'diagnostics")]' in fixed or 'management")]' in fixed:
            fixed = fixed.replace('diagnostics")]', 'diagnostics ")]')
            fixed = fixed.replace('management")]', 'management ")]')
        
        # Fix specific patterns
        if 'required");' in fixed:
            fixed = fixed.replace('required");', 'required ");')
        if 'configured");' in fixed:
            fixed = fixed.replace('configured");', 'configured ");')
        if 'implemented");' in fixed:
            fixed = fixed.replace('implemented");', 'implemented ");')
        
        # Fix path literals with identifiers
        if '"/api/v1"' in fixed or '"/apis"' in fixed or '"/version"' in fixed:
            fixed = fixed.replace('"/api/v1"', '"/api/v1 "')
            fixed = fixed.replace('"/apis"', '"/apis "')
            fixed = fixed.replace('"/version"', '"/version "')
        
        if '"/var/run/docker.sock"' in fixed:
            fixed = fixed.replace('"/var/run/docker.sock"', '"/var/run/docker.sock "')
        
        if '"/.dockerenv"' in fixed:
            fixed = fixed.replace('"/.dockerenv"', '"/.dockerenv "')
        
        if '"/run/.containerenv"' in fixed:
            fixed = fixed.replace('"/run/.containerenv"', '"/run/.containerenv "')
        
        if '"/proc/1/cgroup"' in fixed:
            fixed = fixed.replace('"/proc/1/cgroup"', '"/proc/1/cgroup "')
        
        if '"kubernetes-service-example"' in fixed:
            fixed = fixed.replace('"kubernetes-service-example"', '"kubernetes-service-example "')
        
        if '"container-env"' in fixed:
            fixed = fixed.replace('"container-env"', '"container-env "')
        
        if '"failing-service"' in fixed:
            fixed = fixed.replace('"failing-service"', '"failing-service "')
        
        if '"Connection timeout"' in fixed:
            fixed = fixed.replace('"Connection timeout"', '"Connection timeout "')
        
        # Fix unterminated strings (trailing quote-semicolon)
        if fixed.endswith('");"'):
            fixed = fixed[:-3] + '");'
        
        # Fix delimiter issues
        if 'Utc::now(,' in fixed:
            fixed = fixed.replace('Utc::now(,', 'Utc::now(),')
        
        if '{Healthy)' in fixed:
            fixed = fixed.replace('{Healthy)', '{ Healthy,')
        
        fixed_lines.append(fixed if fixed or not line.strip() else line)
    
    return '\n'.join(fixed_lines)

def fix_file(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        fixed_content = fix_file_content(content, file_path.name)
        
        if content != fixed_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    # Target specific problem files
    problem_files = [
        "crates/songbird-cli/src/bin/gaming_demo.rs",
        "crates/songbird-cli/src/bin/test_runner.rs",
        "crates/songbird-cli/src/cli/commands/mod.rs",
        "crates/songbird-config/src/config/hardcoded_elimination.rs",
        "crates/songbird-config/tests/comprehensive_config_tests.rs",
        "crates/songbird-config/tests/modernized_config_tests.rs",
        "crates/songbird-discovery/src/discovery/backends/container_orchestration.rs",
        "crates/songbird-observability/src/observability/mod.rs",
        "crates/songbird-observability/tests/systematic_observability_coverage.rs",
        "crates/songbird-network-federation/src/network/mod.rs",
    ]
    
    fixed_count = 0
    for file_path_str in problem_files:
        file_path = Path(file_path_str)
        if file_path.exists():
            if fix_file(file_path):
                fixed_count += 1
                print(f"Fixed: {file_path}")
        else:
            print(f"Not found: {file_path}")
    
    print(f"\n✅ Fixed {fixed_count} files")

if __name__ == "__main__":
    main()

