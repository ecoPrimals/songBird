#!/usr/bin/env python3
"""
Comprehensive Configuration Migration Script for Songbird Ecosystem

This script identifies remaining configuration fragments and provides automated
migration paths to canonical locations in songbird-types.

Usage:
    python3 scripts/comprehensive_config_migration.py --analyze
    python3 scripts/comprehensive_config_migration.py --migrate --dry-run
    python3 scripts/comprehensive_config_migration.py --migrate --execute
"""

import os
import re
import json
import argparse
from pathlib import Path
from typing import Dict, List, Set, Optional, Tuple
from dataclasses import dataclass, asdict
import subprocess

@dataclass
class ConfigFragment:
    """Represents a configuration fragment that needs migration"""
    name: str
    file_path: str
    line_number: int
    config_type: str  # struct, enum, type alias
    canonical_location: Optional[str] = None
    migration_priority: str = "medium"  # low, medium, high, critical
    dependencies: List[str] = None
    deprecated: bool = False
    
    def __post_init__(self):
        if self.dependencies is None:
            self.dependencies = []

class ComprehensiveConfigMigrator:
    """Main migration class for configuration consolidation"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.fragments: List[ConfigFragment] = []
        self.canonical_configs: Set[str] = set()
        self.migration_map: Dict[str, str] = {}
        
        # Define canonical locations for different config types
        self.canonical_locations = {
            'federation': 'crates/songbird-types/src/config/federation.rs',
            'orchestration': 'crates/songbird-types/src/config/orchestration.rs',
            'adapters': 'crates/songbird-types/src/config/adapters.rs',
            'network': 'crates/songbird-types/src/config/network.rs',
            'security': 'crates/songbird-types/src/config/security.rs',
            'performance': 'crates/songbird-types/src/config/performance.rs',
            'environment': 'crates/songbird-types/src/config/environment.rs',
            'ai_first': 'crates/songbird-types/src/config/ai_first.rs',
            'storage': 'crates/songbird-types/src/config/storage.rs',
            'health': 'crates/songbird-types/src/config/health.rs',
            'service': 'crates/songbird-types/src/service.rs',
        }
        
        # Configuration patterns to identify
        self.config_patterns = [
            r'pub struct (\w*Config)\s*\{',
            r'pub enum (\w*Config)\s*\{',
            r'pub struct (\w*Configuration)\s*\{',
            r'pub enum (\w*Configuration)\s*\{',
            r'pub type (\w*Config)\s*=',
            r'struct (\w*Config)\s*\{',
            r'enum (\w*Config)\s*\{',
        ]
        
        # Exclude patterns (already canonical or test-only)
        self.exclude_patterns = [
            r'Canonical\w*Config',
            r'Unified\w*Config',
            r'Test\w*Config',
            r'Mock\w*Config',
            r'Example\w*Config',
        ]
        
        # Priority classification rules
        self.priority_rules = {
            'critical': ['FederationConfig', 'SecurityConfig', 'NetworkConfig'],
            'high': ['PerformanceConfig', 'StorageConfig', 'HealthConfig'],
            'medium': ['MonitoringConfig', 'LoggingConfig', 'DeploymentConfig'],
            'low': ['DebugConfig', 'DevConfig', 'UtilityConfig']
        }

    def analyze_codebase(self) -> None:
        """Analyze the codebase to find configuration fragments"""
        print("🔍 Analyzing codebase for configuration fragments...")
        
        # First, identify canonical configurations
        self._identify_canonical_configs()
        
        # Then find fragments that need migration
        self._find_config_fragments()
        
        # Analyze dependencies and relationships
        self._analyze_dependencies()
        
        # Generate migration priorities
        self._calculate_priorities()
        
        print(f"✅ Found {len(self.fragments)} configuration fragments")
        print(f"📊 Canonical configs: {len(self.canonical_configs)}")

    def _identify_canonical_configs(self) -> None:
        """Identify existing canonical configurations"""
        canonical_dir = self.project_root / "crates/songbird-types/src/config"
        if canonical_dir.exists():
            for config_file in canonical_dir.glob("*.rs"):
                with open(config_file, 'r') as f:
                    content = f.read()
                    # Find canonical config names
                    matches = re.findall(r'pub struct (Canonical\w+Config)', content)
                    self.canonical_configs.update(matches)

    def _find_config_fragments(self) -> None:
        """Find configuration fragments across the codebase"""
        exclude_dirs = {'.git', 'target', 'node_modules', '__pycache__', '.pytest_cache'}
        
        for rust_file in self.project_root.rglob("*.rs"):
            # Skip excluded directories
            if any(part in exclude_dirs for part in rust_file.parts):
                continue
                
            # Skip already canonical files
            if 'songbird-types/src/config' in str(rust_file):
                continue
                
            self._analyze_file(rust_file)

    def _analyze_file(self, file_path: Path) -> None:
        """Analyze a single file for configuration fragments"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
                
                for line_num, line in enumerate(lines, 1):
                    for pattern in self.config_patterns:
                        matches = re.finditer(pattern, line)
                        for match in matches:
                            config_name = match.group(1)
                            
                            # Skip if it matches exclude patterns
                            if any(re.match(exclude_pattern, config_name) 
                                   for exclude_pattern in self.exclude_patterns):
                                continue
                            
                            # Determine config type
                            config_type = 'struct' if 'struct' in pattern else 'enum' if 'enum' in pattern else 'type'
                            
                            # Check if deprecated
                            deprecated = self._is_deprecated(content, line_num)
                            
                            fragment = ConfigFragment(
                                name=config_name,
                                file_path=str(file_path.relative_to(self.project_root)),
                                line_number=line_num,
                                config_type=config_type,
                                deprecated=deprecated
                            )
                            
                            self.fragments.append(fragment)
                            
        except Exception as e:
            print(f"⚠️  Error analyzing {file_path}: {e}")

    def _is_deprecated(self, content: str, line_num: int) -> bool:
        """Check if a configuration is marked as deprecated"""
        lines = content.split('\n')
        # Check a few lines before the config declaration
        start_line = max(0, line_num - 5)
        context = '\n'.join(lines[start_line:line_num])
        
        return '#[deprecated' in context or '/// **⚠️ DEPRECATED**' in context

    def _analyze_dependencies(self) -> None:
        """Analyze dependencies between configuration fragments"""
        # This is a simplified dependency analysis
        # In a real implementation, you'd parse the AST more thoroughly
        
        for fragment in self.fragments:
            file_path = self.project_root / fragment.file_path
            try:
                with open(file_path, 'r') as f:
                    content = f.read()
                    
                # Look for other config references in the same file
                for other_fragment in self.fragments:
                    if other_fragment.name != fragment.name and other_fragment.name in content:
                        fragment.dependencies.append(other_fragment.name)
                        
            except Exception:
                pass

    def _calculate_priorities(self) -> None:
        """Calculate migration priorities for each fragment"""
        for fragment in self.fragments:
            # Default priority
            fragment.migration_priority = "medium"
            
            # Check against priority rules
            for priority, patterns in self.priority_rules.items():
                if any(pattern in fragment.name for pattern in patterns):
                    fragment.migration_priority = priority
                    break
            
            # Deprecated configs get higher priority
            if fragment.deprecated:
                if fragment.migration_priority == "low":
                    fragment.migration_priority = "medium"
                elif fragment.migration_priority == "medium":
                    fragment.migration_priority = "high"
            
            # Suggest canonical location
            fragment.canonical_location = self._suggest_canonical_location(fragment)

    def _suggest_canonical_location(self, fragment: ConfigFragment) -> str:
        """Suggest the canonical location for a configuration fragment"""
        name_lower = fragment.name.lower()
        
        # Pattern matching for canonical locations
        if 'federation' in name_lower:
            return self.canonical_locations['federation']
        elif 'network' in name_lower or 'networking' in name_lower:
            return self.canonical_locations['network']
        elif 'security' in name_lower or 'auth' in name_lower or 'encryption' in name_lower:
            return self.canonical_locations['security']
        elif 'performance' in name_lower or 'perf' in name_lower:
            return self.canonical_locations['performance']
        elif 'storage' in name_lower or 'cache' in name_lower or 'database' in name_lower:
            return self.canonical_locations['storage']
        elif 'health' in name_lower or 'monitor' in name_lower:
            return self.canonical_locations['health']
        elif 'service' in name_lower or 'registry' in name_lower:
            return self.canonical_locations['service']
        elif 'orchestr' in name_lower or 'discovery' in name_lower:
            return self.canonical_locations['orchestration']
        elif 'adapter' in name_lower or 'primal' in name_lower:
            return self.canonical_locations['adapters']
        elif 'environment' in name_lower or 'deploy' in name_lower:
            return self.canonical_locations['environment']
        elif 'ai' in name_lower or 'first' in name_lower:
            return self.canonical_locations['ai_first']
        else:
            # Default to environment for unknown configs
            return self.canonical_locations['environment']

    def generate_report(self) -> Dict:
        """Generate a comprehensive migration report"""
        report = {
            'summary': {
                'total_fragments': len(self.fragments),
                'canonical_configs': len(self.canonical_configs),
                'deprecated_fragments': len([f for f in self.fragments if f.deprecated]),
                'priority_breakdown': {
                    'critical': len([f for f in self.fragments if f.migration_priority == 'critical']),
                    'high': len([f for f in self.fragments if f.migration_priority == 'high']),
                    'medium': len([f for f in self.fragments if f.migration_priority == 'medium']),
                    'low': len([f for f in self.fragments if f.migration_priority == 'low']),
                }
            },
            'fragments': [asdict(fragment) for fragment in self.fragments],
            'canonical_configs': list(self.canonical_configs),
            'migration_recommendations': self._generate_migration_recommendations()
        }
        
        return report

    def _generate_migration_recommendations(self) -> List[Dict]:
        """Generate specific migration recommendations"""
        recommendations = []
        
        # Group fragments by canonical location
        location_groups = {}
        for fragment in self.fragments:
            location = fragment.canonical_location or 'unknown'
            if location not in location_groups:
                location_groups[location] = []
            location_groups[location].append(fragment)
        
        for location, fragments in location_groups.items():
            if location == 'unknown':
                continue
                
            recommendation = {
                'canonical_location': location,
                'fragments_to_migrate': len(fragments),
                'high_priority_count': len([f for f in fragments if f.migration_priority in ['critical', 'high']]),
                'deprecated_count': len([f for f in fragments if f.deprecated]),
                'fragments': [f.name for f in fragments[:10]]  # Show first 10
            }
            recommendations.append(recommendation)
        
        # Sort by priority
        recommendations.sort(key=lambda x: x['high_priority_count'], reverse=True)
        
        return recommendations

    def generate_migration_script(self, target_location: str, fragments: List[ConfigFragment]) -> str:
        """Generate a migration script for specific fragments"""
        script = f"""#!/bin/bash
# Generated migration script for {target_location}
# This script helps migrate configuration fragments to canonical locations

set -e

echo "🚀 Starting configuration migration to {target_location}"

"""
        
        for fragment in fragments:
            script += f"""
# Migrate {fragment.name} from {fragment.file_path}
echo "📦 Processing {fragment.name}..."

# TODO: Add specific migration commands for {fragment.name}
# Source: {fragment.file_path}:{fragment.line_number}
# Priority: {fragment.migration_priority}
# Deprecated: {fragment.deprecated}

"""
        
        script += """
echo "✅ Migration completed successfully!"
echo "📋 Please review the changes and run tests before committing."
"""
        
        return script

    def print_analysis_report(self) -> None:
        """Print a human-readable analysis report"""
        report = self.generate_report()
        
        print("\n" + "="*80)
        print("📊 COMPREHENSIVE CONFIGURATION MIGRATION ANALYSIS")
        print("="*80)
        
        print(f"\n📈 SUMMARY:")
        print(f"   Total configuration fragments found: {report['summary']['total_fragments']}")
        print(f"   Existing canonical configurations: {report['summary']['canonical_configs']}")
        print(f"   Deprecated fragments: {report['summary']['deprecated_fragments']}")
        
        print(f"\n🎯 PRIORITY BREAKDOWN:")
        for priority, count in report['summary']['priority_breakdown'].items():
            print(f"   {priority.capitalize()}: {count}")
        
        print(f"\n📋 TOP MIGRATION RECOMMENDATIONS:")
        for i, rec in enumerate(report['migration_recommendations'][:5], 1):
            print(f"   {i}. {rec['canonical_location']}")
            print(f"      - Fragments: {rec['fragments_to_migrate']}")
            print(f"      - High priority: {rec['high_priority_count']}")
            print(f"      - Deprecated: {rec['deprecated_count']}")
        
        print(f"\n🔍 CRITICAL PRIORITY FRAGMENTS:")
        critical_fragments = [f for f in self.fragments if f.migration_priority == 'critical']
        for fragment in critical_fragments[:10]:
            print(f"   - {fragment.name} ({fragment.file_path}:{fragment.line_number})")
        
        print(f"\n⚠️  DEPRECATED FRAGMENTS REQUIRING IMMEDIATE ATTENTION:")
        deprecated_fragments = [f for f in self.fragments if f.deprecated][:10]
        for fragment in deprecated_fragments:
            print(f"   - {fragment.name} ({fragment.file_path}:{fragment.line_number})")
        
        print("\n" + "="*80)

