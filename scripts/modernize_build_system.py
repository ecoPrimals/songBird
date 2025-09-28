#!/usr/bin/env python3
"""
Build System Modernization Script - Phase 4

This script modernizes the build system by fixing workspace configuration warnings,
optimizing profile configurations, and ensuring 100% build success.

Usage:
    python3 scripts/modernize_build_system.py --check     # Check build system health
    python3 scripts/modernize_build_system.py --fix       # Fix build system issues
    python3 scripts/modernize_build_system.py --optimize  # Optimize build profiles
"""

import os
import re
import sys
import json
import argparse
import subprocess
from pathlib import Path
from typing import List, Dict, Set, Tuple, Optional
from dataclasses import dataclass, asdict

@dataclass
class BuildIssue:
    """Represents a build system issue"""
    file_path: str
    issue_type: str  # 'workspace_config', 'profile_config', 'feature_flags', 'dependency'
    severity: str    # 'error', 'warning', 'info'
    description: str
    fix_suggestion: str
    line_number: Optional[int] = None

class BuildSystemModernizer:
    """Main class for modernizing the build system"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.issues: List[BuildIssue] = []
        
        # Cargo.toml files to check
        self.cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        
        # Modern profile configurations
        self.modern_profiles = {
            'dev': {
                'opt-level': 0,
                'debug': True,
                'debug-assertions': True,
                'overflow-checks': True,
                'lto': False,
                'panic': 'unwind',
                'incremental': True,
                'codegen-units': 256,
            },
            'release': {
                'opt-level': 3,
                'debug': False,
                'debug-assertions': False,
                'overflow-checks': False,
                'lto': True,
                'panic': 'abort',
                'incremental': False,
                'codegen-units': 1,
            },
            'test': {
                'opt-level': 0,
                'debug': True,
                'debug-assertions': True,
                'overflow-checks': True,
            },
            'bench': {
                'opt-level': 3,
                'debug': False,
                'lto': True,
            }
        }
        
        # Modern linting configuration
        self.modern_lints = {
            'clippy': {
                'all': 'warn',
                'pedantic': 'warn',
                'nursery': 'warn',
                'cargo': 'warn',
                # Allow some pedantic lints that are too noisy
                'module-name-repetitions': 'allow',
                'similar-names': 'allow',
                'too-many-arguments': 'allow',
                'too-many-lines': 'allow',
                'pub-use': 'allow',  # For re-export crates
            },
            'rust': {
                'unsafe-code': 'deny',
                'missing-docs': 'warn',
                'unused-imports': 'warn',
                'dead-code': 'warn',
            }
        }
    
    def check_build_system(self) -> List[BuildIssue]:
        """Check the build system for issues"""
        print("🔍 Checking build system health...")
        
        # Check workspace configuration
        self._check_workspace_config()
        
        # Check individual crate configurations
        for cargo_file in self.cargo_files:
            self._check_cargo_file(cargo_file)
        
        # Check for compilation issues
        self._check_compilation()
        
        print(f"📊 Found {len(self.issues)} build system issues")
        return self.issues
    
    def _check_workspace_config(self) -> None:
        """Check the main workspace configuration"""
        workspace_cargo = self.project_root / "Cargo.toml"
        
        if not workspace_cargo.exists():
            self.issues.append(BuildIssue(
                file_path=str(workspace_cargo),
                issue_type='workspace_config',
                severity='error',
                description="Missing workspace Cargo.toml",
                fix_suggestion="Create workspace Cargo.toml with proper configuration"
            ))
            return
        
        try:
            with open(workspace_cargo, 'r') as f:
                content = f.read()
            
            # Check for workspace definition
            if '[workspace]' not in content:
                self.issues.append(BuildIssue(
                    file_path=str(workspace_cargo),
                    issue_type='workspace_config',
                    severity='error',
                    description="Missing [workspace] section",
                    fix_suggestion="Add [workspace] section with members list"
                ))
            
            # Check for commented out members
            if '# "crates/' in content:
                self.issues.append(BuildIssue(
                    file_path=str(workspace_cargo),
                    issue_type='workspace_config',
                    severity='warning',
                    description="Commented out workspace members found",
                    fix_suggestion="Enable or remove commented out crate members"
                ))
            
            # Check for workspace lints
            if '[workspace.lints' not in content:
                self.issues.append(BuildIssue(
                    file_path=str(workspace_cargo),
                    issue_type='workspace_config',
                    severity='info',
                    description="Missing workspace-level linting configuration",
                    fix_suggestion="Add [workspace.lints] section for consistent linting"
                ))
            
            # Check for workspace profiles
            if '[workspace.profile' not in content and '[profile.' in content:
                self.issues.append(BuildIssue(
                    file_path=str(workspace_cargo),
                    issue_type='profile_config',
                    severity='warning',
                    description="Profiles should be defined at workspace level",
                    fix_suggestion="Move profile configurations to [workspace.profile.*] sections"
                ))
                
        except Exception as e:
            self.issues.append(BuildIssue(
                file_path=str(workspace_cargo),
                issue_type='workspace_config',
                severity='error',
                description=f"Error reading workspace Cargo.toml: {e}",
                fix_suggestion="Fix file encoding or syntax errors"
            ))
    
    def _check_cargo_file(self, cargo_file: Path) -> None:
        """Check an individual Cargo.toml file"""
        try:
            with open(cargo_file, 'r') as f:
                content = f.read()
            
            # Check for malformed lints sections
            if '[lints.clippy]' in content and 'all = "warn"' in content:
                # This is good, but check for malformed entries
                lines = content.split('\n')
                in_clippy_section = False
                for i, line in enumerate(lines, 1):
                    if '[lints.clippy]' in line:
                        in_clippy_section = True
                    elif line.startswith('[') and in_clippy_section:
                        in_clippy_section = False
                    elif in_clippy_section and '=' in line and not line.strip().startswith('#'):
                        # Check for malformed lint entries
                        if not re.match(r'^[a-z-]+ = "(allow|warn|deny)"', line.strip()):
                            self.issues.append(BuildIssue(
                                file_path=str(cargo_file),
                                issue_type='workspace_config',
                                severity='warning',
                                description=f"Malformed lint entry at line {i}",
                                fix_suggestion="Fix lint entry format: lint-name = \"level\"",
                                line_number=i
                            ))
            
            # Check for profile definitions in non-workspace crates
            if cargo_file.name != "Cargo.toml" or "workspace" not in cargo_file.parent.name:
                if '[profile.' in content:
                    self.issues.append(BuildIssue(
                        file_path=str(cargo_file),
                        issue_type='profile_config',
                        severity='info',
                        description="Profile configuration in crate-level Cargo.toml",
                        fix_suggestion="Move profile configurations to workspace level"
                    ))
            
            # Check for legacy feature flags
            if 'legacy-compat' in content or 'backward-compat' in content:
                self.issues.append(BuildIssue(
                    file_path=str(cargo_file),
                    issue_type='feature_flags',
                    severity='warning',
                    description="Legacy compatibility feature flags found",
                    fix_suggestion="Remove legacy compatibility feature flags"
                ))
                
        except Exception as e:
            self.issues.append(BuildIssue(
                file_path=str(cargo_file),
                issue_type='workspace_config',
                severity='error',
                description=f"Error reading Cargo.toml: {e}",
                fix_suggestion="Fix file encoding or syntax errors"
            ))
    
    def _check_compilation(self) -> None:
        """Check for compilation issues"""
        try:
            # Run cargo check to identify compilation issues
            result = subprocess.run(
                ['cargo', 'check', '--workspace'],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            
            if result.returncode != 0:
                # Parse compilation errors
                error_lines = result.stderr.split('\n')
                for line in error_lines:
                    if 'error:' in line or 'warning:' in line:
                        self.issues.append(BuildIssue(
                            file_path="compilation",
                            issue_type='dependency',
                            severity='error' if 'error:' in line else 'warning',
                            description=line.strip(),
                            fix_suggestion="Fix compilation errors before proceeding"
                        ))
            else:
                print("✅ Compilation check passed")
                
        except subprocess.TimeoutExpired:
            self.issues.append(BuildIssue(
                file_path="compilation",
                issue_type='dependency',
                severity='warning',
                description="Compilation check timed out",
                fix_suggestion="Check for hanging builds or infinite loops"
            ))
        except Exception as e:
            self.issues.append(BuildIssue(
                file_path="compilation",
                issue_type='dependency',
                severity='warning',
                description=f"Could not run compilation check: {e}",
                fix_suggestion="Ensure cargo is available and project is valid"
            ))
    
    def fix_build_issues(self, dry_run: bool = True) -> Dict:
        """Fix identified build system issues"""
        print(f"🔧 {'[DRY RUN] ' if dry_run else ''}Fixing build system issues...")
        
        fix_stats = {
            'files_modified': 0,
            'issues_fixed': 0,
            'workspace_updated': False,
            'profiles_optimized': False,
        }
        
        # Group issues by file for efficient processing
        issues_by_file = {}
        for issue in self.issues:
            if issue.severity in ['error', 'warning']:  # Only fix errors and warnings
                if issue.file_path not in issues_by_file:
                    issues_by_file[issue.file_path] = []
                issues_by_file[issue.file_path].append(issue)
        
        for file_path, file_issues in issues_by_file.items():
            if file_path == "compilation":
                continue  # Skip compilation issues for now
                
            if self._fix_file_issues(file_path, file_issues, dry_run):
                fix_stats['files_modified'] += 1
                fix_stats['issues_fixed'] += len(file_issues)
        
        return fix_stats
    
    def _fix_file_issues(self, file_path: str, issues: List[BuildIssue], dry_run: bool) -> bool:
        """Fix issues in a specific file"""
        try:
            path = Path(file_path)
            if not path.exists():
                return False
            
            with open(path, 'r') as f:
                content = f.read()
            
            original_content = content
            
            for issue in issues:
                if issue.issue_type == 'feature_flags' and 'legacy-compat' in issue.description:
                    # Remove legacy feature flags
                    content = re.sub(r'legacy-compat\s*=\s*\[.*?\].*\n', '', content)
                    content = re.sub(r'backward-compat\s*=\s*\[.*?\].*\n', '', content)
                
                elif issue.issue_type == 'workspace_config' and 'linting' in issue.description:
                    # Add workspace linting configuration
                    if '[workspace.lints' not in content:
                        workspace_lints = self._generate_workspace_lints()
                        # Find insertion point after [workspace] section
                        insertion_point = content.find('[workspace]')
                        if insertion_point != -1:
                            # Find end of workspace section
                            next_section = content.find('\n[', insertion_point + 10)
                            if next_section != -1:
                                content = content[:next_section] + '\n' + workspace_lints + content[next_section:]
                            else:
                                content += '\n' + workspace_lints
            
            if content != original_content:
                if not dry_run:
                    with open(path, 'w') as f:
                        f.write(content)
                    print(f"✅ Fixed issues in {path}")
                else:
                    print(f"🔍 [DRY RUN] Would fix issues in {path}")
                return True
            
            return False
            
        except Exception as e:
            print(f"⚠️  Error fixing {file_path}: {e}")
            return False
    
    def _generate_workspace_lints(self) -> str:
        """Generate modern workspace linting configuration"""
        return """
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"
# Allow some pedantic lints that are too noisy
module-name-repetitions = "allow"
similar-names = "allow"
too-many-arguments = "allow"
too-many-lines = "allow"
pub-use = "allow"  # For re-export crates

