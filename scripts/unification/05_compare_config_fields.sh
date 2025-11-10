#!/usr/bin/env bash
#
# Field-Level Struct Comparison Tool
# Identifies TRUE duplicates (identical fields) vs domain-specific variants
#
# Usage: ./scripts/unification/05_compare_config_fields.sh [struct_name]
#
# Without args: Analyzes all 118 identified duplicate config names
# With arg: Analyzes specific struct name (e.g., NetworkConfig)
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
cd "$PROJECT_ROOT"

OUTPUT_FILE="FIELD_COMPARISON_REPORT_$(date +%Y%m%d_%H%M%S).md"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "🔍 Field-Level Struct Comparison Tool"
echo "====================================="
echo ""

# Function to extract struct definition with fields
extract_struct_fields() {
    local file="$1"
    local struct_name="$2"
    local output_file="$3"
    
    # Use rust-analyzer style extraction - get from 'pub struct Name' to closing brace
    awk -v struct="$struct_name" '
    BEGIN { in_struct=0; brace_count=0; found=0 }
    
    # Match struct declaration
    /^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]+'"$struct_name"'[[:space:]]*(<|{|\()/ {
        in_struct=1
        found=1
        print
        # Count braces in the same line
        for (i=1; i<=length($0); i++) {
            c = substr($0, i, 1)
            if (c == "{") brace_count++
            else if (c == "}") brace_count--
        }
        if (brace_count == 0) in_struct=0
        next
    }
    
    # Continue capturing if in struct
    in_struct {
        print
        # Count braces
        for (i=1; i<=length($0); i++) {
            c = substr($0, i, 1)
            if (c == "{") brace_count++
            else if (c == "}") {
                brace_count--
                if (brace_count == 0) {
                    in_struct=0
                    exit
                }
            }
        }
    }
    
    END { if (!found) exit 1 }
    ' "$file" > "$output_file" 2>/dev/null
    
    return $?
}

# Function to normalize struct fields for comparison
normalize_fields() {
    local file="$1"
    
    # Extract only field names and types, ignore attributes, comments, visibility
    grep -E '^\s*(pub\s+)?[a-z_][a-z0-9_]*\s*:' "$file" 2>/dev/null | \
        sed -E 's/^\s*(pub\s+)?//' | \
        sed -E 's/,?\s*$//' | \
        sed -E 's/\s+/ /g' | \
        sort
}

# Function to extract field signature (name:type only)
get_field_signature() {
    local file="$1"
    normalize_fields "$file" | md5sum | cut -d' ' -f1
}

# Function to compare two struct definitions
compare_structs() {
    local struct_name="$1"
    shift
    local files=("$@")
    
    if [ ${#files[@]} -lt 2 ]; then
        return 0
    fi
    
    echo "### 🔍 $struct_name (${#files[@]} definitions)"
    echo ""
    
    # Extract all definitions
    declare -A signatures
    declare -A definitions
    local idx=0
    
    for file in "${files[@]}"; do
        local temp_file="$TEMP_DIR/${struct_name}_${idx}.txt"
        if extract_struct_fields "$file" "$struct_name" "$temp_file"; then
            local sig=$(get_field_signature "$temp_file")
            signatures["$idx"]="$sig"
            definitions["$idx"]="$file"
            ((idx++))
        fi
    done
    
    if [ $idx -eq 0 ]; then
        echo "⚠️  Could not extract any definitions"
        echo ""
        return 0
    fi
    
    # Group by signature
    declare -A groups
    for i in "${!signatures[@]}"; do
        local sig="${signatures[$i]}"
        if [ -z "${groups[$sig]:-}" ]; then
            groups["$sig"]="$i"
        else
            groups["$sig"]="${groups[$sig]} $i"
        fi
    done
    
    local num_variants=${#groups[@]}
    
    if [ $num_variants -eq 1 ]; then
        echo "✅ **TRUE DUPLICATE** - All ${#definitions[@]} definitions are IDENTICAL"
        echo ""
        echo "**Locations:**"
        for i in "${!definitions[@]}"; do
            echo "- \`${definitions[$i]}\`"
        done
        echo ""
        echo "**Fields:**"
        echo "\`\`\`rust"
        normalize_fields "$TEMP_DIR/${struct_name}_0.txt"
        echo "\`\`\`"
        echo ""
        echo "**✅ CONSOLIDATION: SAFE** - Replace all with re-exports to canonical"
        echo ""
        return 0
    else
        echo "⚠️  **DOMAIN-SPECIFIC VARIANTS** - $num_variants different implementations"
        echo ""
        
        local variant_num=1
        for sig in "${!groups[@]}"; do
            echo "**Variant $variant_num:**"
            local indices=(${groups[$sig]})
            for i in "${indices[@]}"; do
                echo "- \`${definitions[$i]}\`"
            done
            echo ""
            echo "**Fields:**"
            echo "\`\`\`rust"
            normalize_fields "$TEMP_DIR/${struct_name}_${indices[0]}.txt"
            echo "\`\`\`"
            echo ""
            ((variant_num++))
        done
        
        echo "**⚠️  CONSOLIDATION: REVIEW NEEDED** - Determine if variants are legitimate or should be unified"
        echo ""
        return 1
    fi
}

# Main analysis
if [ $# -gt 0 ]; then
    # Single struct analysis
    STRUCT_NAME="$1"
    echo "Analyzing: $STRUCT_NAME"
    echo ""
    
    # Find all files containing the struct
    files=($(grep -r "struct $STRUCT_NAME" crates/ --include="*.rs" -l | sort))
    
    if [ ${#files[@]} -eq 0 ]; then
        echo "❌ No definitions found for $STRUCT_NAME"
        exit 1
    fi
    
    echo "Found ${#files[@]} definitions"
    echo ""
    
    compare_structs "$STRUCT_NAME" "${files[@]}"
    
else
    # Full analysis of all 118 duplicates
    echo "Analyzing all identified duplicate config names..."
    echo ""
    
    {
        echo "# Field-Level Struct Comparison Report"
        echo "Generated: $(date '+%Y-%m-%d %H:%M:%S')"
        echo ""
        echo "## Summary"
        echo ""
        echo "This report analyzes the 118 identified duplicate config names at the FIELD level"
        echo "to distinguish TRUE duplicates (identical fields) from domain-specific variants."
        echo ""
        echo "---"
        echo ""
        
        # Get list of duplicate names from previous report
        if [ ! -f "DUPLICATE_DEFINITIONS_REPORT.md" ]; then
            echo "❌ DUPLICATE_DEFINITIONS_REPORT.md not found. Run 04_find_duplicates.sh first."
            exit 1
        fi
        
        # Extract struct names (those appearing multiple times)
        struct_names=($(grep -E '^\|\s+\S+\s+\|.*\|.*\|.*\|' DUPLICATE_DEFINITIONS_REPORT.md | \
            awk -F'|' '{gsub(/^[[:space:]]+|[[:space:]]+$/,"",$2); print $2}' | \
            sort -u))
        
        echo "## Analysis Results"
        echo ""
        
        local total=0
        local true_dupes=0
        local variants=0
        
        for struct_name in "${struct_names[@]}"; do
            if [ -z "$struct_name" ] || [ "$struct_name" = "Config Name" ]; then
                continue
            fi
            
            ((total++))
            
            # Find all files containing this struct
            files=($(grep -r "struct $struct_name" crates/ --include="*.rs" -l 2>/dev/null | sort))
            
            if [ ${#files[@]} -lt 2 ]; then
                continue
            fi
            
            if compare_structs "$struct_name" "${files[@]}"; then
                ((true_dupes++))
            else
                ((variants++))
            fi
            
            echo "---"
            echo ""
        done
        
        echo "## Final Statistics"
        echo ""
        echo "- **Total Analyzed**: $total structs"
        echo "- **✅ True Duplicates**: $true_dupes (safe to consolidate)"
        echo "- **⚠️  Domain Variants**: $variants (need review)"
        echo "- **Consolidation Rate**: $(( true_dupes * 100 / total ))% can be safely consolidated"
        echo ""
        echo "## Recommendations"
        echo ""
        echo "### Immediate Actions (True Duplicates)"
        echo "1. Consolidate the $true_dupes TRUE duplicates marked with ✅"
        echo "2. Each consolidation: ~30 minutes (proven process)"
        echo "3. Replace all occurrences with re-exports to canonical"
        echo ""
        echo "### Review Actions (Domain Variants)"
        echo "1. Review each of the $variants domain-specific variants"
        echo "2. Determine if differences are legitimate or accidental"
        echo "3. Either:"
        echo "   - Unify if differences are accidental"
        echo "   - Rename for clarity if legitimate (e.g., NetworkConfig → EdgeNetworkConfig)"
        echo ""
        
    } > "$OUTPUT_FILE"
    
    echo "✅ Analysis complete!"
    echo ""
    echo "📄 Report: $OUTPUT_FILE"
    echo ""
    echo "Next steps:"
    echo "1. Review TRUE duplicates (✅) - safe to consolidate"
    echo "2. Review domain variants (⚠️) - decide unify or rename"
    echo "3. Run: ./scripts/unification/consolidate_true_duplicates.sh"
fi

