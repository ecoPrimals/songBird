#!/bin/bash
# Comprehensive syntax error fix script

# Fix .clone(] -> .clone()
find crates -name "*.rs" -type f -exec sed -i 's/\.clone(\]/\.clone()/g' {} \;

# Fix extra closing parens on assert_eq! and similar macros
find crates -name "*.rs" -type f -exec sed -i 's/assert_eq!(\([^)]*\)))/assert_eq!(\1)/g' {} \;

# Fix missing closing parens in function calls
find crates -name "*.rs" -type f -exec sed -i 's/Some(\([0-9]*\);/Some(\1));/g' {} \;

echo "Fixed common syntax patterns. Running cargo fmt to identify remaining issues..."
cargo fmt --all 2>&1 | grep "^error:" | head -20

