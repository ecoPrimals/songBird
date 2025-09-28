#!/usr/bin/env python3
"""
Constants Consolidation Script - Phase 3

This script identifies remaining duplicate constants across the codebase and 
consolidates them into the canonical unified_constants.rs file.

Usage:
    python3 scripts/consolidate_constants.py --scan     # Scan for duplicate constants
    python3 scripts/consolidate_constants.py --merge    # Merge duplicates into canonical file
    python3 scripts/consolidate_constants.py --report   # Generate consolidation report
"""

import os
import re
import sys
import json
import argparse
from pathlib import Path
from typing import List, Dict, Set, Tuple
from dataclasses import dataclass, asdict
from collections import defaultdict

@dataclass
class ConstantDefinition:
    """Represents a constant definition found in the codebase"""
    name: str
    value: str
    file_path: str
    line_number: int
    constant_type: str  # 'const', 'static', 'pub const', etc.
    module_path: str
    is_duplicate: bool = False

class ConstantsConsolidator:
    """Main class for consolidating duplicate constants"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.constants: List[ConstantDefinition] = []
        self.duplicates: Dict[str, List[ConstantDefinition]] = defaultdict(list)
        
        # Patterns to match constant definitions
        self.const_patterns = [
            # pub const CONSTANT_NAME: type = value;
            r'pub\s+const\s+([A-Z_][A-Z0-9_]*)\s*:\s*([^=]+)\s*=\s*([^;]+);',
            # const CONSTANT_NAME: type = value;
            r'(?:^|\s)const\s+([A-Z_][A-Z0-9_]*)\s*:\s*([^=]+)\s*=\s*([^;]+);',
            # pub static CONSTANT_NAME: type = value;
            r'pub\s+static\s+([A-Z_][A-Z0-9_]*)\s*:\s*([^=]+)\s*=\s*([^;]+);',
        ]
        
        # Constants that should be consolidated (high-priority duplicates)
        self.priority_constants = {
            'DEFAULT_PORT', 'DEFAULT_HTTP_PORT', 'DEFAULT_HTTPS_PORT',
            'DEFAULT_TIMEOUT', 'CONNECTION_TIMEOUT', 'REQUEST_TIMEOUT',
            'MAX_CONNECTIONS', 'DEFAULT_MAX_CONNECTIONS', 'MAX_CONCURRENT_CONNECTIONS',
            'DEFAULT_BUFFER_SIZE', 'BUFFER_SIZE', 'MAX_BUFFER_SIZE',
            'DEFAULT_BIND_ADDRESS', 'LOCALHOST_BIND_ADDRESS', 'PRODUCTION_BIND_ADDRESS',
            'DEFAULT_HEALTH_CHECK_INTERVAL', 'HEALTH_CHECK_TIMEOUT',
            'DEFAULT_DISCOVERY_PORT', 'DISCOVERY_PORT',
        }
        
        # Canonical constants file
        self.canonical_file = self.project_root / "crates/songbird-types/src/unified_constants.rs"
    
    def scan_for_constants(self) -> List[ConstantDefinition]:
        """Scan the codebase for constant definitions"""
        print("🔍 Scanning for constant definitions...")
        
        # Scan Rust files
        for rust_file in self.project_root.glob("**/*.rs"):
            if self._should_skip_file(rust_file):
                continue
            self._scan_file(rust_file)
        
        # Identify duplicates
        self._identify_duplicates()
        
        print(f"📊 Found {len(self.constants)} constants, {len(self.duplicates)} with duplicates")
        return self.constants
    
    def _should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped during scanning"""
        skip_patterns = [
            "target/",
            "archive/",
            ".git/",
            "node_modules/",
            "scripts/",
        ]
        
        file_str = str(file_path)
        return any(pattern in file_str for pattern in skip_patterns)
    
    def _scan_file(self, file_path: Path) -> None:
        """Scan a single Rust file for constant definitions"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            for line_num, line in enumerate(lines, 1):
                self._check_line_for_constants(file_path, line_num, line)
                
        except Exception as e:
            print(f"⚠️  Error scanning {file_path}: {e}")
    
    def _check_line_for_constants(self, file_path: Path, line_num: int, line: str) -> None:
        """Check a single line for constant definitions"""
        line_stripped = line.strip()
        
        if line_stripped.startswith('//') or line_stripped.startswith('///'):
            return  # Skip comments
        
        for pattern in self.const_patterns:
            match = re.search(pattern, line_stripped)
            if match:
                const_name = match.group(1)
                const_type = match.group(2).strip() if len(match.groups()) >= 2 else "unknown"
                const_value = match.group(3).strip() if len(match.groups()) >= 3 else "unknown"
                
                # Determine module path
                module_path = self._get_module_path(file_path)
                
                # Determine constant type
                const_decl_type = "pub const" if "pub" in line_stripped else "const"
                if "static" in line_stripped:
                    const_decl_type = "pub static" if "pub" in line_stripped else "static"
                
                constant = ConstantDefinition(
                    name=const_name,
                    value=const_value,
                    file_path=str(file_path),
                    line_number=line_num,
                    constant_type=const_decl_type,
                    module_path=module_path
                )
                
                self.constants.append(constant)
                break
    
    def _get_module_path(self, file_path: Path) -> str:
        """Get the module path for a file"""
        try:
            # Convert file path to module path
            relative_path = file_path.relative_to(self.project_root)
            parts = list(relative_path.parts)
            
            # Remove 'crates' and crate name if present
            if len(parts) > 2 and parts[0] == 'crates':
                parts = parts[2:]  # Remove 'crates' and crate name
            
            # Remove 'src' if present
            if parts and parts[0] == 'src':
                parts = parts[1:]
            
            # Remove file extension and convert to module path
            if parts:
                parts[-1] = parts[-1].replace('.rs', '')
                if parts[-1] == 'mod':
                    parts = parts[:-1]  # Remove 'mod' from path
            
            return '::'.join(parts) if parts else 'root'
            
        except Exception:
            return str(file_path)
    
    def _identify_duplicates(self) -> None:
        """Identify duplicate constants by name"""
        name_counts = defaultdict(list)
        
        for constant in self.constants:
            name_counts[constant.name].append(constant)
        
        # Find duplicates
        for name, constants_list in name_counts.items():
            if len(constants_list) > 1:
                self.duplicates[name] = constants_list
                # Mark as duplicates
                for const in constants_list:
                    const.is_duplicate = True
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive constants consolidation report"""
        report = {
            'scan_timestamp': str(Path.cwd()),
            'total_constants': len(self.constants),
            'duplicate_names': len(self.duplicates),
            'high_priority_duplicates': [],
            'consolidation_candidates': [],
            'by_module': defaultdict(int),
            'by_type': defaultdict(int),
        }
        
        # Analyze by module and type
        for const in self.constants:
            report['by_module'][const.module_path] += 1
            report['by_type'][const.constant_type] += 1
        
        # Find high-priority duplicates
        for name, duplicates_list in self.duplicates.items():
            if name in self.priority_constants:
                report['high_priority_duplicates'].append({
                    'name': name,
                    'count': len(duplicates_list),
                    'locations': [f"{const.file_path}:{const.line_number}" for const in duplicates_list],
                    'values': list(set(const.value for const in duplicates_list))
                })
        
        # All duplicates are consolidation candidates
        for name, duplicates_list in self.duplicates.items():
            if len(duplicates_list) > 1:
                report['consolidation_candidates'].append({
                    'name': name,
                    'count': len(duplicates_list),
                    'locations': [f"{const.file_path}:{const.line_number}" for const in duplicates_list],
                    'canonical_value': self._determine_canonical_value(duplicates_list)
                })
        
        return report
    
    def _determine_canonical_value(self, duplicates_list: List[ConstantDefinition]) -> str:
        """Determine the canonical value for a set of duplicate constants"""
        # Use the most common value, or the first one if all are different
        value_counts = defaultdict(int)
        for const in duplicates_list:
            value_counts[const.value] += 1
        
        # Return the most common value
        return max(value_counts.items(), key=lambda x: x[1])[0]
    
    def consolidate_constants(self, dry_run: bool = True) -> Dict:
        """Consolidate duplicate constants into the canonical file"""
        print(f"🔧 {'[DRY RUN] ' if dry_run else ''}Consolidating duplicate constants...")
        
        consolidation_stats = {
            'constants_added': 0,
            'duplicates_removed': 0,
            'files_modified': 0,
        }
        
        # Generate consolidated constants to add to canonical file
        constants_to_add = []
        
        # Process high-priority duplicates first
        for name in self.priority_constants:
            if name in self.duplicates:
                duplicates_list = self.duplicates[name]
                canonical_value = self._determine_canonical_value(duplicates_list)
                
                # Create canonical constant definition
                canonical_const = f"    /// {name} - consolidated from {len(duplicates_list)} locations\n"
                canonical_const += f"    pub const {name}: {duplicates_list[0].constant_type.split()[-1]} = {canonical_value};\n"
                constants_to_add.append(canonical_const)
                
                consolidation_stats['constants_added'] += 1
        
        if not dry_run and constants_to_add:
            self._add_to_canonical_file(constants_to_add)
        
        return consolidation_stats
    
    def _add_to_canonical_file(self, constants_to_add: List[str]) -> None:
        """Add consolidated constants to the canonical file"""
        try:
            # Read existing canonical file
            if self.canonical_file.exists():
                with open(self.canonical_file, 'r', encoding='utf-8') as f:
                    content = f.read()
            else:
                content = self._create_canonical_file_template()
            
            # Find insertion point (before the last closing brace)
            insertion_point = content.rfind('}')
            if insertion_point == -1:
                insertion_point = len(content)
            
            # Insert new constants
            new_content = content[:insertion_point]
            new_content += "\n// ============================================================================\n"
            new_content += "// CONSOLIDATED CONSTANTS - Added by consolidation script\n"
            new_content += "// ============================================================================\n\n"
            new_content += "".join(constants_to_add)
            new_content += "\n" + content[insertion_point:]
            
            # Write back to file
            with open(self.canonical_file, 'w', encoding='utf-8') as f:
                f.write(new_content)
            
            print(f"✅ Added {len(constants_to_add)} consolidated constants to {self.canonical_file}")
            
        except Exception as e:
            print(f"⚠️  Error updating canonical file: {e}")
    
    def _create_canonical_file_template(self) -> str:
        """Create a template for the canonical constants file if it doesn't exist"""
        return """//! # 🔧 Consolidated Constants - Generated by consolidation script
//!
//! **CONSOLIDATED DUPLICATE CONSTANTS** ✅
//!
//! This section contains constants that were duplicated across multiple files
//! and have been consolidated into this canonical location.

pub mod consolidated {
    use std::time::Duration;
    
    // Consolidated constants will be added here
}
"""

