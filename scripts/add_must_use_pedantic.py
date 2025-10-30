#!/usr/bin/env python3
"""
🔥 PEDANTIC MUST_USE ATTRIBUTE ENFORCER
=====================================

This script adds #[must_use] attributes to ALL functions and types that should 
never be ignored, enforcing PEDANTIC error handling discipline.

TARGETED PATTERNS:
- Functions returning Result<T, E>
- Functions returning Option<T>
- Functions returning Future<Output = Result<T, E>>
- Functions returning impl Future
- Builder pattern methods
- Configuration validation methods
- Resource allocation methods
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Set, Tuple

# Patterns that MUST have #[must_use] attributes
MUST_USE_PATTERNS = [
    # Result returns
    (r'(\s*)(pub\s+(?:async\s+)?fn\s+\w+.*?)\s*->\s*(?:impl\s+.*?)?(?:Result|SongbirdResult)<.*?>\s*\{', 
     r'\1#[must_use = "Result must be handled - ignoring errors is unsafe"]\n\1\2 -> '),
    
    # Option returns  
    (r'(\s*)(pub\s+(?:async\s+)?fn\s+\w+.*?)\s*->\s*Option<.*?>\s*\{',
     r'\1#[must_use = "Option must be handled - ignoring None values can cause bugs"]\n\1\2 -> Option<'),
     
    # Future returns
    (r'(\s*)(pub\s+(?:async\s+)?fn\s+\w+.*?)\s*->\s*(?:impl\s+.*?Future|Future)<.*?>\s*\{',
     r'\1#[must_use = "Future must be awaited - ignoring async operations is incorrect"]\n\1\2 -> '),
     
    # Builder pattern methods
    (r'(\s*)(pub\s+fn\s+(?:with_|set_|add_|enable_|disable_|configure_)\w+.*?)\s*->\s*(?:Self|.*Builder.*?)\s*\{',
     r'\1#[must_use = "Builder methods must be chained - ignoring breaks fluent API"]\n\1\2 -> '),
     
    # Validation methods
    (r'(\s*)(pub\s+fn\s+(?:validate|verify|check|ensure)_\w+.*?)\s*->\s*.*?\s*\{',
     r'\1#[must_use = "Validation results must be checked - ignoring can cause security issues"]\n\1\2 -> '),
     
    # Resource allocation
    (r'(\s*)(pub\s+fn\s+(?:create|new|build|allocate|acquire|connect)_?\w*.*?)\s*->\s*(?:Result|Option|impl|Box|Arc|Rc)<.*?>\s*\{',
     r'\1#[must_use = "Resource allocation must be handled - ignoring can cause leaks"]\n\1\2 -> '),
]

# Types that should have #[must_use] on the type itself
MUST_USE_TYPES = [
    # Result-like types
    (r'(\s*)(pub\s+(?:enum|struct)\s+\w*(?:Result|Error|Response|Status|Outcome)\w*.*?)\s*\{',
     r'\1#[must_use = "This type represents an outcome that must be handled"]\n\1\2 {'),
     
    # Builder types
    (r'(\s*)(pub\s+struct\s+\w*Builder\w*.*?)\s*\{',
     r'\1#[must_use = "Builders must be used to construct the final object"]\n\1\2 {'),
     
    # Guard types  
    (r'(\s*)(pub\s+struct\s+\w*(?:Guard|Lock|Handle|Token)\w*.*?)\s*\{',
     r'\1#[must_use = "Guards and handles must be kept alive for their effect"]\n\1\2 {'),
]

class PedanticMustUseEnforcer:
    """Enforces #[must_use] attributes with PEDANTIC precision"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.files_processed = 0
        self.attributes_added = 0
        self.skipped_files: Set[str] = set()
        
    def should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped"""
        skip_patterns = [
            'target/',
            'benches/',
            'examples/',  # Skip examples for now
            'tests/',     # Skip tests for now  
            '.git/',
            'archive/',
        ]
        
        path_str = str(file_path)
        for pattern in skip_patterns:
            if pattern in path_str:
                return True
                
        return False
        
    def has_must_use_attribute(self, lines: List[str], line_idx: int) -> bool:
        """Check if the function/type already has #[must_use]"""
        # Look backwards for #[must_use] attribute
        for i in range(max(0, line_idx - 5), line_idx):
            if '#[must_use' in lines[i]:
                return True
        return False
        
    def process_file(self, file_path: Path) -> Tuple[int, str]:
        """Process a single Rust file for #[must_use] attributes"""
        
        if self.should_skip_file(file_path):
            self.skipped_files.add(str(file_path))
            return 0, "skipped"
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            lines = content.split('\n')
            changes_made = 0
            
            # Apply function patterns
            for pattern, replacement in MUST_USE_PATTERNS:
                matches = list(re.finditer(pattern, content, re.MULTILINE))
                for match in reversed(matches):  # Process in reverse to maintain positions
                    line_num = content[:match.start()].count('\n')
                    if not self.has_must_use_attribute(lines, line_num):
                        content = content[:match.start()] + re.sub(pattern, replacement, match.group(0)) + content[match.end():]
                        changes_made += 1
                        
            # Apply type patterns  
            for pattern, replacement in MUST_USE_TYPES:
                matches = list(re.finditer(pattern, content, re.MULTILINE))
                for match in reversed(matches):
                    line_num = content[:match.start()].count('\n')
                    if not self.has_must_use_attribute(lines, line_num):
                        content = content[:match.start()] + re.sub(pattern, replacement, match.group(0)) + content[match.end():]
                        changes_made += 1
                        
            # Write back if changes were made
            if changes_made > 0 and content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                    
            return changes_made, "processed"
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            return 0, f"error: {e}"
            
    def run(self) -> None:
        """Run the pedantic must_use enforcer"""
        print("🔥 PEDANTIC MUST_USE ATTRIBUTE ENFORCER ACTIVATED")
        print("=" * 60)
        
        rust_files = list(self.project_root.rglob("*.rs"))
        print(f"📁 Found {len(rust_files)} Rust files to process")
        
        for file_path in rust_files:
            changes, status = self.process_file(file_path)
            
            if status == "processed" and changes > 0:
                print(f"✅ {file_path.relative_to(self.project_root)}: +{changes} #[must_use] attributes")
                self.attributes_added += changes
                
            if status.startswith("processed"):
                self.files_processed += 1
                
        print("\n" + "=" * 60)
        print("🏆 PEDANTIC MUST_USE ENFORCEMENT COMPLETE!")
        print(f"📁 Files processed: {self.files_processed}")
        print(f"⚡ #[must_use] attributes added: {self.attributes_added}")
        print(f"⏭️  Files skipped: {len(self.skipped_files)}")
        
        if self.attributes_added > 0:
            print(f"\n✅ SUCCESS: Added {self.attributes_added} #[must_use] attributes")
            print("🎯 Your code is now MORE PEDANTIC and SAFER!")
        else:
            print("\n💎 ALREADY PERFECT: No #[must_use] attributes needed")
            print("🏆 Your code already follows pedantic best practices!")

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 add_must_use_pedantic.py <project_root>")
        sys.exit(1)
        
    project_root = sys.argv[1]
    if not os.path.exists(project_root):
        print(f"❌ Project root does not exist: {project_root}")
        sys.exit(1)
        
    enforcer = PedanticMustUseEnforcer(project_root)
    enforcer.run()

if __name__ == "__main__":
    main() 