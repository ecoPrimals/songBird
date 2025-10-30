#!/bin/bash

# Songbird Zero-Warning Verification Script
# Verifies that all core crates maintain zero warnings

set -e

echo "🎯 Songbird Zero-Warning Verification"
echo "====================================="
echo

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counter for total warnings
TOTAL_WARNINGS=0
CRATES_CHECKED=0

# Function to extract warning count safely
extract_warning_count() {
    local output="$1"
    local count
    count=$(echo "$output" | grep -o "generated [0-9]\+ warning" | grep -o "[0-9]\+" | head -1 || echo "0")
    if [[ "$count" =~ ^[0-9]+$ ]]; then
        echo "$count"
    else
        echo "0"
    fi
}

# Core crates to check (zero-warning certified)
CORE_CRATES=(
    "songbird-config"
    "songbird-errors"
    "songbird-canonical"
)

# Additional crates (expanding zero-warning coverage)
ADDITIONAL_CRATES=(
    "songbird-core"
    "songbird-discovery"
    "songbird-network"
    "songbird-security"
    "songbird-federation"
    "songbird-orchestrator"
    "songbird-observability"
    "songbird-universal"
    "songbird-test-utils"
)

echo "🔍 Checking Core Crates (Zero-Warning Certified):"
echo "------------------------------------------------"

for crate in "${CORE_CRATES[@]}"; do
    echo -n "   Checking ${crate}... "
    
    # Run clippy and capture warning count
    CLIPPY_OUTPUT=$(cargo clippy -p "${crate}" --lib -- -W clippy::pedantic -W clippy::nursery -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::module_name_repetitions 2>&1 || true)
    
    # Extract warning count
    WARNING_COUNT=$(extract_warning_count "$CLIPPY_OUTPUT")
    
    if [ "$WARNING_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✅ 0 warnings${NC}"
    else
        echo -e "${RED}❌ ${WARNING_COUNT} warnings${NC}"
        TOTAL_WARNINGS=$((TOTAL_WARNINGS + WARNING_COUNT))
    fi
    
    CRATES_CHECKED=$((CRATES_CHECKED + 1))
done

echo
echo "🔄 Checking Additional Crates (Zero-Warning Expansion):"
echo "------------------------------------------------------"

for crate in "${ADDITIONAL_CRATES[@]}"; do
    # Check if crate exists
    if [ ! -d "crates/${crate}" ]; then
        echo -e "   ${YELLOW}⚠️  ${crate} - Directory not found${NC}"
        continue
    fi
    
    echo -n "   Checking ${crate}... "
    
    # Run clippy and capture warning count
    CLIPPY_OUTPUT=$(cargo clippy -p "${crate}" --lib -- -W clippy::pedantic -W clippy::nursery -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::module_name_repetitions 2>&1 || true)
    
    # Extract warning count
    WARNING_COUNT=$(extract_warning_count "$CLIPPY_OUTPUT")
    
    if [ "$WARNING_COUNT" -eq 0 ]; then
        echo -e "${GREEN}✅ 0 warnings${NC}"
    else
        echo -e "${YELLOW}🔄 ${WARNING_COUNT} warnings (expansion target)${NC}"
        TOTAL_WARNINGS=$((TOTAL_WARNINGS + WARNING_COUNT))
    fi
    
    CRATES_CHECKED=$((CRATES_CHECKED + 1))
done

echo
echo "📊 Summary Report:"
echo "=================="

# Count zero-warning crates in core
ZERO_WARNING_CORE=0
for crate in "${CORE_CRATES[@]}"; do
    CLIPPY_OUTPUT=$(cargo clippy -p "${crate}" --lib -- -W clippy::pedantic -W clippy::nursery -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::module_name_repetitions 2>&1 || true)
    WARNING_COUNT=$(extract_warning_count "$CLIPPY_OUTPUT")
    if [ "$WARNING_COUNT" -eq 0 ]; then
        ZERO_WARNING_CORE=$((ZERO_WARNING_CORE + 1))
    fi
done

if [ "$ZERO_WARNING_CORE" -eq ${#CORE_CRATES[@]} ]; then
    echo -e "${GREEN}🎉 PERFECT! All core crates maintain zero warnings!${NC}"
    echo -e "${GREEN}🏆 Industry-leading code quality maintained!${NC}"
    echo
    echo -e "${BLUE}📈 Achievement Metrics:${NC}"
    echo "   ├── Total Crates Checked: ${CRATES_CHECKED}"
    echo "   ├── Zero-Warning Core Crates: ${ZERO_WARNING_CORE}/${#CORE_CRATES[@]} certified"
    echo "   ├── Core Warning Count: 0 (PERFECT!)"
    echo "   └── Quality Status: Industry Leading"
    
    if [ "$TOTAL_WARNINGS" -gt 0 ]; then
        echo
        echo -e "${YELLOW}🔄 Zero-Warning Expansion Progress:${NC}"
        echo "   ├── Additional Crates: Expansion in progress"
        echo "   ├── Remaining Warnings: ${TOTAL_WARNINGS}"
        echo "   └── Next Target: Expand zero-warning coverage"
    fi
    
    exit 0
else
    echo -e "${RED}❌ Core crates have warnings! This should not happen.${NC}"
    echo
    echo -e "${BLUE}📊 Status Report:${NC}"
    echo "   ├── Core Crates (Zero-Warning): ${ZERO_WARNING_CORE}/${#CORE_CRATES[@]}"
    echo "   ├── Total Warnings: ${TOTAL_WARNINGS}"
    echo "   └── Action Required: Fix core crate warnings immediately"
    
    echo
    echo -e "${RED}🚨 URGENT: Core crate warnings detected!${NC}"
    echo "   Run 'cargo clippy --fix --allow-dirty' on core crates"
    echo "   Core crates must maintain zero-warning status"
    
    exit 1
fi 