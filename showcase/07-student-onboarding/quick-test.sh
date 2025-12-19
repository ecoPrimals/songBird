#!/bin/bash
# Quick Test Script - Verify Songbird is Ready
# Run this before deploying to Windows

set -e

echo "🎵 Songbird Quick Test Script"
echo "=============================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Must run from songbird root directory${NC}"
    exit 1
fi

echo "1️⃣  Running integration tests..."
if cargo test --package songbird-orchestrator --test orchestrator_integration_tests --quiet; then
    echo -e "${GREEN}   ✅ All tests passing (10/10)${NC}"
else
    echo -e "${RED}   ❌ Tests failed${NC}"
    exit 1
fi
echo ""

echo "2️⃣  Checking release binary..."
if [ -f "target/release/songbird-orchestrator" ]; then
    SIZE=$(du -h target/release/songbird-orchestrator | cut -f1)
    echo -e "${GREEN}   ✅ Binary exists: $SIZE${NC}"
else
    echo -e "${YELLOW}   ⚠️  Binary not found, building...${NC}"
    cargo build --release --bin songbird-orchestrator
    SIZE=$(du -h target/release/songbird-orchestrator | cut -f1)
    echo -e "${GREEN}   ✅ Binary built: $SIZE${NC}"
fi
echo ""

echo "3️⃣  Checking federation connectivity..."
if curl -s http://192.168.1.144:8000/api/federation/registry > /dev/null 2>&1; then
    NODE_COUNT=$(curl -s http://192.168.1.144:8000/api/federation/registry | grep -o '"node_id"' | wc -l)
    echo -e "${GREEN}   ✅ Registry reachable ($NODE_COUNT nodes)${NC}"
else
    echo -e "${YELLOW}   ⚠️  Registry not reachable (is Eastgate running?)${NC}"
fi
echo ""

echo "4️⃣  Checking config file..."
CONFIG="showcase/07-student-onboarding/config/local-network.toml"
if [ -f "$CONFIG" ]; then
    echo -e "${GREEN}   ✅ Config exists${NC}"
    echo -e "${GREEN}      Registry: $(grep registry_url $CONFIG | cut -d'"' -f2)${NC}"
else
    echo -e "${RED}   ❌ Config not found: $CONFIG${NC}"
    exit 1
fi
echo ""

echo "5️⃣  Checking Python client..."
CLIENT_DIR="showcase/07-student-onboarding/client"
if [ -f "$CLIENT_DIR/setup.py" ]; then
    echo -e "${GREEN}   ✅ Client exists${NC}"
else
    echo -e "${RED}   ❌ Client not found${NC}"
    exit 1
fi
echo ""

echo "=============================="
echo -e "${GREEN}✅ All checks passed!${NC}"
echo ""
echo "Next steps:"
echo "1. Test local deployment:"
echo "   ./target/release/songbird-orchestrator --config $CONFIG"
echo ""
echo "2. Deploy to Windows:"
echo "   See: showcase/07-student-onboarding/DEPLOYMENT_GUIDE.md"
echo ""
echo "3. Test student client:"
echo "   See: showcase/07-student-onboarding/TESTING_CHECKLIST.md"
echo ""

