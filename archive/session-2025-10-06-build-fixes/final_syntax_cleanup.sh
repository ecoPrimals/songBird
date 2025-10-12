#!/usr/bin/env bash
# Final comprehensive syntax fix - fix remaining patterns globally

cd "$(dirname "$0")"

echo "=== Final Syntax Cleanup ==="
echo "Fixing remaining error patterns across all crates..."

# Fix format! with extra semicolon/paren
find crates -name "*.rs" -exec sed -i 's/format!(\([^)]*\));/format!(\1));/g' {} \;
find crates -name "*.rs" -exec sed -i 's/format!$/format!(/' {} \;

# Fix assert_eq with extra paren
find crates -name "*.rs" -exec sed -i 's/assert_eq!(\([^;]*\));$/assert_eq!(\1));/g' {} \;

# Fix .send() and .push() with missing closing paren
find crates -name "*.rs" -exec sed -i 's/\.send(\([^;]*\));$/\.send(\1));/g' {} \;
find crates -name "*.rs" -exec sed -i 's/\.push(\([^;]*\));$/\.push(\1));/g' {} \;

# Fix .insert() with missing closing paren  
find crates -name "*.rs" -exec sed -i 's/\.insert(\([^,]*\), \([^;]*\));$/\.insert(\1, \2));/g' {} \;

# Fix .extend() with extra closing paren
find crates -name "*.rs" -exec sed -i 's/\.extend(\([^)]*\)));$/\.extend(\1));/g' {} \;

# Fix .contains with extra paren
find crates -name "*.rs" -exec sed -i 's/\.contains([^;]*));$/\.contains(\1));/g' {} \;

echo "Syntax fixes applied. Running final check..."
cargo check --workspace 2>&1 | head -100
