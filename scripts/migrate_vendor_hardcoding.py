#!/usr/bin/env python3
"""
🔄 Vendor Hardcoding Migration Script

This script helps identify and migrate hardcoded vendor names (beardog, toadstool, 
nestgate, squirrel) to capability-based discovery patterns.

Usage:
    python3 scripts/migrate_vendor_hardcoding.py --analyze      # Find hardcoded patterns
    python3 scripts/migrate_vendor_hardcoding.py --migrate      # Apply migrations
    python3 scripts/migrate_vendor_hardcoding.py --validate     # Validate migrations
"""

import os
import re
import argparse
from pathlib import Path
from typing import Dict, List, Tuple
import json

# Hardcoded vendor names to migrate
VENDOR_HARDCODING = {
    'beardog': 'security',
    'toadstool': 'compute', 
    'nestgate': 'storage',
    'squirrel': 'ai'
}

# Pattern mappings for migration
MIGRATION_PATTERNS = [
    # Function calls
    (r'register_beardog_provider\(\)', 'register_capability_provider("security", "any-security-vendor")'),
    (r'register_toadstool_provider\(\)', 'register_capability_provider("compute", "any-compute-vendor")'),
    (r'register_nestgate_provider\(\)', 'register_capability_provider("storage", "any-storage-vendor")'),
    (r'register_squirrel_provider\(\)', 'register_capability_provider("ai", "any-ai-vendor")'),
    
    # Endpoint getters
    (r'EcosystemEnvironmentConfig::beardog_endpoint\(\)', 'adapter.discover_by_capability("security").await?.first().map(|p| p.discovered_endpoint)'),
    (r'EcosystemEnvironmentConfig::toadstool_endpoint\(\)', 'adapter.discover_by_capability("compute").await?.first().map(|p| p.discovered_endpoint)'),
    (r'EcosystemEnvironmentConfig::nestgate_endpoint\(\)', 'adapter.discover_by_capability("storage").await?.first().map(|p| p.discovered_endpoint)'),
    (r'EcosystemEnvironmentConfig::squirrel_endpoint\(\)', 'adapter.discover_by_capability("ai").await?.first().map(|p| p.discovered_endpoint)'),
    
    # Direct instantiation
    (r'BearDogPrimal::new\(\)', 'adapter.request_capability("security", "initialize", json!({})).await?'),
    (r'ToadstoolPrimal::new\(\)', 'adapter.request_capability("compute", "initialize", json!({})).await?'),
    (r'NestgatePrimal::new\(\)', 'adapter.request_capability("storage", "initialize", json!({})).await?'),
    (r'SquirrelPrimal::new\(\)', 'adapter.request_capability("ai", "initialize", json!({})).await?'),
]

def find_rust_files(directory: str) -> List[Path]:
    """Find all Rust files in the given directory."""
    rust_files = []
    for root, dirs, files in os.walk(directory):
        # Skip target directory
        if 'target' in dirs:
            dirs.remove('target')
        if 'cache' in dirs:
            dirs.remove('cache')
        
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(Path(root) / file)
    return rust_files

