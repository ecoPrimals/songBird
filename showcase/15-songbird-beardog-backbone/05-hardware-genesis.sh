#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════
# 🔐 Demo 5: Hardware Root of Trust - Genesis with SoloKey
# ═══════════════════════════════════════════════════════════════
# Shows how hardware security keys seed Genesis identity
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
echo -e "${BLUE}     🔐 Hardware Root of Trust - Genesis with SoloKey${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

WORK_DIR="$SCRIPT_DIR/data/hardware-genesis-demo"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}     Scenario: Secure Node Onboarding with Hardware${NC}"
echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo ""
echo "Goal: Create a new node with hardware-backed identity"
echo ""
echo "Supported Hardware:"
echo "  - SoloKey (USB security key)"
echo "  - YubiKey"
echo "  - TPM 2.0 chip"
echo "  - Secure Enclave (macOS/iOS)"
echo "  - Hardware Security Module (HSM)"
echo ""
echo "This demo uses: SoloKey"
echo ""

echo -e "${YELLOW}Step 1: Detect Hardware${NC}"
echo "       Scanning for security keys..."
echo ""

# Simulate hardware detection
sleep 1

if [ -e "/dev/hidraw0" ] || [ -e "/dev/ttyUSB0" ]; then
    echo -e "${GREEN}  ✅ Hardware key detected${NC}"
    HW_DETECTED=true
else
    echo -e "${YELLOW}  ⚠️  No hardware key detected (using mock for demo)${NC}"
    HW_DETECTED=false
fi

HARDWARE_TYPE="SoloKey"
HARDWARE_ID="solokey-$(openssl rand -hex 8)"

cat > hardware_info.json <<EOF
{
  "hardware_type": "$HARDWARE_TYPE",
  "hardware_id": "$HARDWARE_ID",
  "firmware_version": "4.1.5",
  "detected": $HW_DETECTED,
  "capabilities": [
    "ed25519-signing",
    "key-derivation",
    "secure-storage"
  ]
}
EOF

echo -e "${CYAN}Hardware Info:${NC}"
cat hardware_info.json | jq '.'
echo ""

echo -e "${YELLOW}Step 2: Hardware Attestation${NC}"
echo "       BearDog requests hardware attestation..."
echo ""

# Simulate hardware attestation
# In production: beardog attest-hardware --device solokey

echo "  Requesting attestation certificate..."
sleep 1

ATTESTATION_CERT="$(openssl rand -hex 128)"
ATTESTATION_SIG="$(openssl rand -hex 64)"

cat > attestation.json <<EOF
{
  "hardware_id": "$HARDWARE_ID",
  "attestation_certificate": "$ATTESTATION_CERT",
  "attestation_signature": "$ATTESTATION_SIG",
  "attested_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "verified": true
}
EOF

echo -e "${GREEN}  ✅ Hardware attestation successful${NC}"
cat attestation.json | jq '. | {hardware_id, verified, attested_at}'
echo ""

echo -e "${YELLOW}Step 3: Derive Keys from Hardware Seed${NC}"
echo "       BearDog derives keys using hardware PRNG..."
echo ""

# Key derivation from hardware
echo "  Hardware seed → Key derivation (HKDF)"
echo "    - Master seed: From hardware TRNG"
echo "    - Domain: ecoPrimals/songbird"
echo "    - Key type: Ed25519"
echo ""
sleep 1

PUBLIC_KEY="$(openssl rand -hex 32)"
PRIVATE_KEY_HANDLE="solokey://$HARDWARE_ID/key1"

cat > derived_keys.json <<EOF
{
  "public_key": "$PUBLIC_KEY",
  "private_key_handle": "$PRIVATE_KEY_HANDLE",
  "key_type": "Ed25519",
  "derived_from": "$HARDWARE_ID",
  "derivation_path": "m/44'/0'/0'/0/0",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}  ✅ Keys derived from hardware${NC}"
cat derived_keys.json | jq '. | {public_key, private_key_handle, key_type}'
echo ""

echo -e "${YELLOW}Step 4: Physical Proximity Verification${NC}"
echo "       Songbird uses BLE for proximity..."
echo ""

# Simulate BLE proximity
echo "  Scanning for parent node via BLE..."
sleep 1

PARENT_NODE="node-a-parent"
PROXIMITY_PROOF="ble_rssi_-45dBm_$(openssl rand -hex 16)"

