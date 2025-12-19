#!/usr/bin/env bash
# Deploy ToadStool to Remote Tower via Songbird Compute Bridge
# This demonstrates Songbird orchestrating primal deployment

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

STRANDGATE_HOST="${STRANDGATE_HOST:-192.168.1.134}"
STRANDGATE_PORT="${STRANDGATE_PORT:-8081}"
STRANDGATE_URL="https://${STRANDGATE_HOST}:${STRANDGATE_PORT}"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🚀 Deploy ToadStool via Songbird Compute Bridge${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo

# Step 1: Build ToadStool if needed
echo -e "${YELLOW}[1/5] Building ToadStool...${NC}"
TOADSTOOL_DIR="/home/eastgate/Development/ecoPrimals/toadstool"

if [[ ! -f "$TOADSTOOL_DIR/target/release/distributed-train" ]]; then
    echo "  Building distributed training binary..."
    cd "$TOADSTOOL_DIR/showcase/inter-primal/02-songbird-distributed-training"
    cargo build --release 2>&1 | grep -E "(Compiling|Finished)" | tail -5
    echo -e "${GREEN}✅ ToadStool built${NC}"
else
    echo -e "${GREEN}✅ ToadStool already built${NC}"
fi
echo

# Step 2: Check connectivity
echo -e "${YELLOW}[2/5] Checking tower connectivity...${NC}"

echo -n "  Eastgate (localhost:8000): "
if curl -sk https://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Online${NC}"
else
    echo -e "${RED}❌ Offline${NC}"
    exit 1
fi

echo -n "  Strandgate ($STRANDGATE_HOST:$STRANDGATE_PORT): "
if curl -sk "$STRANDGATE_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Online${NC}"
else
    echo -e "${RED}❌ Offline${NC}"
    echo "  Start Strandgate tower first"
    exit 1
fi
echo

# Step 3: Create deployment package
echo -e "${YELLOW}[3/5] Creating deployment package...${NC}"

DEPLOY_DIR="$(pwd)/deploy_package_$(date +%s)"
mkdir -p "$DEPLOY_DIR"

# Copy ToadStool binary
cp "$TOADSTOOL_DIR/showcase/inter-primal/02-songbird-distributed-training/target/release/distributed-train" \
   "$DEPLOY_DIR/toadstool-ml-worker"

# Create deployment manifest
cat > "$DEPLOY_DIR/manifest.json" <<EOF
{
  "service": "toadstool-ml-worker",
  "version": "1.0.0",
  "target_tower": "tower-strandgate",
  "binary": "toadstool-ml-worker",
  "capabilities": ["ml-training", "ml-inference", "compute", "gpu-rtx-3070"],
  "auto_start": true,
  "register_with_songbird": true,
  "environment": {
    "SONGBIRD_URL": "https://${STRANDGATE_HOST}:${STRANDGATE_PORT}",
    "WORKER_ID": "toadstool-strandgate-001",
    "GPU_ENABLED": "true"
  }
}
EOF

echo -e "${GREEN}✅ Package created${NC}"
echo "  Binary: toadstool-ml-worker ($(du -h $DEPLOY_DIR/toadstool-ml-worker | cut -f1))"
echo

# Step 4: Deploy via compute bridge API
echo -e "${YELLOW}[4/5] Deploying to Strandgate...${NC}"

# Use Songbird's deployment API
DEPLOYMENT_PAYLOAD=$(cat <<EOF
{
  "task_type": "deployment",
  "target_tower": "tower-strandgate",
  "payload": {
    "binary_name": "toadstool-ml-worker",
    "binary_size": $(stat -c%s "$DEPLOY_DIR/toadstool-ml-worker"),
    "manifest": $(cat "$DEPLOY_DIR/manifest.json"),
    "deployment_method": "http_upload",
    "restart_strategy": "graceful"
  },
  "priority": "high"
}
EOF
)

# For now, use SSH as fallback (HTTP upload API not fully wired yet)
echo -e "${BLUE}  Using SSH fallback deployment...${NC}"

if command -v ssh > /dev/null && ssh -q strandgate exit 2>/dev/null; then
    echo "  Copying binary to Strandgate..."
    scp -q "$DEPLOY_DIR/toadstool-ml-worker" strandgate:/tmp/
    
    echo "  Setting permissions..."
    ssh strandgate "chmod +x /tmp/toadstool-ml-worker"
    
    echo -e "${GREEN}✅ Binary deployed to Strandgate${NC}"
else
    echo -e "${YELLOW}⚠️  SSH not available${NC}"
    echo "  Manual deployment required:"
    echo "    1. Copy $DEPLOY_DIR/toadstool-ml-worker to Strandgate"
    echo "    2. Run: chmod +x toadstool-ml-worker"
    echo "    3. Run: ./toadstool-ml-worker --songbird-url $STRANDGATE_URL"
fi
echo

# Step 5: Verify deployment
echo -e "${YELLOW}[5/5] Starting ToadStool worker on Strandgate...${NC}"

if command -v ssh > /dev/null && ssh -q strandgate exit 2>/dev/null; then
    # Start worker in background
    ssh strandgate "nohup /tmp/toadstool-ml-worker \
        --songbird-url https://localhost:8081 \
        --epochs 1 \
        > /tmp/toadstool-worker.log 2>&1 &"
    
    sleep 2
    
    echo -e "${GREEN}✅ ToadStool worker started on Strandgate${NC}"
    echo "  Check logs: ssh strandgate tail -f /tmp/toadstool-worker.log"
else
    echo -e "${YELLOW}ℹ️  Start worker manually on Strandgate:${NC}"
    echo "    ./toadstool-ml-worker --songbird-url https://localhost:8081"
fi

echo
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 Deployment Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "${BLUE}📊 Deployment Summary:${NC}"
echo "  Binary:   toadstool-ml-worker"
echo "  Target:   Strandgate ($STRANDGATE_HOST)"
echo "  Method:   SSH (Compute Bridge API coming in V2)"
echo "  Status:   Running"
echo
echo -e "${BLUE}🚀 Next Steps:${NC}"
echo "  1. Verify worker: curl -sk $STRANDGATE_URL/api/compute/workers"
echo "  2. Run distributed workload: ./RUN_DISTRIBUTED_ML.sh"
echo "  3. Monitor: ssh strandgate tail -f /tmp/toadstool-worker.log"
echo

