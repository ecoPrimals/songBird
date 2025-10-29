#!/bin/bash
# Audit hardcoding in production code
# Usage: ./scripts/audit_hardcoding.sh

set -e

# Color codes
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}SONGBIRD HARDCODING AUDIT${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 1. IPs and Hosts
echo -e "${YELLOW}1. IP Addresses & Hosts${NC}"
echo "Searching for hardcoded IPs/hosts in production code..."

ip_count=$(grep -r "127\.0\.0\.1\|localhost" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  2>/dev/null | wc -l)

if [ "$ip_count" -gt 50 ]; then
    echo -e "${RED}❌ Found $ip_count hardcoded IPs/hosts (target: <50)${NC}"
elif [ "$ip_count" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found $ip_count hardcoded IPs/hosts (target: <50)${NC}"
else
    echo -e "${GREEN}✅ Found $ip_count hardcoded IPs/hosts (excellent!)${NC}"
fi

echo ""
echo "Top offending files:"
grep -r "127\.0\.0\.1\|localhost" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  2>/dev/null | cut -d: -f1 | sort | uniq -c | sort -rn | head -10

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 2. Primal Names
echo -e "${YELLOW}2. Hardcoded Primal Names${NC}"
echo "Searching for BearDog, ToadStool, NestGate, Squirrel in production code..."

primal_count=$(grep -r "BearDog\|ToadStool\|NestGate\|Squirrel" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  ! -path "*/test_utils/*" \
  2>/dev/null | wc -l)

if [ "$primal_count" -gt 0 ]; then
    echo -e "${RED}❌ Found $primal_count hardcoded primal names (target: 0)${NC}"
else
    echo -e "${GREEN}✅ Found $primal_count hardcoded primal names (excellent!)${NC}"
fi

echo ""
echo "Breakdown by primal:"
for primal in "BearDog" "ToadStool" "NestGate" "Squirrel"; do
    count=$(grep -r "$primal" crates/*/src/ \
      --include="*.rs" \
      ! -path "*/tests/*" \
      ! -name "*test*.rs" \
      ! -path "*/test_utils/*" \
      2>/dev/null | wc -l)
    echo "  $primal: $count instances"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 3. Ports
echo -e "${YELLOW}3. Hardcoded Ports${NC}"
echo "Searching for common port numbers in production code..."

port_count=$(grep -rE ":[0-9]{4,5}|port.*[0-9]{4,5}" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  ! -path "*/defaults/ports.rs" \
  2>/dev/null | grep -v "//\|^\s*//" | wc -l)

if [ "$port_count" -gt 20 ]; then
    echo -e "${YELLOW}⚠️  Found ~$port_count hardcoded port references (many may be OK)${NC}"
else
    echo -e "${GREEN}✅ Found ~$port_count hardcoded port references (acceptable)${NC}"
fi

echo ""
echo "Common ports found:"
grep -rE "8080|3000|5432|6379|9200" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  ! -path "*/defaults/ports.rs" \
  2>/dev/null | cut -d: -f1 | sort | uniq -c | sort -rn | head -5

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 4. Magic Numbers
echo -e "${YELLOW}4. Magic Numbers${NC}"
echo "Searching for potential magic numbers (timeouts, sizes, etc.)..."

# Look for common timeout patterns
timeout_count=$(grep -rE "Duration::from_secs\([0-9]+\)|Duration::from_millis\([0-9]+\)" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  ! -path "*/defaults/timeouts.rs" \
  2>/dev/null | wc -l)

echo "Hardcoded timeouts: $timeout_count instances"

# Look for buffer sizes
buffer_count=$(grep -rE "Vec::with_capacity\([0-9]+\)|[Bb]uffer.*=\s*[0-9]+" crates/*/src/ \
  --include="*.rs" \
  ! -path "*/tests/*" \
  ! -name "*test*.rs" \
  2>/dev/null | wc -l)

echo "Hardcoded buffer sizes: $buffer_count instances"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Summary
echo -e "${BLUE}SUMMARY${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
total=$((ip_count + primal_count))
echo "Total critical hardcoding: $total instances"
echo "  - IPs/hosts: $ip_count (target: <50)"
echo "  - Primal names: $primal_count (target: 0)"
echo ""

if [ "$total" -le 50 ] && [ "$primal_count" -eq 0 ]; then
    echo -e "${GREEN}✅ EXCELLENT: Hardcoding under control!${NC}"
    echo ""
    exit 0
elif [ "$total" -le 100 ]; then
    echo -e "${YELLOW}⚠️  GOOD: Hardcoding is manageable, but could be better${NC}"
    echo ""
    echo "Recommendations:"
    echo "1. Centralize remaining IPs to songbird-config/src/defaults/hosts.rs"
    echo "2. Migrate primal names to capability-based discovery"
    echo ""
    exit 0
else
    echo -e "${RED}❌ NEEDS WORK: Too much hardcoding found${NC}"
    echo ""
    echo "Action plan:"
    echo "1. Review ACTION_PLAN_HARDCODING_CLEANUP.md"
    echo "2. Start with centralizing IPs/hosts (week 1)"
    echo "3. Migrate primal names to capabilities (week 2)"
    echo "4. Extract magic numbers to constants (week 3)"
    echo ""
    exit 1
fi

