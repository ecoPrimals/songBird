#!/bin/bash
# Deploy TLS-by-default update to Strandgate using Songbird's deployment API

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

STRANDGATE_IP="${STRANDGATE_IP:-192.168.1.134}"
STRANDGATE_PORT="${STRANDGATE_PORT:-8081}"
STRANDGATE_URL="http://$STRANDGATE_IP:$STRANDGATE_PORT"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║   📦 DEPLOYING TLS-BY-DEFAULT UPDATE TO STRANDGATE 📦           ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Check if Strandgate is reachable
echo -e "${BLUE}[1/7]${NC} Checking Strandgate connectivity..."
if ! curl -s -f "$STRANDGATE_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Strandgate not reachable at $STRANDGATE_URL${NC}"
    echo ""
    echo "Please ensure:"
    echo "  1. Strandgate is running"
    echo "  2. IP/Port is correct: $STRANDGATE_IP:$STRANDGATE_PORT"
    echo "  3. Set SONGBIRD_TLS_ENABLED=false on Strandgate temporarily"
    echo ""
    exit 1
fi
echo -e "${GREEN}✓ Strandgate is reachable${NC}"
echo ""

# Step 2: Check deployment capabilities
echo -e "${BLUE}[2/7]${NC} Checking Strandgate deployment capabilities..."
CAPS=$(curl -s "$STRANDGATE_URL/api/deployment/capabilities")
if [ -z "$CAPS" ]; then
    echo -e "${RED}❌ Could not fetch deployment capabilities${NC}"
    exit 1
fi

AVAILABLE_STORAGE=$(echo "$CAPS" | jq -r '.resources.available_storage_gb')
MAX_SIZE=$(echo "$CAPS" | jq -r '.deployment_methods.single.max_size_mb')

echo -e "${GREEN}✓ Deployment capabilities:${NC}"
echo "  Available storage: ${AVAILABLE_STORAGE}GB"
echo "  Max single upload: ${MAX_SIZE}MB"
echo ""

# Step 3: Build latest Songbird orchestrator
echo -e "${BLUE}[3/7]${NC} Building latest Songbird orchestrator with TLS-by-default..."
cd "$(dirname "$0")/../.."
if cargo build --release --bin songbird-orchestrator 2>&1 | tail -3; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

BINARY_PATH="target/release/songbird-orchestrator"
BINARY_SIZE=$(du -m "$BINARY_PATH" | cut -f1)
echo "  Binary size: ${BINARY_SIZE}MB"
echo ""

# Check if binary fits in single upload
if [ "$BINARY_SIZE" -gt "$MAX_SIZE" ]; then
    echo -e "${YELLOW}⚠ Binary too large for single upload (${BINARY_SIZE}MB > ${MAX_SIZE}MB)${NC}"
    echo "  Chunked upload would be needed (not implemented in this script yet)"
    exit 1
fi

# Step 4: Stop old Songbird on Strandgate
echo -e "${BLUE}[4/7]${NC} Stopping old Songbird on Strandgate..."
# First, get list of deployments to find the old Songbird
DEPLOYMENTS=$(curl -s "$STRANDGATE_URL/api/deployment/list")
OLD_DEPLOYMENT_ID=$(echo "$DEPLOYMENTS" | jq -r '.[] | select(.service_name == "songbird-orchestrator") | .deployment_id' | head -1)

if [ -n "$OLD_DEPLOYMENT_ID" ] && [ "$OLD_DEPLOYMENT_ID" != "null" ]; then
    echo "  Found old deployment: $OLD_DEPLOYMENT_ID"
    curl -s -X DELETE "$STRANDGATE_URL/api/deployment/$OLD_DEPLOYMENT_ID" > /dev/null
    echo -e "${GREEN}✓ Old Songbird stopped${NC}"
    sleep 2
else
    echo -e "${YELLOW}ℹ No existing deployment found (will be fresh install)${NC}"
fi
echo ""

