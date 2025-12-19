#!/bin/bash
# Deploy Binary via Songbird Deployment API
# Uses the actual /api/deployment endpoints

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
TARGET_TOWER="${1:-https://192.168.1.134:8081}"
BINARY_PATH="${2:-./simple_toadstool}"
SERVICE_NAME="${3:-toadstool}"
AUTO_START="${4:-true}"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║           🚀 SONGBIRD BINARY DEPLOYMENT 🚀                       ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${CYAN}Target Tower:${NC}  $TARGET_TOWER"
echo -e "${CYAN}Binary:${NC}        $BINARY_PATH"
echo -e "${CYAN}Service Name:${NC}  $SERVICE_NAME"
echo -e "${CYAN}Auto Start:${NC}    $AUTO_START"
echo ""

# Validate binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}❌ Binary not found: $BINARY_PATH${NC}"
    exit 1
fi

BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
echo -e "${GREEN}✅ Binary found (size: $BINARY_SIZE)${NC}"
echo ""

# Step 1: Check capabilities
echo -e "${BLUE}[1/4]${NC} Checking target tower capabilities..."
CAPABILITIES=$(curl -k -s "$TARGET_TOWER/api/deployment/capabilities")

if echo "$CAPABILITIES" | jq . > /dev/null 2>&1; then
    AVAILABLE_GB=$(echo "$CAPABILITIES" | jq -r '.resources.available_storage_gb')
    CPU_CORES=$(echo "$CAPABILITIES" | jq -r '.resources.cpu_cores')
    MEMORY_GB=$(echo "$CAPABILITIES" | jq -r '.resources.available_memory_gb')
    PREFERRED_METHOD=$(echo "$CAPABILITIES" | jq -r '.preferences.preferred_method')
    
    echo -e "${GREEN}✅ Tower capabilities retrieved:${NC}"
    echo "   Storage: ${AVAILABLE_GB} GB"
    echo "   CPU: ${CPU_CORES} cores"
    echo "   Memory: ${MEMORY_GB} GB"
    echo "   Preferred method: ${PREFERRED_METHOD}"
else
    echo -e "${YELLOW}⚠️  Could not retrieve capabilities${NC}"
    echo "$CAPABILITIES"
    exit 1
fi
echo ""

# Step 2: Deploy binary
echo -e "${BLUE}[2/4]${NC} Deploying binary via single upload..."

# Create temp env file
ENV_FILE="/tmp/env_${SERVICE_NAME}.json"
cat > "$ENV_FILE" << EOF
{
  "RUST_LOG": "info",
  "SERVICE_NAME": "$SERVICE_NAME",
  "PORT": "7878"
}
EOF

# Deploy using multipart form
DEPLOY_RESPONSE=$(curl -k -s -X POST "$TARGET_TOWER/api/deployment/binary" \
  -F "binary=@$BINARY_PATH" \
  -F "service_name=$SERVICE_NAME" \
  -F "env_vars=$(cat $ENV_FILE)" \
  -F "auto_start=$AUTO_START")

rm "$ENV_FILE"

if echo "$DEPLOY_RESPONSE" | jq . > /dev/null 2>&1; then
    DEPLOYMENT_ID=$(echo "$DEPLOY_RESPONSE" | jq -r '.deployment_id')
    STATUS=$(echo "$DEPLOY_RESPONSE" | jq -r '.status')
    MESSAGE=$(echo "$DEPLOY_RESPONSE" | jq -r '.message')
    
    echo -e "${GREEN}✅ Deployment initiated:${NC}"
    echo "   ID: $DEPLOYMENT_ID"
    echo "   Status: $STATUS"
    echo "   Message: $MESSAGE"
    
    if [ "$STATUS" = "deployed" ] || [ "$STATUS" = "running" ]; then
        echo -e "${GREEN}✅ Binary deployed successfully!${NC}"
    else
        echo -e "${YELLOW}⚠️  Deployment status: $STATUS${NC}"
    fi
else
    echo -e "${YELLOW}❌ Deployment failed${NC}"
    echo "$DEPLOY_RESPONSE"
    exit 1
fi
echo ""

# Step 3: Check deployment status
echo -e "${BLUE}[3/4]${NC} Checking deployment status..."
sleep 2

STATUS_RESPONSE=$(curl -k -s "$TARGET_TOWER/api/deployment/status/$DEPLOYMENT_ID")

if echo "$STATUS_RESPONSE" | jq . > /dev/null 2>&1; then
    CURRENT_STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')
    PID=$(echo "$STATUS_RESPONSE" | jq -r '.pid // "N/A"')
    PORT=$(echo "$STATUS_RESPONSE" | jq -r '.port // "N/A"')
    
    echo -e "${GREEN}Deployment status:${NC}"
    echo "   Status: $CURRENT_STATUS"
    echo "   PID: $PID"
    echo "   Port: $PORT"
else
    echo -e "${YELLOW}Could not get status${NC}"
fi
echo ""

# Step 4: Verify service
echo -e "${BLUE}[4/4]${NC} Verifying service..."

# Extract host from target tower URL
TARGET_HOST=$(echo "$TARGET_TOWER" | sed -E 's|https?://([^:/]+).*|\1|')

# Try to connect to service
sleep 3
echo "Testing connection to service..."

if curl -s --connect-timeout 3 "http://$TARGET_HOST:7878/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Service is responding!${NC}"
    curl -s "http://$TARGET_HOST:7878/health" | jq '.'
else
    echo -e "${YELLOW}⚠️  Service not responding yet (may need more time)${NC}"
    echo "   Try: curl http://$TARGET_HOST:7878/health"
fi
echo ""

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║               ✨ DEPLOYMENT COMPLETE ✨                          ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Deployment ID: $DEPLOYMENT_ID"
echo "Service endpoint: http://$TARGET_HOST:7878"
echo ""
echo "To check status:"
echo "  curl -k $TARGET_TOWER/api/deployment/status/$DEPLOYMENT_ID | jq ."
echo ""
echo "To stop deployment:"
echo "  curl -k -X DELETE $TARGET_TOWER/api/deployment/$DEPLOYMENT_ID"
echo ""

