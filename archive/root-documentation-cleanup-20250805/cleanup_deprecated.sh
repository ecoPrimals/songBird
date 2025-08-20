#!/bin/bash
# Deprecated Items Cleanup Script
set -e

echo "🧹 REMOVING DEPRECATED ITEMS"
echo "============================="

FIXES_APPLIED=0
LOG_FILE="deprecated_cleanup_log.txt"

# Function to log progress
log_progress() {
    echo "    ✅ $1"
    echo "    ✅ $1" >> "$LOG_FILE"
    FIXES_APPLIED=$((FIXES_APPLIED + 1))
}

# Initialize log
echo "# Deprecated Items Cleanup Log - $(date)" > "$LOG_FILE"
echo "# Systematic removal of deprecated structs, traits, and methods" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# Phase 1: Remove simple deprecated config structs
echo "📝 Phase 1: Removing deprecated config structs..."

# Remove deprecated SecurityConfig structs
files_with_security_config=$(find crates/ -name "*.rs" -exec grep -l "#\[deprecated.*UnifiedSongbirdConfig.*SecurityConfig" {} \;)
for file in $files_with_security_config; do
    if [[ -f "$file" ]]; then
        # Remove SecurityConfig struct definition if it exists
        if grep -q "pub struct SecurityConfig" "$file"; then
            echo "  🗑️  Removing SecurityConfig from $file"
            # Create backup
            cp "$file" "$file.backup"
            # Remove struct definition (simple approach)
            sed -i "/^#\[deprecated.*SecurityConfig/,/^}$/c\\
// REMOVED: Deprecated SecurityConfig struct\\
// Use UnifiedSongbirdConfig.security instead" "$file"
            log_progress "Removed SecurityConfig from $file"
        fi
    fi
done

echo ""
echo "🎉 Deprecated cleanup completed!"
echo "📊 Total fixes applied: $FIXES_APPLIED"
echo "📝 See $LOG_FILE for details"

