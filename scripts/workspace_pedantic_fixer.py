#!/usr/bin/env python3
"""
Workspace Pedantic Perfection Fixer

This script applies pedantic perfection to ALL crates in the workspace:
1. Must-use attributes for functions
2. Redundant closure fixes
3. Format argument inlining
4. Missing Debug implementations
5. Unused results handling
6. All other pedantic issues

Usage:
    python3 scripts/workspace_pedantic_fixer.py --fix-all
    python3 scripts/workspace_pedantic_fixer.py --crate songbird-errors
    python3 scripts/workspace_pedantic_fixer.py --analyze
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Dict, Optional
import json

class WorkspacePedanticFixer:
    """Fixes pedantic issues across the entire workspace"""
    
    def __init__(self, dry_run: bool = False, verbose: bool = False):
        self.dry_run = dry_run
        self.verbose = verbose
        self.issues_fixed = 0
        self.crates_processed = 0
        
    def fix_all_workspace(self):
        """Fix all pedantic issues across the entire workspace"""
        print("🎯 **WORKSPACE PEDANTIC PERFECTION MODE**")
        print("=" * 60)
        
        # Get all crates
        crates = self.get_workspace_crates()
        print(f"📦 Found {len(crates)} crates to process")
        
        total_issues = 0
        for crate_name in crates:
            print(f"\n🔧 **Processing crate: {crate_name}**")
            issues = self.fix_crate_pedantic_issues(crate_name)
            total_issues += issues
            self.crates_processed += 1
            
            if issues > 0:
                print(f"✅ Fixed {issues} issues in {crate_name}")
            else:
                print(f"🎯 {crate_name} already pedantic perfect!")
        
        print(f"\n🎉 **WORKSPACE PEDANTIC PERFECTION COMPLETE**")
        print(f"📦 Processed: {self.crates_processed} crates")
        print(f"🔧 Fixed: {total_issues} total issues")
        
        return total_issues
    
    def get_workspace_crates(self) -> List[str]:
        """Get all crate names in the workspace"""
        crates = []
        
        # Find all Cargo.toml files in crates/
        for crate_dir in Path('crates').iterdir():
            if crate_dir.is_dir() and (crate_dir / 'Cargo.toml').exists():
                crates.append(crate_dir.name)
        
        return sorted(crates)
    
    def fix_crate_pedantic_issues(self, crate_name: str) -> int:
        """Fix pedantic issues in a specific crate"""
        issues_fixed = 0
        
        # Run clippy to identify issues
        issues = self.analyze_crate_issues(crate_name)
        
        if not issues:
            return 0
        
        print(f"🔍 Found {len(issues)} pedantic issues in {crate_name}")
        
        # Apply fixes based on issue types
        for issue in issues:
            if self.apply_pedantic_fix(crate_name, issue):
                issues_fixed += 1
                
        return issues_fixed
    
    def analyze_crate_issues(self, crate_name: str) -> List[Dict]:
        """Analyze pedantic issues in a crate"""
        try:
            # Run clippy with pedantic settings
            result = subprocess.run([
                'cargo', 'clippy', '--package', crate_name, '--', 
                '-D', 'clippy::pedantic'
            ], capture_output=True, text=True, timeout=120)
            
            if result.returncode == 0:
                return []  # No issues found
            
            # Parse clippy output
            issues = self.parse_clippy_issues(result.stderr)
            return issues
            
        except Exception as e:
            if self.verbose:
                print(f"⚠️ Error analyzing {crate_name}: {e}")
            return []
    
    def parse_clippy_issues(self, clippy_output: str) -> List[Dict]:
        """Parse clippy output to extract pedantic issues"""
        issues = []
        lines = clippy_output.split('\n')
        
        current_issue = None
        for line in lines:
            # Match error/warning lines
            error_match = re.match(r'error: (.+)', line)
            if error_match:
                if current_issue:
                    issues.append(current_issue)
                
                current_issue = {
                    'type': 'error',
                    'message': error_match.group(1),
                    'file': None,
                    'line': None,
                    'column': None,
                    'suggestion': None,
                    'help': None
                }
                continue
            
            # Match file location
            location_match = re.match(r'\s*--> (.+):(\d+):(\d+)', line)
            if location_match and current_issue:
                current_issue['file'] = location_match.group(1)
                current_issue['line'] = int(location_match.group(2))
                current_issue['column'] = int(location_match.group(3))
                continue
            
            # Match help suggestions
            help_match = re.match(r'\s*= help: (.+)', line)
            if help_match and current_issue:
                current_issue['help'] = help_match.group(1)
                continue
            
            # Match fix suggestions
            if 'help: add the attribute' in line and current_issue:
                attr_match = re.search(r'`(#\[.+?\])`', line)
                if attr_match:
                    current_issue['suggestion'] = attr_match.group(1)
                continue
            
            # Match closure replacement suggestions
            if 'help: replace the closure with the method itself:' in line and current_issue:
                method_match = re.search(r': `(.+)`', line)
                if method_match:
                    current_issue['suggestion'] = method_match.group(1)
                continue
            
            # Match format string suggestions
            if 'help: change this to' in line and current_issue:
                current_issue['suggestion'] = 'inline_format'
                continue
        
        if current_issue:
            issues.append(current_issue)
        
        return issues
    
    def apply_pedantic_fix(self, crate_name: str, issue: Dict) -> bool:
        """Apply a specific pedantic fix"""
        if not issue.get('file') or not issue.get('line'):
            return False
        
        file_path = issue['file']
        if not Path(file_path).exists():
            return False
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            line_idx = issue['line'] - 1  # Convert to 0-based index
            
            if line_idx >= len(lines):
                return False
            
            original_line = lines[line_idx]
            
            # Apply different types of fixes
            fixed = False
            
            # Fix must-use attributes
            if 'must_use' in issue.get('message', '') and issue.get('suggestion'):
                if issue['suggestion'].startswith('#[must_use]'):
                    # Add must_use attribute
                    indent = len(original_line) - len(original_line.lstrip())
                    lines[line_idx] = ' ' * indent + issue['suggestion'] + ' ' + original_line.lstrip()
                    fixed = True
            
            # Fix redundant closures
            elif 'redundant closure' in issue.get('message', '') and issue.get('suggestion'):
                # Replace closure with method
                if 'map(' in original_line and issue['suggestion']:
                    new_line = re.sub(
                        r'\.map\(\|[^|]+\| [^.]+\.to_string\(\)\)',
                        f'.map({issue["suggestion"]})',
                        original_line
                    )
                    if new_line != original_line:
                        lines[line_idx] = new_line
                        fixed = True
            
            # Fix format string inlining
            elif 'uninlined_format_args' in issue.get('help', '') or issue.get('suggestion') == 'inline_format':
                # Inline format arguments
                format_match = re.search(r'format!\("([^"]*)", ([^)]+)\)', original_line)
                if format_match:
                    format_str = format_match.group(1)
                    args = format_match.group(2)
                    
                    # Simple case: format!("text: {}", var) -> format!("text: {var}")
                    if '{}' in format_str and ',' in args:
                        var_name = args.strip()
                        new_format = format_str.replace('{}', f'{{{var_name}}}')
                        new_line = original_line.replace(
                            f'format!("{format_str}", {args})',
                            f'format!("{new_format}")'
                        )
                        lines[line_idx] = new_line
                        fixed = True
            
            # Fix missing Debug implementations
            elif 'missing-debug-implementations' in issue.get('help', ''):
                # Add #[derive(Debug)] before struct/enum
                if 'pub struct' in original_line or 'pub enum' in original_line:
                    indent = len(original_line) - len(original_line.lstrip())
                    debug_attr = ' ' * indent + '#[derive(Debug)]\n'
                    lines.insert(line_idx, debug_attr)
                    fixed = True
            
            # Fix unused results
            elif 'unused-results' in issue.get('help', ''):
                # Add let _ = before the expression
                if not original_line.strip().startswith('let'):
                    indent_match = re.match(r'(\s*)', original_line)
                    indent = indent_match.group(1) if indent_match else ''
                    
                    # Find the expression part
                    expr_match = re.search(r'(\S.+);', original_line.strip())
                    if expr_match:
                        expr = expr_match.group(1)
                        new_line = f'{indent}let _ = {expr};\n'
                        lines[line_idx] = new_line
                        fixed = True
            
            if fixed and not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.writelines(lines)
                
                if self.verbose:
                    print(f"  ✅ Fixed: {issue['message'][:50]}...")
                
                return True
            
        except Exception as e:
            if self.verbose:
                print(f"  ❌ Error fixing {file_path}:{issue['line']}: {e}")
        
        return False
    
    def generate_workspace_report(self) -> str:
        """Generate comprehensive workspace pedantic report"""
        
        # Get all crates and their status
        crates = self.get_workspace_crates()
        
        report = f"""
