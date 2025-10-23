#!/bin/bash
# Find actual production unwraps (not in tests)

echo "=== TRUE PRODUCTION UNWRAPS ==="
echo ""

count=0
for file in $(grep -r '\.unwrap()' crates/*/src --include='*.rs' -l | grep -v "/tests/"); do
    # Check if file has #[cfg(test)] module
    if grep -q '#\[cfg(test)\]' "$file" 2>/dev/null; then
        # Check unwraps outside test module
        production_unwraps=$(awk '
            /#\[cfg\(test\)\]/ { in_test=1 }
            /^}/ && in_test { in_test=0 }
            !in_test && /\.unwrap\(\)/ { print }
        ' "$file" | wc -l)
        if [ "$production_unwraps" -gt 0 ]; then
            echo "$file: $production_unwraps production unwraps"
            ((count+=production_unwraps))
        fi
    else
        # No test module, count all unwraps
        file_unwraps=$(grep -c '\.unwrap()' "$file")
        echo "$file: $file_unwraps unwraps (no test module)"
        ((count+=file_unwraps))
    fi
done

echo ""
echo "Total true production unwraps: $count"
