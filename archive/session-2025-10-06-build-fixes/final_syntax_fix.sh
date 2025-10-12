#!/bin/bash
# Final batch fix for all remaining )); patterns

echo "🔧 Applying final syntax fixes..."

# Fix all )); patterns to );
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/));$/);/g'

# Fix all )); patterns in middle of code
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/));\s*$/);/g'

# Fix .push(x)); patterns
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.push(\([^)]*\)));/\.push(\1);/g'

# Fix .insert(x)); patterns
find crates -name "*.rs" -type f -print0 | xargs -0 sed -i 's/\.insert(\([^)]*\)));/\.insert(\1);/g'

echo "✅ Final fixes applied. Checking results..."
echo ""
echo "Remaining errors:"
cargo fmt --all --check 2>&1 | grep "Error writing files" | wc -l

echo ""
echo "Files still with errors:"
cargo fmt --all --check 2>&1 | grep "cannot parse" | sed 's/.*cannot parse //'

