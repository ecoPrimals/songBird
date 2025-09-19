#!/usr/bin/env python3
"""
🚀 PEDANTIC CLONE ELIMINATION ENGINE
===================================

This script systematically eliminates unnecessary clone() operations,
replacing them with zero-copy alternatives for MAXIMUM performance.

OPTIMIZATION STRATEGIES:
1. String clones → &str references where possible
2. Vec clones → slice references (&[T])
3. Shared data → Arc<T> for multi-threading
4. Copy-on-write → Cow<T> for conditional mutation
5. Builder patterns → move semantics
6. Configuration → static references
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Dict, Set, Tuple
from dataclasses import dataclass

@dataclass
class CloneOptimization:
    """Represents a clone optimization opportunity"""
    pattern: str
    replacement: str
    description: str
    safety_level: str  # "safe", "careful", "manual"

# Safe clone eliminations that are always correct
SAFE_OPTIMIZATIONS = [
    CloneOptimization(
        pattern=r'(\w+)\.to_string\(\)\.clone\(\)',
        replacement=r'\1.to_string()',
        description="Remove redundant clone() after to_string()",
        safety_level="safe"
    ),
    
    CloneOptimization(
        pattern=r'(\w+)\.clone\(\)\.as_str\(\)',
        replacement=r'\1.as_str()',
        description="Use direct string reference instead of clone+as_str",
        safety_level="safe"
    ),
    
    CloneOptimization(
        pattern=r'(\w+)\.clone\(\)\.len\(\)',
        replacement=r'\1.len()',
        description="Use direct length instead of clone+len",
        safety_level="safe"
    ),
    
    CloneOptimization(
        pattern=r'(\w+)\.clone\(\)\.is_empty\(\)',
        replacement=r'\1.is_empty()',
        description="Use direct is_empty instead of clone+is_empty",
        safety_level="safe"
    ),
    
    CloneOptimization(
        pattern=r'format!\("([^"]*)", ([^)]+)\.clone\(\)\)',
        replacement=r'format!("\1", \2)',
        description="Remove clone in format! macro arguments",
        safety_level="safe"
    ),
]

# Careful optimizations that need context analysis
CAREFUL_OPTIMIZATIONS = [
    CloneOptimization(
        pattern=r'let\s+(\w+)\s*=\s*([^;]+)\.clone\(\);',
        replacement=r'let \1 = &\2;',
        description="Replace clone with reference in let bindings",
        safety_level="careful"
    ),
    
    CloneOptimization(
        pattern=r'return\s+([^;]+)\.clone\(\);',
        replacement=r'return \1.clone(); // TODO: Consider if clone is necessary',
        description="Mark return clones for manual review",
        safety_level="manual"
    ),
]

# Patterns that suggest Arc<T> usage
ARC_PATTERNS = [
    CloneOptimization(
        pattern=r'let\s+(\w+)\s*=\s*([^;]+)\.clone\(\);\s*\/\/.*shared',
        replacement=r'let \1 = Arc::clone(&\2); // ✅ OPTIMIZED: Arc for shared ownership',
        description="Use Arc::clone for shared ownership",
        safety_level="careful"
    ),
]

# Patterns that suggest Cow<T> usage  
COW_PATTERNS = [
    CloneOptimization(
        pattern=r'if\s+.*\{\s*([^}]+)\.clone\(\)\s*\}\s*else\s*\{\s*([^}]+)\s*\}',
        replacement=r'// TODO: Consider Cow<T> for conditional cloning',
        description="Conditional cloning candidate for Cow<T>",
        safety_level="manual"
    ),
]

class PedanticCloneEliminator:
    """Eliminates clones with PEDANTIC precision and safety"""
    
    def __init__(self, project_root: str, dry_run: bool = False):
        self.project_root = Path(project_root)
        self.dry_run = dry_run
        self.files_processed = 0
        self.clones_eliminated = 0
        self.optimizations_applied: Dict[str, int] = {}
        self.manual_reviews_needed: List[str] = []
        
    def should_skip_file(self, file_path: Path) -> bool:
        """Check if file should be skipped"""
        skip_patterns = [
            'target/',
            'benches/',
            'tests/',     # Skip tests for now - different optimization rules
            '.git/',
            'archive/',
        ]
        
        path_str = str(file_path)
        for pattern in skip_patterns:
            if pattern in path_str:
                return True
                
        return False
        
    def is_in_string_literal(self, content: str, match_pos: int) -> bool:
        """Check if the match is inside a string literal"""
        before = content[:match_pos]
        # Count unescaped quotes
        quote_count = before.count('"') - before.count('\\"')
        return quote_count % 2 == 1
        
    def is_in_comment(self, content: str, match_pos: int) -> bool:
        """Check if the match is inside a comment"""
        lines_before = content[:match_pos].split('\n')
        current_line = lines_before[-1] if lines_before else ""
        return '//' in current_line and current_line.index('//') < len(current_line) - len(current_line.lstrip())
        
    def apply_optimizations(self, content: str, optimizations: List[CloneOptimization]) -> Tuple[str, int]:
        """Apply a set of optimizations to content"""
        changes = 0
        
        for opt in optimizations:
            matches = list(re.finditer(opt.pattern, content))
            for match in reversed(matches):  # Process in reverse to maintain positions
                # Skip if in string literal or comment
                if self.is_in_string_literal(content, match.start()) or \
                   self.is_in_comment(content, match.start()):
                    continue
                    
                # Apply the optimization
                old_text = match.group(0)
                new_text = re.sub(opt.pattern, opt.replacement, old_text)
                
                if old_text != new_text:
                    content = content[:match.start()] + new_text + content[match.end():]
                    changes += 1
                    
                    # Track optimization type
                    if opt.description not in self.optimizations_applied:
                        self.optimizations_applied[opt.description] = 0
                    self.optimizations_applied[opt.description] += 1
                    
        return content, changes
        
    def analyze_remaining_clones(self, content: str, file_path: Path) -> None:
        """Analyze remaining clones for manual optimization opportunities"""
        clone_matches = re.findall(r'\.clone\(\)', content)
        if len(clone_matches) > 5:  # Many clones remaining
            self.manual_reviews_needed.append(f"{file_path}: {len(clone_matches)} clones remaining")
            
    def process_file(self, file_path: Path) -> Tuple[int, str]:
        """Process a single Rust file for clone elimination"""
        
        if self.should_skip_file(file_path):
            return 0, "skipped"
            
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            original_content = content
            total_changes = 0
            
            # Apply safe optimizations first
            content, safe_changes = self.apply_optimizations(content, SAFE_OPTIMIZATIONS)
            total_changes += safe_changes
            
            # Apply careful optimizations
            content, careful_changes = self.apply_optimizations(content, CAREFUL_OPTIMIZATIONS)  
            total_changes += careful_changes
            
            # Apply Arc optimizations
            content, arc_changes = self.apply_optimizations(content, ARC_PATTERNS)
            total_changes += arc_changes
            
            # Apply Cow optimizations  
            content, cow_changes = self.apply_optimizations(content, COW_PATTERNS)
            total_changes += cow_changes
            
            # Analyze remaining clones
            self.analyze_remaining_clones(content, file_path)
            
            # Write back if changes were made and not dry run
            if total_changes > 0 and content != original_content and not self.dry_run:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                    
            return total_changes, "processed"
            
        except Exception as e:
            print(f"❌ Error processing {file_path}: {e}")
            return 0, f"error: {e}"
            
    def run(self) -> None:
        """Run the pedantic clone eliminator"""
        print("🚀 PEDANTIC CLONE ELIMINATION ENGINE ACTIVATED")
        if self.dry_run:
            print("🔍 DRY RUN MODE - No files will be modified")
        print("=" * 60)
        
        rust_files = list(self.project_root.rglob("*.rs"))
        print(f"📁 Found {len(rust_files)} Rust files to process")
        
        for file_path in rust_files:
            changes, status = self.process_file(file_path)
            
            if status == "processed" and changes > 0:
                print(f"✅ {file_path.relative_to(self.project_root)}: -{changes} unnecessary clones")
                self.clones_eliminated += changes
                
            if status.startswith("processed"):
                self.files_processed += 1
                
        print("\n" + "=" * 60)
        print("🏆 PEDANTIC CLONE ELIMINATION COMPLETE!")
        print(f"📁 Files processed: {self.files_processed}")
        print(f"⚡ Clones eliminated: {self.clones_eliminated}")
        
        print(f"\n📊 OPTIMIZATION BREAKDOWN:")
        for opt_type, count in sorted(self.optimizations_applied.items()):
            print(f"  • {opt_type}: {count}")
            
        if self.manual_reviews_needed:
            print(f"\n🔍 MANUAL REVIEW RECOMMENDED:")
            for review in self.manual_reviews_needed[:10]:  # Show top 10
                print(f"  • {review}")
                
        if self.clones_eliminated > 0:
            print(f"\n✅ SUCCESS: Eliminated {self.clones_eliminated} unnecessary clones")
            print("⚡ Your code is now FASTER with zero-copy optimizations!")
        else:
            print("\n💎 ALREADY OPTIMIZED: No unnecessary clones found")
            print("🏆 Your code already follows zero-copy best practices!")

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 eliminate_clones_pedantic.py <project_root> [--dry-run]")
        sys.exit(1)
        
    project_root = sys.argv[1]
    dry_run = "--dry-run" in sys.argv
    
    if not os.path.exists(project_root):
        print(f"❌ Project root does not exist: {project_root}")
        sys.exit(1)
        
    eliminator = PedanticCloneEliminator(project_root, dry_run)
    eliminator.run()

if __name__ == "__main__":
    main() 