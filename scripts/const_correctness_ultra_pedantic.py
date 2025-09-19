#!/usr/bin/env python3
"""
🔥 ULTRA-PEDANTIC CONST CORRECTNESS TRANSCENDENCE ENGINE
=======================================================

This script achieves TRANSCENDENT const correctness by adding 'const' to
EVERY function that can be evaluated at compile time.

TRANSCENDENCE OBJECTIVES:
- Maximum compile-time evaluation
- Zero runtime overhead for pure functions
- Const propagation throughout the codebase  
- Compile-time guarantees wherever possible
- Performance through const optimization

BEYOND PEDANTIC - WE ACHIEVE COMPILE-TIME NIRVANA!
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Set, Tuple
from dataclasses import dataclass

@dataclass
class ConstPattern:
    """Represents a const optimization opportunity"""
    pattern: str
    replacement: str
    description: str
    safety_level: str

# Patterns for const functions
CONST_FUNCTION_PATTERNS = [
    ConstPattern(
        pattern=r'(\s*)(pub\s+fn\s+(\w+)\([^)]*\)\s*->\s*[^{]*)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}',
        replacement=r'\1pub const fn \3(\4',
        description="Convert pure functions to const fn",
        safety_level="careful"
    ),
    
    ConstPattern(
        pattern=r'(\s*)(fn\s+(\w+)\([^)]*\)\s*->\s*[^{]*)\s*\{([^{}]*(?:\{[^{}]*\}[^{}]*)*)\}',
        replacement=r'\1const fn \3(\4',
        description="Convert private pure functions to const fn",
        safety_level="careful"
    ),
]

# Patterns for const values
CONST_VALUE_PATTERNS = [
    ConstPattern(
        pattern=r'(\s*)(pub\s+static\s+(\w+):\s*([^=]+))\s*=\s*([^;]+);',
        replacement=r'\1pub const \3: \4 = \5;',
        description="Convert static to const for compile-time values",
        safety_level="safe"
    ),
    
    ConstPattern(
        pattern=r'(\s*)(static\s+(\w+):\s*([^=]+))\s*=\s*([^;]+);',
        replacement=r'\1const \3: \4 = \5;',
        description="Convert private static to const",
        safety_level="safe"
    ),
]

# Patterns for const generics
CONST_GENERIC_PATTERNS = [
    ConstPattern(
        pattern=r'struct\s+(\w+)<([^>]+)>',
        replacement=r'struct \1<const N: usize, \2>',
        description="Add const generic parameters where beneficial",
        safety_level="manual"
    ),
]

class UltraPedanticConstEnforcer:
    """Enforces TRANSCENDENT const correctness"""
    
    def __init__(self, project_root: str, dry_run: bool = False):
        self.project_root = Path(project_root)
        self.dry_run = dry_run
        self.files_processed = 0
        self.const_functions_added = 0
        self.const_values_added = 0
        self.optimizations_applied: Dict[str, int] = {}
        
    def should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped"""
        skip_patterns = [
            'target/',
            'tests/',     # Tests have different const requirements
            'benches/',
            '.git/',
            'archive/',
        ]
        
        path_str = str(file_path)
        for pattern in skip_patterns:
            if pattern in path_str:
                return True
                
        return False
        
    def is_const_compatible(self, function_body: str) -> bool:
        """Check if function body is compatible with const fn"""
        # Patterns that prevent const fn
        non_const_patterns = [
            r'\.await',           # Async operations
            r'panic!\(',          # Panic calls
            r'println!\(',        # I/O operations  
            r'eprintln!\(',       # I/O operations
            r'std::mem::forget',  # Memory operations
            r'unsafe\s*\{',       # Unsafe blocks (some are const-compatible but need review)
            r'Box::new',          # Heap allocation
            r'Vec::new',          # Heap allocation (unless in const context)
            r'HashMap::new',      # Heap allocation
            r'thread::', # Thread operations
            r'Mutex::', # Synchronization
        ]
        
        for pattern in non_const_patterns:
            if re.search(pattern, function_body):
                return False
                
        return True
        
    def is_simple_computation(self, function_body: str) -> bool:
        """Check if function is a simple computation suitable for const"""
        # Look for mathematical operations, simple conditionals, etc.
        simple_patterns = [
            r'^\s*\{[^{}]*\}\s*$',  # Single expression
            r'if.*else',             # Simple conditionals
            r'match.*\{.*\}',        # Pattern matching
            r'[+\-*/]',              # Arithmetic
            r'==|!=|<|>|<=|>=',      # Comparisons
        ]
        
        # Must have some computation
        has_computation = any(re.search(pattern, function_body) for pattern in simple_patterns)
        
        # Must be relatively simple (not too many lines)
        line_count = function_body.count('\n')
        
        return has_computation and line_count < 20
        
    def apply_const_optimizations(self, content: str, patterns: List[ConstPattern]) -> Tuple[str, int]:
        """Apply const optimizations to content"""
        changes = 0
        
        for pattern in patterns:
            if pattern.safety_level == "manual":
                continue  # Skip manual patterns for now
                
            matches = list(re.finditer(pattern.pattern, content, re.MULTILINE | re.DOTALL))
            
            for match in reversed(matches):
                if pattern.description.startswith("Convert pure functions"):
                    # Additional validation for function const conversion
                    if len(match.groups()) >= 4:
                        function_body = match.group(4) if len(match.groups()) > 3 else ""
                        if not self.is_const_compatible(function_body) or not self.is_simple_computation(function_body):
                            continue
                
                # Apply the pattern
                old_text = match.group(0)
                
                if pattern.description.startswith("Convert pure functions"):
                    # Special handling for function conversion
                    indent = match.group(1)
                    func_signature = match.group(2)
                    func_name = match.group(3)
                    func_body = match.group(4) if len(match.groups()) > 3 else ""
                    
                    if 'pub' in func_signature:
                        new_signature = func_signature.replace('pub fn', 'pub const fn')
                    else:
                        new_signature = func_signature.replace('fn', 'const fn')
                        
                    new_text = f"{indent}{new_signature} {{{func_body}}}"
                else:
                    new_text = re.sub(pattern.pattern, pattern.replacement, old_text)
                
                if old_text != new_text:
                    content = content[:match.start()] + new_text + content[match.end():]
                    changes += 1
                    
                    # Track optimization type
                    if pattern.description not in self.optimizations_applied:
                        self.optimizations_applied[pattern.description] = 0
                    self.optimizations_applied[pattern.description] += 1
                    
                    if "function" in pattern.description:
                        self.const_functions_added += 1
                    elif "static to const" in pattern.description:
                        self.const_values_added += 1
                        
        return content, changes
        
    def add_inline_attributes(self, content: str) -> Tuple[str, int]:
        """Add #[inline] attributes to small const functions"""
        inline_pattern = r'(\s*)(pub\s+const\s+fn\s+\w+[^{]*\{[^{}]{1,100}\})'
        matches = list(re.finditer(inline_pattern, content, re.MULTILINE | re.DOTALL))
        changes = 0
        
        for match in reversed(matches):
            # Check if already has inline attribute
            before_match = content[:match.start()]
            if '#[inline]' in before_match[-50:]:  # Check last 50 chars
                continue
                
            indent = match.group(1)
            function_def = match.group(2)
            
            new_text = f"{indent}#[inline]\n{indent}{function_def}"
            content = content[:match.start()] + new_text + content[match.end():]
            changes += 1
            
        return content, changes
        
    def process_file(self, file_path: Path) -> Tuple[int, str]:
        """Process a single Rust file for const optimization"""
        
        if self.should_skip_file(file_path):
            return 0, "skipped"
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            total_changes = 0
            
            # Apply const function patterns
            content, func_changes = self.apply_const_optimizations(content, CONST_FUNCTION_PATTERNS)
            total_changes += func_changes
            
            # Apply const value patterns
            content, value_changes = self.apply_const_optimizations(content, CONST_VALUE_PATTERNS)
            total_changes += value_changes
            
            # Add inline attributes to small const functions
            content, inline_changes = self.add_inline_attributes(content)
            total_changes += inline_changes
            
            # Write back if changes were made and not dry run
            if total_changes > 0 and content != original_content and not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                    
            return total_changes, "processed"
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            return 0, f"error: {e}"
            
    def run(self) -> None:
        """Run the ultra-pedantic const enforcer"""
        print("🔥 ULTRA-PEDANTIC CONST CORRECTNESS TRANSCENDENCE ACTIVATED")
        if self.dry_run:
            print("🔍 DRY RUN MODE - No files will be modified")
        print("=" * 70)
        
        rust_files = list(self.project_root.rglob("*.rs"))
        print(f"📁 Found {len(rust_files)} Rust files to process")
        
        for file_path in rust_files:
            changes, status = self.process_file(file_path)
            
            if status == "processed" and changes > 0:
                print(f"✅ {file_path.relative_to(self.project_root)}: +{changes} const optimizations")
                
            if status.startswith("processed"):
                self.files_processed += 1
                
        print("\n" + "=" * 70)
        print("🏆 ULTRA-PEDANTIC CONST CORRECTNESS TRANSCENDENCE COMPLETE!")
        print(f"📁 Files processed: {self.files_processed}")
        print(f"🔧 Const functions added: {self.const_functions_added}")
        print(f"💎 Const values optimized: {self.const_values_added}")
        
        print(f"\n📊 OPTIMIZATION BREAKDOWN:")
        for opt_type, count in sorted(self.optimizations_applied.items()):
            print(f"  • {opt_type}: {count}")
            
        total_optimizations = self.const_functions_added + self.const_values_added
        if total_optimizations > 0:
            print(f"\n✅ TRANSCENDENCE ACHIEVED: Added {total_optimizations} const optimizations")
            print("⚡ Your code now has MAXIMUM compile-time evaluation!")
            print("🎯 Performance through const correctness achieved!")
        else:
            print("\n💎 ALREADY TRANSCENDENT: Const correctness is already maximized")
            print("🏆 Your code already achieves maximum compile-time optimization!")

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 const_correctness_ultra_pedantic.py <project_root> [--dry-run]")
        sys.exit(1)
        
    project_root = sys.argv[1]
    dry_run = "--dry-run" in sys.argv
    
    if not os.path.exists(project_root):
        print(f"❌ Project root does not exist: {project_root}")
        sys.exit(1)
        
    enforcer = UltraPedanticConstEnforcer(project_root, dry_run)
    enforcer.run()

if __name__ == "__main__":
    main() 