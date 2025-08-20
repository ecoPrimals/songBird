#!/bin/bash

echo "=== BULK PERFORMANCECONFIG STRUCT REMOVAL ==="
echo

echo "📍 Remaining PerformanceConfig locations:"
grep -r "struct PerformanceConfig" crates/ --include="*.rs"

echo
echo "�� SYSTEMATIC REMOVAL:"
echo "====================="

# Remove PerformanceConfig struct definitions by replacing each file's content
files_with_perf_config=$(grep -r "struct PerformanceConfig" crates/ --include="*.rs" -l)

for file in $files_with_perf_config; do
    echo "Processing: $file"
    
    # Create a backup and process the file
    cp "$file" "${file}.bak"
    
    # Use awk to remove PerformanceConfig struct definitions
    awk '
    BEGIN { in_struct = 0; brace_count = 0 }
    /pub struct PerformanceConfig/ { 
        print "// REMOVED: PerformanceConfig deprecated struct - use songbird_config::UnifiedPerformanceConfig instead"
        in_struct = 1
        next
    }
    in_struct && /{/ { brace_count++ }
    in_struct && /}/ { 
        brace_count--
        if (brace_count <= 0) {
            in_struct = 0
        }
        next
    }
    !in_struct { print }
    ' "${file}.bak" > "$file"
    
    rm "${file}.bak"
done

echo
echo "✅ VERIFICATION:"
echo "==============="
remaining=$(grep -r "struct PerformanceConfig" crates/ --include="*.rs" | wc -l)
echo "Remaining PerformanceConfig struct definitions: $remaining"

