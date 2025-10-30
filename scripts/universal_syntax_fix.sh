#!/bin/bash
# Universal syntax fix - fix all remaining HashMap::new()) and similar issues

cd /home/eastgate/Development/ecoPrimals/songbird

# Find all Rust files (excluding target and archive)
find . -name "*.rs" -not -path "./target/*" -not -path "./archive/*" -not -path "./experiments/target/*" | while read file; do
    # Fix HashMap::new()) -> HashMap::new(),
    sed -i 's/HashMap::new())/HashMap::new()/g' "$file"
    
    # Fix Vec::new()) -> Vec::new(),
    sed -i 's/Vec::new())/Vec::new()/g' "$file"
    
    # Fix .into()); -> .into());
    # Sed patterns for missing closing parens before semicolons  
    # This is more complex - skip for now as it needs context
done

echo "✓ Applied universal syntax fixes"

