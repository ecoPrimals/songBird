#!/bin/bash
# Quick Method Wiring Validation
# Tests that all 6 Dark Forest methods are accessible via JSON-RPC
# This validates the wiring fix without needing full STUN connectivity

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if socket path provided or use default
SOCKET="${1:-/primal/songbird}"

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║      🔍 Dark Forest Method Wiring Validation 🔍               ║"
echo "║                                                                ║"
echo "║      Testing that all 6 methods are accessible                ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if socket exists
if [ ! -S "$SOCKET" ]; then
    echo -e "${RED}❌ Socket not found: $SOCKET${NC}"
    echo ""
    echo "Please start Songbird first:"
    echo "  ./songbird server"
    echo ""
    echo "Or specify a custom socket:"
    echo "  $0 /path/to/socket.sock"
    exit 1
fi

echo -e "${GREEN}✅ Found socket: $SOCKET${NC}"
echo ""

PASSED=0
FAILED=0
UNKNOWN=0

test_method() {
    local method=$1
    local params=$2
    local name=$3
    
    echo -e "${CYAN}Testing: ${name}${NC}"
    
    local result=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" | nc -U "$SOCKET" -N 2>/dev/null)
    
    if echo "$result" | grep -q '"error".*"Unknown method"'; then
        echo -e "  ${RED}❌ FAILED - Method not wired!${NC}"
        echo "  Response: $result"
        ((FAILED++))
        return 1
    elif echo "$result" | grep -q '"error"'; then
        # Method exists but returned an error (this is OK - means it's wired)
        echo -e "  ${YELLOW}✅ WIRED (returned expected error)${NC}"
        echo "  Response: $(echo "$result" | jq -c '.' 2>/dev/null || echo "$result")"
        ((PASSED++))
        return 0
    elif echo "$result" | grep -q '"result"'; then
        # Method exists and returned success
        echo -e "  ${GREEN}✅ WIRED (returned result)${NC}"
        echo "  Response: $(echo "$result" | jq -c '.' 2>/dev/null || echo "$result")"
        ((PASSED++))
        return 0
    else
        echo -e "  ${YELLOW}⚠️  UNKNOWN - Unexpected response${NC}"
        echo "  Response: $result"
        ((UNKNOWN++))
        return 2
    fi
}

echo "═══════════════════════════════════════════════════════════════"
echo "Testing Dark Forest Methods (6 total)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Test 1: stun.get_public_address
test_method "stun.get_public_address" '{"server":"stun.nextcloud.com:3478"}' "stun.get_public_address"
echo ""

# Test 2: stun.bind
test_method "stun.bind" '{"server":"stun.nextcloud.com:3478","local_port":54321}' "stun.bind"
echo ""

# Test 3: discovery.peers
test_method "discovery.peers" '{}' "discovery.peers"
echo ""

# Test 4: rendezvous.register
test_method "rendezvous.register" '{"server":"http://test.com","node_id":"test","family_id":"nat0","public_address":"1.2.3.4:5678"}' "rendezvous.register"
echo ""

# Test 5: rendezvous.lookup
test_method "rendezvous.lookup" '{"server":"http://test.com","target":"test-node"}' "rendezvous.lookup"
echo ""

# Test 6: peer.connect
test_method "peer.connect" '{"target_address":"1.2.3.4:5678"}' "peer.connect"
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "Test Summary"
echo "═══════════════════════════════════════════════════════════════"
echo ""

echo "Total Methods: 6"
echo -e "  ${GREEN}Passed:  $PASSED${NC}"
echo -e "  ${RED}Failed:  $FAILED${NC}"
echo -e "  ${YELLOW}Unknown: $UNKNOWN${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo -e "║             ${GREEN}🎉 ALL METHODS WIRED! 🎉${NC}                       ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${GREEN}✅ Wiring fix confirmed!${NC}"
    echo ""
    echo "All 6 Dark Forest methods are accessible:"
    echo "  ✅ stun.get_public_address"
    echo "  ✅ stun.bind"
    echo "  ✅ discovery.peers"
    echo "  ✅ rendezvous.register"
    echo "  ✅ rendezvous.lookup"
    echo "  ✅ peer.connect"
    echo ""
    echo -e "${CYAN}→ Ready for STUN handshake validation!${NC}"
    exit 0
else
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║                                                                ║"
    echo -e "║             ${RED}❌ WIRING INCOMPLETE! ❌${NC}                       ║"
    echo "║                                                                ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${RED}Some methods are not wired correctly!${NC}"
    echo ""
    echo "This means bin_interface.rs is not using IpcServiceHandler."
    echo "Expected: IpcServiceHandler (all 6 methods)"
    echo "Got: HttpHandler (only http.* methods)"
    echo ""
    echo "Fix: Review the wiring fix in bin_interface.rs"
    exit 1
fi

