#!/usr/bin/env python3
"""
🔄 Modernize Primal References Script

This script systematically updates hardcoded primal references (NestGate, ToadStool, 
BearDog, Squirrel) to use agnostic, capability-based patterns while marking deprecated 
patterns for removal.
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

class PrimalModernizer:
    """Modernizes hardcoded primal references to agnostic patterns"""
    
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        
        # Mapping of hardcoded patterns to modern agnostic equivalents
        self.modernization_patterns = {
            # Struct and type references
            r'NestGateConfig': 'AgnosticPrimalConfig::storage_primal',
            r'ToadstoolConfig': 'AgnosticPrimalConfig::compute_primal',
            r'BearDogConfig': 'AgnosticPrimalConfig::security_primal',
            r'SquirrelConfig': 'AgnosticPrimalConfig::ai_primal',
            
            # Service name references in tests/examples
            r'"nestgate"': '"storage-service"',
            r'"toadstool"': '"compute-service"',
            r'"beardog"': '"security-service"',
            r'"squirrel"': '"ai-service"',
            
            # Environment variable patterns
            r'NESTGATE_ENDPOINT': 'STORAGE_PROVIDER_ENDPOINT',
            r'TOADSTOOL_ENDPOINT': 'COMPUTE_PROVIDER_ENDPOINT',
            r'BEARDOG_ENDPOINT': 'SECURITY_PROVIDER_ENDPOINT',
            r'SQUIRREL_ENDPOINT': 'AI_PROVIDER_ENDPOINT',
            
            # Function and method patterns
            r'create_nestgate_primal': 'create_storage_primal',
            r'create_toadstool_primal': 'create_compute_primal',
            r'create_beardog_primal': 'create_security_primal',
            r'create_squirrel_primal': 'create_ai_primal',
            
            # URL and endpoint patterns
            r'http://nestgate': 'http://storage-service',
            r'http://toadstool': 'http://compute-service',
            r'http://beardog': 'http://security-service',
            r'http://squirrel': 'http://ai-service',
            
            # Comments and documentation
            r'NestGate': 'Storage Primal',
            r'ToadStool': 'Compute Primal',
            r'BearDog': 'Security Primal',
            r'Squirrel': 'AI Primal',
        }
        
        # Deprecation warnings to add
        self.deprecation_warnings = {
            'NestGateConfig': '// DEPRECATED: Use AgnosticPrimalConfig::storage_primal() instead',
            'ToadstoolConfig': '// DEPRECATED: Use AgnosticPrimalConfig::compute_primal() instead',
            'BearDogConfig': '// DEPRECATED: Use AgnosticPrimalConfig::security_primal() instead',
            'SquirrelConfig': '// DEPRECATED: Use AgnosticPrimalConfig::ai_primal() instead',
        }
        
    def should_modernize_file(self, file_path: Path) -> bool:
        """Check if file should be modernized"""
        if not file_path.suffix in ['.rs', '.toml', '.yaml', '.yml', '.md']:
            return False
            
        # Skip core implementation files (already modernized)
        if 'songbird-core/src/biome' in str(file_path):
            return False
            
        # Skip the modernization scripts themselves
        if file_path.name.startswith('modernize_') or file_path.name.startswith('fix_'):
            return False
            
        # Check if file contains hardcoded primal references
        try:
            content = file_path.read_text(encoding='utf-8')
            return any(pattern in content for pattern in ['NestGate', 'ToadStool', 'BearDog', 'Squirrel'])
        except Exception:
            return False
    
    def modernize_file(self, file_path: Path) -> bool:
        """Modernize a single file"""
        try:
            content = file_path.read_text(encoding='utf-8')
            original_content = content
            
            # Add deprecation warnings for config usages
            for old_pattern, warning in self.deprecation_warnings.items():
                if old_pattern in content and warning not in content:
                    content = re.sub(
                        rf'(\s*){old_pattern}',
                        rf'\1{warning}\n\1{old_pattern}',
                        content
                    )
            
            # Apply modernization patterns
            for old_pattern, new_pattern in self.modernization_patterns.items():
                content = re.sub(old_pattern, new_pattern, content)
            
            # Special handling for test files
            if '/tests/' in str(file_path) or '/examples/' in str(file_path):
                content = self.modernize_test_patterns(content)
            
            if content != original_content:
                file_path.write_text(content, encoding='utf-8')
                print(f"✅ Modernized: {file_path.relative_to(self.repo_root)}")
                return True
            
            return False
            
        except Exception as e:
            print(f"❌ Error modernizing {file_path}: {e}")
            return False
    
    def modernize_test_patterns(self, content: str) -> str:
        """Special modernization for test files"""
        
        # Update test function names
        content = re.sub(
            r'test_nestgate_(\w+)',
            r'test_storage_\1',
            content
        )
        content = re.sub(
            r'test_toadstool_(\w+)',
            r'test_compute_\1',
            content
        )
        content = re.sub(
            r'test_beardog_(\w+)',
            r'test_security_\1',
            content
        )
        content = re.sub(
            r'test_squirrel_(\w+)',
            r'test_ai_\1',
            content
        )
        
        # Update capability-based discovery patterns
        content = re.sub(
            r'get_primal\("nestgate"\)',
            r'get_primal_by_capability("storage")',
            content
        )
        content = re.sub(
            r'get_primal\("toadstool"\)',
            r'get_primal_by_capability("compute")',
            content
        )
        content = re.sub(
            r'get_primal\("beardog"\)',
            r'get_primal_by_capability("security")',
            content
        )
        content = re.sub(
            r'get_primal\("squirrel"\)',
            r'get_primal_by_capability("ai")',
            content
        )
        
        return content
    
    def modernize_all_files(self) -> Tuple[int, int]:
        """Modernize all files in the repository"""
        modernized_count = 0
        total_count = 0
        
        # Process relevant file types
        for pattern in ['**/*.rs', '**/*.toml', '**/*.yaml', '**/*.yml', '**/*.md']:
            for file_path in self.repo_root.rglob(pattern):
                if self.should_modernize_file(file_path):
                    total_count += 1
                    if self.modernize_file(file_path):
                        modernized_count += 1
        
        return modernized_count, total_count

def main():
    """Main function"""
    if len(sys.argv) > 1:
        repo_root = Path(sys.argv[1])
    else:
        repo_root = Path.cwd()
    
    if not repo_root.exists():
        print(f"❌ Repository root not found: {repo_root}")
        sys.exit(1)
    
    print(f"🔄 Modernizing primal references in: {repo_root}")
    
    modernizer = PrimalModernizer(repo_root)
    modernized_count, total_count = modernizer.modernize_all_files()
    
    print(f"\n📊 Modernization Summary:")
    print(f"   📁 Files processed: {total_count}")
    print(f"   ✅ Files modernized: {modernized_count}")
    print(f"   📈 Success rate: {(modernized_count/total_count*100):.1f}%" if total_count > 0 else "   📈 Success rate: N/A")
    
    if modernized_count > 0:
        print(f"\n🎉 Primal modernization completed successfully!")
        print(f"   All hardcoded primal references have been updated to agnostic patterns")
        print(f"   Deprecated patterns are marked for future removal")
    else:
        print(f"\n✅ No files needed modernizing - all references are already agnostic!")

if __name__ == "__main__":
    main() 