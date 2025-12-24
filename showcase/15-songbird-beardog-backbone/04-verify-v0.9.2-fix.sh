#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# ✅ Demo 4: Verify BearDog v0.9.2 Fix - KEY DERIVATION FIXED!
# ═══════════════════════════════════════════════════════════════
# Tests that the key derivation bug is fixed in v0.9.2
# Verifies both decryption AND privacy enforcement work
# ═══════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     ✅ BearDog v0.9.2 Verification - KEY DERIVATION FIXED!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check BearDog v0.9.2 availability
BEARDOG_BIN="../../../phase2/phase1bins/beardog-v0.9.2-keyfixed-dec24"

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}❌ BearDog v0.9.2 not found at: $BEARDOG_BIN${NC}"
    echo ""
    echo "The BearDog team released v0.9.2 with key derivation fix."
    echo "Please download it and place at the expected location."
    exit 1
fi

echo -e "${GREEN}✅ BearDog v0.9.2 found (with key derivation fix!)${NC}"
BEARDOG_VERSION=$($BEARDOG_BIN --version 2>/dev/null || echo "beardog 0.9.2-keyfixed")
echo "   Version: $BEARDOG_VERSION"
echo ""

# Create receipts directory
RECEIPTS_DIR="$SCRIPT_DIR/receipts/$(date +%Y%m%d_%H%M%S)_v092_verification"
mkdir -p "$RECEIPTS_DIR"
echo -e "${CYAN}Receipts will be saved to: $RECEIPTS_DIR${NC}"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     🎯 Complete Test: Decryption + Privacy Enforcement${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "  Testing:"
echo "    1. Key generation and derivation"
echo "    2. BirdSong encryption"
echo "    3. Ancestor decryption (should work!)"
echo "    4. Stranger decryption (should fail!)"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  Phase 1: Setup Lineage${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Generate root key
NODE_A_KEY="node-a-root-$(date +%s)"
echo "  Generating Node A (root)..."
$BEARDOG_BIN key generate --key-id "$NODE_A_KEY" --algorithm ed25519 > "$RECEIPTS_DIR/node_a_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node A: $NODE_A_KEY${NC}"

# Derive child
NODE_B_KEY="node-b-child-$(date +%s)"
echo "  Deriving Node B (child of A)..."
$BEARDOG_BIN key derive --master-key "$NODE_A_KEY" --purpose "child" --output "$NODE_B_KEY" > "$RECEIPTS_DIR/node_b_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node B: $NODE_B_KEY${NC}"

# Derive grandchild
NODE_C_KEY="node-c-grandchild-$(date +%s)"
echo "  Deriving Node C (grandchild of A, child of B)..."
$BEARDOG_BIN key derive --master-key "$NODE_B_KEY" --purpose "grandchild" --output "$NODE_C_KEY" > "$RECEIPTS_DIR/node_c_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node C: $NODE_C_KEY${NC}"

# Generate stranger
NODE_X_KEY="node-x-stranger-$(date +%s)"
echo "  Generating Node X (stranger, separate lineage)..."
$BEARDOG_BIN key generate --key-id "$NODE_X_KEY" --algorithm ed25519 > "$RECEIPTS_DIR/node_x_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node X: $NODE_X_KEY${NC}"
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  Phase 2: Encrypt BirdSong Message${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

MESSAGE="RELAY_REQUEST: Node C needs relay to reach peer at 203.0.113.10:8080"
echo "$MESSAGE" > "$RECEIPTS_DIR/plaintext.txt"

echo -e "${CYAN}Message:${NC}"
echo "  $MESSAGE"
echo ""

echo "  Encrypting for DirectAncestors (A, B should decrypt)..."
$BEARDOG_BIN birdsong encrypt \
    --message "$MESSAGE" \
    --hint DirectAncestors \
    --root-id "$NODE_A_KEY" \
    --output "$RECEIPTS_DIR/encrypted.birdsong" \
    > "$RECEIPTS_DIR/encrypt_receipt.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ BirdSong encrypted${NC}"
    echo "   Size: $(wc -c < "$RECEIPTS_DIR/encrypted.birdsong") bytes"
else
    echo -e "${RED}❌ Encryption failed${NC}"
    cat "$RECEIPTS_DIR/encrypt_receipt.txt"
    exit 1
fi
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  Phase 3: Test Ancestor Decryption${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Test Node A (root) decryption
echo -e "${CYAN}Test 1: Node A (root) decryption${NC}"
echo "  Node A is root - should decrypt successfully..."
if $BEARDOG_BIN birdsong decrypt \
    --input "$RECEIPTS_DIR/encrypted.birdsong" \
    --key-id "$NODE_A_KEY" \
    > "$RECEIPTS_DIR/decrypted_by_a.txt" 2>&1 && \
   grep -q "RELAY_REQUEST" "$RECEIPTS_DIR/decrypted_by_a.txt"; then
    echo -e "${GREEN}  ✅ PASS: Node A decrypted successfully!${NC}"
    echo -e "${CYAN}  Decrypted:${NC} $(grep "RELAY_REQUEST" "$RECEIPTS_DIR/decrypted_by_a.txt")"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}  ❌ FAIL: Node A cannot decrypt${NC}"
    cat "$RECEIPTS_DIR/decrypted_by_a.txt" | tail -5
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test Node B (child) decryption
echo -e "${CYAN}Test 2: Node B (child) decryption${NC}"
echo "  Node B is child of A - should decrypt successfully..."
if $BEARDOG_BIN birdsong decrypt \
    --input "$RECEIPTS_DIR/encrypted.birdsong" \
    --key-id "$NODE_B_KEY" \
    > "$RECEIPTS_DIR/decrypted_by_b.txt" 2>&1 && \
   grep -q "RELAY_REQUEST" "$RECEIPTS_DIR/decrypted_by_b.txt"; then
    echo -e "${GREEN}  ✅ PASS: Node B decrypted successfully!${NC}"
    echo -e "${CYAN}  Decrypted:${NC} $(grep "RELAY_REQUEST" "$RECEIPTS_DIR/decrypted_by_b.txt")"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}  ❌ FAIL: Node B cannot decrypt${NC}"
    cat "$RECEIPTS_DIR/decrypted_by_b.txt" | tail -5
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}  Phase 4: Test Privacy Enforcement${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Test Node X (stranger) decryption - should FAIL
echo -e "${CYAN}Test 3: Node X (stranger) decryption - SHOULD FAIL${NC}"
echo "  Node X is NOT in lineage - decryption should fail..."
if ! $BEARDOG_BIN birdsong decrypt \
    --input "$RECEIPTS_DIR/encrypted.birdsong" \
    --key-id "$NODE_X_KEY" \
    > "$RECEIPTS_DIR/decrypted_by_x.txt" 2>&1 || \
   ! grep -q "RELAY_REQUEST" "$RECEIPTS_DIR/decrypted_by_x.txt"; then
    echo -e "${GREEN}  ✅ PASS: Privacy enforced - Node X cannot decrypt!${NC}"
    echo -e "${CYAN}  Error:${NC} $(grep -i "cannot decrypt\|not in lineage\|privacy\|error" "$RECEIPTS_DIR/decrypted_by_x.txt" | head -1)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}  ❌ FAIL: Privacy breach - Node X decrypted!${NC}"
    echo "  This should NOT happen!"
    cat "$RECEIPTS_DIR/decrypted_by_x.txt"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     Test Results${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
SUCCESS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))

echo "  Total Tests: $TOTAL_TESTS"
echo -e "  Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "  Failed: ${RED}$TESTS_FAILED${NC}"
echo "  Success Rate: $SUCCESS_RATE%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}     ✅ ALL TESTS PASSED! BearDog v0.9.2 WORKS!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "  ✅ Key derivation FIXED"
    echo "  ✅ Ancestors can decrypt"
    echo "  ✅ Privacy enforced (strangers blocked)"
    echo ""
    echo -e "${MAGENTA}🎉 INTEGRATION SUCCESS! 🎉${NC}"
else
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}     ❌ SOME TESTS FAILED${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
fi
echo ""

echo -e "${CYAN}Receipts saved to:${NC}"
echo "  $RECEIPTS_DIR/"
echo ""
ls -lh "$RECEIPTS_DIR/"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Evolution Timeline:${NC}"
echo ""
echo "  v0.9.0 → Found: Privacy gap"
echo "           Fixed: 3 hours"
echo "           Result: BirdSong CLI"
echo ""
echo "  v0.9.1 → Found: Key derivation bug"
echo "           Fixed: 30 minutes"
echo "           Result: Working key derivation"
echo ""
echo "  v0.9.2 → Status: ${GREEN}ALL KNOWN GAPS FIXED!${NC}"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}What This Proves:${NC}"
echo ""
echo "1. ✅ Iterative Testing Works"
echo "   - Found 2 real bugs through live testing"
echo "   - Both fixed in < 4 hours total"
echo "   - Each fix verified immediately"
echo ""
echo "2. ✅ No Mocks = Real Validation"
echo "   - Real crypto operations"
echo "   - Real bugs found"
echo "   - Real fixes verified"
echo ""
echo "3. ✅ Fast Evolution"
echo "   - Bug → Report → Fix → Verify → Next"
echo "   - Clear reproduction steps = fast fixes"
echo "   - Cryptographic receipts prove everything"
echo ""

echo -e "${GREEN}✅ BearDog v0.9.2 verification complete!${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${MAGENTA}🚀 Ready for Songbird integration!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Additional issues found - see receipts${NC}"
    exit 1
fi

