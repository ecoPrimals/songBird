#!/usr/bin/env python3
"""
Songbird Modernization Verification Script
==========================================

Validates that the canonical modernization has been successfully applied
and identifies any remaining legacy patterns or issues.
"""

import os
import re
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple

class ModernizationValidator:
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.crates_path = self.root_path / "crates"
        
        # Patterns that should NOT exist after modernization
        self.legacy_patterns = {
            # Legacy Result patterns
            r'Result<[^,>]+,\s*Box<dyn\s+std::error::Error>': 'Legacy Box<dyn Error> pattern',
            r'Result<[^,>]+,\s*Box<dyn\s+Error>': 'Legacy Box<dyn Error> pattern',
            r'std::result::Result<[^,>]+,\s*SongbirdError>': 'Should use SongbirdResult<T>',
            
            # Malformed function signatures
            r'fn\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\([^)]*\)\s*->\s*\{': 'Malformed function signature',
            
            # Duplicate imports
            r'use\s+songbird_errors::\{[^}]*,\s*SongbirdError,\s*SongbirdError\}': 'Duplicate SongbirdError import',
            
            # Deprecated patterns
            r'\.unwrap_data\(\)': 'Deprecated unwrap_data() call',
            r'Ok\(SongbirdResponse::success\(\(\)\)\)': 'Should use Ok(success(()))',
        }
        
        # Patterns that SHOULD exist (canonical patterns)
        self.required_patterns = {
            r'use\s+songbird_errors::\{[^}]*SongbirdResult[^}]*\}': 'SongbirdResult import',
            r'use\s+songbird_errors::\{[^}]*SongbirdError[^}]*\}': 'SongbirdError import',
            r'pub\s+async\s+fn\s+[a-zA-Z_][a-zA-Z0-9_]*\s*\([^)]*\)\s*->\s*SongbirdResult<[^>]+>': 'Canonical async function',
        }

    def check_file(self, file_path: Path) -> Dict[str, List[Tuple[int, str]]]:
        """Check a single file for legacy patterns and canonical compliance"""
        if not file_path.suffix == '.rs':
            return {}
            
        results = {
            'legacy_issues': [],
            'missing_patterns': [],
            'canonical_compliance': []
        }
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()
            
            # Check for legacy patterns
            for i, line in enumerate(lines, 1):
                for pattern, description in self.legacy_patterns.items():
                    if re.search(pattern, line):
                        results['legacy_issues'].append((i, f"{description}: {line.strip()}"))
            
            # Check for canonical patterns in the entire file
            content = ''.join(lines)
            for pattern, description in self.required_patterns.items():
                if re.search(pattern, content):
                    results['canonical_compliance'].append(description)
                    
        except Exception as e:
            results['legacy_issues'].append((0, f"Error reading file: {e}"))
            
        return results

    def validate_crate(self, crate_name: str) -> Dict[str, any]:
        """Validate a specific crate"""
        crate_path = self.crates_path / crate_name
        if not crate_path.exists():
            return {"error": f"Crate not found: {crate_name}"}
        
        print(f"\n🔍 Validating crate: {crate_name}")
        
        total_files = 0
        files_with_issues = 0
        total_legacy_issues = 0
        canonical_files = 0
        
        results = {
            "files_checked": 0,
            "files_with_legacy_issues": 0,
            "total_legacy_issues": 0,
            "canonical_compliant_files": 0,
            "issues": []
        }
        
        # Check all Rust files
        for rust_file in crate_path.rglob("*.rs"):
            total_files += 1
            file_results = self.check_file(rust_file)
            
            if file_results.get('legacy_issues'):
                files_with_issues += 1
                total_legacy_issues += len(file_results['legacy_issues'])
                
                relative_path = rust_file.relative_to(self.root_path)
                results["issues"].append({
                    "file": str(relative_path),
                    "legacy_issues": file_results['legacy_issues']
                })
                
                print(f"  ⚠️  {relative_path}: {len(file_results['legacy_issues'])} legacy issues")
            
            if file_results.get('canonical_compliance'):
                canonical_files += 1
        
        results.update({
            "files_checked": total_files,
            "files_with_legacy_issues": files_with_issues,
            "total_legacy_issues": total_legacy_issues,
            "canonical_compliant_files": canonical_files
        })
        
        if files_with_issues == 0:
            print(f"  ✅ No legacy issues found in {total_files} files")
        else:
            print(f"  🔧 Found {total_legacy_issues} legacy issues in {files_with_issues}/{total_files} files")
            
        return results

    def check_compilation(self) -> Dict[str, any]:
        """Check if the codebase compiles successfully"""
        print("\n🏗️  Testing compilation...")
        
        try:
            result = subprocess.run(
                ["cargo", "check", "--all"],
                cwd=self.root_path,
                capture_output=True,
                text=True,
                timeout=300  # 5 minutes timeout
            )
            
            if result.returncode == 0:
                print("  ✅ All crates compile successfully!")
                return {"success": True, "errors": []}
            else:
                print("  ❌ Compilation errors found:")
                errors = result.stderr.split('\n')[:10]  # Show first 10 errors
                for error in errors:
                    if error.strip():
                        print(f"    {error}")
                return {"success": False, "errors": errors}
                
        except subprocess.TimeoutExpired:
            print("  ⏰ Compilation timeout (5 minutes)")
            return {"success": False, "errors": ["Compilation timeout"]}
        except Exception as e:
            print(f"  ❌ Compilation check failed: {e}")
            return {"success": False, "errors": [str(e)]}

    def validate_all(self) -> Dict[str, any]:
        """Validate the entire modernization"""
        print("🚀 Starting Modernization Validation")
        print("=" * 50)
        
        # Priority crates to check first
        priority_crates = [
            "songbird-errors",
            "songbird-discovery", 
            "songbird-test-utils",
            "songbird-universal",
            "songbird-core",
            "songbird-network",
            "songbird-federation",
            "songbird-security"
        ]
        
        validation_results = {
            "overall_success": True,
            "crates_validated": 0,
            "total_files_checked": 0,
            "total_legacy_issues": 0,
            "compilation_success": False,
            "crate_results": {}
        }
        
        # Validate priority crates
        for crate_name in priority_crates:
            if (self.crates_path / crate_name).exists():
                results = self.validate_crate(crate_name)
                validation_results["crate_results"][crate_name] = results
                
                if not isinstance(results, dict) or results.get("total_legacy_issues", 0) > 0:
                    validation_results["overall_success"] = False
                
                validation_results["crates_validated"] += 1
                validation_results["total_files_checked"] += results.get("files_checked", 0)
                validation_results["total_legacy_issues"] += results.get("total_legacy_issues", 0)
        
        # Check compilation
        compilation_results = self.check_compilation()
        validation_results["compilation_success"] = compilation_results["success"]
        
        if not compilation_results["success"]:
            validation_results["overall_success"] = False
        
        # Summary
        print("\n" + "=" * 50)
        print("🎯 Modernization Validation Summary")
        print("=" * 50)
        
        print(f"📊 Crates validated: {validation_results['crates_validated']}")
        print(f"📁 Files checked: {validation_results['total_files_checked']}")
        print(f"🔧 Legacy issues found: {validation_results['total_legacy_issues']}")
        print(f"🏗️  Compilation: {'✅ Success' if validation_results['compilation_success'] else '❌ Failed'}")
        
        if validation_results["overall_success"]:
            print("\n🎉 Modernization SUCCESSFUL!")
            print("✅ All crates follow canonical patterns")
            print("✅ No legacy patterns detected")
            print("✅ Compilation successful")
        else:
            print("\n⚠️  Modernization needs attention:")
            if validation_results["total_legacy_issues"] > 0:
                print(f"🔧 {validation_results['total_legacy_issues']} legacy issues need fixing")
            if not validation_results["compilation_success"]:
                print("❌ Compilation errors need resolution")
        
        return validation_results

def main():
    """Main entry point"""
    root_path = "."
    validator = ModernizationValidator(root_path)
    
    if len(os.sys.argv) > 1 and os.sys.argv[1] == "--crate":
        # Validate specific crate
        crate_name = os.sys.argv[2] if len(os.sys.argv) > 2 else "songbird-discovery"
        results = validator.validate_crate(crate_name)
        print(f"\n🎯 Validation complete for {crate_name}")
    else:
        # Validate all
        results = validator.validate_all()

if __name__ == "__main__":
    main() 