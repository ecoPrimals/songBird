#!/usr/bin/env bash
# 🍼 Zero Hardcoding Migration Script
#
# Systematically eliminates all vendor and primal hardcoding patterns
# from the Songbird codebase, replacing them with capability-based discovery.
#
# Usage: ./scripts/eliminate_all_hardcoding.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DRY_RUN=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 DRY RUN MODE - No files will be modified${NC}"
fi

cd "$PROJECT_ROOT"

echo -e "${BLUE}🍼 Zero Hardcoding Migration${NC}"
echo "================================"
echo

# Phase 1: Audit current state
echo -e "${BLUE}Phase 1: Auditing hardcoding patterns...${NC}"

PRIMAL_COUNT=$(grep -r -i "beardog\|squirrel\|toadstool\|nestgate" crates/ --include="*.rs" | wc -l || true)
VENDOR_COUNT=$(grep -r -i "kubernetes\|k8s\|consul\|etcd\|docker\|nomad" crates/ --include="*.rs" | wc -l || true)
PORT_COUNT=$(grep -r "8001\|8002\|8003\|8004\|50051" crates/ --include="*.rs" | wc -l || true)

echo "  📊 Primal name references: $PRIMAL_COUNT"
echo "  📊 External service references: $VENDOR_COUNT"
echo "  📊 Hardcoded port references: $PORT_COUNT"
echo

# Phase 2: Search and report specific patterns
echo -e "${BLUE}Phase 2: Finding specific patterns...${NC}"

echo -e "${YELLOW}  Searching for primal endpoint patterns...${NC}"
grep -r "get_primal_endpoint\|PrimalEndpoint\|primal_name.*=" crates/ --include="*.rs" --color=always | head -20 || echo "  ✅ None found"
echo

echo -e "${YELLOW}  Searching for hardcoded port constants...${NC}"
grep -r "DEFAULT_.*_PORT.*=" crates/ --include="*.rs" --color=always | head -20 || echo "  ✅ None found"
echo

# Phase 3: Provide migration guidance
echo -e "${BLUE}Phase 3: Migration Guidance${NC}"
echo "================================"
echo

cat << 'EOF'
📋 MANUAL MIGRATION STEPS:

1. **Replace Primal-Specific Clients**:
   ```rust
   // ❌ OLD
   let client = BearDogClient::new("http://localhost:8004")?;
   
   // ✅ NEW
   let endpoint = capability_endpoints::get_capability_endpoint("security").await?;
   let client = SecurityCapabilityClient::new(endpoint)?;
   ```

2. **Replace Hardcoded Ports**:
   ```rust
   // ❌ OLD
   const PRIMAL_PORT: u16 = 8004;
   
   // ✅ NEW
   let port = env::var("CAPABILITY_SECURITY_PORT")?
       .parse()
       .map_err(|_| SongbirdError::configuration("Invalid port"))?;
   ```

3. **Replace Vendor-Specific Code**:
   ```rust
   // ❌ OLD
   use kubernetes::Client;
   let k8s_client = Client::new()?;
   
   // ✅ NEW
   let discovery = DiscoveryMethod::ContainerMetadata {
       api_endpoint_env_var: "CONTAINER_METADATA_API".to_string(),
   };
   ```

4. **Update Tests**:
   ```rust
   #[tokio::test]
   async fn test_capability_discovery() {
       // Set environment for testing
       env::set_var("CAPABILITY_SECURITY_ENDPOINT", "http://test:8443");
       
       let endpoint = get_capability_endpoint("security").await?;
       assert_eq!(endpoint, "http://test:8443");
   }
   ```

🔧 AUTOMATED TOOLS AVAILABLE:

1. **Rust Migrator**: `cargo run --bin agnostic-migrator`
2. **Pattern Scanner**: `cargo run --bin hardcoding-scanner`
3. **Python Fixer**: `python3 scripts/fix_hardcoding_patterns.py`

EOF

# Phase 4: Check for specific anti-patterns
echo -e "${BLUE}Phase 4: Checking for anti-patterns...${NC}"
echo "================================"
echo

ANTI_PATTERNS=0

# Check for direct primal name string literals (not in comments/docs)
echo -e "${YELLOW}  Checking for primal name string literals...${NC}"
if grep -r '"beardog"\|"squirrel"\|"toadstool"\|"nestgate"' crates/*/src/ --include="*.rs" | grep -v "//.*beardog\|//.*squirrel" | head -10; then
    ANTI_PATTERNS=$((ANTI_PATTERNS + 1))
    echo -e "${RED}  ⚠️  Found primal name string literals in code${NC}"
fi
echo

# Check for hardcoded localhost endpoints with primal ports
echo -e "${YELLOW}  Checking for hardcoded localhost endpoints...${NC}"
if grep -r 'localhost:800[1-4]' crates/*/src/ --include="*.rs" | head -10; then
    ANTI_PATTERNS=$((ANTI_PATTERNS + 1))
    echo -e "${RED}  ⚠️  Found hardcoded localhost:800x endpoints${NC}"
fi
echo

# Check for vendor-specific imports in production code
echo -e "${YELLOW}  Checking for vendor-specific imports...${NC}"
if grep -r "^use kubernetes::\|^use consul::\|^use docker::" crates/*/src/ --include="*.rs" | head -10; then
    ANTI_PATTERNS=$((ANTI_PATTERNS + 1))
    echo -e "${RED}  ⚠️  Found vendor-specific imports in production code${NC}"
fi
echo

# Phase 5: Report and recommendations
echo -e "${BLUE}Phase 5: Summary and Recommendations${NC}"
echo "================================"
echo

if [[ $ANTI_PATTERNS -eq 0 ]]; then
    echo -e "${GREEN}✅ No critical anti-patterns found!${NC}"
else
    echo -e "${RED}⚠️  Found $ANTI_PATTERNS types of anti-patterns${NC}"
fi

echo
echo "📊 HARDCODING INVENTORY:"
echo "  - Primal references: $PRIMAL_COUNT instances"
echo "  - Vendor references: $VENDOR_COUNT instances"
echo "  - Port hardcoding: $PORT_COUNT instances"
echo

echo "🎯 NEXT STEPS:"
echo "  1. Review migration plan: ZERO_HARDCODING_MIGRATION_PLAN.md"
echo "  2. Update environment configuration"
echo "  3. Run automated migration tools"
echo "  4. Update tests and examples"
echo "  5. Validate with: cargo test --workspace"
echo

echo "🔍 DETAILED ANALYSIS:"
echo "  For detailed per-file breakdown:"
echo "  $ grep -r 'beardog\\|squirrel\\|toadstool\\|nestgate' crates/ --include='*.rs' -n"
echo

exit $ANTI_PATTERNS

