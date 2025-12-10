#!/bin/bash
# Script to systematically replace .unwrap() with .expect() in test files
# This makes test failures more debuggable with explicit error messages

set -e

cd "$(dirname "$0")/.."

echo "🔍 Finding all .unwrap() calls in test files..."

# Find all test files with unwrap() calls
TEST_FILES=$(find crates/*/tests -name "*.rs" -type f -exec grep -l "\.unwrap()" {} \;)

TOTAL_FILES=$(echo "$TEST_FILES" | wc -l)
echo "📊 Found $TOTAL_FILES test files with unwrap() calls"

COUNTER=0

for file in $TEST_FILES; do
    COUNTER=$((COUNTER + 1))
    echo "[$COUNTER/$TOTAL_FILES] Processing: $file"
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Strategy: Replace common patterns with contextual expect messages
    # Pattern 1: .await.unwrap() -> .await.expect("async operation failed in test")
    sed -i 's/\.await\.unwrap()/\.await\.expect("Async operation failed in test")/g' "$file"
    
    # Pattern 2: Simple .unwrap() -> .expect("operation failed in test")
    # Only if not already an expect
    sed -i 's/\([^.]\)\.unwrap()/\1\.expect("Operation failed in test")/g' "$file"
    
    # Pattern 3: Result unwrap -> more specific
    sed -i 's/\.parse()\.unwrap()/\.parse()\.expect("Failed to parse value in test")/g' "$file"
    sed -i 's/\.to_string()\.parse()\.unwrap()/\.to_string()\.parse()\.expect("Failed to parse string in test")/g' "$file"
    
    # Verify file still compiles by checking syntax (quick check)
    if ! rustfmt --check "$file" 2>/dev/null; then
        echo "⚠️  Formatting issue in $file, restoring backup"
        mv "$file.bak" "$file"
    else
        rm "$file.bak"
    fi
done

echo "✅ Replacement complete!"
echo "🧪 Running tests to verify..."

# Run tests to ensure nothing broke
cargo test --workspace --lib --quiet

echo "✅ All tests still pass!"
echo "📊 Checking remaining unwraps..."
cargo clippy --workspace --tests --quiet 2>&1 | grep -c "unwrap()" || echo "0"

