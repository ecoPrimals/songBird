#!/bin/bash
# Deploy Songbird with Intelligent Routing to All Towers via HTTP

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Deploying Songbird with Intelligent Routing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
AGENT_PORT="9020"
BINARY_PATH="./target/release/songbird-orchestrator"

if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Binary not found. Building..."
    cargo build --release --bin songbird-orchestrator
fi

echo "✅ Binary ready: $(ls -lh $BINARY_PATH | awk '{print $5}')"
echo ""

deploy_to_tower() {
    local tower_name=$1
    local tower_ip=$2
    local agent_url="http://$tower_ip:$AGENT_PORT"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 Deploying to $tower_name ($tower_ip)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if ! curl -s -f "$agent_url/health" > /dev/null; then
        echo "❌ Execution Agent not responding"
        return 1
    fi
    echo "✅ Execution Agent ready"
    
    echo "📤 Uploading binary..."
    RESPONSE=$(curl -s -X POST "$agent_url/api/v1/deployment/binary" \
      -F "binary=@$BINARY_PATH" \
      -F "service_name=songbird-orchestrator" \
      -F 'env_vars={}' \
      -F "auto_start=false")
    
    if echo "$RESPONSE" | jq -e '.deployment_id' > /dev/null 2>&1; then
        DEPLOYMENT_ID=$(echo "$RESPONSE" | jq -r '.deployment_id')
        echo "✅ Deployed! ID: $DEPLOYMENT_ID"
        echo "   Binary: /tmp/songbird-deployments/$DEPLOYMENT_ID/service"
        echo ""
        echo "💡 To start on $tower_name:"
        echo "   pkill -f songbird-orchestrator"
        echo "   nohup /tmp/songbird-deployments/$DEPLOYMENT_ID/service > /tmp/songbird.log 2>&1 &"
        echo ""
    else
        echo "❌ Deployment failed: $RESPONSE"
        return 1
    fi
}

deploy_to_tower "Tower B (Strandgate)" "$TOWER_B_IP"
deploy_to_tower "Tower C (Southgate)" "$TOWER_C_IP"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Deployment Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Now restart Songbird on each tower via SSH or execution agent"
echo ""