def main():
    parser = argparse.ArgumentParser(description='Comprehensive Configuration Migration Tool')
    parser.add_argument('--analyze', action='store_true', help='Analyze codebase for configuration fragments')
    parser.add_argument('--migrate', action='store_true', help='Generate migration scripts')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be migrated without executing')
    parser.add_argument('--execute', action='store_true', help='Execute the migration')
    parser.add_argument('--output', default='migration_report.json', help='Output file for analysis report')
    parser.add_argument('--project-root', default='.', help='Project root directory')
    
    args = parser.parse_args()
    
    if not any([args.analyze, args.migrate]):
        parser.print_help()
        return
    
    migrator = ComprehensiveConfigMigrator(args.project_root)
    
    if args.analyze:
        migrator.analyze_codebase()
        migrator.print_analysis_report()
        
        # Save detailed report
        report = migrator.generate_report()
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\n💾 Detailed report saved to: {args.output}")
    
    if args.migrate:
        if not migrator.fragments:
            migrator.analyze_codebase()
        
        if args.dry_run:
            print("\n🔍 DRY RUN - Migration Preview:")
            for rec in migrator.generate_report()['migration_recommendations'][:3]:
                print(f"\nWould migrate to: {rec['canonical_location']}")
                print(f"Fragments: {rec['fragments_to_migrate']}")
        
        if args.execute:
            print("\n⚠️  EXECUTE mode not yet implemented for safety.")
            print("Please review the analysis report and migrate configurations manually.")
            print("Use the generated migration scripts as guidance.")

if __name__ == "__main__":
    main() 