[workspace.lints.rust]
unsafe-code = "deny"
missing-docs = "warn"
unused-imports = "warn"
dead-code = "warn"
"""
    
    def optimize_profiles(self, dry_run: bool = True) -> Dict:
        """Optimize build profiles for performance"""
        print(f"⚡ {'[DRY RUN] ' if dry_run else ''}Optimizing build profiles...")
        
        workspace_cargo = self.project_root / "Cargo.toml"
        
        if not workspace_cargo.exists():
            return {'profiles_added': 0}
        
        try:
            with open(workspace_cargo, 'r') as f:
                content = f.read()
            
            # Add optimized profiles if they don't exist
            profiles_section = self._generate_optimized_profiles()
            
            # Check if profiles already exist
            if '[workspace.profile' not in content and '[profile.' not in content:
                content += '\n' + profiles_section
                
                if not dry_run:
                    with open(workspace_cargo, 'w') as f:
                        f.write(content)
                    print("✅ Added optimized build profiles")
                else:
                    print("🔍 [DRY RUN] Would add optimized build profiles")
                
                return {'profiles_added': len(self.modern_profiles)}
            else:
                print("ℹ️  Build profiles already configured")
                return {'profiles_added': 0}
                
        except Exception as e:
            print(f"⚠️  Error optimizing profiles: {e}")
            return {'profiles_added': 0}
    
    def _generate_optimized_profiles(self) -> str:
        """Generate optimized profile configurations"""
        profiles_toml = "\n# ============================================================================\n"
        profiles_toml += "# OPTIMIZED BUILD PROFILES\n"
        profiles_toml += "# ============================================================================\n\n"
        
        for profile_name, config in self.modern_profiles.items():
            profiles_toml += f"[workspace.profile.{profile_name}]\n"
            for key, value in config.items():
                if isinstance(value, bool):
                    profiles_toml += f"{key.replace('_', '-')} = {str(value).lower()}\n"
                elif isinstance(value, str):
                    profiles_toml += f"{key.replace('_', '-')} = \"{value}\"\n"
                else:
                    profiles_toml += f"{key.replace('_', '-')} = {value}\n"
            profiles_toml += "\n"
        
        return profiles_toml
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive build system report"""
        report = {
            'total_issues': len(self.issues),
            'by_severity': {'error': 0, 'warning': 0, 'info': 0},
            'by_type': {},
            'critical_issues': [],
            'recommendations': [],
        }
        
        # Analyze issues
        for issue in self.issues:
            report['by_severity'][issue.severity] += 1
            
            if issue.issue_type not in report['by_type']:
                report['by_type'][issue.issue_type] = 0
            report['by_type'][issue.issue_type] += 1
            
            if issue.severity == 'error':
                report['critical_issues'].append({
                    'file': issue.file_path,
                    'description': issue.description,
                    'fix': issue.fix_suggestion
                })
        
        # Generate recommendations
        if report['by_severity']['error'] > 0:
            report['recommendations'].append("Fix compilation errors before proceeding with other improvements")
        
        if report['by_type'].get('workspace_config', 0) > 0:
            report['recommendations'].append("Modernize workspace configuration for better build performance")
        
        if report['by_type'].get('profile_config', 0) > 0:
            report['recommendations'].append("Optimize build profiles for development and release builds")
        
        return report

