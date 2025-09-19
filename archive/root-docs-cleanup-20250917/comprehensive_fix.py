#!/usr/bin/env python3
"""
Comprehensive fix for all songbird-types compilation issues.
"""

import os
import re
import glob

def fix_serde_imports():
    """Fix all serde import semicolon issues."""
    files = glob.glob('crates/songbird-types/src/**/*.rs', recursive=True)
    
    for file_path in files:
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix serde imports
        content = re.sub(r'use serde:\{Deserialize, Serialize\}$', 
                        r'use serde::{Deserialize, Serialize};', content, flags=re.MULTILINE)
        content = re.sub(r'use chrono:\{DateTime, Utc\}$', 
                        r'use chrono::{DateTime, Utc};', content, flags=re.MULTILINE)
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"Fixed imports in {file_path}")

def fix_const_declarations():
    """Fix const declaration semicolons."""
    with open('crates/songbird-types/src/constants.rs', 'r') as f:
        content = f.read()
    
    # Fix const declarations that are missing semicolons
    content = re.sub(r'(pub const [^=]+ = [^;]+)\n    /// ', r'\1;\n    /// ', content)
    content = re.sub(r'(pub const [^=]+ = [^;]+)\n\n/// ', r'\1;\n\n/// ', content)
    
    # Fix specific cases
    content = re.sub(r'CanonicalPerformanceDefaults::DEFAULT_METRICS_INTERVAL\n\n/// ', 
                    r'CanonicalPerformanceDefaults::DEFAULT_METRICS_INTERVAL;\n\n/// ', content)
    
    with open('crates/songbird-types/src/constants.rs', 'w') as f:
        f.write(content)
    print("Fixed constants.rs")

def fix_trait_methods():
    """Fix trait method declarations."""
    with open('crates/songbird-types/src/traits.rs', 'r') as f:
        content = f.read()
    
    # Fix trait method declarations - add semicolons
    content = re.sub(r'(async fn [^(]+\([^)]*\)) -> ([^;]+)\n\n    /// ', r'\1 -> \2;\n\n    /// ', content)
    content = re.sub(r'(fn [^(]+\([^)]*\)) -> ([^;]+)\n\n    /// ', r'\1 -> \2;\n\n    /// ', content)
    content = re.sub(r'(fn [^(]+\([^)]*\))\n\n    /// ', r'\1;\n\n    /// ', content)
    content = re.sub(r'(async fn [^(]+\([^)]*\))\n\n    /// ', r'\1;\n\n    /// ', content)
    
    # Fix specific patterns
    content = re.sub(r'async fn ([^(]+\([^)]*\)) -> ([^;]+)\n\}', r'async fn \1 -> \2;\n}', content)
    content = re.sub(r'fn ([^(]+\([^)]*\)) -> ([^;]+)\n\}', r'fn \1 -> \2;\n}', content)
    content = re.sub(r'fn ([^(]+\([^)]*\))\n\}', r'fn \1;\n}', content)
    
    # Remove extra semicolons after trait blocks
    content = re.sub(r'\n;\n\n', r'\n\n', content)
    content = re.sub(r'\n;\n$', r'\n', content)
    
    with open('crates/songbird-types/src/traits.rs', 'w') as f:
        f.write(content)
    print("Fixed traits.rs")

def fix_config_files():
    """Fix config file semicolons and struct issues."""
    config_files = [
        'crates/songbird-types/src/config/orchestration.rs',
        'crates/songbird-types/src/config/performance.rs', 
        'crates/songbird-types/src/config/storage.rs',
        'crates/songbird-types/src/config/system.rs',
        'crates/songbird-types/src/config/unified.rs'
    ]
    
    for file_path in config_files:
        if not os.path.exists(file_path):
            continue
            
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Fix use statement endings
        content = re.sub(r'use serde:\{Deserialize, Serialize\}\n\n/// ', 
                        r'use serde::{Deserialize, Serialize};\n\n/// ', content)
        
        # Fix enum/struct endings
        content = re.sub(r'\}\n\n/// ', r'};\n\n/// ', content)
        
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"Fixed {file_path}")

def fix_migration_file():
    """Fix specific migration.rs issues."""
    with open('crates/songbird-types/src/config/migration.rs', 'r') as f:
        content = f.read()
    
    # Fix specific patterns in migration.rs
    content = re.sub(r'        \}\n\n        ([a-zA-Z_])', r'        };\n\n        \1', content)
    
    with open('crates/songbird-types/src/config/migration.rs', 'w') as f:
        f.write(content)
    print("Fixed migration.rs")

def fix_service_file():
    """Fix service.rs builder pattern issues."""
    with open('crates/songbird-types/src/service.rs', 'r') as f:
        content = f.read()
    
    # Fix builder pattern methods
    content = re.sub(r'self\.([^=]+) = ([^;]+)\n        self', r'self.\1 = \2;\n        self', content)
    
    with open('crates/songbird-types/src/service.rs', 'w') as f:
        f.write(content)
    print("Fixed service.rs")

def fix_memory_optimized():
    """Fix memory_optimized.rs specific issues."""
    with open('crates/songbird-types/src/memory_optimized.rs', 'r') as f:
        content = f.read()
    
    # Fix missing expressions
    content = re.sub(r'let \(opt_cap, trad_cap, cap_improvement\) =;', 
                    r'let (opt_cap, trad_cap, cap_improvement) = Self::memory_comparison();', content)
    
    with open('crates/songbird-types/src/memory_optimized.rs', 'w') as f:
        f.write(content)
    print("Fixed memory_optimized.rs")

def fix_errors_file():
    """Fix errors.rs test issues.""" 
    with open('crates/songbird-types/src/errors.rs', 'r') as f:
        content = f.read()
    
    # Fix test patterns
    content = re.sub(r'        \}\n\n        (assert|let|match)', r'        };\n\n        \1', content)
    content = re.sub(r'let deserialized: SecurityError =;', 
                    r'let deserialized: SecurityError = serde_json::from_str(&serialized).expect("Should deserialize");', content)
    
    with open('crates/songbird-types/src/errors.rs', 'w') as f:
        f.write(content)
    print("Fixed errors.rs")

def fix_response_file():
    """Fix response.rs semicolon issue."""
    with open('crates/songbird-types/src/response.rs', 'r') as f:
        content = f.read()
    
    # Remove stray semicolon
    content = re.sub(r'\n;\n\n', r'\n\n', content)
    
    with open('crates/songbird-types/src/response.rs', 'w') as f:
        f.write(content)
    print("Fixed response.rs")

def fix_types_file():
    """Fix types.rs test issues."""
    with open('crates/songbird-types/src/types.rs', 'r') as f:
        content = f.read()
    
    # Fix test function delimiter issues
    content = re.sub(r'        \)\n        \)\n        \)', r'        )\n        )\n        );', content)
    
    with open('crates/songbird-types/src/types.rs', 'w') as f:
        f.write(content)
    print("Fixed types.rs")

def main():
    """Apply all fixes."""
    print("Starting comprehensive fixes...")
    
    fix_serde_imports()
    fix_const_declarations()
    fix_trait_methods()
    fix_config_files()
    fix_migration_file()
    fix_service_file()
    fix_memory_optimized()
    fix_errors_file()
    fix_response_file()
    fix_types_file()
    
    print("All comprehensive fixes applied!")

if __name__ == "__main__":
    main()
