#!/bin/bash
# Proper Toadstool + Songbird Integration Setup
# Uses Songbird HTTP API to manage everything

set -e

# Configuration
SONGBIRD_TOWER_A="http://192.168.1.144:8080"
SONGBIRD_TOWER_B="http://192.168.1.134:8081"
SONGBIRD_TOWER_C="http://192.168.1.207:8082"

TOADSTOOL_ROOT="/home/eastgate/Development/ecoPrimals/toadstool"
TOADSTOOL_BINARY="$TOADSTOOL_ROOT/target/release/toadstool-cli"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🍄🐦 Toadstool + Songbird Integration Setup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "This is the PROPER architecture:"
echo "  1. Songbird orchestrates via HTTP API"
echo "  2. Toadstool registers with Songbird"
echo "  3. Songbird routes compute tasks to Toadstool"
echo "  4. Toadstool handles GPU/ML execution"
echo ""

# Function to check Songbird health
check_songbird() {
    local tower_name=$1
    local tower_url=$2
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🐦 Checking Songbird on $tower_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if curl -s -f "$tower_url/health" > /dev/null; then
        echo "✅ Songbird responding on $tower_name"
        curl -s "$tower_url/health" | jq -r '.' 2>/dev/null || echo "  Status: OK"
        return 0
    else
        echo "❌ Songbird not responding on $tower_name"
        return 1
    fi
}

# Function to register Toadstool with Songbird
register_toadstool() {
    local tower_name=$1
    local songbird_url=$2
    local toadstool_host=$3
    local toadstool_port=$4
    
    echo ""
    echo "📡 Registering Toadstool ($toadstool_host:$toadstool_port) with $tower_name"
    
    REGISTRATION_DATA=$(cat <<EOF
{
  "service_type": "compute",
  "service_id": "toadstool-$toadstool_host",
  "host": "$toadstool_host",
  "port": "$toadstool_port",
  "capabilities": {
    "gpu": true,
    "python": true,
    "pytorch": true,
    "distributed_training": true
  }
}
EOF
)
    
    RESPONSE=$(curl -s -X POST "$songbird_url/api/v1/services/register" \
      -H "Content-Type: application/json" \
      -d "$REGISTRATION_DATA" 2>&1)
    
    if echo "$RESPONSE" | jq -e '.' > /dev/null 2>&1; then
        echo "✅ Registration successful"
        echo "$RESPONSE" | jq '.'
    else
        echo "⚠️  Registration response: $RESPONSE"
        echo "   (This is OK if endpoint doesn't exist yet - Toadstool will register on startup)"
    fi
}

# Function to start Toadstool with Songbird integration
start_toadstool() {
    local tower_name=$1
    local tower_host=$2
    local toadstool_port=$3
    local songbird_endpoint=$4
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🍄 Starting Toadstool on $tower_name ($tower_host:$toadstool_port)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "   Songbird: $songbird_endpoint"
    
    echo ""
    echo "Command to run on $tower_name:"
    echo "─────────────────────────────────────────────────────────────────────"
    cat <<EOF
cd $TOADSTOOL_ROOT
TOADSTOOL_HOST=$tower_host \\
TOADSTOOL_PORT=$toadstool_port \\
TOADSTOOL_SONGBIRD_ENDPOINT=$songbird_endpoint \\
TOADSTOOL_GPU_ENABLED=true \\
nohup ./target/release/toadstool-server > /tmp/toadstool_$tower_name.log 2>&1 &
EOF
    echo "─────────────────────────────────────────────────────────────────────"
}

# Check Songbird on all towers
echo "🔍 Step 1: Checking Songbird instances"
echo ""

check_songbird "Tower A (Eastgate)" "$SONGBIRD_TOWER_A"
TOWER_A_STATUS=$?

check_songbird "Tower B (Strandgate)" "$SONGBIRD_TOWER_B"
TOWER_B_STATUS=$?

check_songbird "Tower C (Southgate)" "$SONGBIRD_TOWER_C"
TOWER_C_STATUS=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Status Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $TOWER_A_STATUS -eq 0 ]; then echo "✅ Tower A: Songbird OK"; else echo "❌ Tower A: Songbird DOWN"; fi
if [ $TOWER_B_STATUS -eq 0 ]; then echo "✅ Tower B: Songbird OK"; else echo "❌ Tower B: Songbird DOWN"; fi
if [ $TOWER_C_STATUS -eq 0 ]; then echo "✅ Tower C: Songbird OK"; else echo "❌ Tower C: Songbird DOWN"; fi

# Try to register Toadstool with each Songbird
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔗 Step 2: Pre-registering Toadstool services"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $TOWER_A_STATUS -eq 0 ]; then
    register_toadstool "Tower A" "$SONGBIRD_TOWER_A" "192.168.1.144" "9000"
fi

if [ $TOWER_B_STATUS -eq 0 ]; then
    register_toadstool "Tower B" "$SONGBIRD_TOWER_B" "192.168.1.134" "9000"
fi

if [ $TOWER_C_STATUS -eq 0 ]; then
    register_toadstool "Tower C" "$SONGBIRD_TOWER_C" "192.168.1.207" "9000"
fi

# Show how to start Toadstool on each tower
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Step 3: Start Toadstool on each tower"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

start_toadstool "Tower A" "192.168.1.144" "9000" "$SONGBIRD_TOWER_A"
start_toadstool "Tower B" "192.168.1.134" "9000" "$SONGBIRD_TOWER_B"
start_toadstool "Tower C" "192.168.1.207" "9000" "$SONGBIRD_TOWER_C"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Step 4: Submit distributed training job via Songbird"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "After Toadstool instances are running, submit training job to Songbird:"
echo "─────────────────────────────────────────────────────────────────────"
cat <<'EOF'
curl -X POST http://192.168.1.144:8080/api/v1/compute/distributed \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "pytorch_distributed_training",
    "world_size": 3,
    "script": "/path/to/train_distributed.py",
    "requirements": {
      "gpu": true,
      "python": "3.10",
      "pytorch": "2.0"
    },
    "env": {
      "MASTER_ADDR": "192.168.1.144",
      "MASTER_PORT": "29501"
    }
  }'
EOF
echo "─────────────────────────────────────────────────────────────────────"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ SETUP COMPLETE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  1. Run the Toadstool commands above on each tower"
echo "  2. Verify registration: curl $SONGBIRD_TOWER_A/api/v1/services | jq '.'"
echo "  3. Submit training job through Songbird"
echo "  4. Songbird routes to Toadstool instances automatically"
echo ""
echo "This is the PROPER ecoPrimals architecture! 🐦🍄"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

