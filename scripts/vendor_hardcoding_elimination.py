#!/usr/bin/env python3
"""
🔄 Vendor Hardcoding Elimination Script

MISSION: Systematically eliminate ALL hardcoded vendor names and replace with
capability-based discovery patterns.

TARGETS:
- Primal names: beardog → capability_security, nestgate → capability_storage, etc.
- External services: k8s → container_orchestration, consul → service_discovery
- Hardcoded endpoints, ports, and connections
- 2^n connection patterns → universal adapter routing

STRATEGY:
1. Scan codebase for hardcoded patterns
2. Generate migration plan with confidence scores
3. Apply safe transformations with backup
4. Verify migrations don't break functionality
5. Generate migration report

USAGE:
    python3 scripts/vendor_hardcoding_elimination.py --scan
    python3 scripts/vendor_hardcoding_elimination.py --migrate --pattern beardog
    python3 scripts/vendor_hardcoding_elimination.py --migrate-all --dry-run
    python3 scripts/vendor_hardcoding_elimination.py --report
"""

import os
import re
import json
import shutil
import argparse
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Set
from dataclasses import dataclass, asdict
from enum import Enum
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class MigrationUrgency(Enum):
    CRITICAL = "critical"
    HIGH = "high" 
    MEDIUM = "medium"
    LOW = "low"

class MigrationComplexity(Enum):
    SIMPLE = "simple"
    MODERATE = "moderate"
    COMPLEX = "complex"
    CRITICAL = "critical"

@dataclass
class HardcodedPattern:
    """Represents a hardcoded pattern found in the codebase"""
    pattern: str
    file_path: str
    line_number: int
    line_content: str
    context_lines: List[str]
    pattern_type: str  # 'primal', 'external_service', 'endpoint', 'connection'
    confidence: float
    suggested_replacement: str
    urgency: MigrationUrgency
    complexity: MigrationComplexity

@dataclass 
class MigrationRule:
    """Rule for transforming a hardcoded pattern"""
    from_pattern: str
    to_capability: str
    pattern_regex: str
    replacement_template: str
    confidence: float
    urgency: MigrationUrgency
    complexity: MigrationComplexity
    context_hints: List[str]
    breaking_change: bool

@dataclass
class MigrationResult:
    """Result of applying a migration"""
    original_pattern: str
    migrated_pattern: str
    file_path: str
    line_number: int
    success: bool
    error_message: Optional[str]
    confidence: float
    requires_manual_review: bool

