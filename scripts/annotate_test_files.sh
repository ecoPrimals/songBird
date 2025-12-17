#!/bin/bash
# Annotate test files with clippy allow for test-specific patterns
# Usage: ./scripts/annotate_test_files.sh

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Annotating test files with clippy allows...${NC}"

# Annotation to add (preserves existing module doc comments)
ANNOTATION="
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::unnecessary_literal_unwrap)]"

# Counter
count=0

# Find all test files and add annotations if not already present
find crates tests -type f -name "*test*.rs" -o -name "tests.rs" | while read -r file; do
    # Check if file already has the annotations
    if ! grep -q "#\!\[allow(clippy::unwrap_used)\]" "$file"; then
        # Get first line
        first_line=$(head -n 1 "$file")
        
        # If first line is a module doc comment, insert after it
        if [[ "$first_line" == "//!"* ]]; then
            # Find where module docs end
            line_num=$(grep -n "^[^/]" "$file" | head -n 1 | cut -d: -f1)
            if [ -z "$line_num" ]; then
                line_num=$(wc -l < "$file")
            fi
            
            # Insert annotation after module docs
            {
                head -n "$((line_num - 1))" "$file"
                echo "$ANNOTATION"
                tail -n +"$line_num" "$file"
            } > "$file.tmp"
            mv "$file.tmp" "$file"
        else
            # Insert at beginning
            {
                echo "$ANNOTATION"
                cat "$file"
            } > "$file.tmp"
            mv "$file.tmp" "$file"
        fi
        
        echo -e "${GREEN}✓${NC} Annotated: $file"
        ((count++))
    fi
done

echo -e "${BLUE}Annotated $count test files${NC}"
echo -e "${GREEN}Done!${NC}"

