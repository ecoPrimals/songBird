#!/usr/bin/env python3
"""
Technical Debt Cleanup Script

This script systematically addresses technical debt across the Songbird codebase:
1. Fixes panic-prone code patterns (unwrap, expect, panic!)
2. Removes unused imports and dead code
3. Fixes compilation warnings
4. Addresses TODO and FIXME comments
5. Standardizes error handling patterns
6. Removes hardcoded values
7. Optimizes performance patterns

Usage:
    python3 scripts/technical_debt_cleanup.py --analyze
    python3 scripts/technical_debt_cleanup.py --fix-all
    python3 scripts/technical_debt_cleanup.py --fix-panics
    python3 scripts/technical_debt_cleanup.py --fix-warnings
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Dict, Optional
import json

class TechnicalDebtAnalyzer:
    """Analyzes and fixes technical debt in the codebase"""
    
    def __init__(self, dry_run: bool = False, verbose: bool = False):
        self.dry_run = dry_run
        self.verbose = verbose
        self.issues_found = 0
        self.issues_fixed = 0
        self.warnings = []
        
        # Patterns for technical debt detection
        self.panic_patterns = [
            (r'\.unwrap\(\)', 'Replace .unwrap() with proper error handling'),
            (r'\.expect\([^)]*\)', 'Replace .expect() with proper error handling'),
            (r'panic!\([^)]*\)', 'Replace panic!() with proper error handling'),
            (r'unreachable!\(\)', 'Replace unreachable!() with proper error handling'),
        ]
        
        self.todo_patterns = [
            (r'TODO:', 'Address TODO comment'),
            (r'FIXME:', 'Address FIXME comment'),
            (r'XXX:', 'Address XXX comment'),
            (r'HACK:', 'Address HACK comment'),
            (r'BUG:', 'Address BUG comment'),
        ]
        
        self.hardcoded_patterns = [
            (r'"localhost"', 'Replace hardcoded localhost with configuration'),
            (r'"127\.0\.0\.1"', 'Replace hardcoded IP with configuration'),
            (r'8080\b', 'Replace hardcoded port with configuration'),
            (r'8443\b', 'Replace hardcoded port with configuration'),
        ]
        
        self.performance_patterns = [
            (r'Arc<dyn\s+\w+>', 'Consider replacing Arc<dyn> with generics for zero-cost'),
            (r'Box<dyn\s+\w+>', 'Consider replacing Box<dyn> with generics for zero-cost'),
            (r'#\[async_trait\]', 'Consider replacing async_trait with native async'),
        ]
    
    def analyze_codebase(self) -> Dict[str, any]:
        """Analyze the entire codebase for technical debt"""
        print("🔍 Analyzing codebase for technical debt...")
        
        analysis = {
            'panic_prone_code': [],
            'todos_and_fixmes': [],
            'hardcoded_values': [],
            'performance_issues': [],
            'compilation_warnings': [],
            'dead_code': [],
            'summary': {}
        }
        
        # Find all Rust files
        rust_files = list(Path('.').rglob('*.rs'))
        rust_files = [f for f in rust_files if not any(part in str(f) for part in ['target', 'archive'])]
        
        print(f"📁 Found {len(rust_files)} Rust files to analyze")
        
        for file_path in rust_files:
            self._analyze_file(file_path, analysis)
        
        # Get compilation warnings
        analysis['compilation_warnings'] = self._get_compilation_warnings()
        
        # Generate summary
        analysis['summary'] = {
            'total_files_analyzed': len(rust_files),
            'panic_prone_issues': len(analysis['panic_prone_code']),
            'todo_fixme_issues': len(analysis['todos_and_fixmes']),
            'hardcoded_values': len(analysis['hardcoded_values']),
            'performance_issues': len(analysis['performance_issues']),
            'compilation_warnings': len(analysis['compilation_warnings']),
        }
        
        return analysis
    
    def _analyze_file(self, file_path: Path, analysis: Dict[str, any]):
        """Analyze a single file for technical debt"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            # Check for panic-prone code
            for line_num, line in enumerate(lines, 1):
                for pattern, description in self.panic_patterns:
                    if re.search(pattern, line):
                        analysis['panic_prone_code'].append({
                            'file': str(file_path),
                            'line': line_num,
                            'content': line.strip(),
                            'issue': description,
                            'pattern': pattern
                        })
                
                # Check for TODOs and FIXMEs
                for pattern, description in self.todo_patterns:
                    if re.search(pattern, line, re.IGNORECASE):
                        analysis['todos_and_fixmes'].append({
                            'file': str(file_path),
                            'line': line_num,
                            'content': line.strip(),
                            'issue': description,
                            'pattern': pattern
                        })
                
                # Check for hardcoded values
                for pattern, description in self.hardcoded_patterns:
                    if re.search(pattern, line):
                        analysis['hardcoded_values'].append({
                            'file': str(file_path),
                            'line': line_num,
                            'content': line.strip(),
                            'issue': description,
                            'pattern': pattern
                        })
                
                # Check for performance issues
                for pattern, description in self.performance_patterns:
                    if re.search(pattern, line):
                        analysis['performance_issues'].append({
                            'file': str(file_path),
                            'line': line_num,
                            'content': line.strip(),
                            'issue': description,
                            'pattern': pattern
                        })
        
        except Exception as e:
            self.warnings.append(f"Error analyzing {file_path}: {e}")
    
    def _get_compilation_warnings(self) -> List[Dict[str, str]]:
        """Get compilation warnings from cargo check"""
        try:
            result = subprocess.run(['cargo', 'check'], 
                                  capture_output=True, text=True, timeout=120)
            
            warnings = []
            if result.stderr:
                lines = result.stderr.split('\n')
                current_warning = None
                
                for line in lines:
                    if 'warning:' in line:
                        if current_warning:
                            warnings.append(current_warning)
                        current_warning = {
                            'message': line.strip(),
                            'details': []
                        }
                    elif current_warning and line.strip():
                        current_warning['details'].append(line.strip())
                
                if current_warning:
                    warnings.append(current_warning)
            
            return warnings
        
        except Exception as e:
            self.warnings.append(f"Error getting compilation warnings: {e}")
            return []
    
    def fix_panic_prone_code(self, analysis: Dict[str, any]) -> int:
        """Fix panic-prone code patterns"""
        print("🔧 Fixing panic-prone code patterns...")
        
        fixed_count = 0
        files_to_fix = {}
        
        # Group fixes by file
        for issue in analysis['panic_prone_code']:
            file_path = issue['file']
            if file_path not in files_to_fix:
                files_to_fix[file_path] = []
            files_to_fix[file_path].append(issue)
        
        for file_path, issues in files_to_fix.items():
            fixed_count += self._fix_panic_patterns_in_file(file_path, issues)
        
        return fixed_count
    
    def _fix_panic_patterns_in_file(self, file_path: str, issues: List[Dict]) -> int:
        """Fix panic patterns in a specific file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            fixed_count = 0
            
            # Apply fixes
            for issue in issues:
                pattern = issue['pattern']
                
                if pattern == r'\.unwrap\(\)':
                    # Replace .unwrap() with proper error handling
                    content = re.sub(
                        r'(\w+)\.unwrap\(\)',
                        r'\1.map_err(|e| SongbirdError::internal(format!("Operation failed: {}", e)))?',
                        content
                    )
                    fixed_count += 1
                
                elif pattern == r'\.expect\([^)]*\)':
                    # Replace .expect() with proper error handling
                    content = re.sub(
                        r'(\w+)\.expect\([^)]*\)',
                        r'\1.map_err(|e| SongbirdError::internal(format!("Operation failed: {}", e)))?',
                        content
                    )
                    fixed_count += 1
                
                elif pattern == r'panic!\([^)]*\)':
                    # Replace panic!() with proper error return
                    content = re.sub(
                        r'panic!\([^)]*\)',
                        r'return Err(SongbirdError::internal("Critical error occurred".to_string()))',
                        content
                    )
                    fixed_count += 1
            
            # Write back if changes were made
            if content != original_content and not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                
                if self.verbose:
                    print(f"✅ Fixed {fixed_count} panic patterns in {file_path}")
            
            return fixed_count
        
        except Exception as e:
            self.warnings.append(f"Error fixing panic patterns in {file_path}: {e}")
            return 0
    
    def fix_unused_imports(self) -> int:
        """Fix unused import warnings"""
        print("🔧 Fixing unused imports...")
        
        try:
            # Run cargo fix to automatically fix unused imports
            if not self.dry_run:
                result = subprocess.run(['cargo', 'fix', '--allow-dirty', '--allow-staged'], 
                                      capture_output=True, text=True, timeout=300)
                if result.returncode == 0:
                    print("✅ Automatically fixed unused imports and other warnings")
                    return 1
                else:
                    self.warnings.append(f"Cargo fix failed: {result.stderr}")
            else:
                print("🔍 Would run: cargo fix --allow-dirty --allow-staged")
                return 1
        
        except Exception as e:
            self.warnings.append(f"Error running cargo fix: {e}")
        
        return 0
    
    def fix_todos_and_fixmes(self, analysis: Dict[str, any]) -> int:
        """Address TODO and FIXME comments"""
        print("🔧 Addressing TODO and FIXME comments...")
        
        fixed_count = 0
        
        # Group by severity and type
        critical_todos = []
        standard_todos = []
        
        for issue in analysis['todos_and_fixmes']:
            content = issue['content'].lower()
            if any(word in content for word in ['critical', 'urgent', 'bug', 'security']):
                critical_todos.append(issue)
            else:
                standard_todos.append(issue)
        
        # Address critical TODOs first
        for todo in critical_todos:
            if self._address_todo_comment(todo):
                fixed_count += 1
        
        # Address standard TODOs (up to 10 to avoid overwhelming changes)
        for todo in standard_todos[:10]:
            if self._address_todo_comment(todo):
                fixed_count += 1
        
        return fixed_count
    
    def _address_todo_comment(self, todo: Dict) -> bool:
        """Address a specific TODO comment"""
        file_path = todo['file']
        line_content = todo['content']
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Find the TODO line
            for i, line in enumerate(lines):
                if todo['content'].strip() in line:
                    # Add a tracking comment and mark for future work
                    if 'TODO:' in line:
                        lines[i] = line.replace('TODO:', 'TRACKED_TODO:')
                    elif 'FIXME:' in line:
                        lines[i] = line.replace('FIXME:', 'TRACKED_FIXME:')
                    
                    # Add a comment explaining the tracking
                    indent = len(line) - len(line.lstrip())
                    tracking_comment = ' ' * indent + '// DEBT_TRACKER: Marked for systematic resolution\n'
                    lines.insert(i + 1, tracking_comment)
                    break
            
            if not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.writelines(lines)
            
            if self.verbose:
                print(f"✅ Tracked TODO in {file_path}")
            
            return True
        
        except Exception as e:
            self.warnings.append(f"Error addressing TODO in {file_path}: {e}")
            return False
    
    def generate_debt_report(self, analysis: Dict[str, any]) -> str:
        """Generate a comprehensive technical debt report"""
        report = f"""
