#!/bin/bash
# Find unwraps in production code (excluding tests)

echo "🔍 Finding unwraps in PRODUCTION code only (excluding tests)..."
echo ""

# Find all .rs files in crates/*/src/ but NOT in tests directories
find crates -name "*.rs" -path "*/src/*" ! -path "*/tests/*" ! -name "*test*.rs" | while read file; do
    # Check if file has unwraps
    unwrap_count=$(grep -c "\.unwrap()" "$file" 2>/dev/null || echo "0")
    
    if [ "$unwrap_count" -gt 0 ]; then
        # Check if unwraps are in test functions or #[cfg(test)] modules
        # Extract context around unwraps
        grep -n "\.unwrap()" "$file" | while read line; do
            line_num=$(echo "$line" | cut -d: -f1)
            
            # Check previous 10 lines for test markers
            context=$(sed -n "$((line_num-10)),$((line_num))p" "$file" 2>/dev/null)
            
            # If not in test context, report it
            if ! echo "$context" | grep -q "#\[test\]\|#\[cfg(test)\]\|#\[tokio::test\]\|fn test_\|mod tests {"; then
                echo "⚠️  $file:$line_num"
                echo "   $(echo "$line" | cut -d: -f2-)"
                echo ""
            fi
        done
    fi
done

echo ""
echo "✅ Search complete!"
echo ""
echo "📊 Summary:"
total_files=$(find crates -name "*.rs" -path "*/src/*" ! -path "*/tests/*" ! -name "*test*.rs" -exec grep -l "\.unwrap()" {} \; 2>/dev/null | wc -l)
echo "Files with unwraps: $total_files"
