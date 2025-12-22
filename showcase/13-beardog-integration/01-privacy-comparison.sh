#!/usr/bin/env bash
set -euo pipefail

# Privacy Comparison: Plaintext vs BirdSong
#
# Demonstrates the privacy gain from BearDog integration

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  🔒 Privacy Comparison: Plaintext vs BirdSong                     ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Parse mode
MODE="${1:---both}"

show_plaintext() {
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  Mode 1: Plaintext Discovery (Current - No BearDog)"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "Broadcast Message (UDP):"
    echo "{"
    echo "  \"node_id\": \"eastgate-12345678\","
    echo "  \"node_name\": \"eastgate\","
    echo "  \"capabilities\": ["
    echo "    \"orchestration\","
    echo "    \"compute\","
    echo "    \"gpu-nvidia-rtx-4090\""
    echo "  ],"
    echo "  \"endpoints\": ["
    echo "    \"https://192.168.1.100:8080\","
    echo "    \"https://10.0.0.50:8080\""
    echo "  ],"
    echo "  \"federation_id\": \"university-cs-dept\""
    echo "}"
    echo ""
    echo "👁️  **OBSERVER CAN SEE:**"
    echo "  ✓ Node identity: eastgate-12345678"
    echo "  ✓ Node name: eastgate"
    echo "  ✓ Capabilities: GPU type (RTX 4090)"
    echo "  ✓ Network addresses: 192.168.1.100, 10.0.0.50"
    echo "  ✓ Federation membership: university-cs-dept"
    echo "  ✓ Complete topology"
    echo ""
    echo "📊 **PRIVACY LEVEL: LOW**"
    echo "  - Suitable for trusted LANs only"
    echo "  - Campus networks, research labs"
    echo "  - NOT suitable for internet deployment"
    echo ""
}

show_birdsong() {
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  Mode 2: BirdSong Discovery (With BearDog)"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "Broadcast Message (UDP):"
    echo "{"
    echo "  \"version\": 1,"
    echo "  \"ciphertext\": \"7f8e9a2c5d1b3f6e9c8a7d5b2e4f1a8c...\","
    echo "  \"lineage_hint\": \"LineageRoot(university-root)\","
    echo "  \"timestamp\": \"2025-12-21T12:34:56Z\","
    echo "  \"signature\": \"3a5c7e9f2b4d6a8c1e3f5b7d9a2c4e6f...\""
    echo "}"
    echo ""
    echo "👁️  **OBSERVER (NON-FAMILY) SEES:**"
    echo "  ✓ Encrypted blob (no meaning)"
    echo "  ✓ Packet size (~500 bytes)"
    echo "  ✓ Timestamp (but not content)"
    echo "  ✗ Cannot decrypt ciphertext"
    echo "  ✗ Cannot see node identity"
    echo "  ✗ Cannot see capabilities"
    echo "  ✗ Cannot see endpoints"
    echo "  ✗ Cannot map topology"
    echo ""
    echo "👨‍👩‍👧‍👦 **FAMILY MEMBER SEES:**"
    echo "  1. Receives same broadcast"
    echo "  2. Recognizes lineage hint"
    echo "  3. Requests key from BearDog"
    echo "  4. BearDog verifies lineage proof"
    echo "  5. BearDog provides decryption key"
    echo "  6. Decrypts and sees full message:"
    echo "     ✓ Node identity"
    echo "     ✓ Capabilities"
    echo "     ✓ Endpoints"
    echo "     ✓ Everything"
    echo ""
    echo "📊 **PRIVACY LEVEL: HIGH**"
    echo "  - Suitable for untrusted networks"
    echo "  - Internet, public WiFi, cellular"
    echo "  - Privacy-preserving by design"
    echo ""
}

show_comparison() {
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  Privacy Comparison Summary"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    printf "%-30s | %-20s | %-20s\n" "Property" "Plaintext" "BirdSong"
    echo "───────────────────────────────────────────────────────────────────"
    printf "%-30s | %-20s | %-20s\n" "Node ID visible" "✓ YES" "✗ NO (family only)"
    printf "%-30s | %-20s | %-20s\n" "Capabilities visible" "✓ YES" "✗ NO (family only)"
    printf "%-30s | %-20s | %-20s\n" "Endpoints visible" "✓ YES" "✗ NO (family only)"
    printf "%-30s | %-20s | %-20s\n" "Topology mappable" "✓ YES" "✗ NO"
    printf "%-30s | %-20s | %-20s\n" "Encryption" "✗ NONE" "✓ AES-256-GCM"
    printf "%-30s | %-20s | %-20s\n" "Lineage-gated" "✗ NO" "✓ YES"
    printf "%-30s | %-20s | %-20s\n" "Suitable for internet" "✗ NO" "✓ YES"
    echo ""
    echo "🎵 **KEY INSIGHT:** BirdSong = Obvious to family, noise to others"
    echo ""
}

show_use_cases() {
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  When to Use Each Mode"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""
    echo "**Plaintext (No BearDog):**"
    echo "  ✓ Trusted university campus LAN"
    echo "  ✓ Research lab private network"
    echo "  ✓ Development/testing environment"
    echo "  ✓ Fast, simple, zero setup"
    echo ""
    echo "**BirdSong (With BearDog):**"
    echo "  ✓ Internet deployment"
    echo "  ✓ Public WiFi (coffee shop, airport)"
    echo "  ✓ Cellular networks (mobile devices)"
    echo "  ✓ Untrusted networks"
    echo "  ✓ Privacy-sensitive federations"
    echo "  ✓ Cross-organization collaboration"
    echo ""
    echo "**Transformation:**"
    echo "  Songbird alone = Trusted LAN orchestrator"
    echo "  Songbird + BearDog = True P2P platform"
    echo ""
}

# Main execution
case "$MODE" in
    --plaintext)
        show_plaintext
        ;;
    --birdsong)
        show_birdsong
        ;;
    --both)
        show_plaintext
        sleep 2
        show_birdsong
        sleep 2
        show_comparison
        sleep 2
        show_use_cases
        ;;
    *)
        echo "Usage: $0 [--plaintext|--birdsong|--both]"
        exit 1
        ;;
esac

echo "═══════════════════════════════════════════════════════════════════"
echo "  Demo Complete"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "**Status**: Conceptual demonstration (BearDog integration pending)"
echo "**Next**: Implement BearDog Phase 1 (Lineage Foundation)"
echo "**Timeline**: 14-20 weeks for full integration"
echo ""
echo "See: ../specs/SONGBIRD_BEARDOG_INTEGRATION.md"
echo ""

