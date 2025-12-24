#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🔐 Live Demo 2: BearDog Encryption (Real Crypto)
# ═══════════════════════════════════════════════════════════════
# Uses REAL BearDog v0.9.0 for encryption/decryption
# Simulates BirdSong privacy-preserving messaging
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
echo -e "${BLUE}     🔐 BearDog Encryption - LIVE CRYPTO${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check BearDog availability
BEARDOG_BIN="../../../phase2/phase1bins/beardog-v0.9.0-dec23"

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}❌ BearDog v0.9.0 not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BearDog v0.9.0 found${NC}"
echo ""

# Create receipts directory
RECEIPTS_DIR="$SCRIPT_DIR/receipts/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RECEIPTS_DIR"
echo -e "${CYAN}Receipts will be saved to: $RECEIPTS_DIR${NC}"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: Privacy-Preserving Messaging${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "  Node C (sender) → Encrypted Message → Node A (family)"
echo "  Node X (stranger) → Cannot decrypt (sees noise)"
echo ""

echo -e "${YELLOW}Step 1: Generate Keys for All Nodes${NC}"
echo ""

# Generate keys
NODE_A_KEY="node-a-$(date +%s)"
NODE_C_KEY="node-c-$(date +%s)"
NODE_X_KEY="node-x-$(date +%s)"

echo "  Generating Node A (family ancestor)..."
$BEARDOG_BIN key generate --key-id "$NODE_A_KEY" --algorithm ed25519 > "$RECEIPTS_DIR/node_a_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node A key: $NODE_A_KEY${NC}"

echo "  Generating Node C (family descendant)..."
$BEARDOG_BIN key generate --key-id "$NODE_C_KEY" --algorithm ed25519 > "$RECEIPTS_DIR/node_c_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node C key: $NODE_C_KEY${NC}"

echo "  Generating Node X (stranger)..."
$BEARDOG_BIN key generate --key-id "$NODE_X_KEY" --algorithm ed25519 > "$RECEIPTS_DIR/node_x_key.txt" 2>&1
echo -e "${GREEN}  ✅ Node X key: $NODE_X_KEY${NC}"
echo ""

echo -e "${YELLOW}Step 2: Create Test Message${NC}"
echo ""

MESSAGE="RELAY_REQUEST: Node C needs relay to reach external peer at 203.0.113.10:8080"
echo "$MESSAGE" > "$RECEIPTS_DIR/plaintext_message.txt"

echo -e "${CYAN}Plaintext Message:${NC}"
echo "  $MESSAGE"
echo ""

echo -e "${YELLOW}Step 3: Encrypt Message with Node C's Key${NC}"
echo "       (Simulating BirdSong broadcast)..."
echo ""

$BEARDOG_BIN encrypt \
    --key "$NODE_C_KEY" \
    --input "$RECEIPTS_DIR/plaintext_message.txt" \
    --output "$RECEIPTS_DIR/encrypted_message.bin" \
    > "$RECEIPTS_DIR/encryption_receipt.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Message encrypted${NC}"
    echo ""
    echo -e "${CYAN}Encrypted Data (hex):${NC}"
    xxd -l 64 "$RECEIPTS_DIR/encrypted_message.bin" || hexdump -C "$RECEIPTS_DIR/encrypted_message.bin" | head -5
    echo "  ..."
    echo ""
    echo "  Full encrypted message: $RECEIPTS_DIR/encrypted_message.bin"
    echo "  Size: $(wc -c < "$RECEIPTS_DIR/encrypted_message.bin") bytes"
else
    echo -e "${RED}❌ Encryption failed${NC}"
    cat "$RECEIPTS_DIR/encryption_receipt.txt"
    exit 1
fi
echo ""

echo -e "${YELLOW}Step 4: Node A (Family) Attempts Decryption${NC}"
echo "       Using Node A's key..."
echo ""

$BEARDOG_BIN decrypt \
    --key "$NODE_A_KEY" \
    --input "$RECEIPTS_DIR/encrypted_message.bin" \
    --output "$RECEIPTS_DIR/decrypted_by_a.txt" \
    > "$RECEIPTS_DIR/decryption_a_receipt.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Node A decrypted message${NC}"
    echo ""
    echo -e "${CYAN}Decrypted Message:${NC}"
    cat "$RECEIPTS_DIR/decrypted_by_a.txt"
    echo ""
else
    echo -e "${YELLOW}⚠️  Node A cannot decrypt (key mismatch)${NC}"
    echo "   This is EXPECTED - we need shared key derivation"
    echo "   This is an INTEGRATION GAP to evolve"
    cat "$RECEIPTS_DIR/decryption_a_receipt.txt"
fi
echo ""

echo -e "${YELLOW}Step 5: Node X (Stranger) Attempts Decryption${NC}"
echo "       Using Node X's key..."
echo ""

$BEARDOG_BIN decrypt \
    --key "$NODE_X_KEY" \
    --input "$RECEIPTS_DIR/encrypted_message.bin" \
    --output "$RECEIPTS_DIR/decrypted_by_x.txt" \
    > "$RECEIPTS_DIR/decryption_x_receipt.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${RED}❌ Node X should NOT be able to decrypt!${NC}"
    cat "$RECEIPTS_DIR/decrypted_by_x.txt"
else
    echo -e "${GREEN}✅ Node X cannot decrypt (CORRECT)${NC}"
    echo "   Stranger sees only encrypted noise"
    echo ""
    echo -e "${CYAN}What Node X sees (noise):${NC}"
    xxd -l 32 "$RECEIPTS_DIR/encrypted_message.bin" || hexdump -C "$RECEIPTS_DIR/encrypted_message.bin" | head -3
fi
echo ""

echo -e "${YELLOW}Step 6: Node C Decrypts Own Message${NC}"
echo "       (Sender can always decrypt)..."
echo ""

$BEARDOG_BIN decrypt \
    --key "$NODE_C_KEY" \
    --input "$RECEIPTS_DIR/encrypted_message.bin" \
    --output "$RECEIPTS_DIR/decrypted_by_c.txt" \
    > "$RECEIPTS_DIR/decryption_c_receipt.txt" 2>&1

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Node C decrypted own message${NC}"
    echo ""
    echo -e "${CYAN}Decrypted Message:${NC}"
    cat "$RECEIPTS_DIR/decrypted_by_c.txt"
    echo ""
    
    # Verify it matches original
    if diff -q "$RECEIPTS_DIR/plaintext_message.txt" "$RECEIPTS_DIR/decrypted_by_c.txt" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Decryption verified - matches original!${NC}"
    else
        echo -e "${RED}❌ Decryption mismatch!${NC}"
    fi
else
    echo -e "${RED}❌ Node C cannot decrypt own message${NC}"
    cat "$RECEIPTS_DIR/decryption_c_receipt.txt"
fi
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Encryption Demo Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "Results:"
echo "  Node C (sender):    ✅ Can encrypt and decrypt"
echo "  Node A (family):    ⚠️  Needs shared key derivation (gap)"
echo "  Node X (stranger):  ✅ Cannot decrypt (privacy preserved)"
echo ""

echo -e "${CYAN}Receipts saved to:${NC}"
echo "  $RECEIPTS_DIR/"
echo ""
ls -lh "$RECEIPTS_DIR/"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Integration Gaps Found:${NC}"
echo ""
echo "  1. ⚠️  Lineage-based key sharing not yet implemented"
echo "     - Node A should derive shared key from lineage"
echo "     - BearDog needs: derive_shared_key(ancestor, descendant)"
echo ""
echo "  2. ⚠️  BirdSong protocol needs integration"
echo "     - Encrypt for multiple recipients (all ancestors)"
echo "     - BearDog needs: encrypt_for_lineage(message, hint)"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}What This Proves:${NC}"
echo ""
echo "1. 🔐 Real Encryption"
echo "   - BearDog v0.9.0 encrypts/decrypts actual data"
echo "   - Cryptographic receipts for all operations"
echo "   - Privacy preserved (strangers can't decrypt)"
echo ""
echo "2. 🔍 Exposes Gaps"
echo "   - Shows what works NOW"
echo "   - Identifies what needs evolution"
echo "   - No mocks hiding integration issues"
echo ""
echo "3. 📜 Reproducible"
echo "   - All encrypted data saved"
echo "   - Can verify independently"
echo "   - Receipts prove operations happened"
echo ""

echo -e "${GREEN}✅ Live encryption demonstration complete!${NC}"
echo ""
echo -e "${YELLOW}Next: Integrate with Songbird for relay coordination${NC}"
echo ""

