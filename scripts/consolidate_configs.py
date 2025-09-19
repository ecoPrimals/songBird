#!/usr/bin/env python3
"""
Configuration Consolidation Script

This script identifies duplicate and scattered configuration structures
and helps consolidate them under the canonical UnifiedSongbirdConfig.
"""

import os
import re
import subprocess
from pathlib import Path
from collections import defaultdict

# Configuration structures that should be consolidated
DUPLICATE_CONFIGS = {
    # Discovery configs
    "DiscoveryConfig": [
        "crates/songbird-universal/src/discovery.rs",
        "crates/songbird-discovery/src/discovery/core.rs",
        "crates/songbird-discovery/src/discovery/config/mod.rs",
        "crates/songbird-discovery/src/traits/discovery.rs",
    ],
    
    # Network configs
    "NetworkConfig": [
        "crates/songbird-discovery/src/discovery/config/mod.rs",
        "crates/songbird-config/src/canonical_network.rs",
    ],
    
    # Service configs
    "ServiceConfig": [
        "crates/songbird-config/src/canonical/service.rs",
        "crates/songbird-config/src/unified/core.rs",
        "crates/songbird-config/src/performance.rs",
        "examples/comprehensive_unified_error_migration.rs",
    ],
    
    # Health check configs
    "HealthCheckConfig": [
        "crates/songbird-config/src/canonical/service.rs",
        "crates/songbird-universal/src/types.rs",
        "crates/songbird-discovery/src/traits/health.rs",
        "crates/songbird-config/src/unified/api.rs",
        "crates/songbird-config/src/unified/robustness.rs",
    ],
    
    # Security configs
    "SecurityConfig": [
        "crates/songbird-universal/src/types.rs",
    ],
    
    # Circuit breaker configs
    "CircuitBreakerConfig": [
        "crates/songbird-config/src/canonical/resilience.rs",
        "crates/songbird-universal/src/types.rs",
        "crates/songbird-config/src/unified/robustness.rs",
        "crates/songbird-config/src/unified/api.rs",
    ],
}

# Test-only configs that are acceptable
TEST_CONFIGS = [
    "TestExecutionConfig",
    "TestFederationConfig", 
    "ChaosTestConfig",
    "TestConfig",
]

# Example configs that are acceptable
EXAMPLE_CONFIGS = [
    "NasServiceConfig",
    "EnvironmentConfig",  # in examples/
    "FederationConfig",   # in examples/
]

def analyze_config_fragmentation():
    """Analyze configuration fragmentation across the codebase."""
    print("🔍 Analyzing configuration fragmentation...")
    
    config_usage = defaultdict(list)
    
    # Find all config struct definitions
    result = subprocess.run([
        'grep', '-r', '--include=*.rs', 
        'pub struct.*Config.*{', 'crates/'
    ], capture_output=True, text=True)
    
    if result.returncode == 0:
        for line in result.stdout.strip().split('\n'):
            if ':' in line:
                file_path, content = line.split(':', 1)
                # Extract config name
                match = re.search(r'pub struct (\w*Config)', content)
                if match:
                    config_name = match.group(1)
                    config_usage[config_name].append(file_path)
    
    return config_usage

def categorize_configs(config_usage):
    """Categorize configs into consolidation targets and acceptable configs."""
    print("📊 Categorizing configuration structures...")
    
    consolidation_targets = {}
    acceptable_configs = {}
    
    for config_name, files in config_usage.items():
        # Skip test configs
        if any(test in config_name for test in TEST_CONFIGS):
            acceptable_configs[config_name] = files
            continue
            
        # Skip example configs in examples/
        if any('examples/' in f for f in files) and any(ex in config_name for ex in EXAMPLE_CONFIGS):
            acceptable_configs[config_name] = files
            continue
            
        # Check if it's a known duplicate
        if config_name in DUPLICATE_CONFIGS:
            consolidation_targets[config_name] = files
        elif len(files) > 1:
            # Multiple definitions - potential consolidation target
            consolidation_targets[config_name] = files
        else:
            acceptable_configs[config_name] = files
    
    return consolidation_targets, acceptable_configs

