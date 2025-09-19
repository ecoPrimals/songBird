#!/usr/bin/env python3
"""
🎯 Vendor Hardcoding Elimination Script

This script systematically eliminates hardcoded primal names and replaces them
with agnostic, capability-based patterns that support the "each primal only 
knows itself" architecture.

CORE PRINCIPLE: No primal names hardcoded anywhere in production code.
"""

import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Set
import json

class VendorHardcodingEliminator:
    """Eliminates vendor hardcoding and implements agnostic patterns"""
    
    def __init__(self, repo_root: Path):
        self.repo_root = repo_root
        self.hardcoded_primals = {
            'beardog', 'toadstool', 'nestgate', 'squirrel',
            'BearDog', 'ToadStool', 'NestGate', 'Squirrel',
            'BEARDOG', 'TOADSTOOL', 'NESTGATE', 'SQUIRREL'
        }
        
        # Patterns to replace with agnostic equivalents
        self.replacement_patterns = {
            # Configuration patterns
            r'beardog_config': 'security_provider_config',
            r'toadstool_config': 'compute_provider_config', 
            r'nestgate_config': 'storage_provider_config',
            r'squirrel_config': 'ai_provider_config',
            
            # Endpoint patterns
            r'beardog_endpoint': 'security_endpoint',
            r'toadstool_endpoint': 'compute_endpoint',
            r'nestgate_endpoint': 'storage_endpoint', 
            r'squirrel_endpoint': 'ai_endpoint',
            
            # Service patterns
            r'beardog_service': 'security_service',
            r'toadstool_service': 'compute_service',
            r'nestgate_service': 'storage_service',
            r'squirrel_service': 'ai_service',
            
            # Environment variable patterns
            r'BEARDOG_ENDPOINT': 'SECURITY_PROVIDER_ENDPOINT',
            r'TOADSTOOL_ENDPOINT': 'COMPUTE_PROVIDER_ENDPOINT',
            r'NESTGATE_ENDPOINT': 'STORAGE_PROVIDER_ENDPOINT',
            r'SQUIRREL_ENDPOINT': 'AI_PROVIDER_ENDPOINT',
        }
        
        # Files to exclude from migration (tests, examples, docs)
        self.excluded_patterns = {
            'test', 'example', 'demo', 'benchmark', 'doc', 'README', 'CHANGELOG'
        }
        
        self.migration_stats = {
            'files_processed': 0,
            'hardcoded_instances_found': 0,
            'hardcoded_instances_replaced': 0,
            'files_with_changes': 0,
        }

    def should_migrate_file(self, file_path: Path) -> bool:
        """Determine if file should be migrated (exclude tests, examples, docs)"""
        path_str = str(file_path).lower()
        
        # Skip certain directories
        skip_dirs = {'test', 'tests', 'example', 'examples', 'demo', 'demos', 
                    'bench', 'benches', 'doc', 'docs', 'target', '.git'}
        
        for part in file_path.parts:
            if part.lower() in skip_dirs:
                return False
                
        # Skip certain file patterns
        for pattern in self.excluded_patterns:
            if pattern in path_str:
                return False
                
        return file_path.suffix in {'.rs', '.toml', '.yaml', '.yml', '.json'}

    def find_hardcoded_instances(self, content: str) -> List[Tuple[str, int, str]]:
        """Find all hardcoded primal name instances in content"""
        instances = []
        lines = content.split('\n')
        
        for line_num, line in enumerate(lines, 1):
            for primal in self.hardcoded_primals:
                # Look for primal names in various contexts
                patterns = [
                    rf'\b{primal}\b',  # Word boundary
                    rf'"{primal}"',    # In quotes
                    rf"'{primal}'",    # In single quotes
                    rf'{primal}_',     # As prefix
                    rf'_{primal}',     # As suffix
                ]
                
                for pattern in patterns:
                    matches = re.finditer(pattern, line, re.IGNORECASE)
                    for match in matches:
                        instances.append((match.group(), line_num, line.strip()))
                        
        return instances

    def create_agnostic_replacement(self, hardcoded_text: str, context: str) -> str:
        """Create agnostic replacement for hardcoded primal name"""
        lower_text = hardcoded_text.lower()
        
        # Map primal names to capability types
        capability_map = {
            'beardog': 'security',
            'toadstool': 'compute', 
            'nestgate': 'storage',
            'squirrel': 'ai'
        }
        
        # Determine capability type
        capability = None
        for primal, cap in capability_map.items():
            if primal in lower_text:
                capability = cap
                break
                
        if not capability:
            capability = 'unknown'
            
        # Context-aware replacements
        if 'config' in context.lower():
            return f'{capability}_provider_config'
        elif 'endpoint' in context.lower():
            return f'{capability}_provider_endpoint'  
        elif 'service' in context.lower():
            return f'{capability}_provider_service'
        elif context.isupper():  # Environment variable
            return f'{capability.upper()}_PROVIDER'
        else:
            return f'{capability}_provider'

    def migrate_file_content(self, content: str, file_path: Path) -> Tuple[str, int]:
        """Migrate file content to remove hardcoded primal names"""
        original_content = content
        changes_made = 0
        
        # Apply pattern-based replacements
        for pattern, replacement in self.replacement_patterns.items():
            old_content = content
            content = re.sub(pattern, replacement, content, flags=re.IGNORECASE)
            if content != old_content:
                changes_made += content.count(replacement) - old_content.count(replacement)
                
        # Handle remaining hardcoded instances contextually
        instances = self.find_hardcoded_instances(content)
        for instance, line_num, line_context in instances:
            # Skip if this looks like a test or example
            if any(skip in line_context.lower() for skip in ['test', 'example', 'demo']):
                continue
                
            # Create contextual replacement
            replacement = self.create_agnostic_replacement(instance, line_context)
            content = content.replace(instance, replacement, 1)
            changes_made += 1
            
        return content, changes_made

    def generate_migration_report(self) -> str:
        """Generate comprehensive migration report"""
        report = f"""
# 🎯 Vendor Hardcoding Elimination Report

## 📊 Migration Statistics
- **Files Processed**: {self.migration_stats['files_processed']}
- **Files Modified**: {self.migration_stats['files_with_changes']}
- **Hardcoded Instances Found**: {self.migration_stats['hardcoded_instances_found']}
- **Hardcoded Instances Replaced**: {self.migration_stats['hardcoded_instances_replaced']}

## ✅ Agnostic Pattern Implementation

### Before (Hardcoded)
```rust
// ❌ Hardcoded primal names
beardog_config.endpoint = "https://beardog.internal:8443";
toadstool_service.connect();
nestgate_storage.upload(data);
squirrel_ai.process(request);
```

### After (Agnostic)
```rust
// ✅ Capability-based agnostic patterns
security_provider_config.endpoint = env::var("SECURITY_PROVIDER_ENDPOINT")
    .unwrap_or_else(|| discover_capability_endpoint("security"));
compute_provider_service.connect();
storage_provider.upload(data);
ai_provider.process(request);
```

## 🌟 Self-Discovery Architecture Benefits

1. **Zero Hardcoded Dependencies**: No primal names in production code
2. **Infinite Extensibility**: Any primal can provide any capability
3. **Network Effects**: Primals discover each other dynamically
4. **Service Mesh Ready**: Compatible with any service mesh architecture
5. **Environment Agnostic**: Works in K8s, Docker, bare-metal

## 🚀 Next Steps

1. Update environment variable patterns to use capability-based naming
2. Implement universal adapter integration in all service discovery
3. Add capability announcement mechanisms for dynamic discovery
4. Test network effects with multiple primal providers per capability
"""
        return report

    def migrate_repository(self) -> None:
        """Migrate entire repository to eliminate vendor hardcoding"""
        print("🎯 Starting vendor hardcoding elimination...")
        
        rust_files = list(self.repo_root.rglob("*.rs"))
        config_files = list(self.repo_root.rglob("*.toml")) + list(self.repo_root.rglob("*.yaml"))
        
        all_files = rust_files + config_files
        
        for file_path in all_files:
            if not self.should_migrate_file(file_path):
                continue
                
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    original_content = f.read()
                    
                # Find hardcoded instances
                instances = self.find_hardcoded_instances(original_content)
                self.migration_stats['hardcoded_instances_found'] += len(instances)
                
                if instances:
                    print(f"📁 Processing {file_path} ({len(instances)} hardcoded instances)")
                    
                    # Migrate content
                    migrated_content, changes = self.migrate_file_content(original_content, file_path)
                    
                    if changes > 0:
                        # Write migrated content
                        with open(file_path, 'w', encoding='utf-8') as f:
                            f.write(migrated_content)
                            
                        self.migration_stats['files_with_changes'] += 1
                        self.migration_stats['hardcoded_instances_replaced'] += changes
                        
                        print(f"  ✅ Replaced {changes} hardcoded instances")
                        
                self.migration_stats['files_processed'] += 1
                        
            except Exception as e:
                print(f"  ❌ Error processing {file_path}: {e}")
                
        # Generate migration report
        report = self.generate_migration_report()
        report_path = self.repo_root / "VENDOR_HARDCODING_ELIMINATION_REPORT.md"
        
        with open(report_path, 'w') as f:
            f.write(report)
            
        print(f"\n🎉 Migration completed!")
        print(f"📊 Report saved to: {report_path}")
        print(f"📈 Statistics: {self.migration_stats}")

def main():
    """Main migration entry point"""
    if len(sys.argv) > 1:
        repo_root = Path(sys.argv[1])
    else:
        repo_root = Path.cwd()
        
    if not repo_root.exists():
        print(f"❌ Repository path not found: {repo_root}")
        sys.exit(1)
        
    print(f"🎯 Eliminating vendor hardcoding in: {repo_root}")
    
    eliminator = VendorHardcodingEliminator(repo_root)
    eliminator.migrate_repository()

if __name__ == "__main__":
    main() 