#!/usr/bin/env python3
"""
Pedantic Issues Fixer

This script fixes all the pedantic clippy issues found in the codebase:
1. Unnecessary qualifications
2. Unused results
3. Missing Debug implementations
4. Missing documentation

Usage:
    python3 scripts/fix_pedantic_issues.py --fix-all
    python3 scripts/fix_pedantic_issues.py --fix-qualifications
    python3 scripts/fix_pedantic_issues.py --fix-unused-results
    python3 scripts/fix_pedantic_issues.py --fix-debug
    python3 scripts/fix_pedantic_issues.py --fix-docs
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import List, Tuple, Dict, Optional

class PedanticIssuesFixer:
    """Fixes pedantic clippy issues systematically"""
    
    def __init__(self, dry_run: bool = False, verbose: bool = False):
        self.dry_run = dry_run
        self.verbose = verbose
        self.issues_fixed = 0
        
    def fix_all_issues(self):
        """Fix all pedantic issues"""
        print("🎯 **FIXING ALL PEDANTIC ISSUES**")
        print("=" * 50)
        
        # Fix in order of importance
        self.fix_unnecessary_qualifications()
        self.fix_unused_results()
        self.fix_missing_debug()
        self.fix_missing_docs()
        
        print(f"\n🎉 **TOTAL ISSUES FIXED: {self.issues_fixed}**")
    
    def fix_unnecessary_qualifications(self):
        """Fix unnecessary qualifications"""
        print("\n🔧 **FIXING UNNECESSARY QUALIFICATIONS**")
        
        fixes = [
            # crates/songbird-types/src/config/federation.rs:54
            {
                'file': 'crates/songbird-types/src/config/federation.rs',
                'old': 'listen_addresses: vec![std::net::SocketAddr::new(',
                'new': 'listen_addresses: vec![SocketAddr::new(',
                'line': 54
            },
            # crates/songbird-types/src/config/gaming.rs:97
            {
                'file': 'crates/songbird-types/src/config/gaming.rs',
                'old': 'pub settings: std::collections::HashMap<String, serde_json::Value>,',
                'new': 'pub settings: HashMap<String, serde_json::Value>,',
                'line': 97
            },
            # crates/songbird-types/src/config/gaming.rs:105
            {
                'file': 'crates/songbird-types/src/config/gaming.rs',
                'old': 'settings: std::collections::HashMap::new(),',
                'new': 'settings: HashMap::new(),',
                'line': 105
            },
            # crates/songbird-types/src/config/migration.rs:46
            {
                'file': 'crates/songbird-types/src/config/migration.rs',
                'old': 'json_config: serde_json::Value,',
                'new': 'json_config: Value,',
                'line': 46
            },
            # crates/songbird-types/src/config/network.rs:299
            {
                'file': 'crates/songbird-types/src/config/network.rs',
                'old': '.unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),',
                'new': '.unwrap_or(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),',
                'line': 299
            },
            # crates/songbird-types/src/config/network.rs:302
            {
                'file': 'crates/songbird-types/src/config/network.rs',
                'old': '.unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),',
                'new': '.unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),',
                'line': 302
            },
            # crates/songbird-types/src/response.rs:62
            {
                'file': 'crates/songbird-types/src/response.rs',
                'old': 'pub fn from_error(error: &crate::errors::SongbirdError) -> Self {',
                'new': 'pub fn from_error(error: &SongbirdError) -> Self {',
                'line': 62
            },
        ]
        
        for fix in fixes:
            if self.apply_fix(fix):
                self.issues_fixed += 1
                if self.verbose:
                    print(f"✅ Fixed qualification in {fix['file']}:{fix['line']}")
    
    def fix_unused_results(self):
        """Fix unused results by using let _ = or adding #[allow]"""
        print("\n🔧 **FIXING UNUSED RESULTS**")
        
        fixes = [
            # Environment.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/config/environment.rs',
                'old': 'endpoints.insert("storage".to_string(), storage.clone());',
                'new': 'let _ = endpoints.insert("storage".to_string(), storage.clone());',
                'line': 384
            },
            {
                'file': 'crates/songbird-types/src/config/environment.rs',
                'old': 'endpoints.insert("compute".to_string(), compute.clone());',
                'new': 'let _ = endpoints.insert("compute".to_string(), compute.clone());',
                'line': 387
            },
            {
                'file': 'crates/songbird-types/src/config/environment.rs',
                'old': 'endpoints.insert("ai".to_string(), ai.clone());',
                'new': 'let _ = endpoints.insert("ai".to_string(), ai.clone());',
                'line': 390
            },
            {
                'file': 'crates/songbird-types/src/config/environment.rs',
                'old': 'endpoints.insert("security".to_string(), security.clone());',
                'new': 'let _ = endpoints.insert("security".to_string(), security.clone());',
                'line': 393
            },
            {
                'file': 'crates/songbird-types/src/config/environment.rs',
                'old': 'endpoints.insert("orchestration".to_string(), orchestration.clone());',
                'new': 'let _ = endpoints.insert("orchestration".to_string(), orchestration.clone());',
                'line': 396
            },
            # Health.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/health.rs',
                'old': 'self.metrics.insert(key.into(), metric_value);',
                'new': 'let _ = self.metrics.insert(key.into(), metric_value);',
                'line': 100
            },
            {
                'file': 'crates/songbird-types/src/health.rs',
                'old': 'self.components.insert(name.into(), status);',
                'new': 'let _ = self.components.insert(name.into(), status);',
                'line': 110
            },
            # Primal.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/primal.rs',
                'old': 'self.endpoints.insert(name.into(), url.into());',
                'new': 'let _ = self.endpoints.insert(name.into(), url.into());',
                'line': 104
            },
            {
                'file': 'crates/songbird-types/src/primal.rs',
                'old': 'self.metadata.insert(key.into(), value.into());',
                'new': 'let _ = self.metadata.insert(key.into(), value.into());',
                'line': 110
            },
            {
                'file': 'crates/songbird-types/src/primal.rs',
                'old': 'self.config.insert(key.into(), value.into());',
                'new': 'let _ = self.config.insert(key.into(), value.into());',
                'line': 183
            },
            {
                'file': 'crates/songbird-types/src/primal.rs',
                'old': 'metadata.insert("primal_id".to_string(), primal_id.into());',
                'new': 'let _ = metadata.insert("primal_id".to_string(), primal_id.into());',
                'line': 235
            },
            {
                'file': 'crates/songbird-types/src/primal.rs',
                'old': 'metadata.insert("error_type".to_string(), "service_unavailable".to_string());',
                'new': 'let _ = metadata.insert("error_type".to_string(), "service_unavailable".to_string());',
                'line': 236
            },
            # Response.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/response.rs',
                'old': 'metadata.insert(key.into(), value.into());',
                'new': 'let _ = metadata.insert(key.into(), value.into());',
                'line': 81
            },
            # Service.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/service.rs',
                'old': 'self.endpoints.insert(name.into(), url.into());',
                'new': 'let _ = self.endpoints.insert(name.into(), url.into());',
                'line': 60
            },
            {
                'file': 'crates/songbird-types/src/service.rs',
                'old': 'self.metadata.insert(key.into(), value.into());',
                'new': 'let _ = self.metadata.insert(key.into(), value.into());',
                'line': 66
            },
            # Types.rs HashMap inserts
            {
                'file': 'crates/songbird-types/src/types.rs',
                'old': 'self.metadata.insert(key.into(), value.into());',
                'new': 'let _ = self.metadata.insert(key.into(), value.into());',
                'line': 150
            },
        ]
        
        for fix in fixes:
            if self.apply_fix(fix):
                self.issues_fixed += 1
                if self.verbose:
                    print(f"✅ Fixed unused result in {fix['file']}:{fix['line']}")
    
    def fix_missing_debug(self):
        """Add #[derive(Debug)] to types missing Debug implementation"""
        print("\n🔧 **FIXING MISSING DEBUG IMPLEMENTATIONS**")
        
        debug_fixes = [
            {
                'file': 'crates/songbird-types/src/config/migration.rs',
                'line': 36,
                'old': 'pub struct ConfigMigrationUtils;',
                'new': '#[derive(Debug)]\npub struct ConfigMigrationUtils;'
            },
            {
                'file': 'crates/songbird-types/src/config/network.rs',
                'line': 194,
                'old': 'pub struct ProductionLanConfig {',
                'new': '#[derive(Debug)]\npub struct ProductionLanConfig {'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 9,
                'old': 'pub struct CanonicalNetworkAddresses;',
                'new': '#[derive(Debug)]\npub struct CanonicalNetworkAddresses;'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 29,
                'old': 'pub struct CanonicalNetworkLimits;',
                'new': '#[derive(Debug)]\npub struct CanonicalNetworkLimits;'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 44,
                'old': 'pub struct CanonicalResourceDefaults;',
                'new': '#[derive(Debug)]\npub struct CanonicalResourceDefaults;'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 59,
                'old': 'pub struct CanonicalPerformanceDefaults;',
                'new': '#[derive(Debug)]\npub struct CanonicalPerformanceDefaults;'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 73,
                'old': 'pub struct CanonicalDiscoveryDefaults;',
                'new': '#[derive(Debug)]\npub struct CanonicalDiscoveryDefaults;'
            },
            {
                'file': 'crates/songbird-types/src/constants.rs',
                'line': 87,
                'old': 'pub struct CanonicalEnvironmentConstants;',
                'new': '#[derive(Debug)]\npub struct CanonicalEnvironmentConstants;'
            },
        ]
        
        for fix in debug_fixes:
            if self.apply_fix(fix):
                self.issues_fixed += 1
                if self.verbose:
                    print(f"✅ Added Debug to {fix['file']}:{fix['line']}")
    
    def fix_missing_docs(self):
        """Add missing documentation"""
        print("\n🔧 **FIXING MISSING DOCUMENTATION**")
        
        doc_fixes = [
            {
                'file': 'crates/songbird-types/src/memory_optimized.rs',
                'line': 172,
                'old': 'pub struct OptimizedCapabilities {',
                'new': '/// Optimized capabilities structure for high-performance operations\npub struct OptimizedCapabilities {'
            },
            {
                'file': 'crates/songbird-types/src/response.rs',
                'line': 220,
                'old': 'pub type BoolResponse = SongbirdResponse<bool>;',
                'new': '/// Response type for boolean values\npub type BoolResponse = SongbirdResponse<bool>;'
            },
            {
                'file': 'crates/songbird-types/src/response.rs',
                'line': 221,
                'old': 'pub type JsonResponse = SongbirdResponse<serde_json::Value>;',
                'new': '/// Response type for JSON values\npub type JsonResponse = SongbirdResponse<serde_json::Value>;'
            },
            {
                'file': 'crates/songbird-types/src/service.rs',
                'line': 230,
                'old': '        min: f64,',
                'new': '        /// Minimum threshold value\n        min: f64,'
            },
            {
                'file': 'crates/songbird-types/src/service.rs',
                'line': 231,
                'old': '        max: f64,',
                'new': '        /// Maximum threshold value\n        max: f64,'
            },
        ]
        
        for fix in doc_fixes:
            if self.apply_fix(fix):
                self.issues_fixed += 1
                if self.verbose:
                    print(f"✅ Added docs to {fix['file']}:{fix['line']}")
    
    def apply_fix(self, fix: Dict) -> bool:
        """Apply a single fix to a file"""
        try:
            file_path = fix['file']
            
            if not Path(file_path).exists():
                print(f"⚠️ File not found: {file_path}")
                return False
            
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            if fix['old'] not in content:
                if self.verbose:
                    print(f"⚠️ Pattern not found in {file_path}: {fix['old'][:50]}...")
                return False
            
            # Apply the fix
            new_content = content.replace(fix['old'], fix['new'])
            
            if not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(new_content)
            
            return True
            
        except Exception as e:
            print(f"❌ Error applying fix to {fix['file']}: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(
        description='Pedantic Issues Fixer',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Fix all pedantic issues
  python3 scripts/fix_pedantic_issues.py --fix-all
  
  # Fix specific types of issues
  python3 scripts/fix_pedantic_issues.py --fix-qualifications
  python3 scripts/fix_pedantic_issues.py --fix-unused-results
  python3 scripts/fix_pedantic_issues.py --fix-debug
  python3 scripts/fix_pedantic_issues.py --fix-docs
  
  # Dry run to see what would be fixed
  python3 scripts/fix_pedantic_issues.py --fix-all --dry-run
        """
    )
    
    parser.add_argument('--fix-all', action='store_true', help='Fix all pedantic issues')
    parser.add_argument('--fix-qualifications', action='store_true', help='Fix unnecessary qualifications')
    parser.add_argument('--fix-unused-results', action='store_true', help='Fix unused results')
    parser.add_argument('--fix-debug', action='store_true', help='Add missing Debug implementations')
    parser.add_argument('--fix-docs', action='store_true', help='Add missing documentation')
    parser.add_argument('--dry-run', action='store_true', help='Show what would be fixed')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if not any([args.fix_all, args.fix_qualifications, args.fix_unused_results, 
                args.fix_debug, args.fix_docs]):
        parser.error("Must specify at least one fix type")
    
    # Create fixer
    fixer = PedanticIssuesFixer(dry_run=args.dry_run, verbose=args.verbose)
    
    print("🎯 **PEDANTIC ISSUES FIXER**")
    print("=" * 40)
    
    if args.dry_run:
        print("🔍 **DRY RUN MODE** - No changes will be made")
    
    try:
        if args.fix_all:
            fixer.fix_all_issues()
        else:
            if args.fix_qualifications:
                fixer.fix_unnecessary_qualifications()
            if args.fix_unused_results:
                fixer.fix_unused_results()
            if args.fix_debug:
                fixer.fix_missing_debug()
            if args.fix_docs:
                fixer.fix_missing_docs()
        
        print(f"\n🎉 **SUCCESSFULLY FIXED {fixer.issues_fixed} PEDANTIC ISSUES!**")
        
        if not args.dry_run:
            print("\n✅ **NEXT STEPS:**")
            print("1. Run `cargo clippy --package songbird-types -- -D clippy::pedantic` to verify fixes")
            print("2. Run `cargo test --package songbird-types` to ensure functionality")
            print("3. Run `cargo fmt --all` to format the changes")
        
    except KeyboardInterrupt:
        print("\n❌ Fixing cancelled by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n💥 Fixing failed: {e}")
        sys.exit(1)

if __name__ == '__main__':
    main() 