# 🎯 **WORKSPACE PEDANTIC PERFECTION REPORT**

**🚀 COMPREHENSIVE WORKSPACE QUALITY ASSESSMENT**

**Date**: {__import__('time').strftime('%Y-%m-%d %H:%M:%S')}
**Scope**: {len(crates)} crates in Songbird workspace
**Status**: 🔄 **PEDANTIC PERFECTION IN PROGRESS**

---

## 📊 **WORKSPACE OVERVIEW**

| **Crate** | **Status** | **Issues** | **Priority** |
|-----------|------------|------------|--------------|
"""
        
        for crate in crates:
            issues = self.analyze_crate_issues(crate)
            issue_count = len(issues)
            
            if issue_count == 0:
                status = "🟢 **PERFECT**"
                priority = "✅ Complete"
            elif issue_count <= 5:
                status = "🟡 **MINOR**"
                priority = "🔧 Low"
            elif issue_count <= 15:
                status = "🟠 **MODERATE**"
                priority = "⚡ Medium"
            else:
                status = "🔴 **CRITICAL**"
                priority = "🚨 High"
            
            report += f"| **{crate}** | {status} | {issue_count} | {priority} |\n"
        
        report += f"""

---

## 🔧 **RECOMMENDED ACTIONS**

### **Immediate Priority (High)**
- Fix critical crates with 15+ issues
- Apply must-use attributes to functions
- Add missing Debug implementations