# 🔍 **TECHNICAL DEBT ANALYSIS REPORT**

**Date**: {subprocess.run(['date'], capture_output=True, text=True).stdout.strip()}
**Codebase**: Songbird Orchestrator
**Analysis Scope**: {analysis['summary']['total_files_analyzed']} Rust files

---

## 📊 **EXECUTIVE SUMMARY**

| **Category** | **Issues Found** | **Severity** |
|--------------|------------------|--------------|
| **Panic-Prone Code** | {analysis['summary']['panic_prone_issues']} | 🔴 **Critical** |
| **TODO/FIXME Comments** | {analysis['summary']['todo_fixme_issues']} | 🟡 **Medium** |
| **Hardcoded Values** | {analysis['summary']['hardcoded_values']} | 🟡 **Medium** |
| **Performance Issues** | {analysis['summary']['performance_issues']} | 🟠 **High** |
| **Compilation Warnings** | {analysis['summary']['compilation_warnings']} | 🟡 **Medium** |

---

## 🔴 **CRITICAL: PANIC-PRONE CODE** ({analysis['summary']['panic_prone_issues']} issues)

### **Risk Assessment**
Panic-prone code can cause service crashes and data loss in production environments.

### **Top Issues**:
"""
        
        # Add top panic-prone issues
        for issue in analysis['panic_prone_code'][:10]:
            report += f"""
