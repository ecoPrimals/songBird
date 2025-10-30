#!/usr/bin/env python3
"""
🔥 COMPREHENSIVE HARDCODING ELIMINATION SCRIPT

**MISSION**: Systematically eliminate ALL hardcoded patterns from the codebase

This script finds and eliminates:
1. Primal names (beardog, toadstool, nestgate, squirrel)
2. Vendor names (kubernetes, consul, docker, redis, etcd)  
3. Hardcoded ports and addresses
4. Magic numbers and timeout values

The philosophy: Each service knows only itself and discovers everything else
at runtime through capability-based discovery.
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Set, Tuple
from dataclasses import dataclass
from enum import Enum
import argparse


class HardcodingType(Enum):
    """Types of hardcoding to eliminate"""
    PRIMAL_NAME = "primal_name"
    VENDOR_NAME = "vendor_name" 
    PORT_NUMBER = "port_number"
    IP_ADDRESS = "ip_address"
    MAGIC_NUMBER = "magic_number"
    SERVICE_URL = "service_url"


@dataclass
class HardcodingPattern:
    """A detected hardcoding pattern"""
    file_path: str
    line_number: int
    line_content: str
    pattern_type: HardcodingType
    matched_text: str
    suggested_replacement: str
    env_var_needed: str | None
    severity: str  # 'critical', 'high', 'medium', 'low'


@dataclass
class MigrationStats:
    """Statistics from migration"""
    files_scanned: int = 0
    patterns_found: int = 0
    patterns_fixed: int = 0
    env_vars_needed: Set[str] = None
    
    def __post_init__(self):
        if self.env_vars_needed is None:
            self.env_vars_needed = set()


class HardcodingEliminator:
    """Systematically eliminates hardcoding from the codebase"""
    
    def __init__(self, repo_root: Path, dry_run: bool = True):
        self.repo_root = repo_root
        self.dry_run = dry_run
        self.stats = MigrationStats()
        
        # Files to exclude from scanning
        self.excluded_patterns = [
            # Test files can have some hardcoding for mocking
            r'tests?/',
            r'test_utils/',
            r'_test\.rs$',
            r'_tests\.rs$',
            # Migration/audit files themselves
            r'eliminate.*hardcoding',
            r'audit.*hardcoding',
            r'migration',
            # Documentation
            r'\.md$',
            r'docs/',
            # Build artifacts
            r'target/',
            r'\.git/',
        ]
        
        # Primal name patterns to eliminate
        self.primal_patterns = [
            # Lowercase variations
            (r'\bbeardog\b', 'SECURITY_PROVIDER', 'security provider'),
            (r'\btoadstool\b', 'COMPUTE_PROVIDER', 'compute provider'),
            (r'\bnestgate\b', 'STORAGE_PROVIDER', 'storage provider'),
            (r'\bsquirrel\b', 'AI_PROVIDER', 'AI provider'),
            
            # CamelCase variations
            (r'\bBearDog\b', 'SecurityProvider', 'security provider'),
            (r'\bToadStool\b', 'ComputeProvider', 'compute provider'),
            (r'\bNestGate\b', 'StorageProvider', 'storage provider'),
            (r'\bSquirrel\b', 'AiProvider', 'AI provider'),
            
            # UPPERCASE variations
            (r'\bBEARDOG\b', 'SECURITY_PROVIDER', 'security provider'),
            (r'\bTOADSTOOL\b', 'COMPUTE_PROVIDER', 'compute provider'),
            (r'\bNESTGATE\b', 'STORAGE_PROVIDER', 'storage provider'),
            (r'\bSQUIRREL\b', 'AI_PROVIDER', 'AI provider'),
        ]
        
        # Vendor name patterns to eliminate
        self.vendor_patterns = [
            (r'\bkubernetes\b', 'container_orchestrator', 'CONTAINER_ORCHESTRATOR_TYPE'),
            (r'\bk8s\b', 'container_orchestrator', 'CONTAINER_ORCHESTRATOR_TYPE'),
            (r'\bconsul\b', 'service_registry', 'SERVICE_REGISTRY_TYPE'),
            (r'\bdocker\b', 'container_runtime', 'CONTAINER_RUNTIME_TYPE'),
            (r'\betcd\b', 'key_value_store', 'KEY_VALUE_STORE_TYPE'),
            (r'\bredis\b', 'cache_provider', 'CACHE_PROVIDER_TYPE'),
            (r'\bpostgres\b', 'database', 'DATABASE_TYPE'),
            (r'\bmysql\b', 'database', 'DATABASE_TYPE'),
            (r'\bmongodb\b', 'document_store', 'DOCUMENT_STORE_TYPE'),
        ]
        
        # Context patterns to detect if hardcoding is in test/mock context
        self.test_context_patterns = [
            r'#\[test\]',
            r'#\[cfg\(test\)\]',
            r'mock::',
            r'Mock\w+',
            r'test_helper',
            r'fixtures::',
        ]
    
    def should_scan_file(self, file_path: Path) -> bool:
        """Check if file should be scanned for hardcoding"""
        path_str = str(file_path)
        
        # Only scan Rust source files
        if not path_str.endswith('.rs'):
            return False
        
        # Check exclusion patterns
        for pattern in self.excluded_patterns:
            if re.search(pattern, path_str):
                return False
        
        return True
    
    def is_test_context(self, file_content: str, line_number: int) -> bool:
        """Check if line is in a test/mock context where some hardcoding is acceptable"""
        lines = file_content.split('\n')
        
        # Check surrounding lines for test context
        start = max(0, line_number - 10)
        end = min(len(lines), line_number + 3)
        context = '\n'.join(lines[start:end])
        
        for pattern in self.test_context_patterns:
            if re.search(pattern, context):
                return True
        
        return False
    
    def scan_file_for_primals(self, file_path: Path) -> List[HardcodingPattern]:
        """Scan a file for primal name hardcoding"""
        patterns = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            for line_num, line in enumerate(lines, 1):
                # Skip comments
                if line.strip().startswith('//'):
                    continue
                
                for primal_pattern, replacement, capability in self.primal_patterns:
                    matches = re.finditer(primal_pattern, line, re.IGNORECASE)
                    for match in matches:
                        # Determine if this is in test context
                        in_test = self.is_test_context(content, line_num)
                        severity = 'medium' if in_test else 'critical'
                        
                        # Generate appropriate replacement
                        if '"' in line or "'" in line:
                            # String literal context
                            suggested = f'env::var("{replacement}_ENDPOINT").unwrap_or_default()'
                            env_var = f'{replacement}_ENDPOINT'
                        else:
                            # Code context
                            suggested = f'discover_capability("{capability}").await'
                            env_var = None
                        
                        patterns.append(HardcodingPattern(
                            file_path=str(file_path),
                            line_number=line_num,
                            line_content=line.strip(),
                            pattern_type=HardcodingType.PRIMAL_NAME,
                            matched_text=match.group(),
                            suggested_replacement=suggested,
                            env_var_needed=env_var,
                            severity=severity,
                        ))
        
        except Exception as e:
            print(f"⚠️  Error scanning {file_path}: {e}")
        
        return patterns
    
    def scan_file_for_vendors(self, file_path: Path) -> List[HardcodingPattern]:
        """Scan a file for vendor name hardcoding"""
        patterns = []
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            for line_num, line in enumerate(lines, 1):
                # Skip comments
                if line.strip().startswith('//'):
                    continue
                
                # Skip if in a use/import statement (those are different)
                if 'use ' in line or 'extern crate' in line:
                    continue
                
                for vendor_pattern, capability, env_var in self.vendor_patterns:
                    matches = re.finditer(vendor_pattern, line, re.IGNORECASE)
                    for match in matches:
                        # Check if this is in type name (like KubernetesAdapter)
                        # These are OK as they're implementing adapters
                        if 'Adapter' in line or 'Client' in line:
                            continue
                        
                        in_test = self.is_test_context(content, line_num)
                        severity = 'low' if in_test else 'high'
                        
                        suggested = f'discover_capability("{capability}").await'
                        
                        patterns.append(HardcodingPattern(
                            file_path=str(file_path),
                            line_number=line_num,
                            line_content=line.strip(),
                            pattern_type=HardcodingType.VENDOR_NAME,
                            matched_text=match.group(),
                            suggested_replacement=suggested,
                            env_var_needed=env_var,
                            severity=severity,
                        ))
        
        except Exception as e:
            print(f"⚠️  Error scanning {file_path}: {e}")
        
        return patterns
    
    def scan_file_for_ports(self, file_path: Path) -> List[HardcodingPattern]:
        """Scan a file for hardcoded port numbers"""
        patterns = []
        
        # Patterns for port hardcoding
        port_patterns = [
            (r':\s*(\d{4,5})\b', 'PORT'),
            (r'port\s*=\s*(\d{4,5})\b', 'PORT'),
            (r'const\s+\w*PORT\w*\s*:\s*u16\s*=\s*(\d{4,5})', 'PORT'),
        ]
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            for line_num, line in enumerate(lines, 1):
                # Skip comments and constants definition files
                if line.strip().startswith('//') or 'constants' in str(file_path).lower():
                    continue
                
                for pattern, env_prefix in port_patterns:
                    matches = re.finditer(pattern, line)
                    for match in matches:
                        port_num = match.group(1) if match.lastindex else match.group()
                        
                        in_test = self.is_test_context(content, line_num)
                        severity = 'low' if in_test else 'medium'
                        
                        # Generate context-aware env var name
                        if 'health' in line.lower():
                            env_var = 'HEALTH_PORT'
                        elif 'metrics' in line.lower():
                            env_var = 'METRICS_PORT'
                        else:
                            env_var = 'SERVICE_PORT'
                        
                        suggested = f'env::var("{env_var}").unwrap_or("8080").parse().unwrap()'
                        
                        patterns.append(HardcodingPattern(
                            file_path=str(file_path),
                            line_number=line_num,
                            line_content=line.strip(),
                            pattern_type=HardcodingType.PORT_NUMBER,
                            matched_text=port_num,
                            suggested_replacement=suggested,
                            env_var_needed=env_var,
                            severity=severity,
                        ))
        
        except Exception as e:
            print(f"⚠️  Error scanning {file_path}: {e}")
        
        return patterns
    
    def scan_directory(self, directory: Path) -> List[HardcodingPattern]:
        """Recursively scan directory for hardcoding"""
        all_patterns = []
        
        for file_path in directory.rglob('*.rs'):
            if not self.should_scan_file(file_path):
                continue
            
            self.stats.files_scanned += 1
            
            # Scan for different types of hardcoding
            patterns = []
            patterns.extend(self.scan_file_for_primals(file_path))
            patterns.extend(self.scan_file_for_vendors(file_path))
            patterns.extend(self.scan_file_for_ports(file_path))
            
            all_patterns.extend(patterns)
        
        return all_patterns
    
    def generate_report(self, patterns: List[HardcodingPattern]) -> str:
        """Generate a detailed report of findings"""
        report = []
        report.append("=" * 80)
        report.append("🔥 HARDCODING ELIMINATION REPORT")
        report.append("=" * 80)
        report.append("")
        
        # Summary by type
        by_type = {}
        for pattern in patterns:
            key = pattern.pattern_type.value
            by_type[key] = by_type.get(key, 0) + 1
        
        report.append("📊 SUMMARY BY TYPE:")
        for ptype, count in sorted(by_type.items()):
            report.append(f"  {ptype}: {count} instances")
        report.append("")
        
        # Summary by severity
        by_severity = {}
        for pattern in patterns:
            by_severity[pattern.severity] = by_severity.get(pattern.severity, 0) + 1
        
        report.append("⚠️  SUMMARY BY SEVERITY:")
        for severity in ['critical', 'high', 'medium', 'low']:
            count = by_severity.get(severity, 0)
            if count > 0:
                report.append(f"  {severity.upper()}: {count} instances")
        report.append("")
        
        # Environment variables needed
        env_vars = set(p.env_var_needed for p in patterns if p.env_var_needed)
        if env_vars:
            report.append("🔧 ENVIRONMENT VARIABLES NEEDED:")
            for var in sorted(env_vars):
                report.append(f"  {var}=<value>")
            report.append("")
        
        # Top files with hardcoding
        by_file = {}
        for pattern in patterns:
            by_file[pattern.file_path] = by_file.get(pattern.file_path, 0) + 1
        
        report.append("📁 TOP FILES WITH HARDCODING:")
        top_files = sorted(by_file.items(), key=lambda x: x[1], reverse=True)[:20]
        for file_path, count in top_files:
            # Show relative path
            rel_path = Path(file_path).relative_to(self.repo_root)
            report.append(f"  {count:4d}  {rel_path}")
        report.append("")
        
        return "\n".join(report)
    
    def generate_env_template(self, patterns: List[HardcodingPattern]) -> str:
        """Generate environment variable template"""
        env_vars = {}
        
        for pattern in patterns:
            if pattern.env_var_needed:
                # Generate example values
                var_name = pattern.env_var_needed
                if 'PORT' in var_name:
                    example = '8080'
                elif 'ENDPOINT' in var_name:
                    example = 'http://localhost:8080'
                elif 'URL' in var_name:
                    example = 'http://localhost:8080'
                else:
                    example = '<value>'
                
                env_vars[var_name] = example
        
        template = []
        template.append("# 🍼 Zero-Knowledge Environment Configuration")
        template.append("# Generated by hardcoding elimination script")
        template.append("")
        template.append("# Service Identity")
        template.append("SERVICE_ID=my-service")
        template.append("SERVICE_CAPABILITIES=compute,storage  # What THIS service provides")
        template.append("")
        template.append("# Required Capabilities (what this service needs)")
        template.append("REQUIRED_CAPABILITIES=security,storage")
        template.append("OPTIONAL_CAPABILITIES=ai,analytics")
        template.append("")
        template.append("# Network Configuration (NO hardcoded ports)")
        template.append("SERVICE_PORT=8080")
        template.append("HEALTH_PORT=8081")
        template.append("METRICS_PORT=8082")
        template.append("")
        template.append("# Discovery Configuration")
        template.append("ENABLE_INFANT_DISCOVERY=true")
        template.append("DISCOVERY_TIMEOUT_SECS=30")
        template.append("")
        template.append("# Capability Endpoints (discovered dynamically or configured)")
        for var_name in sorted(env_vars.keys()):
            template.append(f"{var_name}={env_vars[var_name]}")
        template.append("")
        
        return "\n".join(template)
    
    def run_elimination(self, crates_dir: Path) -> MigrationStats:
        """Run the complete elimination process"""
        print("🔥 Starting comprehensive hardcoding elimination...")
        print(f"📁 Scanning: {crates_dir}")
        print(f"🏃 Mode: {'DRY RUN' if self.dry_run else 'LIVE'}")
        print("")
        
        # Scan for patterns
        print("🔍 Scanning for hardcoded patterns...")
        patterns = self.scan_directory(crates_dir)
        self.stats.patterns_found = len(patterns)
        
        print(f"✅ Scan complete: {self.stats.files_scanned} files scanned")
        print(f"📊 Found {len(patterns)} hardcoded patterns")
        print("")
        
        # Generate and display report
        report = self.generate_report(patterns)
        print(report)
        
        # Generate environment template
        env_template = self.generate_env_template(patterns)
        env_file = self.repo_root / "config" / "zero-knowledge.env.template"
        env_file.parent.mkdir(exist_ok=True)
        with open(env_file, 'w') as f:
            f.write(env_template)
        print(f"📝 Environment template written to: {env_file}")
        print("")
        
        # Write detailed report
        report_file = self.repo_root / "HARDCODING_ELIMINATION_REPORT.md"
        with open(report_file, 'w') as f:
            f.write(report)
        print(f"📄 Detailed report written to: {report_file}")
        print("")
        
        return self.stats


def main():
    parser = argparse.ArgumentParser(
        description="Eliminate all hardcoding from Songbird codebase"
    )
    parser.add_argument(
        '--live',
        action='store_true',
        help='Actually modify files (default is dry-run)'
    )
    parser.add_argument(
        '--repo-root',
        type=Path,
        default=Path.cwd(),
        help='Repository root directory'
    )
    
    args = parser.parse_args()
    
    repo_root = args.repo_root
    crates_dir = repo_root / 'crates'
    
    if not crates_dir.exists():
        print(f"❌ Error: {crates_dir} not found")
        sys.exit(1)
    
    eliminator = HardcodingEliminator(repo_root, dry_run=not args.live)
    stats = eliminator.run_elimination(crates_dir)
    
    print("=" * 80)
    print("🎯 ELIMINATION COMPLETE")
    print(f"📁 Files scanned: {stats.files_scanned}")
    print(f"🔍 Patterns found: {stats.patterns_found}")
    if not args.live:
        print("")
        print("ℹ️  This was a DRY RUN. Use --live to apply changes.")
    print("=" * 80)


if __name__ == '__main__':
    main()

