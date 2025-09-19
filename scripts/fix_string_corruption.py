#!/usr/bin/env python3
"""
String Corruption Cleanup Script

This script systematically fixes the widespread string corruption issues
where file paths have been embedded into string literals, causing syntax errors.

Usage: python3 scripts/fix_string_corruption.py
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict

# Common corruption patterns we need to fix
CORRUPTION_PATTERNS = [
    # Embedded file paths in strings
    (r'crates/songbird-[^/]*/[^/]*\.rs["\']', ''),
    
    # Malformed string literals
    (r'"([^"]*)"([^"]*)"([^"]*)"', r'"\1\2\3"'),
    
    # Broken quote patterns
    (r'"([^"]*) ([^"]*)"([^"]*)"', r'"\1 \2\3"'),
    
    # Missing closing quotes
    (r'"([^"]*$)', r'"\1"'),
    
    # Double quotes in middle of strings
    (r'"([^"]*)"([^"]*)"', r'"\1\2"'),
    
    # File paths embedded in error messages
    (r'format!\("([^"]*)"([^"]*)"([^"]*)"', r'format!("\1\2\3"'),
    
    # Broken template strings
    (r'r#"([^"]*)"([^"]*)"#', r'r#"\1\2"#'),
    
    # Missing semicolons after strings
    (r'"([^"]*)"([^;}\n])', r'"\1";\2'),
    
    # Malformed function calls with embedded paths
    (r'\.to_owned\(\)([^;,}\n])', r'.to_owned();\1'),
    
    # Broken field assignments
    (r'field: Some\("([^"]*)"([^"]*)"', r'field: Some("\1\2"'),
    
    # Malformed struct initialization
    (r'\{([^}]*)"([^"]*)"([^}]*)\}', r'{\1"\2"\3}'),
]

# Specific file path corruption patterns
FILE_PATH_PATTERNS = [
    (r'crates/songbird-cli/src/cli/commands/init\.rs""', ''),
    (r'crates/songbird-security/src/[^"]*\.rs""', ''),
    (r'crates/songbird-federation/src/[^"]*\.rs""', ''),
    (r'crates/songbird-[^/]*/src/[^"]*\.rs""', ''),
    (r'songbird-[^/]*/src/[^"]*\.rs""', ''),
]

# Patterns for fixing common syntax errors
SYNTAX_FIXES = [
    # Fix unterminated strings
    (r'"([^"]*)\n', r'"\1"\n'),
    
    # Fix broken format strings
    (r'format!\("([^"]*){([^}]*)"([^"]*)"', r'format!("\1{}\3", \2'),
    
    # Fix malformed error constructors
    (r'SongbirdError::([a-zA-Z_]+) \{([^}]*)"([^"]*)"([^}]*)\}', 
     r'SongbirdError::\1 {\2"\3"\4}'),
    
    # Fix broken string concatenation
    (r'"([^"]*)" \+ "([^"]*)"', r'"\1\2"'),
    
    # Fix template literal issues
    (r'r#"([^"#]*)"([^"#]*)"#', r'r#"\1\2"#'),
]

class StringCorruptionFixer:
    def __init__(self, root_dir: str = "crates"):
        self.root_dir = Path(root_dir)
        self.fixed_files = 0
        self.total_fixes = 0
        self.error_files = []

    def find_corrupted_files(self) -> List[Path]:
        """Find all files with string corruption patterns."""
        corrupted_files = []
        
        for rust_file in self.root_dir.rglob("*.rs"):
            if self.has_corruption(rust_file):
                corrupted_files.append(rust_file)
        
        return corrupted_files

    def has_corruption(self, file_path: Path) -> bool:
        """Check if a file contains corruption patterns."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                
            # Check for embedded file paths
            if re.search(r'crates/songbird-[^/]*/[^/]*\.rs', content):
                return True
                
            # Check for malformed strings
            if re.search(r'"[^"]*"[^"]*"[^"]*"', content):
                return True
                
            # Check for unterminated strings
            lines = content.split('\n')
            for line in lines:
                # Skip comments
                if line.strip().startswith('//'):
                    continue
                # Check for unterminated string literals
                if '"' in line and line.count('"') % 2 != 0:
                    return True
                    
            return False
            
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return False

    def fix_file(self, file_path: Path) -> bool:
        """Fix corruption in a single file."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                original_content = f.read()
            
            content = original_content
            fixes_applied = 0
            
            # Apply file path corruption fixes
            for pattern, replacement in FILE_PATH_PATTERNS:
                new_content = re.sub(pattern, replacement, content)
                if new_content != content:
                    fixes_applied += 1
                    content = new_content
            
            # Apply general corruption fixes
            for pattern, replacement in CORRUPTION_PATTERNS:
                new_content = re.sub(pattern, replacement, content)
                if new_content != content:
                    fixes_applied += 1
                    content = new_content
            
            # Apply syntax fixes
            for pattern, replacement in SYNTAX_FIXES:
                new_content = re.sub(pattern, replacement, content)
                if new_content != content:
                    fixes_applied += 1
                    content = new_content
            
            # Manual fixes for specific patterns
            content = self.apply_manual_fixes(content)
            
            # Only write if changes were made
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                
                self.total_fixes += fixes_applied
                print(f"✅ Fixed {file_path} ({fixes_applied} fixes)")
                return True
            else:
                print(f"⚪ No fixes needed for {file_path}")
                return False
                
        except Exception as e:
            print(f"❌ Error fixing {file_path}: {e}")
            self.error_files.append(str(file_path))
            return False

    def apply_manual_fixes(self, content: str) -> str:
        """Apply specific manual fixes for known patterns."""
        
        # Fix broken template strings in init.rs
        content = re.sub(
            r'let service_template = r#"([^"#]*)"([^"#]*)"([^"#]*)"#',
            r'let service_template = r#"\1\2\3"#',
            content
        )
        
        # Fix broken error message formatting
        content = re.sub(
            r'message: format!\("([^"]*)"([^"]*)"([^"]*)"',
            r'message: format!("\1\2\3"',
            content
        )
        
        # Fix broken field assignments
        content = re.sub(
            r'field: Some\("([^"]*)"([^"]*)"',
            r'field: Some("\1\2"',
            content
        )
        
        # Fix broken string literals in println/info/debug macros
        content = re.sub(
            r'(println!|info!|debug!|warn!|error!)\("([^"]*)"([^"]*)"([^"]*)"',
            r'\1("\2\3\4"',
            content
        )
        
        # Fix broken struct field initialization
        content = re.sub(
            r'(\w+): "([^"]*)"([^"]*)"([^,}]*)',
            r'\1: "\2\3"\4',
            content
        )
        
        return content

    def run_cleanup(self) -> Dict[str, int]:
        """Run the complete cleanup process."""
        print("🔍 Finding corrupted files...")
        corrupted_files = self.find_corrupted_files()
        
        print(f"📊 Found {len(corrupted_files)} corrupted files")
        
        if not corrupted_files:
            print("✅ No corrupted files found!")
            return {"total_files": 0, "fixed_files": 0, "total_fixes": 0, "errors": 0}
        
        print("🔧 Starting cleanup process...")
        
        for file_path in corrupted_files:
            if self.fix_file(file_path):
                self.fixed_files += 1
        
        print(f"\n📈 Cleanup Summary:")
        print(f"   Total files processed: {len(corrupted_files)}")
        print(f"   Files fixed: {self.fixed_files}")
        print(f"   Total fixes applied: {self.total_fixes}")
        print(f"   Files with errors: {len(self.error_files)}")
        
        if self.error_files:
            print(f"\n❌ Files with errors:")
            for error_file in self.error_files:
                print(f"   - {error_file}")
        
        return {
            "total_files": len(corrupted_files),
            "fixed_files": self.fixed_files,
            "total_fixes": self.total_fixes,
            "errors": len(self.error_files)
        }

def main():
    """Main function to run the string corruption cleanup."""
    print("🛠️  Songbird String Corruption Cleanup Tool")
    print("=" * 50)
    
    fixer = StringCorruptionFixer()
    results = fixer.run_cleanup()
    
    if results["errors"] == 0:
        print("\n✅ Cleanup completed successfully!")
    else:
        print(f"\n⚠️  Cleanup completed with {results['errors']} errors")
    
    # Test compilation after fixes
    print("\n🧪 Testing compilation after fixes...")
    import subprocess
    try:
        result = subprocess.run(
            ["cargo", "fmt", "--check"], 
            capture_output=True, 
            text=True,
            cwd="."
        )
        if result.returncode == 0:
            print("✅ Formatting check passed!")
        else:
            print("⚠️  Some formatting issues remain")
            print(result.stdout)
    except Exception as e:
        print(f"❌ Could not test compilation: {e}")

if __name__ == "__main__":
    main() 