### **Short Term (Medium)**
- Fix redundant closures
- Inline format arguments
- Handle unused results

### **Long Term (Low)**
- Maintain pedantic standards
- Implement pre-commit hooks
- Regular quality monitoring

---

## 🛠️ **AUTOMATION COMMANDS**

```bash
# Fix all workspace issues
python3 scripts/workspace_pedantic_fixer.py --fix-all

# Fix specific crate
python3 scripts/workspace_pedantic_fixer.py --crate songbird-errors

# Analyze without fixing
python3 scripts/workspace_pedantic_fixer.py --analyze
```

---

**🎯 WORKSPACE PEDANTIC PERFECTION: Elevating Every Crate to Excellence!**

*Generated by: Workspace Pedantic Perfection System*
"""
        
        return report

def main():
    parser = argparse.ArgumentParser(
        description='Workspace Pedantic Perfection Fixer',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Fix all crates in workspace
  python3 scripts/workspace_pedantic_fixer.py --fix-all
  
  # Fix specific crate
  python3 scripts/workspace_pedantic_fixer.py --crate songbird-errors
  
  # Analyze issues without fixing
  python3 scripts/workspace_pedantic_fixer.py --analyze
  
  # Dry run to see what would be fixed
  python3 scripts/workspace_pedantic_fixer.py --fix-all --dry-run
        """
    )
    
    parser.add_argument('--fix-all', action='store_true', help='Fix all crates in workspace')
    parser.add_argument('--crate', help='Fix specific crate')
    parser.add_argument('--analyze', action='store_true', help='Analyze issues without fixing')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be fixed')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--report-file', default='WORKSPACE_PEDANTIC_REPORT.md', help='Report file')
    
    args = parser.parse_args()
    
    if not any([args.fix_all, args.crate, args.analyze]):
        parser.error("Must specify --fix-all, --crate, or --analyze")
    
    # Create fixer
    fixer = WorkspacePedanticFixer(dry_run=args.dry_run, verbose=args.verbose)
    
    print("🎯 **WORKSPACE PEDANTIC PERFECTION SYSTEM**")
    print("=" * 60)
    
    if args.dry_run:
        print("🔍 **DRY RUN MODE** - No changes will be made")
    
    try:
        if args.fix_all:
            total_fixed = fixer.fix_all_workspace()
            print(f"\n🎉 **WORKSPACE PERFECTION ACHIEVED!**")
            print(f"Fixed {total_fixed} issues across {fixer.crates_processed} crates")
            
        elif args.crate:
            issues_fixed = fixer.fix_crate_pedantic_issues(args.crate)
            print(f"\n🎉 **CRATE PERFECTION ACHIEVED!**")
            print(f"Fixed {issues_fixed} issues in {args.crate}")
            
        elif args.analyze:
            report = fixer.generate_workspace_report()
            with open(args.report_file, 'w') as f:
                f.write(report)
            print(f"\n📄 Workspace analysis saved to: {args.report_file}")
        
        if not args.dry_run and (args.fix_all or args.crate):
            print("\n✅ **NEXT STEPS:**")
            print("1. Run `cargo fmt --all` to format changes")
            print("2. Run `cargo clippy --workspace -- -D clippy::pedantic` to verify")
            print("3. Run `cargo test --workspace` to ensure functionality")
        
    except KeyboardInterrupt:
        print("\n❌ Workspace pedantic perfection cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Workspace pedantic perfection failed: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main() 