def generate_consolidation_report(consolidation_targets, acceptable_configs):
    """Generate a detailed consolidation report."""
    print("\n📋 CONFIGURATION CONSOLIDATION REPORT")
    print("=" * 60)
    
    print(f"\n🎯 CONSOLIDATION TARGETS ({len(consolidation_targets)} types)")
    print("-" * 40)
    
    total_duplicates = 0
    for config_name, files in consolidation_targets.items():
        duplicate_count = len(files) - 1
        total_duplicates += duplicate_count
        
        print(f"\n📦 {config_name}")
        print(f"   Duplicates: {duplicate_count}")
        for i, file_path in enumerate(files):
            status = "✅ CANONICAL" if i == 0 else "🔄 DUPLICATE"
            print(f"   {status}: {file_path}")
    
    print(f"\n✅ ACCEPTABLE CONFIGS ({len(acceptable_configs)} types)")
    print("-" * 40)
    for config_name, files in acceptable_configs.items():
        reason = "Test-only" if any(test in config_name for test in TEST_CONFIGS) else "Single definition"
        print(f"   {config_name}: {reason}")
    
    print(f"\n📈 CONSOLIDATION IMPACT")
    print("-" * 40)
    print(f"   Total config types: {len(consolidation_targets) + len(acceptable_configs)}")
    print(f"   Consolidation targets: {len(consolidation_targets)}")
    print(f"   Duplicate definitions: {total_duplicates}")
    print(f"   Potential reduction: {total_duplicates} definitions")
    
    return total_duplicates

def suggest_consolidation_actions(consolidation_targets):
    """Suggest specific actions for consolidation."""
    print(f"\n🚀 RECOMMENDED CONSOLIDATION ACTIONS")
    print("-" * 50)
    
    high_priority = []
    medium_priority = []
    
    for config_name, files in consolidation_targets.items():
        duplicate_count = len(files) - 1
        if duplicate_count >= 3:
            high_priority.append((config_name, files, duplicate_count))
        else:
            medium_priority.append((config_name, files, duplicate_count))
    
    if high_priority:
        print(f"\n🔥 HIGH PRIORITY (3+ duplicates)")
        for config_name, files, count in high_priority:
            print(f"   {config_name}: {count} duplicates")
            print(f"      Action: Consolidate to songbird-types::config::{config_name}")
    
    if medium_priority:
        print(f"\n⚡ MEDIUM PRIORITY (1-2 duplicates)")
        for config_name, files, count in medium_priority:
            print(f"   {config_name}: {count} duplicates")
            print(f"      Action: Review for consolidation opportunity")

def check_unified_config_usage():
    """Check how well UnifiedSongbirdConfig is being adopted."""
    print(f"\n📊 UNIFIED CONFIG ADOPTION STATUS")
    print("-" * 40)
    
    # Find usages of UnifiedSongbirdConfig
    result = subprocess.run([
        'grep', '-r', '--include=*.rs',
        'UnifiedSongbirdConfig', 'crates/'
    ], capture_output=True, text=True)
    
    if result.returncode == 0:
        usage_count = len(result.stdout.strip().split('\n'))
        print(f"   UnifiedSongbirdConfig usages: {usage_count}")
    else:
        print(f"   UnifiedSongbirdConfig usages: 0")
    
    # Find usages of old config patterns
    old_patterns = ['SongbirdConfig', 'CanonicalConfig', 'UniversalPrimalConfig']
    for pattern in old_patterns:
        result = subprocess.run([
            'grep', '-r', '--include=*.rs',
            pattern, 'crates/'
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            usage_count = len(result.stdout.strip().split('\n'))
            print(f"   {pattern} usages: {usage_count} (should migrate)")
        else:
            print(f"   {pattern} usages: 0 ✅")

def main():
    """Main consolidation analysis."""
    print("🔄 Starting configuration consolidation analysis...")
    
    # Analyze current state
    config_usage = analyze_config_fragmentation()
    
    if not config_usage:
        print("✅ No configuration fragmentation found!")
        return
    
    # Categorize configs
    consolidation_targets, acceptable_configs = categorize_configs(config_usage)
    
    # Generate report
    total_duplicates = generate_consolidation_report(consolidation_targets, acceptable_configs)
    
    # Suggest actions
    suggest_consolidation_actions(consolidation_targets)
    
    # Check unified config adoption
    check_unified_config_usage()
    
    print(f"\n🎉 ANALYSIS COMPLETE")
    print("-" * 30)
    if total_duplicates > 0:
        print(f"   Found {total_duplicates} duplicate config definitions")
        print(f"   Consolidation will improve code consistency and reduce maintenance")
        print(f"   Next step: Implement consolidation script for high-priority targets")
    else:
        print("   No significant configuration fragmentation detected!")
        print("   Configuration system is well-consolidated")

if __name__ == "__main__":
    main() 