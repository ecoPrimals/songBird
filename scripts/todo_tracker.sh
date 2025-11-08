#!/bin/bash
# TODO Tracker Script - Extract and categorize TODO/FIXME markers
# For Songbird codebase unification project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/todo_reports"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Songbird TODO Tracker ===${NC}"
echo "Project: $PROJECT_ROOT"
echo "Output: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to extract TODOs
extract_todos() {
    local pattern=$1
    local label=$2
    local output_file=$3
    
    echo -e "${YELLOW}Extracting ${label}...${NC}"
    
    grep -rn "$pattern" "$PROJECT_ROOT/crates" \
        --include="*.rs" \
        --exclude-dir="target" \
        --exclude-dir="tests" 2>/dev/null \
        > "$output_file" || true
    
    local count=$(wc -l < "$output_file" 2>/dev/null || echo 0)
    echo -e "${GREEN}Found $count ${label} markers${NC}"
}

# Extract all TODO patterns
echo -e "${BLUE}Scanning codebase...${NC}"

extract_todos "TODO" "TODO" "$OUTPUT_DIR/todos.txt"
extract_todos "FIXME" "FIXME" "$OUTPUT_DIR/fixmes.txt"
extract_todos "XXX" "XXX" "$OUTPUT_DIR/xxx.txt"
extract_todos "HACK" "HACK" "$OUTPUT_DIR/hacks.txt"

echo ""
echo -e "${BLUE}=== Summary Report ===${NC}"

# Generate summary
{
    echo "# TODO Tracking Report"
    echo "Generated: $(date)"
    echo "Project: Songbird"
    echo ""
    echo "## Summary"
    echo ""
    echo "| Type | Count | File |"
    echo "|------|-------|------|"
    echo "| TODO | $(wc -l < "$OUTPUT_DIR/todos.txt" 2>/dev/null || echo 0) | todo_reports/todos.txt |"
    echo "| FIXME | $(wc -l < "$OUTPUT_DIR/fixmes.txt" 2>/dev/null || echo 0) | todo_reports/fixmes.txt |"
    echo "| XXX | $(wc -l < "$OUTPUT_DIR/xxx.txt" 2>/dev/null || echo 0) | todo_reports/xxx.txt |"
    echo "| HACK | $(wc -l < "$OUTPUT_DIR/hacks.txt" 2>/dev/null || echo 0) | todo_reports/hacks.txt |"
    echo ""
    
    # Calculate totals
    local total=$(($(wc -l < "$OUTPUT_DIR/todos.txt" 2>/dev/null || echo 0) + \
                    $(wc -l < "$OUTPUT_DIR/fixmes.txt" 2>/dev/null || echo 0) + \
                    $(wc -l < "$OUTPUT_DIR/xxx.txt" 2>/dev/null || echo 0) + \
                    $(wc -l < "$OUTPUT_DIR/hacks.txt" 2>/dev/null || echo 0)))
    
    echo "**Total**: $total markers"
    echo ""
    echo "## Top Files by TODO Count"
    echo ""
    
    # Find top files with TODOs
    if [ -s "$OUTPUT_DIR/todos.txt" ]; then
        cat "$OUTPUT_DIR/todos.txt" | \
            awk -F: '{print $1}' | \
            sort | uniq -c | sort -rn | head -20 | \
            awk '{printf "- `%s`: %d TODOs\n", $2, $1}'
    fi
    
    echo ""
    echo "## Categorization Guide"
    echo ""
    echo "### TODO - Feature Work & Improvements"
    echo "- Estimated: ~250 markers"
    echo "- Priority: P2-P3 (Medium-Low)"
    echo "- Timeline: 3-6 months"
    echo ""
    echo "### FIXME - Bug Fixes"
    echo "- Estimated: ~100 markers"
    echo "- Priority: P1-P2 (High-Medium)"
    echo "- Timeline: 1-2 months"
    echo ""
    echo "### XXX - Refactoring Needs"
    echo "- Estimated: ~200 markers"
    echo "- Priority: P2-P3 (Medium-Low)"
    echo "- Timeline: 3-6 months"
    echo ""
    echo "### HACK - Temporary Workarounds"
    echo "- Estimated: ~20 markers"
    echo "- Priority: P1-P2 (High-Medium)"
    echo "- Timeline: 1-2 months"
    echo ""
    echo "## Next Steps"
    echo ""
    echo "1. Review each file: \`todo_reports/*.txt\`"
    echo "2. Create GitHub issues for each marker"
    echo "3. Categorize by priority (P0-P3)"
    echo "4. Assign to milestones"
    echo "5. Track progress in project board"
    echo ""
    echo "## Cleanup Target"
    echo ""
    echo "- Current: $total markers"
    echo "- Target: 0 markers"
    echo "- Timeline: 6 months"
    echo "- Rate: ~100 markers/month"
    
} > "$OUTPUT_DIR/summary.md"

# Display summary
cat "$OUTPUT_DIR/summary.md"

echo ""
echo -e "${GREEN}=== Report Complete ===${NC}"
echo -e "Full reports saved to: ${BLUE}$OUTPUT_DIR${NC}"
echo ""
echo "Files generated:"
echo "  - $OUTPUT_DIR/todos.txt"
echo "  - $OUTPUT_DIR/fixmes.txt"
echo "  - $OUTPUT_DIR/xxx.txt"
echo "  - $OUTPUT_DIR/hacks.txt"
echo "  - $OUTPUT_DIR/summary.md"
echo ""
echo -e "${YELLOW}Next: Review reports and create GitHub issues${NC}"

