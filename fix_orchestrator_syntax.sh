#!/bin/bash
# Automated syntax fix script for orchestrator crate
# Created: October 12, 2025
# Purpose: Fix systematic syntax corruption from failed search/replace

set -e

CRATE_DIR="crates/songbird-orchestrator/src/core"
BACKUP_DIR="orchestrator-syntax-backup-$(date +%Y%m%d-%H%M%S)"

echo "🔧 Songbird Orchestrator Syntax Fix Script"
echo "==========================================="
echo ""

# Create backup
echo "📦 Creating backup..."
mkdir -p "$BACKUP_DIR"
cp -r "$CRATE_DIR" "$BACKUP_DIR/"
echo "✅ Backup created in: $BACKUP_DIR"
echo ""

# Function to fix a file
fix_file() {
    local file="$1"
    local basename=$(basename "$file")
    
    echo "🔨 Fixing: $basename"
    
    # Create temporary file
    local tmpfile=$(mktemp)
    
    # Apply fixes using sed
    sed -e 's/pub enum \([A-Za-z_][A-Za-z0-9_]*\)  {/pub enum \1 {/g' \
        -e 's/pub struct \([A-Za-z_][A-Za-z0-9_]*\)  {/pub struct \1 {/g' \
        -e 's/impl \([A-Za-z_][A-Za-z0-9_]*\)  {/impl \1 {/g' \
        -e 's/\([A-Za-z][A-Za-z0-9_]*\))/\1,/g' \
        -e 's/config)/config,/g' \
        -e 's/(&self)self,/(&self)/g' \
        -e 's/\.to_string(),/\.to_string()),/g' \
        -e 's/thresholds\.insert(\([^)]*\)));/thresholds.insert(\1);/g' \
        "$file" > "$tmpfile"
    
    # Move temp file back
    mv "$tmpfile" "$file"
    echo "   ✓ Fixed"
}

# Find and fix all Rust files in core directory
echo "🔍 Finding files to fix..."
FILES=$(find "$CRATE_DIR" -name "*.rs" -type f 2>/dev/null || true)

if [ -z "$FILES" ]; then
    echo "⚠️  No files found in $CRATE_DIR"
    exit 1
fi

echo "📝 Found $(echo "$FILES" | wc -l) files"
echo ""

# Fix each file
for file in $FILES; do
    fix_file "$file"
done

echo ""
echo "🧪 Testing compilation..."
if cargo build -p songbird-orchestrator 2>&1 | grep -q "Finished"; then
    echo "✅ SUCCESS! Orchestrator crate compiles!"
    echo ""
    echo "💾 Backup location: $BACKUP_DIR"
    echo "   (You can delete this if everything works)"
else
    echo "⚠️  Still have compilation errors. Manual fixes may be needed."
    echo "   Check the output above for details."
    echo ""
    echo "🔄 To restore backup:"
    echo "   rm -rf $CRATE_DIR"
    echo "   cp -r $BACKUP_DIR/core $CRATE_DIR"
fi

echo ""
echo "✨ Script complete!"

