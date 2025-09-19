#!/usr/bin/env python3
"""
Targeted fixes for songbird-types crate compilation issues.
"""

import os
import re

def fix_file(file_path, fixes):
    """Apply targeted fixes to a specific file."""
    if not os.path.exists(file_path):
        print(f"File not found: {file_path}")
        return
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        print(f"Fixed {file_path}")

def main():
    """Apply all fixes."""
    
    # Fix constants.rs
    fix_file('crates/songbird-types/src/constants.rs', [
        (r'pub const ([^=]+= [^;]+)\n    /// ', r'pub const \1;\n    /// '),
        (r'let is_container =;', r'let is_container = std::env::var("CONTAINER").is_ok();'),
        (r'CanonicalPerformanceDefaults::DEFAULT_EVALUATION_TIMEOUT\n/// ', r'CanonicalPerformanceDefaults::DEFAULT_EVALUATION_TIMEOUT;\n\n/// '),
    ])
    
    # Fix errors.rs  
    fix_file('crates/songbird-types/src/errors.rs', [
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
        (r'pub type Result<T> = std::result::Result<T, SongbirdError>\n/// ', r'pub type Result<T> = std::result::Result<T, SongbirdError>;\n\n/// '),
    ])
    
    # Fix health.rs
    fix_file('crates/songbird-types/src/health.rs', [
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
    ])
    
    # Fix memory_optimized.rs
    fix_file('crates/songbird-types/src/memory_optimized.rs', [
        (r'use crate:\{[^}]+\}\n', r'use crate::{CanonicalHealthStatus, CanonicalPrimalType, SongbirdError};\n'),
        (r'        \}\n        Self::', r'        }\n        Self::'),
        (r'self\.([^|]+) \|= ([^;]+)\n        self', r'self.\1 |= \2;\n        self'),
        (r'self\.([^.]+)\.push\(([^)]+)\)\n        self', r'self.\1.push(\2);\n        self'),
        (r'let mut capabilities = Vec::new\(\)\n\n        if', r'let mut capabilities = Vec::new();\n\n        if'),
        (r'type MemoryComparisonResult = \([^)]+\)\n\nimpl', r'type MemoryComparisonResult = (usize, usize, f64);\n\nimpl'),
        (r'let optimized_size = [^;]+\n        let traditional_size =;', r'let optimized_size = std::mem::size_of::<OptimizedCapabilities>();\n        let traditional_size = std::mem::size_of::<Vec<String>>();'),
        (r'let \(optimized, traditional, improvement\) =;', r'let (optimized, traditional, improvement) = Self::compare_memory_usage();'),
        (r'type Target = T\n\n    fn deref', r'type Target = T;\n\n    fn deref'),
    ])
    
    # Fix primal.rs
    fix_file('crates/songbird-types/src/primal.rs', [
        (r'use chrono:\{DateTime, Utc\}\n', r'use chrono::{DateTime, Utc};\n'),
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
    ])
    
    # Fix response.rs
    fix_file('crates/songbird-types/src/response.rs', [
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
        (r'pub type SongbirdResult<T> = Result<T, SongbirdError>\n\n/// ', r'pub type SongbirdResult<T> = Result<T, SongbirdError>;\n\n/// '),
    ])
    
    # Fix service.rs
    fix_file('crates/songbird-types/src/service.rs', [
        (r'use chrono:\{DateTime, Utc\}\n', r'use chrono::{DateTime, Utc};\n'),
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
        (r'self\.([^=]+) = ([^;]+)\n        self', r'self.\1 = \2;\n        self'),
        (r'self\.([^.]+)\.push\(([^)]+)\)\n            self', r'self.\1.push(\2);\n            self'),
        (r'type AllowedValues = [^;]+\n\n/// ', r'type AllowedValues = Option<Vec<serde_json::Value>>;\n\n/// '),
    ])
    
    # Fix traits.rs - this needs special handling for trait methods
    fix_file('crates/songbird-types/src/traits.rs', [
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
        (r'use crate::service:\{[^}]+\}\n', r'use crate::service::{CanonicalServiceEndpoint, CanonicalServiceInfo};\n'),
        (r'async fn ([^(]+\([^)]*\)) -> ([^;]+)\n\n    /// ', r'async fn \1 -> \2;\n\n    /// '),
        (r'fn ([^(]+\([^)]*\)) -> ([^;]+)\n\n    /// ', r'fn \1 -> \2;\n\n    /// '),
        (r'fn ([^(]+\([^)]*\))\n\n    /// ', r'fn \1;\n\n    /// '),
        (r'async fn ([^(]+\([^)]*\))\n\n    /// ', r'async fn \1;\n\n    /// '),
        (r'}\n;\n\n', r'}\n\n'),
        (r'}\n;\n$', r'}\n'),
        (r'async fn stop\(&mut self\) -> SongbirdResult<\(\)>\n\}', r'async fn stop(&mut self) -> SongbirdResult<()>;\n}'),
    ])
    
    # Fix types.rs
    fix_file('crates/songbird-types/src/types.rs', [
        (r'use chrono:\{DateTime, Utc\}\n', r'use chrono::{DateTime, Utc};\n'),
        (r'use serde:\{Deserialize, Serialize\}\n', r'use serde::{Deserialize, Serialize};\n'),
        (r'use super::\*\n\n    #\[test\]', r'use super::*;\n\n    #[test]'),
        (r'let error_response = CanonicalResponse::error\(;', r'let error_response = CanonicalResponse::error("Test error".to_string());'),
        (r'        \}\n\n        match', r'        };\n\n        match'),
        (r'        \}\n\n        assert', r'        };\n\n        assert'),
    ])
    
    # Fix zero_copy.rs
    fix_file('crates/songbird-types/src/zero_copy.rs', [
        (r'pub type ZeroCopyString = Cow<\'static, str>\n\n/// ', r'pub type ZeroCopyString = Cow<\'static, str>;\n\n/// '),
    ])
    
    print("All targeted fixes applied!")

if __name__ == "__main__":
    main() 