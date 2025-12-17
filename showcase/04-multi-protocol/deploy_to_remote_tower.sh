#!/bin/bash
# Deploy Multi-Protocol Update to Remote Tower via Compute Bridge
#
# This script deploys the new multi-protocol features to a remote tower
# (like Strandgate) using Songbird's compute bridge.

set -e

# Configuration
REMOTE_TOWER="${REMOTE_TOWER:-strandgate}"
COMPUTE_BRIDGE="${COMPUTE_BRIDGE:-http://localhost:8080}"
DRY_RUN="${DRY_RUN:-false}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║     🚀 Deploy Multi-Protocol Update to Remote Tower 🚀        ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${BLUE}Target:${NC} $REMOTE_TOWER"
echo -e "${BLUE}Compute Bridge:${NC} $COMPUTE_BRIDGE"
echo -e "${BLUE}Dry Run:${NC} $DRY_RUN"
echo ""

# Step 1: Build the update
echo -e "${BLUE}[1/6]${NC} Building release binary..."
if [ "$DRY_RUN" = "false" ]; then
    cargo build --release --bin songbird-orchestrator
    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${YELLOW}⊘ Dry run - skipping build${NC}"
fi
echo ""

# Step 2: Check remote tower connectivity
echo -e "${BLUE}[2/6]${NC} Checking remote tower connectivity..."
if curl -s -f -m 5 "$COMPUTE_BRIDGE/health" > /dev/null 2>&1; then
    REMOTE_STATUS=$(curl -s "$COMPUTE_BRIDGE/health" | jq -r '.status')
    echo -e "${GREEN}✓ Remote tower is ${REMOTE_STATUS}${NC}"
else
    echo -e "${RED}✗ Cannot connect to remote tower at $COMPUTE_BRIDGE${NC}"
    echo "  Please check:"
    echo "  1. Tower is running"
    echo "  2. Compute bridge is accessible"
    echo "  3. COMPUTE_BRIDGE env var is correct"
    exit 1
fi
echo ""

# Step 3: Check current remote capabilities
echo -e "${BLUE}[3/6]${NC} Checking current remote capabilities..."
CURRENT_CAPS=$(curl -s "$COMPUTE_BRIDGE/api/protocol/capabilities" 2>/dev/null || echo "{}")

if echo "$CURRENT_CAPS" | jq -e '.protocols' > /dev/null 2>&1; then
    echo "Current protocols on remote:"
    echo "$CURRENT_CAPS" | jq -r '.protocols | keys[]' | while read proto; do
        echo -e "  • $proto"
    done
    
    HAS_TARPC=$(echo "$CURRENT_CAPS" | jq -r '.protocols | has("tarpc")')
    if [ "$HAS_TARPC" = "true" ]; then
        echo -e "${GREEN}✓ Remote already has multi-protocol support${NC}"
        echo "  Proceeding with update anyway..."
    else
        echo -e "${YELLOW}⚠ Remote does not have tarpc yet${NC}"
        echo "  This update will add multi-protocol support"
    fi
else
    echo -e "${YELLOW}⚠ Cannot determine current capabilities${NC}"
    echo "  Remote may be running older version"
fi
echo ""

# Step 4: Create deployment package
echo -e "${BLUE}[4/6]${NC} Creating deployment package..."
if [ "$DRY_RUN" = "false" ]; then
    DEPLOY_DIR="/tmp/songbird-deploy-$(date +%s)"
    mkdir -p "$DEPLOY_DIR"
    
    # Copy binary
    cp target/release/songbird-orchestrator "$DEPLOY_DIR/"
    
    # Create deployment manifest
    cat > "$DEPLOY_DIR/manifest.json" <<EOF
{
  "version": "$(cargo pkgid | cut -d'#' -f2)",
  "timestamp": "$(date -Iseconds)",
  "features": [
    "multi-protocol",
    "tarpc",
    "json-rpc",
    "btsp",
    "tls"
  ],
  "protocols": [
    "http",
    "https",
    "json-rpc",
    "tarpc",
    "websocket",
    "wss",
    "btsp"
  ]
}
EOF
    
    echo -e "${GREEN}✓ Package created at $DEPLOY_DIR${NC}"
    echo "  Manifest:"
    cat "$DEPLOY_DIR/manifest.json" | jq .
else
    echo -e "${YELLOW}⊘ Dry run - skipping package creation${NC}"
fi
echo ""