class VendorHardcodingEliminator:
    """Main class for eliminating vendor hardcoding"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.migration_rules = self._initialize_migration_rules()
        self.found_patterns: List[HardcodedPattern] = []
        self.migration_results: List[MigrationResult] = []
        
        # Directories to scan
        self.scan_dirs = [
            "crates",
            "examples", 
            "src",
            "tests",
        ]
        
        # File extensions to scan
        self.scan_extensions = [".rs", ".toml", ".md", ".yaml", ".yml", ".json"]
        
        # Directories to exclude
        self.exclude_dirs = {
            "target", 
            ".git", 
            "archive", 
            "node_modules",
            "coverage-report",
            "cache"
        }
        
        # Files to exclude from migration (preserve for reference)
        self.exclude_files = {
            # Test files can keep hardcoded examples for validation
            "vendor_hardcoding_elimination_tests.rs",
            "agnostic_integration_test.rs",
            # Archive files are preserved for historical reference
        }

    def _initialize_migration_rules(self) -> Dict[str, MigrationRule]:
        """Initialize migration rules for all hardcoded patterns"""
        rules = {}
        
        # Primal hardcoding elimination rules
        primal_rules = [
            # Security Primal (beardog)
            MigrationRule(
                from_pattern="beardog",
                to_capability="security",
                pattern_regex=r'\bbeardog\b',
                replacement_template="capability_security",
                confidence=0.95,
                urgency=MigrationUrgency.CRITICAL,
                complexity=MigrationComplexity.MODERATE,
                context_hints=["security", "auth", "encrypt", "token"],
                breaking_change=True
            ),
            # Storage Primal (nestgate)
            MigrationRule(
                from_pattern="nestgate",
                to_capability="storage", 
                pattern_regex=r'\bnestgate\b',
                replacement_template="capability_storage",
                confidence=0.95,
                urgency=MigrationUrgency.CRITICAL,
                complexity=MigrationComplexity.MODERATE,
                context_hints=["storage", "store", "retrieve", "data", "file"],
                breaking_change=True
            ),
            # Compute Primal (toadstool)
            MigrationRule(
                from_pattern="toadstool",
                to_capability="compute",
                pattern_regex=r'\btoadstool\b',
                replacement_template="capability_compute", 
                confidence=0.95,
                urgency=MigrationUrgency.CRITICAL,
                complexity=MigrationComplexity.MODERATE,
                context_hints=["compute", "execute", "process", "container", "job"],
                breaking_change=True
            ),
            # AI Primal (squirrel)
            MigrationRule(
                from_pattern="squirrel",
                to_capability="ai",
                pattern_regex=r'\bsquirrel\b',
                replacement_template="capability_ai",
                confidence=0.95,
                urgency=MigrationUrgency.CRITICAL, 
                complexity=MigrationComplexity.MODERATE,
                context_hints=["ai", "analyze", "inference", "ml", "model"],
                breaking_change=True
            ),
        ]
        
        # External service agnostic rules
        external_rules = [
            # Kubernetes
            MigrationRule(
                from_pattern="kubernetes",
                to_capability="container_orchestration",
                pattern_regex=r'\bkubernetes\b',
                replacement_template="container_orchestration",
                confidence=0.90,
                urgency=MigrationUrgency.HIGH,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["orchestration", "container", "deploy", "k8s"],
                breaking_change=False
            ),
            # k8s (Kubernetes alias)
            MigrationRule(
                from_pattern="k8s",
                to_capability="container_orchestration", 
                pattern_regex=r'\bk8s\b',
                replacement_template="container_orchestration",
                confidence=0.90,
                urgency=MigrationUrgency.HIGH,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["orchestration", "container", "deploy"],
                breaking_change=False
            ),
            # Consul
            MigrationRule(
                from_pattern="consul",
                to_capability="service_discovery",
                pattern_regex=r'\bconsul\b',
                replacement_template="service_discovery",
                confidence=0.85,
                urgency=MigrationUrgency.MEDIUM,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["discovery", "registry", "service"],
                breaking_change=False
            ),
            # Docker
            MigrationRule(
                from_pattern="docker",
                to_capability="container_runtime",
                pattern_regex=r'\bdocker\b',
                replacement_template="container_runtime",
                confidence=0.85,
                urgency=MigrationUrgency.MEDIUM,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["container", "runtime", "image"],
                breaking_change=False
            ),
            # Prometheus
            MigrationRule(
                from_pattern="prometheus",
                to_capability="metrics_collection",
                pattern_regex=r'\bprometheus\b',
                replacement_template="metrics_collection",
                confidence=0.80,
                urgency=MigrationUrgency.LOW,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["metrics", "monitoring", "collect"],
                breaking_change=False
            ),
            # Grafana
            MigrationRule(
                from_pattern="grafana",
                to_capability="metrics_visualization",
                pattern_regex=r'\bgrafana\b',
                replacement_template="metrics_visualization",
                confidence=0.80,
                urgency=MigrationUrgency.LOW,
                complexity=MigrationComplexity.SIMPLE,
                context_hints=["dashboard", "visualization", "graph"],
                breaking_change=False
            ),
        ]
        
        # Combine all rules
        all_rules = primal_rules + external_rules
        
        # Index by pattern
        for rule in all_rules:
            rules[rule.from_pattern] = rule
            
        return rules

    def scan_codebase(self) -> List[HardcodedPattern]:
        """Scan the entire codebase for hardcoded patterns"""
        logger.info("🔍 Scanning codebase for hardcoded vendor patterns...")
        
        found_patterns = []
        
        for scan_dir in self.scan_dirs:
            dir_path = self.project_root / scan_dir
            if not dir_path.exists():
                continue
                
            logger.info(f"📂 Scanning directory: {scan_dir}")
            patterns = self._scan_directory(dir_path)
            found_patterns.extend(patterns)
            
        self.found_patterns = found_patterns
        logger.info(f"✅ Scan complete: {len(found_patterns)} patterns found")
        
        return found_patterns

    def _scan_directory(self, directory: Path) -> List[HardcodedPattern]:
        """Scan a directory recursively for patterns"""
        patterns = []
        
        for file_path in directory.rglob("*"):
            # Skip directories
            if file_path.is_dir():
                continue
                
            # Skip excluded directories
            if any(exclude in file_path.parts for exclude in self.exclude_dirs):
                continue
                
            # Skip excluded files
            if file_path.name in self.exclude_files:
                continue
                
            # Check file extension
            if file_path.suffix not in self.scan_extensions:
                continue
                
            # Scan file for patterns
            file_patterns = self._scan_file(file_path)
            patterns.extend(file_patterns)
            
        return patterns

    def _scan_file(self, file_path: Path) -> List[HardcodedPattern]:
        """Scan a single file for hardcoded patterns"""
        patterns = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
                
            for line_num, line in enumerate(lines, 1):
                line_patterns = self._scan_line(file_path, line_num, line, lines)
                patterns.extend(line_patterns)
                
        except Exception as e:
            logger.warning(f"⚠️ Error scanning {file_path}: {e}")
            
        return patterns

    def _scan_line(self, file_path: Path, line_num: int, line: str, all_lines: List[str]) -> List[HardcodedPattern]:
        """Scan a single line for hardcoded patterns"""
        patterns = []
        
        # Skip comments in Rust files
        if file_path.suffix == '.rs':
            stripped = line.strip()
            if stripped.startswith('//') or stripped.startswith('///'):
                return patterns
                
        # Check each migration rule
        for rule in self.migration_rules.values():
            matches = re.finditer(rule.pattern_regex, line, re.IGNORECASE)
            
            for match in matches:
                # Get context lines
                context_start = max(0, line_num - 3)
                context_end = min(len(all_lines), line_num + 2)
                context_lines = [l.rstrip() for l in all_lines[context_start:context_end]]
                
                # Calculate confidence based on context
                confidence = self._calculate_pattern_confidence(rule, line, context_lines)
                
                # Determine pattern type
                pattern_type = self._classify_pattern_type(rule, line)
                
                pattern = HardcodedPattern(
                    pattern=match.group(),
                    file_path=str(file_path.relative_to(self.project_root)),
                    line_number=line_num,
                    line_content=line.rstrip(),
                    context_lines=context_lines,
                    pattern_type=pattern_type,
                    confidence=confidence,
                    suggested_replacement=rule.replacement_template,
                    urgency=rule.urgency,
                    complexity=rule.complexity
                )
                
                patterns.append(pattern)
                
        return patterns

    def _calculate_pattern_confidence(self, rule: MigrationRule, line: str, context: List[str]) -> float:
        """Calculate confidence score for a pattern match"""
        base_confidence = rule.confidence
        
        # Boost confidence if context hints are present
        context_text = ' '.join(context).lower()
        hint_matches = sum(1 for hint in rule.context_hints if hint in context_text)
        hint_boost = min(0.1 * hint_matches, 0.2)
        
        # Reduce confidence if in test files or examples
        if any(test_indicator in line.lower() for test_indicator in ['test', 'example', 'demo']):
            base_confidence *= 0.7
            
        return min(base_confidence + hint_boost, 1.0)

    def _classify_pattern_type(self, rule: MigrationRule, line: str) -> str:
        """Classify the type of hardcoded pattern"""
        if rule.from_pattern in ['beardog', 'nestgate', 'toadstool', 'squirrel']:
            return 'primal'
        elif rule.from_pattern in ['kubernetes', 'k8s', 'consul', 'docker']:
            return 'external_service'
        elif any(indicator in line.lower() for indicator in ['http://', 'https://', ':', 'endpoint']):
            return 'endpoint'
        else:
            return 'connection'

    def migrate_pattern(self, pattern: HardcodedPattern, dry_run: bool = False) -> MigrationResult:
        """Migrate a single hardcoded pattern"""
        logger.info(f"🔄 Migrating pattern: {pattern.pattern} in {pattern.file_path}:{pattern.line_number}")
        
        try:
            # Get migration rule
            rule = self.migration_rules.get(pattern.pattern.lower())
            if not rule:
                return MigrationResult(
                    original_pattern=pattern.pattern,
                    migrated_pattern=f"capability_{pattern.pattern.lower()}",
                    file_path=pattern.file_path,
                    line_number=pattern.line_number,
                    success=False,
                    error_message="No migration rule found",
                    confidence=0.5,
                    requires_manual_review=True
                )
            
            # Read file
            file_path = self.project_root / pattern.file_path
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Apply migration
            original_line = lines[pattern.line_number - 1]
            migrated_line = re.sub(
                rule.pattern_regex,
                rule.replacement_template,
                original_line,
                flags=re.IGNORECASE
            )
            
            if not dry_run:
                # Create backup
                backup_path = file_path.with_suffix(file_path.suffix + '.backup')
                if not backup_path.exists():
                    shutil.copy2(file_path, backup_path)
                
                # Write migrated file
                lines[pattern.line_number - 1] = migrated_line
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.writelines(lines)
            
            return MigrationResult(
                original_pattern=pattern.pattern,
                migrated_pattern=rule.replacement_template,
                file_path=pattern.file_path,
                line_number=pattern.line_number,
                success=True,
                error_message=None,
                confidence=pattern.confidence,
                requires_manual_review=rule.breaking_change or pattern.confidence < 0.8
            )
            
        except Exception as e:
            logger.error(f"❌ Migration failed: {e}")
            return MigrationResult(
                original_pattern=pattern.pattern,
                migrated_pattern="",
                file_path=pattern.file_path,
                line_number=pattern.line_number,
                success=False,
                error_message=str(e),
                confidence=0.0,
                requires_manual_review=True
            )

    def migrate_all_patterns(self, dry_run: bool = False, pattern_filter: Optional[str] = None) -> List[MigrationResult]:
        """Migrate all found patterns"""
        logger.info(f"🚀 Starting migration of {len(self.found_patterns)} patterns (dry_run={dry_run})")
        
        results = []
        
        for pattern in self.found_patterns:
            # Apply pattern filter if specified
            if pattern_filter and pattern_filter.lower() not in pattern.pattern.lower():
                continue
                
            result = self.migrate_pattern(pattern, dry_run)
            results.append(result)
            
            if result.success:
                logger.info(f"✅ Migrated: {result.original_pattern} → {result.migrated_pattern}")
            else:
                logger.warning(f"❌ Failed: {result.original_pattern} - {result.error_message}")
        
        self.migration_results = results
        return results

    def generate_migration_report(self) -> Dict:
        """Generate comprehensive migration report"""
        logger.info("📊 Generating migration report...")
        
        # Analyze patterns by type
        pattern_types = {}
        for pattern in self.found_patterns:
            pattern_types[pattern.pattern_type] = pattern_types.get(pattern.pattern_type, 0) + 1
        
        # Analyze patterns by urgency
        urgency_breakdown = {}
        for pattern in self.found_patterns:
            urgency = pattern.urgency.value
            urgency_breakdown[urgency] = urgency_breakdown.get(urgency, 0) + 1
        
        # Analyze migration results
        successful_migrations = sum(1 for r in self.migration_results if r.success)
        failed_migrations = len(self.migration_results) - successful_migrations
        manual_review_required = sum(1 for r in self.migration_results if r.requires_manual_review)
        
        # Calculate effort estimates
        effort_hours = self._calculate_migration_effort()
        
        report = {
            "scan_summary": {
                "total_patterns_found": len(self.found_patterns),
                "pattern_types": pattern_types,
                "urgency_breakdown": urgency_breakdown,
                "files_affected": len(set(p.file_path for p in self.found_patterns))
            },
            "migration_summary": {
                "total_migrations_attempted": len(self.migration_results),
                "successful_migrations": successful_migrations,
                "failed_migrations": failed_migrations,
                "manual_review_required": manual_review_required,
                "success_rate": successful_migrations / len(self.migration_results) if self.migration_results else 0
            },
            "effort_estimation": {
                "estimated_hours": effort_hours,
                "critical_patterns": len([p for p in self.found_patterns if p.urgency == MigrationUrgency.CRITICAL]),
                "breaking_changes": len([r for r in self.migration_results if r.requires_manual_review])
            },
            "detailed_patterns": [asdict(p) for p in self.found_patterns],
            "migration_results": [asdict(r) for r in self.migration_results]
        }
        
        return report

    def _calculate_migration_effort(self) -> int:
        """Calculate estimated effort in hours"""
        effort_map = {
            MigrationComplexity.SIMPLE: 1,
            MigrationComplexity.MODERATE: 4,
            MigrationComplexity.COMPLEX: 16,
            MigrationComplexity.CRITICAL: 40
        }
        
        total_hours = 0
        for pattern in self.found_patterns:
            total_hours += effort_map.get(pattern.complexity, 4)
            
        return total_hours

    def verify_migrations(self) -> Dict[str, bool]:
        """Verify that migrations don't break compilation"""
        logger.info("🔍 Verifying migrations don't break compilation...")
        
        verification_results = {}
        
        try:
            # Run cargo check to verify compilation
            result = subprocess.run(
                ["cargo", "check", "--all-targets"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            
            verification_results["cargo_check"] = result.returncode == 0
            if result.returncode != 0:
                logger.warning(f"❌ Cargo check failed:\n{result.stderr}")
            else:
                logger.info("✅ Cargo check passed")
                
        except subprocess.TimeoutExpired:
            logger.warning("⚠️ Cargo check timed out")
            verification_results["cargo_check"] = False
        except Exception as e:
            logger.error(f"❌ Cargo check error: {e}")
            verification_results["cargo_check"] = False
            
        try:
            # Run cargo test to verify tests still pass
            result = subprocess.run(
                ["cargo", "test", "--no-run"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=600  # 10 minute timeout
            )
            
            verification_results["cargo_test_compile"] = result.returncode == 0
            if result.returncode != 0:
                logger.warning(f"❌ Test compilation failed:\n{result.stderr}")
            else:
                logger.info("✅ Test compilation passed")
                
        except Exception as e:
            logger.error(f"❌ Test compilation error: {e}")
            verification_results["cargo_test_compile"] = False
            
        return verification_results

    def rollback_migrations(self) -> bool:
        """Rollback all migrations using backup files"""
        logger.info("🔙 Rolling back migrations...")
        
        try:
            backup_files = list(self.project_root.rglob("*.backup"))
            
            for backup_file in backup_files:
                original_file = backup_file.with_suffix('')
                shutil.copy2(backup_file, original_file)
                backup_file.unlink()  # Remove backup
                logger.info(f"📄 Restored: {original_file.relative_to(self.project_root)}")
            
            logger.info(f"✅ Rollback complete: {len(backup_files)} files restored")
            return True
            
        except Exception as e:
            logger.error(f"❌ Rollback failed: {e}")
            return False

def main():
    parser = argparse.ArgumentParser(description="Vendor Hardcoding Elimination Tool")
    parser.add_argument("--project-root", default=".", help="Project root directory")
    parser.add_argument("--scan", action="store_true", help="Scan codebase for hardcoded patterns")
    parser.add_argument("--migrate", help="Migrate specific pattern")
    parser.add_argument("--migrate-all", action="store_true", help="Migrate all patterns")
    parser.add_argument("--dry-run", action="store_true", help="Perform dry run without making changes")
    parser.add_argument("--report", action="store_true", help="Generate migration report")
    parser.add_argument("--verify", action="store_true", help="Verify migrations don't break compilation")
    parser.add_argument("--rollback", action="store_true", help="Rollback all migrations")
    parser.add_argument("--output", help="Output file for reports")
    
    args = parser.parse_args()
    
    # Initialize eliminator
    eliminator = VendorHardcodingEliminator(args.project_root)
    
    # Scan for patterns
    if args.scan or args.migrate_all or args.migrate:
        patterns = eliminator.scan_codebase()
        print(f"🔍 Found {len(patterns)} hardcoded patterns")
        
        # Show summary by urgency
        urgency_counts = {}
        for pattern in patterns:
            urgency = pattern.urgency.value
            urgency_counts[urgency] = urgency_counts.get(urgency, 0) + 1
            
        for urgency, count in sorted(urgency_counts.items()):
            print(f"   {urgency.upper()}: {count} patterns")
    
    # Migrate specific pattern
    if args.migrate:
        matching_patterns = [p for p in eliminator.found_patterns 
                           if args.migrate.lower() in p.pattern.lower()]
        if matching_patterns:
            results = []
            for pattern in matching_patterns:
                result = eliminator.migrate_pattern(pattern, args.dry_run)
                results.append(result)
            eliminator.migration_results = results
            print(f"🔄 Migrated {len(results)} patterns matching '{args.migrate}'")
        else:
            print(f"❌ No patterns found matching '{args.migrate}'")
    
    # Migrate all patterns  
    if args.migrate_all:
        results = eliminator.migrate_all_patterns(args.dry_run)
        successful = sum(1 for r in results if r.success)
        print(f"🚀 Migration complete: {successful}/{len(results)} successful")
    
    # Verify migrations
    if args.verify:
        verification = eliminator.verify_migrations()
        for check, passed in verification.items():
            status = "✅ PASSED" if passed else "❌ FAILED"
            print(f"{check}: {status}")
    
    # Rollback migrations
    if args.rollback:
        success = eliminator.rollback_migrations()
        if success:
            print("✅ Rollback successful")
        else:
            print("❌ Rollback failed")
    
    # Generate report
    if args.report or args.output:
        report = eliminator.generate_migration_report()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(report, f, indent=2)
            print(f"📊 Report saved to: {args.output}")
        else:
            print("\n📊 MIGRATION REPORT")
            print("=" * 50)
            scan = report["scan_summary"]
            print(f"Patterns found: {scan['total_patterns_found']}")
            print(f"Files affected: {scan['files_affected']}")
            
            if eliminator.migration_results:
                migration = report["migration_summary"]
                print(f"Migrations attempted: {migration['total_migrations_attempted']}")
                print(f"Success rate: {migration['success_rate']:.1%}")
                print(f"Manual review required: {migration['manual_review_required']}")
            
            effort = report["effort_estimation"]
            print(f"Estimated effort: {effort['estimated_hours']} hours")
            print(f"Critical patterns: {effort['critical_patterns']}")

if __name__ == "__main__":
    main() 