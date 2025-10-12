#!/usr/bin/env bash
# Comprehensive syntax error fix script

cd "$(dirname "$0")"

echo "Fixing systematic syntax errors..."

# Fix .insert() patterns with missing closing parens
find crates tests -name "*.rs" -type f -exec sed -i 's/\.insert(\([^;]*\));/\.insert(\1));/g' {} \;

# Fix HashMap/Vec initialization patterns
find crates tests -name "*.rs" -type f -exec sed -i 's/Vec::new(),/Vec::new())/g' {} \;
find crates tests -name "*.rs" -type f -exec sed -i 's/HashMap::new(),/HashMap::new())/g' {} \;
find crates tests -name "*.rs" -type f -exec sed -i 's/HashSet::new(),/HashSet::new())/g' {} \;

# Fix assert! macro patterns
find crates tests -name "*.rs" -type f -exec sed -i 's/assert!(\([^;]*\));/assert!(\1));/g' {} \;
find crates tests -name "*.rs" -type f -exec sed -i 's/assert_eq!(\([^;]*\));/assert_eq!(\1));/g' {} \;

# Fix extra closing parens after .to_string()
find crates tests -name "*.rs" -type f -exec sed -i 's/\.to_string());/.to_string();/g' {} \;

# Fix match arms with missing closing parens
find crates tests -name "*.rs" -type f -exec sed -i 's/=> println!("\([^"]*\)"),/=> println!("\1"),/g' {} \;

# Fix string literals with prefix errors (for known patterns)
find crates tests -name "*.rs" -type f -exec sed -i 's/"api".to_string()/  "api".to_string()/g' {} \;

echo "Syntax fixes applied. Running cargo check..."
cargo check --workspace 2>&1 | head -100