cat > proximity.json <<EOF
{
  "parent_node": "$PARENT_NODE",
  "detection_method": "BLE",
  "signal_strength": "-45 dBm",
  "distance_estimate": "< 2 meters",
  "proximity_proof": "$PROXIMITY_PROOF",
  "verified_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}  ✅ Parent node detected nearby${NC}"
cat proximity.json | jq '. | {parent_node, detection_method, distance_estimate}'
echo ""

echo -e "${YELLOW}Step 5: Witness Gathering${NC}"
echo "       Songbird coordinates witness network..."
echo ""

# Generate witnesses
WITNESS_COUNT=3
WITNESSES=()

for i in $(seq 1 $WITNESS_COUNT); do
    WITNESS_ID="witness-hw-$i-$(openssl rand -hex 4)"
    WITNESS_SIG="$(openssl rand -hex 64)"
    WITNESSES+=("$WITNESS_ID")
    
    cat > "witness_${i}.json" <<EOF
{
  "witness_id": "$WITNESS_ID",
  "witnessed_node": "$HARDWARE_ID",
  "signature": "$WITNESS_SIG",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
done

echo -e "${GREEN}  ✅ $WITNESS_COUNT witnesses gathered${NC}"
echo ""

echo -e "${YELLOW}Step 6: Genesis Ceremony (Hardware-Backed)${NC}"
echo "       BearDog signs lineage with hardware-backed keys..."
echo ""

CEREMONY_ID="ceremony-hw-$(uuidgen)"
NEW_NODE_ID="node-hw-$(openssl rand -hex 4)"

# Parent signs lineage (using hardware key)
echo "  Parent node signs with hardware key..."
sleep 1

LINEAGE_SIG="$(openssl rand -hex 64)"

cat > hardware_genesis.json <<EOF
{
  "ceremony_id": "$CEREMONY_ID",
  "node_id": "$NEW_NODE_ID",
  "hardware_backed": true,
  "hardware_type": "$HARDWARE_TYPE",
  "hardware_id": "$HARDWARE_ID",
  "public_key": "$PUBLIC_KEY",
  "lineage": {
    "parent": "$PARENT_NODE",
    "ancestors": ["$PARENT_NODE"],
    "signature": "$LINEAGE_SIG",
    "signed_by_hardware": true
  },
  "attestation": {
    "certificate": "$ATTESTATION_CERT",
    "verified": true
  },
  "witnesses": $(jq -s '[.[] | {witness_id, signature, timestamp}]' witness_*.json),
  "proximity_proof": "$PROXIMITY_PROOF",
  "birth_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo -e "${GREEN}  ✅ Genesis ceremony complete${NC}"
cat hardware_genesis.json | jq '. | {ceremony_id, node_id, hardware_backed, hardware_type, lineage: .lineage | {parent, signed_by_hardware}}'
echo ""

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}     ✅ Hardware-Backed Genesis Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "New Node Identity:"
cat hardware_genesis.json | jq '. | {node_id, hardware_type, hardware_id, public_key, lineage: .lineage.parent, witnesses: (.witnesses | length)}'
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Security Benefits:${NC}"
echo ""
echo "1. 🔐 Tamper-Resistant Keys"
echo "   ✅ Private key never leaves hardware"
echo "   ✅ Signing happens inside secure element"
echo "   ✅ Physical extraction extremely difficult"
echo ""
echo "2. 🎲 High-Quality Entropy"
echo "   ✅ Hardware TRNG (True Random Number Generator)"
echo "   ✅ Not dependent on OS entropy"
echo "   ✅ Suitable for long-term keys"
echo ""
echo "3. 📜 Attestation"
echo "   ✅ Hardware certifies it's genuine"
echo "   ✅ Firmware version verified"
echo "   ✅ Chain of trust to manufacturer"
echo ""
echo "4. 🔒 Secure Storage"
echo "   ✅ Keys stored in hardware, not filesystem"
echo "   ✅ Protected from memory dumps"
echo "   ✅ Survives OS reinstall"
echo ""
echo "5. 👤 User Verification"
echo "   ✅ Can require PIN for signing"
echo "   ✅ Button press for authorization"
echo "   ✅ Multi-factor authentication"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Comparison:${NC}"
echo ""
echo "Software-Only Genesis:"
echo "  ❌ Keys in filesystem (can be copied)"
echo "  ❌ OS entropy (may be predictable)"
echo "  ❌ No hardware attestation"
echo "  ❌ Vulnerable to malware"
echo ""
echo "Hardware-Backed Genesis:"
echo "  ✅ Keys in secure element (tamper-resistant)"
echo "  ✅ Hardware TRNG (high quality)"
echo "  ✅ Attestation verified"
echo "  ✅ Protected from malware"
echo ""

echo -e "${BLUE}───────────────────────────────────────────────────────────────${NC}"
echo -e "${YELLOW}Integration with Lineage Relay:${NC}"
echo ""
echo "Hardware-backed nodes can:"
echo ""
echo "  🧬 Prove lineage cryptographically"
echo "     - Hardware signs lineage proofs"
echo "     - Cannot be forged"
echo ""
echo "  🔐 Relay with strong identity"
echo "     - Relay authorization backed by hardware"
echo "     - Descendants trust hardware attestation"
echo ""
echo "  🎵 BirdSong with hardware keys"
echo "     - Encryption keys from hardware"
echo "     - Decryption in secure element"
echo ""
echo "  🔄 Long-term identity"
echo "     - Keys survive device changes"
echo "     - Move SoloKey to new machine"
echo "     - Same identity preserved"
echo ""

echo -e "${GREEN}✅ Sovereign identity with hardware root of trust!${NC}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Supported Hardware:${NC}"
echo ""
echo "  - SoloKey: https://solokeys.com/"
echo "  - YubiKey: https://www.yubico.com/"
echo "  - TPM 2.0: Built into most modern laptops"
echo "  - Secure Enclave: macOS/iOS devices"
echo "  - Nitrokey: https://www.nitrokey.com/"
echo ""
echo -e "${YELLOW}Next Demo: 06-full-integration.sh${NC}"
echo "           (Complete end-to-end scenario)"
echo ""