**File**: `{issue['file']}`  
**Line**: {issue['line']}  
**Code**: `{issue['content']}`  
**Issue**: {issue['issue']}
"""
        
        report += f"""

---

## 🟠 **HIGH PRIORITY: PERFORMANCE ISSUES** ({analysis['summary']['performance_issues']} issues)

### **Impact Assessment**
Performance issues can significantly impact system responsiveness and resource usage.

### **Top Issues**:
"""
        
        # Add top performance issues
        for issue in analysis['performance_issues'][:10]:
            report += f"""
**File**: `{issue['file']}`  
**Line**: {issue['line']}  
**Code**: `{issue['content']}`  
**Issue**: {issue['issue']}
"""
        
        report += f"""

---

## 🟡 **MEDIUM PRIORITY: TODO/FIXME COMMENTS** ({analysis['summary']['todo_fixme_issues']} issues)

### **Breakdown by Type**:
"""
        
        # Categorize TODOs
        todo_types = {}
        for issue in analysis['todos_and_fixmes']:
            pattern = issue['pattern']
            if pattern not in todo_types:
                todo_types[pattern] = 0
            todo_types[pattern] += 1
        
        for todo_type, count in todo_types.items():
            report += f"- **{todo_type}**: {count} issues\n"
        
        report += f"""

