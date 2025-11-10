#!/bin/bash
# Deploy Toadstool Compute Platform via Execution Agent
# This is the CORRECT bootstrap pattern:
#   Execution Agent (lightweight) → deploys → Toadstool (heavyweight)

set -e

# Configuration
TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
AGENT_PORT="9020"
TOADSTOOL_PORT="9000"
SONGBIRD_ORCHESTRATOR_B_PORT="8081"
SONGBIRD_ORCHESTRATOR_C_PORT="8082"

# Local paths
LOCAL_TOADSTOOL_ROOT="/home/eastgate/Development/ecoPrimals/toadstool"
TOADSTOOL_BINARY="$LOCAL_TOADSTOOL_ROOT/target/release/toadstool-cli"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🍄 Toadstool Deployment via Songbird Execution Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Architecture:"
echo "  • Execution Agent: Lightweight admin/bootstrap (Port $AGENT_PORT)"
echo "  • Toadstool: Heavy ML/GPU compute (Port $TOADSTOOL_PORT)"
echo ""
echo "This is the PROPER bootstrap pattern!"
echo ""

# Step 1: Check if Toadstool binary exists
if [ ! -f "$TOADSTOOL_BINARY" ]; then
    echo "📦 Toadstool binary not found. Building..."
    cd "$LOCAL_TOADSTOOL_ROOT"
    cargo build --release --bin toadstool
    cd -
    echo "✅ Toadstool built successfully"
else
    echo "✅ Toadstool binary found: $TOADSTOOL_BINARY"
fi

# Get binary size
BINARY_SIZE=$(ls -lh "$TOADSTOOL_BINARY" | awk '{print $5}')
echo "   Size: $BINARY_SIZE"
echo ""

# Function to deploy to a tower
deploy_toadstool() {
    local tower_name=$1
    local tower_ip=$2
    local orchestrator_port=$3
    local agent_endpoint="http://$tower_ip:$AGENT_PORT"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🚀 Deploying Toadstool to $tower_name ($tower_ip)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if execution agent is running
    if ! curl -s -f "$agent_endpoint/api/v1/health" > /dev/null; then
        echo "❌ Execution Agent not responding on $tower_name"
        echo "   Please ensure the agent is running on port $AGENT_PORT"
        return 1
    fi
    echo "✅ Execution Agent responding on $tower_name"
    
    # Deploy Toadstool binary
    echo "📤 Uploading Toadstool binary..."
    
    DEPLOY_RESPONSE=$(curl -s -X POST "$agent_endpoint/api/deployment/binary" \
      -F "binary=@$TOADSTOOL_BINARY" \
      -F "service_name=toadstool-compute" \
      -F "env_vars={\"TOADSTOOL_HOST\":\"$tower_ip\",\"TOADSTOOL_PORT\":\"$TOADSTOOL_PORT\",\"SONGBIRD_ENDPOINT\":\"http://$tower_ip:$orchestrator_port\",\"TOADSTOOL_GPU_ENABLED\":\"true\",\"RUST_LOG\":\"info\"}" \
      -F "auto_start=false")
    
    if echo "$DEPLOY_RESPONSE" | jq -e '.deployment_id' > /dev/null 2>&1; then
        DEPLOYMENT_ID=$(echo "$DEPLOY_RESPONSE" | jq -r '.deployment_id')
        DEPLOYMENT_PATH=$(echo "$DEPLOY_RESPONSE" | jq -r '.service_path')
        echo "✅ Toadstool deployed successfully!"
        echo "   Deployment ID: $DEPLOYMENT_ID"
        echo "   Binary path: $DEPLOYMENT_PATH"
        echo ""
        
        echo "📋 To start Toadstool on $tower_name:"
        echo "   1. SSH to tower (if needed for systemd setup):"
        echo "      ssh $tower_ip"
        echo ""
        echo "   2. Create systemd service (one-time):"
        echo "      cat << 'EOF' | sudo tee /etc/systemd/system/toadstool.service"
        echo "[Unit]"
        echo "Description=Toadstool Compute Platform"
        echo "After=network.target"
        echo ""
        echo "[Service]"
        echo "Type=simple"
        echo "User=$(whoami)"
        echo "WorkingDirectory=/tmp/songbird-deployments/$DEPLOYMENT_ID"
        echo "Environment=\"TOADSTOOL_HOST=$tower_ip\""
        echo "Environment=\"TOADSTOOL_PORT=$TOADSTOOL_PORT\""
        echo "Environment=\"SONGBIRD_ENDPOINT=http://$tower_ip:$orchestrator_port\""
        echo "Environment=\"TOADSTOOL_GPU_ENABLED=true\""
        echo "Environment=\"RUST_LOG=info\""
        echo "ExecStart=$DEPLOYMENT_PATH"
        echo "Restart=always"
        echo "RestartSec=10"
        echo ""
        echo "[Install]"
        echo "WantedBy=multi-user.target"
        echo "EOF"
        echo ""
        echo "   3. Enable and start:"
        echo "      sudo systemctl daemon-reload"
        echo "      sudo systemctl enable toadstool"
        echo "      sudo systemctl start toadstool"
        echo ""
        echo "   4. Verify:"
        echo "      curl http://$tower_ip:$TOADSTOOL_PORT/health"
        echo ""
        
        return 0
    else
        echo "❌ Deployment failed on $tower_name"
        echo "   Response: $DEPLOY_RESPONSE"
        return 1
    fi
}

# Deploy to Tower B
deploy_toadstool "Tower B (Strandgate)" "$TOWER_B_IP" "$SONGBIRD_ORCHESTRATOR_B_PORT"
TOWER_B_STATUS=$?

echo ""

# Deploy to Tower C
deploy_toadstool "Tower C (Southgate)" "$TOWER_C_IP" "$SONGBIRD_ORCHESTRATOR_C_PORT"
TOWER_C_STATUS=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Deployment Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
if [ $TOWER_B_STATUS -eq 0 ]; then
    echo "✅ Tower B: Toadstool binary deployed"
else
    echo "❌ Tower B: Deployment failed"
fi

if [ $TOWER_C_STATUS -eq 0 ]; then
    echo "✅ Tower C: Toadstool binary deployed"
else
    echo "❌ Tower C: Deployment failed"
fi

echo ""
echo "📖 Next Steps:"
echo "   1. Follow the systemd setup instructions above for each tower"
echo "   2. Verify Toadstool health endpoints"
echo "   3. Check Songbird discovers Toadstool capabilities:"
echo "      curl http://192.168.1.144:8080/api/federation/nodes | jq '.'"
echo ""
echo "🎯 After Toadstool is running:"
echo "   • Use Execution Agent for: Songbird admin, simple commands"
echo "   • Use Toadstool for: ML training, GPU workloads, distributed compute"
echo ""
echo "See: COMPUTE_LAYER_DECISION_GUIDE.md for detailed usage"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