def analyze_file(file_path: Path) -> Dict[str, List[Tuple[int, str]]]:
    """Analyze a file for hardcoded vendor patterns."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
    except Exception as e:
        print(f"❌ Error reading {file_path}: {e}")
        return {}
    
    findings = {vendor: [] for vendor in VENDOR_HARDCODING.keys()}
    
    lines = content.split('\n')
    for line_num, line in enumerate(lines, 1):
        for vendor in VENDOR_HARDCODING.keys():
            if re.search(rf'\b{vendor}\b', line, re.IGNORECASE):
                findings[vendor].append((line_num, line.strip()))
    
    # Remove empty findings
    return {k: v for k, v in findings.items() if v}

def analyze_codebase(directory: str) -> Dict[str, Dict[str, List[Tuple[int, str]]]]:
    """Analyze entire codebase for hardcoded vendor patterns."""
    print(f"🔍 Analyzing codebase in {directory} for vendor hardcoding...")
    
    rust_files = find_rust_files(directory)
    print(f"   Found {len(rust_files)} Rust files to analyze")
    
    results = {}
    files_with_issues = 0
    
    for file_path in rust_files:
        findings = analyze_file(file_path)
        if findings:
            results[str(file_path)] = findings
            files_with_issues += 1
    
    print(f"   📊 Analysis complete: {files_with_issues} files contain vendor hardcoding")
    return results

def generate_migration_report(analysis_results: Dict) -> None:
    """Generate a comprehensive migration report."""
    print("\n" + "="*80)
    print("📋 VENDOR HARDCODING MIGRATION REPORT")
    print("="*80)
    
    total_issues = 0
    vendor_counts = {vendor: 0 for vendor in VENDOR_HARDCODING.keys()}
    
    for file_path, findings in analysis_results.items():
        if findings:
            print(f"\n📁 {file_path}")
            for vendor, occurrences in findings.items():
                vendor_counts[vendor] += len(occurrences)
                total_issues += len(occurrences)
                
                print(f"   🔴 {vendor.upper()} ({len(occurrences)} occurrences):")
                for line_num, line_content in occurrences[:3]:  # Show first 3
                    print(f"      Line {line_num}: {line_content}")
                if len(occurrences) > 3:
                    print(f"      ... and {len(occurrences) - 3} more")
    
    print(f"\n📊 SUMMARY:")
    print(f"   Total hardcoded references: {total_issues}")
    for vendor, count in vendor_counts.items():
        capability = VENDOR_HARDCODING[vendor]
        print(f"   {vendor} → {capability}: {count} references")
    
    print(f"\n🎯 MIGRATION PRIORITY:")
    sorted_vendors = sorted(vendor_counts.items(), key=lambda x: x[1], reverse=True)
    for i, (vendor, count) in enumerate(sorted_vendors, 1):
        capability = VENDOR_HARDCODING[vendor]
        if count > 0:
            print(f"   {i}. {vendor} → {capability} capability ({count} refs)")

def create_migration_suggestions(analysis_results: Dict) -> Dict[str, List[str]]:
    """Create specific migration suggestions for each file."""
    suggestions = {}
    
    for file_path, findings in analysis_results.items():
        if not findings:
            continue
            
        file_suggestions = []
        
        # Add migration header comment
        file_suggestions.append(
            "// ⚠️ MIGRATION NOTICE: This file contains hardcoded vendor names"
        )
        file_suggestions.append(
            "// See PRIMAL_HARDCODING_MIGRATION_GUIDE.md for migration patterns"
        )
        
        # Add universal adapter import
        if any(findings.values()):
            file_suggestions.append(
                "use songbird_universal::{AgnosticUniversalAdapter, UniversalAdapterTrait};"
            )
        
        # Specific patterns for each vendor
        for vendor, occurrences in findings.items():
            capability = VENDOR_HARDCODING[vendor]
            if occurrences:
                file_suggestions.append(
                    f"// 🔄 MIGRATE: Replace {vendor} references with '{capability}' capability requests"
                )
        
        suggestions[file_path] = file_suggestions
    
    return suggestions

def validate_migrations(directory: str) -> Dict[str, bool]:
    """Validate that migrations have been applied correctly."""
    print(f"✅ Validating migrations in {directory}...")
    
    rust_files = find_rust_files(directory)
    validation_results = {}
    
    for file_path in rust_files:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Check for remaining hardcoded patterns
            has_hardcoding = any(
                re.search(rf'\b{vendor}\b', content, re.IGNORECASE) 
                for vendor in VENDOR_HARDCODING.keys()
            )
            
            # Check for migration markers
            has_migration_markers = any(marker in content for marker in [
                'MIGRATION NOTICE',
                'MIGRATED:',
                'DEPRECATED:',
                'capability-based',
                'universal adapter'
            ])
            
            validation_results[str(file_path)] = {
                'has_hardcoding': has_hardcoding,
                'has_migration_markers': has_migration_markers,
                'migration_complete': has_migration_markers and not has_hardcoding
            }
            
        except Exception as e:
            print(f"❌ Error validating {file_path}: {e}")
            validation_results[str(file_path)] = {'error': str(e)}
    
    # Summary
    total_files = len(validation_results)
    migrated_files = sum(1 for result in validation_results.values() 
                        if result.get('migration_complete', False))
    files_with_hardcoding = sum(1 for result in validation_results.values() 
                               if result.get('has_hardcoding', False))
    
    print(f"📊 Validation Results:")
    print(f"   Total files: {total_files}")
    print(f"   Fully migrated: {migrated_files}")
    print(f"   Still have hardcoding: {files_with_hardcoding}")
    print(f"   Migration progress: {migrated_files/total_files*100:.1f}%")
    
    return validation_results

def main():
    parser = argparse.ArgumentParser(description='Migrate vendor hardcoding to capability-based patterns')
    parser.add_argument('--analyze', action='store_true', help='Analyze codebase for hardcoded patterns')
    parser.add_argument('--migrate', action='store_true', help='Apply automated migrations')
    parser.add_argument('--validate', action='store_true', help='Validate migration completeness')
    parser.add_argument('--directory', default='.', help='Directory to analyze (default: current)')
    parser.add_argument('--output', help='Output file for analysis results')
    
    args = parser.parse_args()
    
    if args.analyze:
        analysis_results = analyze_codebase(args.directory)
        generate_migration_report(analysis_results)
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(analysis_results, f, indent=2)
            print(f"\n💾 Analysis results saved to {args.output}")
        
        # Generate migration suggestions
        suggestions = create_migration_suggestions(analysis_results)
        print(f"\n🔧 Migration suggestions generated for {len(suggestions)} files")
    
    elif args.validate:
        validation_results = validate_migrations(args.directory)
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(validation_results, f, indent=2)
            print(f"\n💾 Validation results saved to {args.output}")
    
    elif args.migrate:
        print("🚧 Automated migration not implemented yet")
        print("   Use the analysis results and PRIMAL_HARDCODING_MIGRATION_GUIDE.md")
        print("   for manual migration guidance")
    
    else:
        parser.print_help()

if __name__ == '__main__':
    main() 