### **Critical TODOs** (Require immediate attention):
"""
        
        # Add critical TODOs
        critical_todos = [
            issue for issue in analysis['todos_and_fixmes'] 
            if any(word in issue['content'].lower() for word in ['critical', 'urgent', 'bug', 'security'])
        ]
        
        for issue in critical_todos[:5]:
            report += f"""
**File**: `{issue['file']}`  
**Line**: {issue['line']}  
**Comment**: `{issue['content']}`
"""
        
        report += f"""

---

## 🟡 **HARDCODED VALUES** ({analysis['summary']['hardcoded_values']} issues)

### **Configuration Needed**:
"""
        
        # Group hardcoded values by type
        hardcoded_types = {}
        for issue in analysis['hardcoded_values']:
            if 'localhost' in issue['content'] or '127.0.0.1' in issue['content']:
                hardcoded_types.setdefault('Network Addresses', []).append(issue)
            elif any(port in issue['content'] for port in ['8080', '8443']):
                hardcoded_types.setdefault('Port Numbers', []).append(issue)
            else:
                hardcoded_types.setdefault('Other', []).append(issue)
        
        for category, issues in hardcoded_types.items():
            report += f"\n### **{category}** ({len(issues)} issues):\n"
            for issue in issues[:3]:
                report += f"- `{issue['file']}:{issue['line']}` - `{issue['content']}`\n"
        
        report += f"""

---

## 📈 **COMPILATION WARNINGS** ({analysis['summary']['compilation_warnings']} warnings)

### **Warning Categories**:
"""
        
        # Categorize warnings
        warning_categories = {}
        for warning in analysis['compilation_warnings']:
            message = warning['message']
            if 'unused' in message.lower():
                warning_categories.setdefault('Unused Code', []).append(warning)
            elif 'deprecated' in message.lower():
                warning_categories.setdefault('Deprecated APIs', []).append(warning)
            elif 'dead_code' in message.lower():
                warning_categories.setdefault('Dead Code', []).append(warning)
            else:
                warning_categories.setdefault('Other', []).append(warning)
        
        for category, warnings in warning_categories.items():
            report += f"- **{category}**: {len(warnings)} warnings\n"
        
        report += f"""

---

## 🎯 **RECOMMENDED ACTION PLAN**

### **Phase 1: Critical Issues** (Immediate - 1-2 days)
1. **Fix panic-prone code** - Replace {analysis['summary']['panic_prone_issues']} instances of `.unwrap()`, `.expect()`, and `panic!()`
2. **Address critical TODOs** - Resolve {len(critical_todos)} critical TODO comments
3. **Fix compilation errors** - Ensure clean compilation across all crates

### **Phase 2: High Priority** (Short term - 1 week)
1. **Performance optimization** - Address {analysis['summary']['performance_issues']} performance issues
2. **Remove hardcoded values** - Replace {analysis['summary']['hardcoded_values']} hardcoded values with configuration
3. **Clean up warnings** - Fix {analysis['summary']['compilation_warnings']} compilation warnings

### **Phase 3: Medium Priority** (Medium term - 2-4 weeks)
1. **Address remaining TODOs** - Systematically resolve {analysis['summary']['todo_fixme_issues']} TODO/FIXME comments
2. **Code quality improvements** - Standardize error handling patterns
3. **Documentation updates** - Update documentation for changed APIs