# Step 5: Upload new binary
echo -e "${BLUE}[5/7]${NC} Uploading new Songbird binary to Strandgate..."
DEPLOY_RESPONSE=$(curl -s -X POST "$STRANDGATE_URL/api/deployment/binary" \
  -F "binary=@$BINARY_PATH" \
  -F "service_name=songbird-orchestrator" \
  -F "auto_start=true" \
  -F 'env_vars={"SONGBIRD_HTTP_PORT":"8081","SONGBIRD_TLS_ENABLED":"true","SONGBIRD_NODE_ID":"tower-b-strandgate","RUST_LOG":"info"}')

DEPLOYMENT_ID=$(echo "$DEPLOY_RESPONSE" | jq -r '.deployment_id')
DEPLOY_STATUS=$(echo "$DEPLOY_RESPONSE" | jq -r '.status')

if [ "$DEPLOYMENT_ID" = "null" ] || [ -z "$DEPLOYMENT_ID" ]; then
    echo -e "${RED}❌ Deployment failed${NC}"
    echo "$DEPLOY_RESPONSE" | jq .
    exit 1
fi

echo -e "${GREEN}✓ Binary uploaded successfully${NC}"
echo "  Deployment ID: $DEPLOYMENT_ID"
echo "  Status: $DEPLOY_STATUS"
echo ""

# Step 6: Wait for service to start and check status
echo -e "${BLUE}[6/7]${NC} Waiting for new Songbird to start..."
sleep 5

for i in {1..10}; do
    STATUS=$(curl -s "$STRANDGATE_URL/api/deployment/status/$DEPLOYMENT_ID" | jq -r '.status')
    
    if [ "$STATUS" = "running" ]; then
        echo -e "${GREEN}✓ Songbird is running!${NC}"
        break
    elif [ "$STATUS" = "failed" ]; then
        echo -e "${RED}❌ Deployment failed${NC}"
        curl -s "$STRANDGATE_URL/api/deployment/status/$DEPLOYMENT_ID" | jq .
        exit 1
    else
        echo "  Status: $STATUS... (${i}/10)"
        sleep 2
    fi
done
echo ""

# Step 7: Test TLS connection
echo -e "${BLUE}[7/7]${NC} Testing TLS connection to updated Strandgate..."
echo "  Old endpoint: http://$STRANDGATE_IP:8081"
echo "  New endpoint: https://$STRANDGATE_IP:8443 (with TLS)"
echo ""

# Give it a moment for TLS to initialize
sleep 3

# Try HTTPS connection (allow self-signed cert)
if curl -k -s -f "https://$STRANDGATE_IP:8443/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓✓✓ TLS connection successful!${NC}"
    echo ""
    
    # Check protocol capabilities
    echo "Checking new protocol capabilities..."
    CAPS=$(curl -k -s "https://$STRANDGATE_IP:8443/api/protocol/capabilities")
    echo "$CAPS" | jq -r '.protocols | keys[] | "  ✓ " + .'
    echo ""
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${GREEN}✅ DEPLOYMENT COMPLETE!${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Strandgate Tower B is now running with:"
    echo "  ✅ TLS enabled by default (fail-secure)"
    echo "  ✅ HTTPS on port 8443"
    echo "  ✅ Multi-protocol support (HTTP, JSON-RPC, tarpc)"
    echo "  ✅ Auto-generated TLS certificates"
    echo ""
    echo "Access Strandgate via:"
    echo "  https://$STRANDGATE_IP:8443/health"
    echo "  curl -k https://$STRANDGATE_IP:8443/api/protocol/capabilities"
    echo ""
    echo "Next: Reconnect Tower A to Tower B with TLS federation!"
    echo ""
elif curl -s -f "http://$STRANDGATE_IP:8081/health" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Still running on HTTP (TLS might not have initialized)${NC}"
    echo "  Check logs or try manual restart with:"
    echo "  export SONGBIRD_TLS_ENABLED=true"
    echo ""
else
    echo -e "${RED}❌ Cannot connect to Strandgate on either HTTP or HTTPS${NC}"
    echo "  Check deployment status:"
    echo "  curl http://$STRANDGATE_IP:8081/api/deployment/status/$DEPLOYMENT_ID"
    echo ""
fi