def main():
    parser = argparse.ArgumentParser(description="Consolidate duplicate constants in Songbird")
    parser.add_argument("--scan", action="store_true", help="Scan for duplicate constants")
    parser.add_argument("--merge", action="store_true", help="Merge duplicates into canonical file")
    parser.add_argument("--report", action="store_true", help="Generate consolidation report")
    parser.add_argument("--dry-run", action="store_true", help="Perform dry run (no actual changes)")
    parser.add_argument("--output", help="Output file for report (JSON format)")
    
    args = parser.parse_args()
    
    if not any([args.scan, args.merge, args.report]):
        parser.print_help()
        return
    
    # Determine project root (assume script is in scripts/ directory)
    project_root = Path(__file__).parent.parent
    
    consolidator = ConstantsConsolidator(str(project_root))
    
    if args.scan or args.merge or args.report:
        constants = consolidator.scan_for_constants()
    
    if args.report:
        report = consolidator.generate_report()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(report, f, indent=2, default=str)
            print(f"📊 Report saved to {args.output}")
        else:
            print("\n📊 CONSTANTS CONSOLIDATION REPORT")
            print("=" * 50)
            print(f"Total constants found: {report['total_constants']}")
            print(f"Duplicate constant names: {report['duplicate_names']}")
            print(f"High priority duplicates: {len(report['high_priority_duplicates'])}")
            print(f"Consolidation candidates: {len(report['consolidation_candidates'])}")
            
            if report['high_priority_duplicates']:
                print("\n🎯 High Priority Duplicates:")
                for duplicate in report['high_priority_duplicates'][:5]:  # Show top 5
                    print(f"  • {duplicate['name']}: {duplicate['count']} instances")
                    print(f"    Values: {', '.join(duplicate['values'][:3])}{'...' if len(duplicate['values']) > 3 else ''}")
    
    if args.merge:
        dry_run = args.dry_run
        stats = consolidator.consolidate_constants(dry_run)
        
        print(f"\n🔧 {'[DRY RUN] ' if dry_run else ''}CONSOLIDATION SUMMARY")
        print("=" * 50)
        print(f"Constants added to canonical file: {stats['constants_added']}")
        
        if not dry_run:
            print("\n✅ Constants consolidation complete!")
            print("⚠️  Please update imports in affected files to use canonical constants")
        else:
            print("\n🔍 Dry run complete. Use --merge without --dry-run to apply changes")

if __name__ == "__main__":
    main()
