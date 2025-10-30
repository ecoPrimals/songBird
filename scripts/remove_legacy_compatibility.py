#!/usr/bin/env python3
"""
Legacy Compatibility Removal Script - Phase 2

This script identifies and removes legacy compatibility layers, backward compatibility
wrappers, and deprecated environment variable support across the Songbird codebase.

Usage:
    python scripts/remove_legacy_compatibility.py --scan    # Scan for legacy code
    python scripts/remove_legacy_compatibility.py --remove  # Remove legacy code
    python scripts/remove_legacy_compatibility.py --report  # Generate report
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Set, Tuple
from dataclasses import dataclass, asdict

@dataclass
class LegacyItem:
    """Represents a legacy compatibility item found in the codebase"""
    file_path: str
    line_number: int
    item_type: str  # 'feature_flag', 'env_var', 'deprecated_trait', 'compatibility_layer'
    content: str
    severity: str  # 'high', 'medium', 'low'
    migration_suggestion: str

class LegacyCompatibilityRemover:
    """Main class for identifying and removing legacy compatibility code"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.legacy_items: List[LegacyItem] = []
        
        # Patterns to identify legacy compatibility code
        self.legacy_patterns = {
            'feature_flags': [
                r'#\[cfg\(feature\s*=\s*["\']legacy-compat["\']',
                r'#\[cfg\(feature\s*=\s*["\']backward-compat["\']',
                r'#\[cfg\(feature\s*=\s*["\']legacy["\']',
            ],
            'env_vars': [
                r'SONGBIRD_LEGACY_',
                r'legacy-[a-z-]+',
                r'LEGACY_[A-Z_]+',
                r'backward_compat',
                r'legacy_primal_names',
            ],
            'deprecated_traits': [
                r'#\[deprecated.*Use.*instead.*\]',
                r'pub\s+use.*// Legacy compatibility',
                r'pub\s+use.*// Backward compatibility',
            ],
            'compatibility_layers': [
                r'pub\s+mod\s+legacy\s*\{',
                r'pub\s+mod\s+compat\s*\{',
                r'pub\s+mod\s+backward_compat\s*\{',
                r'// Legacy compatibility',
                r'// Backward compatibility',
                r'// For backward compatibility',
            ]
        }
        
        # High-priority removal targets
        self.high_priority_removals = {
            'legacy-compat feature flags',
            'deprecated environment variables',
            'backward compatibility wrappers',
            'legacy primal name mapping',
        }
    
    def scan_for_legacy_code(self) -> List[LegacyItem]:
        """Scan the codebase for legacy compatibility code"""
        print("🔍 Scanning for legacy compatibility code...")
        
        # Scan Rust files
        for rust_file in self.project_root.glob("**/*.rs"):
            if self._should_skip_file(rust_file):
                continue
            self._scan_file(rust_file)
        
        # Scan TOML files for feature flags
        for toml_file in self.project_root.glob("**/Cargo.toml"):
            self._scan_toml_file(toml_file)
        
        print(f"📊 Found {len(self.legacy_items)} legacy compatibility items")
        return self.legacy_items
    
    def _should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped during scanning"""
        skip_patterns = [
            "target/",
            "archive/",
            ".git/",
            "node_modules/",
            "scripts/",  # Skip this script itself
        ]
        
        file_str = str(file_path)
        return any(pattern in file_str for pattern in skip_patterns)
    
    def _scan_file(self, file_path: Path) -> None:
        """Scan a single Rust file for legacy patterns"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            for line_num, line in enumerate(lines, 1):
                self._check_line_for_legacy(file_path, line_num, line)
                
        except Exception as e:
            print(f"⚠️  Error scanning {file_path}: {e}")
    
    def _scan_toml_file(self, file_path: Path) -> None:
        """Scan a TOML file for legacy feature flags"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Look for legacy feature flags
            if 'legacy-compat' in content or 'backward-compat' in content:
                self.legacy_items.append(LegacyItem(
                    file_path=str(file_path),
                    line_number=0,
                    item_type='feature_flag',
                    content=f"Legacy feature flags in {file_path.name}",
                    severity='high',
                    migration_suggestion="Remove legacy-compat and backward-compat feature flags"
                ))
                
        except Exception as e:
            print(f"⚠️  Error scanning TOML {file_path}: {e}")
    
    def _check_line_for_legacy(self, file_path: Path, line_num: int, line: str) -> None:
        """Check a single line for legacy patterns"""
        line_stripped = line.strip()
        
        # Check each pattern category
        for category, patterns in self.legacy_patterns.items():
            for pattern in patterns:
                if re.search(pattern, line_stripped, re.IGNORECASE):
                    severity = self._determine_severity(category, line_stripped)
                    suggestion = self._get_migration_suggestion(category, line_stripped)
                    
                    self.legacy_items.append(LegacyItem(
                        file_path=str(file_path),
                        line_number=line_num,
                        item_type=category,
                        content=line_stripped,
                        severity=severity,
                        migration_suggestion=suggestion
                    ))
                    break
    
    def _determine_severity(self, category: str, content: str) -> str:
        """Determine the severity level of a legacy item"""
        high_severity_indicators = [
            'legacy-compat',
            'backward-compat', 
            'LEGACY_',
            'deprecated',
        ]
        
        if any(indicator in content.lower() for indicator in high_severity_indicators):
            return 'high'
        elif category in ['feature_flags', 'env_vars']:
            return 'high'
        elif category == 'deprecated_traits':
            return 'medium'
        else:
            return 'low'
    
    def _get_migration_suggestion(self, category: str, content: str) -> str:
        """Get migration suggestion for a legacy item"""
        suggestions = {
            'feature_flags': "Remove legacy feature flag and update code to use canonical types",
            'env_vars': "Replace with canonical environment variables from unified config",
            'deprecated_traits': "Update to use canonical trait from songbird-types",
            'compatibility_layers': "Remove compatibility layer and use canonical APIs directly",
        }
        
        return suggestions.get(category, "Update to use canonical implementation")
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive report of legacy code findings"""
        report = {
            'scan_timestamp': str(Path.cwd()),
            'total_items': len(self.legacy_items),
            'by_severity': {},
            'by_type': {},
            'high_priority_files': [],
            'migration_priorities': [],
        }
        
        # Group by severity
        for item in self.legacy_items:
            severity = item.severity
            if severity not in report['by_severity']:
                report['by_severity'][severity] = 0
            report['by_severity'][severity] += 1
            
            # Group by type
            item_type = item.item_type
            if item_type not in report['by_type']:
                report['by_type'][item_type] = 0
            report['by_type'][item_type] += 1
        
        # Identify high priority files
        high_priority_files = set()
        for item in self.legacy_items:
            if item.severity == 'high':
                high_priority_files.add(item.file_path)
        
        report['high_priority_files'] = list(high_priority_files)
        
        # Create migration priorities
        priority_map = {
            'feature_flags': 1,
            'env_vars': 2, 
            'compatibility_layers': 3,
            'deprecated_traits': 4,
        }
        
        migration_priorities = []
        for item_type, count in report['by_type'].items():
            priority = priority_map.get(item_type, 5)
            migration_priorities.append({
                'type': item_type,
                'count': count,
                'priority': priority,
                'description': self._get_migration_description(item_type)
            })
        
        migration_priorities.sort(key=lambda x: x['priority'])
        report['migration_priorities'] = migration_priorities
        
        return report
    
    def _get_migration_description(self, item_type: str) -> str:
        """Get description for migration priority"""
        descriptions = {
            'feature_flags': "Remove legacy feature flags from Cargo.toml files",
            'env_vars': "Replace deprecated environment variables with canonical config",
            'compatibility_layers': "Remove backward compatibility modules and wrappers",
            'deprecated_traits': "Update trait usage to canonical implementations",
        }
        return descriptions.get(item_type, "Update legacy code patterns")
    
    def remove_legacy_code(self, dry_run: bool = True) -> Dict:
        """Remove identified legacy code (with dry-run option)"""
        print(f"🧹 {'[DRY RUN] ' if dry_run else ''}Removing legacy compatibility code...")
        
        removal_stats = {
            'files_modified': 0,
            'lines_removed': 0,
            'feature_flags_removed': 0,
            'env_vars_updated': 0,
        }
        
        # Group items by file for efficient processing
        files_to_modify = {}
        for item in self.legacy_items:
            if item.severity == 'high':  # Only remove high-severity items
                if item.file_path not in files_to_modify:
                    files_to_modify[item.file_path] = []
                files_to_modify[item.file_path].append(item)
        
        for file_path, items in files_to_modify.items():
            if self._remove_from_file(file_path, items, dry_run):
                removal_stats['files_modified'] += 1
                removal_stats['lines_removed'] += len(items)
        
        return removal_stats
    
    def _remove_from_file(self, file_path: str, items: List[LegacyItem], dry_run: bool) -> bool:
        """Remove legacy items from a specific file"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Sort items by line number in reverse order to maintain line numbers
            items_sorted = sorted(items, key=lambda x: x.line_number, reverse=True)
            
            lines_to_remove = set()
            for item in items_sorted:
                if item.line_number > 0:  # Skip items without specific line numbers
                    lines_to_remove.add(item.line_number - 1)  # Convert to 0-based index
            
            if not dry_run and lines_to_remove:
                # Remove lines in reverse order
                for line_index in sorted(lines_to_remove, reverse=True):
                    if 0 <= line_index < len(lines):
                        del lines[line_index]
                
                # Write back the modified file
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.writelines(lines)
                
                print(f"✅ Modified {file_path}: removed {len(lines_to_remove)} lines")
            elif dry_run:
                print(f"🔍 [DRY RUN] Would modify {file_path}: remove {len(lines_to_remove)} lines")
            
            return len(lines_to_remove) > 0
            
        except Exception as e:
            print(f"⚠️  Error processing {file_path}: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(description="Remove legacy compatibility code from Songbird")
    parser.add_argument("--scan", action="store_true", help="Scan for legacy code")
    parser.add_argument("--remove", action="store_true", help="Remove legacy code")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")
    parser.add_argument("--dry-run", action="store_true", help="Perform dry run (no actual changes)")
    parser.add_argument("--output", help="Output file for report (JSON format)")
    
    args = parser.parse_args()
    
    if not any([args.scan, args.remove, args.report]):
        parser.print_help()
        return
    
    # Determine project root (assume script is in scripts/ directory)
    project_root = Path(__file__).parent.parent
    
    remover = LegacyCompatibilityRemover(str(project_root))
    
    if args.scan or args.remove or args.report:
        legacy_items = remover.scan_for_legacy_code()
    
    if args.report:
        report = remover.generate_report()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(report, f, indent=2)
            print(f"📊 Report saved to {args.output}")
        else:
            print("\n📊 LEGACY COMPATIBILITY REPORT")
            print("=" * 50)
            print(f"Total legacy items found: {report['total_items']}")
            print(f"High severity: {report['by_severity'].get('high', 0)}")
            print(f"Medium severity: {report['by_severity'].get('medium', 0)}")
            print(f"Low severity: {report['by_severity'].get('low', 0)}")
            print(f"\nHigh priority files: {len(report['high_priority_files'])}")
            
            print("\n🎯 Migration Priorities:")
            for priority in report['migration_priorities']:
                print(f"  {priority['priority']}. {priority['type']}: {priority['count']} items")
                print(f"     {priority['description']}")
    
    if args.remove:
        dry_run = args.dry_run
        stats = remover.remove_legacy_code(dry_run)
        
        print(f"\n🧹 {'[DRY RUN] ' if dry_run else ''}REMOVAL SUMMARY")
        print("=" * 50)
        print(f"Files modified: {stats['files_modified']}")
        print(f"Lines removed: {stats['lines_removed']}")
        
        if not dry_run:
            print("\n✅ Legacy compatibility removal complete!")
            print("⚠️  Please run tests to ensure functionality is preserved")
        else:
            print("\n🔍 Dry run complete. Use --remove without --dry-run to apply changes")

if __name__ == "__main__":
    main() 