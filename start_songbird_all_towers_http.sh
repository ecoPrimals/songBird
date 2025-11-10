#!/bin/bash
# Start Songbird on all towers via HTTP Execution Agent

set -e

TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
AGENT_PORT="9020"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Starting Songbird on All Towers (via HTTP)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

start_songbird() {
    local tower_name=$1
    local tower_ip=$2
    local agent_url="http://$tower_ip:$AGENT_PORT"
    local songbird_port=$3
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🐦 Starting Songbird on $tower_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Kill existing Songbird
    echo "Stopping existing Songbird..."
    curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d '{"command":"pkill","args":["-f","songbird-orchestrator"],"env":{},"background":false}' > /dev/null 2>&1
    sleep 2
    
    # Find latest deployment
    DEPLOYMENTS=$(curl -s "$agent_url/api/v1/deployment/list" | jq -r '.deployments[] | select(.service_name == "songbird-orchestrator") | .deployment_id' | head -1)
    
    if [ -z "$DEPLOYMENTS" ]; then
        echo "❌ No deployment found. Run deploy_songbird_all_towers.sh first"
        return 1
    fi
    
    LATEST_DEPLOYMENT=$(echo "$DEPLOYMENTS" | head -1)
    BINARY_PATH="/tmp/songbird-deployments/$LATEST_DEPLOYMENT/service"
    
    echo "✅ Using deployment: $LATEST_DEPLOYMENT"
    
    # Start Songbird
    echo "Starting Songbird..."
    RESPONSE=$(curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d "{
        \"command\": \"bash\",
        \"args\": [\"-c\", \"cd /tmp && RUST_LOG=info SONGBIRD_PORT=$songbird_port nohup $BINARY_PATH > /tmp/songbird.log 2>&1 &\"],
        \"env\": {
          \"RUST_LOG\": \"info\",
          \"SONGBIRD_PORT\": \"$songbird_port\"
        },
        \"background\": true,
        \"capture_output\": false
      }")
    
    if echo "$RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
        JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id')
        echo "✅ Songbird started (Job: $JOB_ID)"
        sleep 3
        
        # Test health
        if curl -s -f "http://$tower_ip:$songbird_port/health" > /dev/null; then
            echo "✅ Health check passed: http://$tower_ip:$songbird_port/health"
        else
            echo "⚠️  Health check pending (might still be starting)"
        fi
    else
        echo "❌ Failed to start: $RESPONSE"
        return 1
    fi
    echo ""
}

# Start on all towers
start_songbird "Tower B (Strandgate)" "$TOWER_B_IP" "8081"
start_songbird "Tower C (Southgate)" "$TOWER_C_IP" "8082"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Songbird Started on All Towers!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Verify:"
echo "  curl http://192.168.1.144:8080/health  # Tower A"
echo "  curl http://192.168.1.134:8081/health  # Tower B"
echo "  curl http://192.168.1.207:8082/health  # Tower C"
echo ""

