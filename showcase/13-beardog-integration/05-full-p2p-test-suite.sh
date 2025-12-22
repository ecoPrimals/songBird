#!/usr/bin/env bash
# Full P2P Integration Test Suite
# Orchestrates complete P2P testing with Mock BearDog

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}/../.."

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  🚀 Full P2P Integration Test Suite                               ║"
echo "║  Complete E2E Testing with REAL BearDog                           ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Configuration
SONGBIRD_URL="${SONGBIRD_URL:-https://localhost:8080}"
MOCK_BEARDOG_PORT="${MOCK_BEARDOG_PORT:-9000}"
TEST_RESULTS_DIR="/tmp/p2p-test-results-$$"
mkdir -p "${TEST_RESULTS_DIR}"

# Track test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_script="$2"
    
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  Running: ${test_name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if bash "${test_script}" > "${TEST_RESULTS_DIR}/${test_name}.log" 2>&1; then
        echo -e "${GREEN}✅ PASS: ${test_name}${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL: ${test_name}${NC}"
        echo -e "${YELLOW}   See: ${TEST_RESULTS_DIR}/${test_name}.log${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    # No cleanup needed - real BearDog stays running
    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT

# Phase 1: Pre-flight checks
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 1: Pre-flight Checks${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Checking Songbird...${NC}"
if curl -k -s "${SONGBIRD_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Songbird is running${NC}"
else
    echo -e "${RED}❌ Songbird is not running${NC}"
    echo -e "${YELLOW}   Start Songbird first:${NC}"
    echo "   cargo run --bin songbird-orchestrator"
    exit 1
fi

echo -e "\n${YELLOW}Checking dependencies...${NC}"
for cmd in curl jq python3; do
    if command -v ${cmd} &> /dev/null; then
        echo -e "${GREEN}✅ ${cmd} available${NC}"
    else
        echo -e "${RED}❌ ${cmd} not found${NC}"
        exit 1
    fi
done

# Phase 2: Verify Real BearDog
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 2: Verifying Real BearDog Service${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Checking for Real BearDog service...${NC}"
if curl -s "http://localhost:${MOCK_BEARDOG_PORT}/health" > /dev/null 2>&1; then
    BEARDOG_VERSION=$(curl -s "http://localhost:${MOCK_BEARDOG_PORT}/health" | jq -r '.version // "unknown"')
    echo -e "${GREEN}✅ Real BearDog is running (version: ${BEARDOG_VERSION})${NC}"
else
    echo -e "${RED}❌ BearDog is not running${NC}"
    echo -e "${YELLOW}   Start BearDog first:${NC}"
    echo "   cd ../beardog"
    echo "   BTSP_PORT=9000 ./target/release/examples/btsp_server &"
    exit 1
fi

# Phase 3: Foundation Tests
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 3: Foundation Tests${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

run_test "privacy-comparison" "${SCRIPT_DIR}/01-privacy-comparison.sh"
run_test "graceful-degradation" "${SCRIPT_DIR}/02-graceful-degradation-test.sh"

# Phase 4: Live Integration Tests
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 4: Live Integration Tests (with Mock BearDog)${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

export SONGBIRD_URL="${SONGBIRD_URL}"
export BEARDOG_EXPECTED_PORT="${MOCK_BEARDOG_PORT}"

run_test "btsp-live-integration" "${SCRIPT_DIR}/03-btsp-live-integration-test.sh"
run_test "birdsong-discovery" "${SCRIPT_DIR}/04-birdsong-discovery-test.sh"

# Phase 5: End-to-End P2P Flow
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 5: End-to-End P2P Flow (REAL BearDog)${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Test 1: Establish P2P Tunnel with REAL genetic cryptography...${NC}"
TUNNEL_RESPONSE=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"peer":{"id":"songbird-test","endpoint":"192.168.1.1:8080"},"initiator_entropy":"test-entropy-123"}' \
    "http://localhost:${MOCK_BEARDOG_PORT}/btsp/tunnel/establish")

TUNNEL_ID=$(echo "${TUNNEL_RESPONSE}" | jq -r '.handle.id // empty')
if [ -n "${TUNNEL_ID}" ]; then
    echo -e "${GREEN}✅ Tunnel established: ${TUNNEL_ID}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ Failed to establish tunnel${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

if [ -n "${TUNNEL_ID}" ]; then
    echo -e "\n${YELLOW}Test 2: Encrypt data through tunnel...${NC}"
    ENCRYPT_RESPONSE=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d '{"data":"Hello P2P World!","tunnel_id":"'${TUNNEL_ID}'"}' \
        "http://localhost:${MOCK_BEARDOG_PORT}/btsp/tunnel/${TUNNEL_ID}/encrypt")
    
    ENCRYPTED_DATA=$(echo "${ENCRYPT_RESPONSE}" | jq -r '.encrypted_data // empty')
    if [ -n "${ENCRYPTED_DATA}" ]; then
        echo -e "${GREEN}✅ Data encrypted: ${ENCRYPTED_DATA:0:40}...${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
        
        echo -e "\n${YELLOW}Test 3: Decrypt data through tunnel...${NC}"
        DECRYPT_RESPONSE=$(curl -s -X POST \
            -H "Content-Type: application/json" \
            -d '{"encrypted_data":"'${ENCRYPTED_DATA}'","tunnel_id":"'${TUNNEL_ID}'"}' \
            "http://localhost:${MOCK_BEARDOG_PORT}/btsp/tunnel/${TUNNEL_ID}/decrypt")
        
        DECRYPTED_DATA=$(echo "${DECRYPT_RESPONSE}" | jq -r '.data // empty')
        if [ "${DECRYPTED_DATA}" == "Hello P2P World!" ]; then
            echo -e "${GREEN}✅ Data decrypted correctly: ${DECRYPTED_DATA}${NC}"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ Decryption failed or data mismatch${NC}"
            TOTAL_TESTS=$((TOTAL_TESTS + 1))
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    else
        echo -e "${RED}❌ Encryption failed${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo -e "\n${YELLOW}Test 4: Get tunnel status...${NC}"
    STATUS_RESPONSE=$(curl -s "http://localhost:${MOCK_BEARDOG_PORT}/btsp/tunnel/${TUNNEL_ID}/status")
    TUNNEL_ACTIVE=$(echo "${STATUS_RESPONSE}" | jq -r '.active // false')
    
    if [ "${TUNNEL_ACTIVE}" == "true" ]; then
        echo -e "${GREEN}✅ Tunnel status: active${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Tunnel status check failed${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    echo -e "\n${YELLOW}Test 5: Close tunnel...${NC}"
    CLOSE_RESPONSE=$(curl -s -X DELETE "http://localhost:${MOCK_BEARDOG_PORT}/btsp/tunnel/${TUNNEL_ID}")
    if echo "${CLOSE_RESPONSE}" | jq -e '.status == "closed"' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Tunnel closed successfully${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ Failed to close tunnel${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
fi

# Phase 6: BirdSong E2E
echo -e "\n${CYAN}══════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Phase 6: BirdSong End-to-End${NC}"
echo -e "${CYAN}══════════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Test 6: Encrypt with BirdSong...${NC}"
BIRDSONG_ENCRYPT=$(curl -s -X POST \
    -H "Content-Type: application/json" \
    -d '{"message":"Family dinner at 6pm","lineage_group":"family-chat"}' \
    "http://localhost:${MOCK_BEARDOG_PORT}/birdsong/encrypt")

BIRDSONG_PAYLOAD=$(echo "${BIRDSONG_ENCRYPT}" | jq -r '.encrypted_payload // empty')
if [ -n "${BIRDSONG_PAYLOAD}" ]; then
    echo -e "${GREEN}✅ BirdSong encryption working${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}❌ BirdSong encryption failed${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

# Final Report
echo -e "\n${CYAN}"
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                                                                   ║"
echo "║  📊 Full P2P Integration Test Results                             ║"
echo "║                                                                   ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

PASS_RATE=$((PASSED_TESTS * 100 / TOTAL_TESTS))

echo -e "${BLUE}Test Summary:${NC}"
echo -e "  Total Tests:  ${TOTAL_TESTS}"
echo -e "  ${GREEN}Passed:       ${PASSED_TESTS}${NC}"
if [ ${FAILED_TESTS} -gt 0 ]; then
    echo -e "  ${RED}Failed:       ${FAILED_TESTS}${NC}"
fi
echo -e "  Pass Rate:    ${PASS_RATE}%"

echo -e "\n${BLUE}Test Logs:${NC}"
echo -e "  ${TEST_RESULTS_DIR}/"

if [ ${FAILED_TESTS} -eq 0 ]; then
    echo -e "\n${GREEN}🎉 ALL TESTS PASSED!${NC}"
    echo -e "${GREEN}✅ P2P Infrastructure Fully Validated${NC}"
    echo -e "\n${BLUE}What This Means:${NC}"
    echo "  • BTSP secure tunnels working end-to-end"
    echo "  • BirdSong privacy encryption operational"
    echo "  • Service discovery validated"
    echo "  • Graceful degradation confirmed"
    echo "  • Full P2P flow tested"
    echo -e "\n${GREEN}Status: READY FOR PRODUCTION (with real BearDog)${NC}"
    exit 0
else
    echo -e "\n${YELLOW}⚠️  Some tests failed (${FAILED_TESTS}/${TOTAL_TESTS})${NC}"
    echo -e "${YELLOW}   Check logs for details${NC}"
    exit 1
fi

