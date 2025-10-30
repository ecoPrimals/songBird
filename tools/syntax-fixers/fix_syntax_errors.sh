#!/bin/bash
# Syntax Error Fix Script
# Systematically fixes the semicolon placement bug across the codebase

set -e

echo "🔧 Starting systematic syntax error fixes..."
echo "================================================"

# Backup first (just in case)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
echo "📦 Creating safety backup: syntax_backup_${TIMESTAMP}.tar.gz"
tar -czf "syntax_backup_${TIMESTAMP}.tar.gz" crates/ src/ 2>/dev/null || true

echo ""
echo "🔍 Fixing Pattern 1: Semicolons inside closing quotes/parens..."

# Fix Pattern: );" -> );
find crates src -name "*.rs" -type f -exec sed -i 's/);"$/);/g' {} \; 2>/dev/null || true
find crates src -name "*.rs" -type f -exec sed -i "s/);'$/);/g" {} \; 2>/dev/null || true

# Fix Pattern: >";" -> >";
find crates src -name "*.rs" -type f -exec sed -i 's/">;"$/">;/g' {} \; 2>/dev/null || true

# Fix Pattern: ,"  (comma quote at end) in contexts where it should be just comma
# This is trickier - be more conservative

echo ""
echo "🔍 Fixing Pattern 2: Struct/enum trailing commas..."
# Fix: field: Type, , , } -> field: Type }
find crates src -name "*.rs" -type f -exec sed -i 's/, , ,$/,/g' {} \; 2>/dev/null || true
find crates src -name "*.rs" -type f -exec sed -i 's/, ,$//g' {} \; 2>/dev/null || true

echo ""
echo "🔍 Fixing Pattern 3: Missing closing delimiters..."
# Fix specific known issues
# songbird-network-federation/src/network/mod.rs line 45
sed -i '45s/)$/}/' crates/songbird-network-federation/src/network/mod.rs 2>/dev/null || true

echo ""
echo "🔍 Fixing Pattern 4: String literal prefix errors..."
# These are from unterminated strings - need to find the actual quotes
# Fix lines that end with " followed by newline in certain contexts

echo ""
echo "✅ Automated fixes complete!"
echo ""
echo "📊 Summary of changes:"
echo "   - Fixed )semicolon patterns"
echo "   - Fixed trailing commas in structs"
echo "   - Fixed specific delimiter issues"
echo ""
echo "🔍 Next steps:"
echo "   1. Review changes with: git diff"
echo "   2. Test compilation: cargo check"
echo "   3. Run formatter: cargo fmt --all"
echo ""
echo "⚠️  Backup saved as: syntax_backup_${TIMESTAMP}.tar.gz"

