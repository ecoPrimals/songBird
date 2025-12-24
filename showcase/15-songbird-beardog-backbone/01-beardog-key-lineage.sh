#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🔑 Live Demo 1: BearDog Key Lineage (Real Crypto)
# ═══════════════════════════════════════════════════════════════
# Uses REAL BearDog v0.9.0 to establish cryptographic lineage
# Generates cryptographic receipts for validation
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     🔑 BearDog Key Lineage - LIVE CRYPTO${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check BearDog availability
BEARDOG_BIN="../../../phase2/phase1bins/beardog-v0.9.0-dec23"

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}❌ BearDog v0.9.0 not found at: $BEARDOG_BIN${NC}"
    echo ""
    echo "Expected location: ../phase2/phase1bins/beardog-v0.9.0-dec23"
    echo ""
    echo "Download from: https://github.com/ecoPrimals/bearDog/releases/tag/v0.9.0-integration-dec23"
    exit 1
fi

echo -e "${GREEN}✅ BearDog v0.9.0 found${NC}"
BEARDOG_VERSION=$($BEARDOG_BIN --version)
echo "   Version: $BEARDOG_VERSION"
echo ""

# Create receipts directory
RECEIPTS_DIR="$SCRIPT_DIR/receipts/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RECEIPTS_DIR"
echo -e "${CYAN}Receipts will be saved to: $RECEIPTS_DIR${NC}"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: Establish 3-Generation Key Lineage${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "  Root Key (A) → Child Key (B) → Grandchild Key (C)"
echo ""

echo -e "${YELLOW}Step 1: Generate Root Key (Node A)${NC}"
echo "       Using BearDog to generate master key..."
echo ""

# Generate root key
ROOT_KEY_ID="node-a-root-$(date +%s)"
$BEARDOG_BIN key generate \
    --key-id "$ROOT_KEY_ID" \
    --algorithm ed25519 \
    > "$RECEIPTS_DIR/01_root_key_generation.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Root key generated${NC}"
    echo "   Key ID: $ROOT_KEY_ID"
    cat "$RECEIPTS_DIR/01_root_key_generation.txt"
else
    echo -e "${RED}❌ Root key generation failed${NC}"
    cat "$RECEIPTS_DIR/01_root_key_generation.txt"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 2: Derive Child Key (Node B) from Root${NC}"
echo "       Using BearDog key derivation..."
echo ""

# Derive child key
CHILD_KEY_ID="node-b-child-$(date +%s)"
$BEARDOG_BIN key derive \
    --master-key "$ROOT_KEY_ID" \
    --purpose "child-node-b" \
    --output "$CHILD_KEY_ID" \
    > "$RECEIPTS_DIR/02_child_key_derivation.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Child key derived${NC}"
    echo "   Key ID: $CHILD_KEY_ID"
    echo "   Parent: $ROOT_KEY_ID"
    cat "$RECEIPTS_DIR/02_child_key_derivation.txt"
else
    echo -e "${RED}❌ Child key derivation failed${NC}"
    cat "$RECEIPTS_DIR/02_child_key_derivation.txt"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 3: Derive Grandchild Key (Node C) from Child${NC}"
echo "       Continuing lineage chain..."
echo ""

# Derive grandchild key
GRANDCHILD_KEY_ID="node-c-grandchild-$(date +%s)"
$BEARDOG_BIN key derive \
    --master-key "$CHILD_KEY_ID" \
    --purpose "grandchild-node-c" \
    --output "$GRANDCHILD_KEY_ID" \
    > "$RECEIPTS_DIR/03_grandchild_key_derivation.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Grandchild key derived${NC}"
    echo "   Key ID: $GRANDCHILD_KEY_ID"
    echo "   Parent: $CHILD_KEY_ID"
    cat "$RECEIPTS_DIR/03_grandchild_key_derivation.txt"
else
    echo -e "${RED}❌ Grandchild key derivation failed${NC}"
    cat "$RECEIPTS_DIR/03_grandchild_key_derivation.txt"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 4: Verify Key Lineage${NC}"
echo "       Querying BearDog for lineage chain..."
echo ""

# Query lineage for grandchild
$BEARDOG_BIN key lineage \
    --key-id "$GRANDCHILD_KEY_ID" \
    --json \
    > "$RECEIPTS_DIR/04_lineage_verification.json" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Lineage verified${NC}"
    echo ""
    echo -e "${CYAN}Lineage Chain:${NC}"
    cat "$RECEIPTS_DIR/04_lineage_verification.json" | jq '.' || cat "$RECEIPTS_DIR/04_lineage_verification.json"
else
    echo -e "${YELLOW}⚠️  Lineage query not yet implemented in BearDog v0.9.0${NC}"
    echo "   This is an INTEGRATION GAP we need to evolve"
    cat "$RECEIPTS_DIR/04_lineage_verification.json"
fi
echo ""

echo -e "${YELLOW}Step 5: List All Keys${NC}"
echo "       Verifying all keys exist..."
echo ""

$BEARDOG_BIN key list > "$RECEIPTS_DIR/05_key_list.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Keys listed${NC}"
    cat "$RECEIPTS_DIR/05_key_list.txt"
else
    echo -e "${RED}❌ Key listing failed${NC}"
    cat "$RECEIPTS_DIR/05_key_list.txt"
fi
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Key Lineage Established!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "Cryptographic Lineage:"
echo "  Root:        $ROOT_KEY_ID"
echo "  Child:       $CHILD_KEY_ID"
echo "  Grandchild:  $GRANDCHILD_KEY_ID"
echo ""

echo -e "${CYAN}Receipts saved to:${NC}"
echo "  $RECEIPTS_DIR/"
echo ""
ls -lh "$RECEIPTS_DIR/"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Integration Gaps Found:${NC}"
echo ""

# Check what worked and what didn't
GAPS_FOUND=0

if ! grep -q "lineage" "$RECEIPTS_DIR/04_lineage_verification.json" 2>/dev/null; then
    echo "  ⚠️  BearDog key lineage query may need evolution"
    ((GAPS_FOUND++))
fi

if [ $GAPS_FOUND -eq 0 ]; then
    echo "  ✅ No integration gaps found - all operations successful!"
else
    echo ""
    echo "  Found $GAPS_FOUND integration gap(s) to evolve"
fi
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}What This Proves:${NC}"
echo ""
echo "1. 🔑 Real Cryptography"
echo "   - BearDog v0.9.0 generates actual keys"
echo "   - Key derivation creates parent→child relationships"
echo "   - All operations produce cryptographic receipts"
echo ""
echo "2. 📜 Reproducible"
echo "   - All receipts saved to disk"
echo "   - Can be verified independently"
echo "   - Timestamps and key IDs preserved"
echo ""
echo "3. 🔍 Validates Integration"
echo "   - Exposes what works NOW"
echo "   - Identifies gaps to evolve"
echo "   - No mocks hiding issues"
echo ""

echo -e "${GREEN}✅ Live crypto demonstration complete!${NC}"
echo ""
echo -e "${YELLOW}Next: Use these keys for encryption/decryption${NC}"
echo ""

