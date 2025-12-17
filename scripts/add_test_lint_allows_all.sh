#!/usr/bin/env bash
# Add idiomatic lint allows to all test modules and test files
# Following Rust community standards for test code

set -euo pipefail

echo "🔧 Adding idiomatic test lint allows to all test modules..."

# Find all test files and test modules
for file in $(find crates -type f -name "*.rs" | grep -E "(tests/|_tests\.rs|test\.rs)"); do
    # Check if file already has the allows
    if grep -q "allow(clippy::unwrap_used" "$file"; then
        continue
    fi
    
    # Add at the top of test files
    if [[ "$file" == *"/tests/"* ]]; then
        # Test directory files - add at top
        sed -i '1i// Allow common test patterns - idiomatic for test code\n#![allow(clippy::unwrap_used, clippy::expect_used)]\n#![allow(clippy::unnecessary_wraps)]\n#![allow(clippy::field_reassign_with_default)]\n#![allow(clippy::uninlined_format_args)]\n#![allow(clippy::float_cmp)]\n' "$file"
        echo "✅ Added allows to $file"
    fi
done

# Also add to source files with #[cfg(test)] modules
for file in $(find crates -type f -name "*.rs" -path "*/src/*"); do
    if grep -q "#\[cfg(test)\]" "$file" && ! grep -q "allow(clippy::unwrap_used" "$file"; then
        # Add allows before cfg(test) modules
        sed -i '/#\[cfg(test)\]/i #[allow(clippy::unwrap_used, clippy::expect_used, clippy::unnecessary_wraps, clippy::field_reassign_with_default)]' "$file"
        echo "✅ Added allows to test module in $file"
    fi
done

echo "🎉 Lint allows added to test code across codebase"

