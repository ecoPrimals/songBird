#!/bin/bash
# Comprehensive syntax fix for all remaining errors

echo "🔧 Starting comprehensive syntax fix..."

# Fix pattern: .clone(] -> .clone()
echo "Fixing .clone(] patterns..."
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.clone(\]/\.clone()/g'

# Fix pattern: )); at end of assert_eq where it should be );
echo "Fixing double closing parens in assert_eq..."
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/assert_eq!(\([^)]*\), Some(\([^)]*\)))));/assert_eq!(\1, Some(\2)));/g'

# Fix missing closing parens in Some()
echo "Fixing Some() patterns..."
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/Some(\([0-9]*\);/Some(\1));/g'
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/Some(\("[^"]*"\)\.to_string();/Some(\1.to_string());/g'
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/Some(\("[^"]*"\.into());/Some(\1.into()));/g'

# Fix vec![] patterns with wrong closing
echo "Fixing vec![] patterns..."
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/vec!\[\([^]]*\)])/vec![\1]/g'

# Fix table.row patterns
echo "Fixing table.row patterns..."
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.row(vec!\[/\.row(vec![/g'

echo "✅ Automated fixes complete. Checking results..."
cargo fmt --all --check 2>&1 | grep -E "^(error|Error)" | head -30

