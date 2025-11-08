#!/usr/bin/env bash
# 🔄 Primal Reference Migration Script
#
# Systematically migrates primal-specific references to capability-based patterns
#
# Usage: ./scripts/migrate_primal_references.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DRY_RUN=false

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo -e "${YELLOW}🔍 DRY RUN MODE - No files will be modified${NC}"
fi

cd "$PROJECT_ROOT"

echo -e "${BLUE}🔄 Primal Reference Migration${NC}"
echo "================================"
echo

# Step 1: Find and report primal-specific patterns
echo -e "${BLUE}Step 1: Analyzing primal-specific patterns...${NC}"

BEARDOG_REFS=$(grep -r "beardog" crates/*/src/ --include="*.rs" -i | wc -l || true)
SQUIRREL_REFS=$(grep -r "squirrel" crates/*/src/ --include="*.rs" -i | wc -l || true)
TOADSTOOL_REFS=$(grep -r "toadstool" crates/*/src/ --include="*.rs" -i | wc -l || true)
NESTGATE_REFS=$(grep -r "nestgate" crates/*/src/ --include="*.rs" -i | wc -l || true)

echo "  📊 BearDog references: $BEARDOG_REFS"
echo "  📊 Squirrel references: $SQUIRREL_REFS"
echo "  📊 Toadstool references: $TOADSTOOL_REFS"
echo "  📊 NestGate references: $NESTGATE_REFS"
echo "  📊 Total: $((BEARDOG_REFS + SQUIRREL_REFS + TOADSTOOL_REFS + NESTGATE_REFS))"
echo

# Step 2: Migrate specific function calls
echo -e "${BLUE}Step 2: Migration patterns...${NC}"

cat << 'EOF'
🔄 MIGRATION PATTERNS:

1. **get_primal_endpoint() → get_capability_endpoint()**
   Pattern: get_primal_endpoint("beardog")
   Replace: capability_endpoints::get_capability_endpoint("security").await?
   
2. **BeardogClient → SecurityCapabilityClient**
   Pattern: BeardogClient::new(endpoint)
   Replace: SecurityCapabilityClient::new().await?
   
3. **SquirrelClient → AiCapabilityClient**
   Pattern: SquirrelClient::new(endpoint)
   Replace: AiCapabilityClient::new().await?
   
4. **ToadstoolClient → ComputeCapabilityClient**
   Pattern: ToadstoolClient::new(endpoint)
   Replace: ComputeCapabilityClient::new().await?
   
5. **NestGateClient → StorageCapabilityClient**
   Pattern: NestGateClient::new(endpoint)
   Replace: StorageCapabilityClient::new().await?

EOF

# Step 3: Show files that need migration
echo -e "${BLUE}Step 3: Files requiring migration...${NC}"
echo

echo -e "${YELLOW}Files with get_primal_endpoint:${NC}"
grep -r "get_primal_endpoint" crates/*/src/ --include="*.rs" -l || echo "  ✅ None found"
echo

echo -e "${YELLOW}Files with primal-specific clients:${NC}"
grep -r "BeardogClient\|SquirrelClient\|ToadstoolClient\|NestGateClient" crates/*/src/ --include="*.rs" -l || echo "  ✅ None found"
echo

# Step 4: Migration instructions
echo -e "${BLUE}Step 4: Manual migration steps...${NC}"
echo "================================"
echo

cat << 'EOF'
📝 MANUAL MIGRATION STEPS:

For each file found above:

1. **Update imports**:
   ```rust
   // Remove old primal-specific imports
   // use songbird_primal_sdk::beardog::BeardogClient;
   
   // Add capability-based imports
   use songbird_primal_sdk::security_capability_client::SecurityCapabilityClient;
   use songbird_config::capability_endpoints;
   ```

2. **Replace client initialization**:
   ```rust
   // ❌ OLD
   let endpoint = get_primal_endpoint("beardog")?;
   let client = BeardogClient::new(endpoint)?;
   
   // ✅ NEW
   let client = SecurityCapabilityClient::new().await?;
   // Endpoint discovery happens automatically!
   ```

3. **Update function signatures** (if needed):
   ```rust
   // If function becomes async due to discovery:
   pub async fn initialize_security() -> SongbirdResult<()>
   ```

4. **Test the changes**:
   ```bash
   # Set required environment variables
   export CAPABILITY_SECURITY_ENDPOINT="http://security:8443"
   
   # Run tests
   cargo test
   ```

EOF

# Step 5: Validate deprecated modules
echo -e "${BLUE}Step 5: Checking deprecated module usage...${NC}"
echo

echo -e "${YELLOW}Files using deprecated primal modules:${NC}"
grep -r "use.*songbird_primal_sdk::(beardog|squirrel|toadstool)::" crates/*/src/ --include="*.rs" || echo "  ✅ None found"
echo

# Step 6: Summary
echo -e "${BLUE}Summary${NC}"
echo "================================"
echo

TOTAL_REFS=$((BEARDOG_REFS + SQUIRREL_REFS + TOADSTOOL_REFS + NESTGATE_REFS))

if [[ $TOTAL_REFS -eq 0 ]]; then
    echo -e "${GREEN}✅ No primal-specific references found! Migration complete!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Found $TOTAL_REFS primal-specific references${NC}"
    echo
    echo "Next steps:"
    echo "  1. Review files listed above"
    echo "  2. Update imports to use capability-based clients"
    echo "  3. Replace primal-specific calls with capability discovery"
    echo "  4. Test with environment variables set"
    echo "  5. Run: cargo test --workspace"
    echo
    exit 1
fi