# Step 5: Deploy via compute bridge
echo -e "${BLUE}[5/6]${NC} Deploying to remote tower..."
if [ "$DRY_RUN" = "false" ]; then
    echo "Creating deployment workload..."
    
    # Use compute API to schedule deployment
    WORKLOAD=$(curl -s -X POST "$COMPUTE_BRIDGE/api/compute/submit" \
        -H "Content-Type: application/json" \
        -d "{
            \"task_type\": \"deployment\",
            \"payload\": {
                \"binary_path\": \"$DEPLOY_DIR/songbird-orchestrator\",
                \"manifest\": $(cat $DEPLOY_DIR/manifest.json),
                \"restart_strategy\": \"graceful\",
                \"backup_current\": true
            },
            \"priority\": \"high\"
        }")
    
    WORKLOAD_ID=$(echo "$WORKLOAD" | jq -r '.workload_id // .id // "unknown"')
    
    if [ "$WORKLOAD_ID" != "unknown" ] && [ "$WORKLOAD_ID" != "null" ]; then
        echo -e "${GREEN}✓ Deployment workload submitted: $WORKLOAD_ID${NC}"
        
        # Monitor deployment
        echo "Monitoring deployment progress..."
        for i in {1..30}; do
            sleep 2
            STATUS=$(curl -s "$COMPUTE_BRIDGE/api/compute/status/$WORKLOAD_ID" | jq -r '.status // "unknown"')
            
            if [ "$STATUS" = "completed" ]; then
                echo -e "${GREEN}✓ Deployment completed successfully!${NC}"
                break
            elif [ "$STATUS" = "failed" ]; then
                echo -e "${RED}✗ Deployment failed${NC}"
                exit 1
            else
                echo -e "  Status: ${YELLOW}$STATUS${NC} (${i}/30)"
            fi
        done
    else
        echo -e "${YELLOW}⚠ Deployment queued (workload tracking unavailable)${NC}"
        echo "  Waiting 10 seconds for deployment to complete..."
        sleep 10
    fi
else
    echo -e "${YELLOW}⊘ Dry run - would deploy via compute bridge${NC}"
fi
echo ""

# Step 6: Verify deployment
echo -e "${BLUE}[6/6]${NC} Verifying deployment..."
echo "Waiting for remote tower to restart (15 seconds)..."
sleep 15

# Check if tower is back up
for i in {1..10}; do
    if curl -s -f -m 5 "$COMPUTE_BRIDGE/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Remote tower is back online${NC}"
        break
    else
        echo -e "  Waiting for tower... (${i}/10)"
        sleep 3
    fi
done

# Verify new capabilities
NEW_CAPS=$(curl -s "$COMPUTE_BRIDGE/api/protocol/capabilities" 2>/dev/null || echo "{}")

if echo "$NEW_CAPS" | jq -e '.protocols' > /dev/null 2>&1; then
    echo ""
    echo "Updated protocols on remote:"
    echo "$NEW_CAPS" | jq -r '.protocols | keys[]' | while read proto; do
        echo -e "  ${GREEN}✓${NC} $proto"
    done
    
    # Check for new protocols
    HAS_TARPC=$(echo "$NEW_CAPS" | jq -r '.protocols | has("tarpc")')
    HAS_JSONRPC=$(echo "$NEW_CAPS" | jq -r '.protocols | has("json-rpc")')
    
    if [ "$HAS_TARPC" = "true" ] && [ "$HAS_JSONRPC" = "true" ]; then
        echo ""
        echo -e "${GREEN}✓✓✓ Multi-protocol deployment successful!${NC}"
    else
        echo ""
        echo -e "${YELLOW}⚠ Some protocols may not be enabled${NC}"
        echo "  Check remote tower configuration"
    fi
else
    echo -e "${RED}✗ Cannot verify new capabilities${NC}"
    echo "  Remote tower may not be responding correctly"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                 Deployment Complete!                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  1. Test protocol escalation:"
echo "     ./showcase/04-multi-protocol/test_remote_protocol_escalation.sh"
echo ""
echo "  2. Monitor remote tower:"
echo "     watch -n 2 'curl -s $COMPUTE_BRIDGE/api/protocol/capabilities | jq .'"
echo ""
echo "  3. Check logs on remote tower for any issues"
echo ""

# Cleanup
if [ "$DRY_RUN" = "false" ] && [ -n "$DEPLOY_DIR" ]; then
    rm -rf "$DEPLOY_DIR"
fi

