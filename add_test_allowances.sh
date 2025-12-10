#!/bin/bash
# Script to add #[allow(clippy::unwrap_used)] to test files
# This makes test code exempt from unwrap warnings since test failures are acceptable

set -e

echo "Adding clippy::unwrap_used allowances to test files..."

# Find all test files and add the annotation if not present
find crates -type f -path "*/tests/*.rs" | while read -r file; do
    # Check if file already has the annotation
    if ! grep -q "allow(clippy::unwrap_used)" "$file"; then
        # Check if file starts with //! or //
        if head -n1 "$file" | grep -q "^//"; then
            # Insert after any leading comments
            line_num=$(grep -n "^[^/]" "$file" | head -n1 | cut -d: -f1)
            if [ -z "$line_num" ]; then
                # File is all comments, append at end
                echo "" >> "$file"
                echo "#![allow(clippy::unwrap_used)]" >> "$file"
                echo "" >> "$file"
            else
                # Insert before first non-comment line
                sed -i "${line_num}i\\#![allow(clippy::unwrap_used)]\\n" "$file"
            fi
        else
            # No leading comments, insert at top
            sed -i '1i\\#![allow(clippy::unwrap_used)]\n' "$file"
        fi
        echo "✓ Added to: $file"
    else
        echo "  Skip (already has): $file"
    fi
done

echo ""
echo "✅ Complete! All test files now have unwrap allowances."