def main():
    parser = argparse.ArgumentParser(description="Modernize Songbird build system")
    parser.add_argument("--check", action="store_true", help="Check build system health")
    parser.add_argument("--fix", action="store_true", help="Fix build system issues")
    parser.add_argument("--optimize", action="store_true", help="Optimize build profiles")
    parser.add_argument("--report", action="store_true", help="Generate detailed report")
    parser.add_argument("--dry-run", action="store_true", help="Perform dry run (no actual changes)")
    parser.add_argument("--output", help="Output file for report (JSON format)")
    
    args = parser.parse_args()
    
    if not any([args.check, args.fix, args.optimize, args.report]):
        parser.print_help()
        return
    
    # Determine project root (assume script is in scripts/ directory)
    project_root = Path(__file__).parent.parent
    
    modernizer = BuildSystemModernizer(str(project_root))
    
    if args.check or args.fix or args.report:
        issues = modernizer.check_build_system()
    
    if args.report:
        report = modernizer.generate_report()
        
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(report, f, indent=2)
            print(f"📊 Report saved to {args.output}")
        else:
            print("\n📊 BUILD SYSTEM REPORT")
            print("=" * 50)
            print(f"Total issues: {report['total_issues']}")
            print(f"Errors: {report['by_severity']['error']}")
            print(f"Warnings: {report['by_severity']['warning']}")
            print(f"Info: {report['by_severity']['info']}")
            
            if report['critical_issues']:
                print("\n🚨 Critical Issues:")
                for issue in report['critical_issues'][:3]:  # Show top 3
                    print(f"  • {issue['file']}: {issue['description']}")
            
            if report['recommendations']:
                print("\n💡 Recommendations:")
                for rec in report['recommendations']:
                    print(f"  • {rec}")
    
    if args.fix:
        dry_run = args.dry_run
        stats = modernizer.fix_build_issues(dry_run)
        
        print(f"\n🔧 {'[DRY RUN] ' if dry_run else ''}BUILD SYSTEM FIX SUMMARY")
        print("=" * 50)
        print(f"Files modified: {stats['files_modified']}")
        print(f"Issues fixed: {stats['issues_fixed']}")
    
    if args.optimize:
        dry_run = args.dry_run
        stats = modernizer.optimize_profiles(dry_run)
        
        print(f"\n⚡ {'[DRY RUN] ' if dry_run else ''}PROFILE OPTIMIZATION SUMMARY")
        print("=" * 50)
        print(f"Profiles added: {stats['profiles_added']}")
        
        if not dry_run and stats['profiles_added'] > 0:
            print("\n✅ Build system modernization complete!")
            print("🚀 Optimized profiles will improve build performance")
        elif dry_run:
            print("\n🔍 Dry run complete. Use --optimize without --dry-run to apply changes")

if __name__ == "__main__":
    main() 