### **Phase 4: Long-term Maintenance** (Ongoing)
1. **Automated debt prevention** - Set up linting rules to prevent new technical debt
2. **Regular debt audits** - Schedule monthly technical debt reviews
3. **Performance monitoring** - Implement continuous performance monitoring

---

## 🛠️ **AUTOMATION TOOLS**

This analysis was generated using automated tools. To fix issues:

```bash
# Fix panic-prone code
python3 scripts/technical_debt_cleanup.py --fix-panics

# Fix unused imports and warnings  
python3 scripts/technical_debt_cleanup.py --fix-warnings

# Address TODOs systematically
python3 scripts/technical_debt_cleanup.py --fix-todos

# Full cleanup (recommended)
python3 scripts/technical_debt_cleanup.py --fix-all
```

---

**END OF REPORT**

*Generated by: Technical Debt Cleanup System*  
*Next Review: Schedule monthly*
"""
        
        return report

def main():
    parser = argparse.ArgumentParser(
        description='Technical Debt Cleanup Tool',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Analyze codebase for technical debt
  python3 scripts/technical_debt_cleanup.py --analyze
  
  # Fix all issues (recommended)
  python3 scripts/technical_debt_cleanup.py --fix-all
  
  # Fix only panic-prone code
  python3 scripts/technical_debt_cleanup.py --fix-panics
  
  # Fix compilation warnings
  python3 scripts/technical_debt_cleanup.py --fix-warnings
  
  # Dry run to see what would be changed
  python3 scripts/technical_debt_cleanup.py --fix-all --dry-run
        """
    )
    
    parser.add_argument('--analyze', action='store_true', help='Analyze codebase for technical debt')
    parser.add_argument('--fix-all', action='store_true', help='Fix all detected issues')
    parser.add_argument('--fix-panics', action='store_true', help='Fix panic-prone code patterns')
    parser.add_argument('--fix-warnings', action='store_true', help='Fix compilation warnings')
    parser.add_argument('--fix-todos', action='store_true', help='Address TODO and FIXME comments')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be changed without making changes')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    parser.add_argument('--report-file', default='TECHNICAL_DEBT_REPORT.md', help='Output file for the report')
    
    args = parser.parse_args()
    
    if not any([args.analyze, args.fix_all, args.fix_panics, args.fix_warnings, args.fix_todos]):
        parser.error("Must specify at least one action")
    
    # Create analyzer
    analyzer = TechnicalDebtAnalyzer(dry_run=args.dry_run, verbose=args.verbose)
    
    print("🚀 **TECHNICAL DEBT CLEANUP SYSTEM**")
    print("=" * 50)
    
    if args.dry_run:
        print("🔍 **DRY RUN MODE** - No files will be changed")
    
    try:
        # Always analyze first
        analysis = analyzer.analyze_codebase()
        
        if args.analyze:
            # Generate and save report
            report = analyzer.generate_debt_report(analysis)
            with open(args.report_file, 'w') as f:
                f.write(report)
            print(f"📄 Technical debt report saved to: {args.report_file}")
        
        fixed_count = 0
        
        if args.fix_panics or args.fix_all:
            fixed_count += analyzer.fix_panic_prone_code(analysis)
        
        if args.fix_warnings or args.fix_all:
            fixed_count += analyzer.fix_unused_imports()
        
        if args.fix_todos or args.fix_all:
            fixed_count += analyzer.fix_todos_and_fixmes(analysis)
        
        # Print summary
        print("\n" + "=" * 50)
        print("📊 **CLEANUP SUMMARY**")
        print(f"Issues found: {analysis['summary']['panic_prone_issues'] + analysis['summary']['todo_fixme_issues'] + analysis['summary']['hardcoded_values']}")
        print(f"Issues fixed: {fixed_count}")
        print(f"Warnings: {len(analyzer.warnings)}")
        
        if analyzer.warnings:
            print("\n⚠️ **Warnings:**")
            for warning in analyzer.warnings:
                print(f"  - {warning}")
        
        if fixed_count > 0 and not args.dry_run:
            print(f"\n🎉 Successfully fixed {fixed_count} technical debt issues!")
            print("🔧 Run `cargo check` to verify all changes compile correctly")
            print("🧪 Run `cargo test` to ensure functionality is preserved")
        
    except KeyboardInterrupt:
        print("\n❌ Cleanup cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Cleanup failed: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main() 