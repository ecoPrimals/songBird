#!/bin/bash
# Launch Distributed Training via Songbird HTTP API (NO SSH!)
# This is the PROPER ecoPrimals architecture

set -e

SONGBIRD_MASTER="http://192.168.1.144:8080"
TOADSTOOL_BINARY="/home/eastgate/Development/ecoPrimals/toadstool/target/release/toadstool-cli"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed GPU Training via Songbird (SSH-FREE!)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Architecture: Songbird → Toadstool → PyTorch DDP"
echo ""

# Step 1: Deploy Toadstool binary to remote towers via Songbird
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Step 1: Deploying Toadstool to all towers via Songbird"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

deploy_toadstool() {
    local tower_name=$1
    local tower_ip=$2
    local tower_port=$3
    
    echo ""
    echo "📡 Deploying Toadstool to $tower_name ($tower_ip)"
    
    # Use Songbird's deployment API
    RESPONSE=$(curl -s -X POST "http://$tower_ip:$tower_port/api/deployment/binary" \
      -F "binary=@$TOADSTOOL_BINARY" \
      -F "service_name=toadstool-server" \
      -F "env_vars={\"TOADSTOOL_HOST\":\"$tower_ip\",\"TOADSTOOL_PORT\":\"9000\",\"TOADSTOOL_SONGBIRD_ENDPOINT\":\"http://$tower_ip:$tower_port\",\"TOADSTOOL_GPU_ENABLED\":\"true\"}" \
      -F "auto_start=true" 2>&1)
    
    if echo "$RESPONSE" | jq -e '.deployment_id' > /dev/null 2>&1; then
        DEPLOYMENT_ID=$(echo "$RESPONSE" | jq -r '.deployment_id')
        echo "✅ Toadstool deployed to $tower_name"
        echo "   Deployment ID: $DEPLOYMENT_ID"
        return 0
    else
        echo "⚠️  Deployment response: $RESPONSE"
        return 1
    fi
}

# Deploy to Tower B
deploy_toadstool "Tower B" "192.168.1.134" "8081"

# Deploy to Tower C
deploy_toadstool "Tower C" "192.168.1.207" "8082"

# Start Toadstool locally
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🍄 Starting Toadstool on Tower A (local)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

pkill -f "toadstool-byob-server" 2>/dev/null || true
sleep 2

cd /home/eastgate/Development/ecoPrimals/toadstool
TOADSTOOL_HOST=192.168.1.144 \
TOADSTOOL_PORT=9000 \
TOADSTOOL_SONGBIRD_ENDPOINT=http://192.168.1.144:8080 \
TOADSTOOL_GPU_ENABLED=true \
nohup ./target/release/toadstool-server > /tmp/toadstool_tower_a.log 2>&1 &

echo "✅ Toadstool started on Tower A"
echo "   Log: /tmp/toadstool_tower_a.log"

# Wait for Toadstool instances to start and register
echo ""
echo "⏳ Waiting 10 seconds for Toadstool instances to start and register..."
sleep 10

# Step 2: Verify Toadstool registration
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Step 2: Verifying Toadstool registrations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

check_toadstool() {
    local tower_name=$1
    local toadstool_url=$2
    
    if curl -s -f "$toadstool_url/health" > /dev/null 2>&1; then
        echo "✅ $tower_name: Toadstool responding"
        return 0
    else
        echo "⚠️  $tower_name: Toadstool not responding yet"
        return 1
    fi
}

check_toadstool "Tower A" "http://192.168.1.144:9000"
check_toadstool "Tower B" "http://192.168.1.134:9000"
check_toadstool "Tower C" "http://192.168.1.207:9000"

# Step 3: Submit distributed training job via Songbird
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Step 3: Submitting Distributed Training Job via Songbird"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

TRAINING_JOB=$(cat <<EOF
{
  "job_type": "pytorch_distributed_training",
  "job_name": "imagenet_training_3towers",
  "world_size": 3,
  "epochs": 2,
  "batch_size": 64,
  "script_path": "/home/*/Development/*/songbird/experiments/imagenet_training/training/train_distributed.py",
  "requirements": {
    "gpu": true,
    "python": "3.10",
    "pytorch": true,
    "distributed": true
  },
  "env": {
    "MASTER_ADDR": "192.168.1.144",
    "MASTER_PORT": "29501"
  },
  "towers": [
    {"rank": 0, "host": "192.168.1.144", "toadstool_endpoint": "http://192.168.1.144:9000"},
    {"rank": 1, "host": "192.168.1.134", "toadstool_endpoint": "http://192.168.1.134:9000"},
    {"rank": 2, "host": "192.168.1.207", "toadstool_endpoint": "http://192.168.1.207:9000"}
  ]
}
EOF
)

echo ""
echo "Submitting job to Songbird orchestrator..."
echo ""

JOB_RESPONSE=$(curl -s -X POST "$SONGBIRD_MASTER/api/v1/compute/distributed" \
  -H "Content-Type: application/json" \
  -d "$TRAINING_JOB" 2>&1)

if echo "$JOB_RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
    JOB_ID=$(echo "$JOB_RESPONSE" | jq -r '.job_id')
    echo "✅ Training job submitted successfully!"
    echo "   Job ID: $JOB_ID"
    echo ""
    echo "   Monitor: curl $SONGBIRD_MASTER/api/v1/compute/jobs/$JOB_ID | jq '.'"
else
    echo "⚠️  Job submission response: $JOB_RESPONSE"
    echo ""
    echo "   Note: If the endpoint doesn't exist yet, we'll submit directly to Toadstool instances"
    
    # Fallback: Submit directly to each Toadstool instance
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔄 Fallback: Submitting directly to Toadstool instances"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    submit_to_toadstool() {
        local tower_name=$1
        local endpoint=$2
        local rank=$3
        local training_dir=$4
        local output_dir=$5
        
        echo ""
        echo "📡 Submitting rank $rank to $tower_name"
        
        TASK_REQUEST=$(cat <<EOFTASK
{
  "workload_type": "python_script",
  "script_path": "$training_dir/train_distributed.py",
  "args": ["--rank", "$rank", "--world-size", "3", "--epochs", "2", "--batch-size", "64", "--output-dir", "$output_dir"],
  "env": {
    "MASTER_ADDR": "192.168.1.144",
    "MASTER_PORT": "29501"
  },
  "requirements": {
    "gpu": true
  }
}
EOFTASK
)
        
        RESPONSE=$(curl -s -X POST "$endpoint/api/v1/workloads" \
          -H "Content-Type: application/json" \
          -d "$TASK_REQUEST" 2>&1)
        
        if echo "$RESPONSE" | jq -e '.' > /dev/null 2>&1; then
            echo "✅ Task submitted to $tower_name"
            echo "$RESPONSE" | jq '.'
        else
            echo "⚠️  Response: $RESPONSE"
        fi
    }
    
    # Submit to each tower
    submit_to_toadstool "Tower A" "http://192.168.1.144:9000" "0" \
      "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training" \
      "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/toadstool"
    
    submit_to_toadstool "Tower B" "http://192.168.1.134:9000" "1" \
      "/home/strandgate/Development/songbird/experiments/imagenet_training/training" \
      "/home/strandgate/Development/songbird/experiments/imagenet_training/results/toadstool"
    
    submit_to_toadstool "Tower C" "http://192.168.1.207:9000" "2" \
      "/home/southgate/Development/songbird/experiments/imagenet_training/training" \
      "/home/southgate/Development/songbird/experiments/imagenet_training/results/toadstool"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎊 DISTRIBUTED TRAINING LAUNCHED VIA SONGBIRD!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor:"
echo "  • Local GPU:  watch -n 2 nvidia-smi"
echo "  • Tower A Log: tail -f /tmp/toadstool_tower_a.log"
echo "  • Songbird API: curl $SONGBIRD_MASTER/api/v1/services | jq '.'"
echo ""
echo "This is the PROPER ecoPrimals architecture! 🐦🍄"
echo "  • Songbird: Orchestration via HTTP"
echo "  • Toadstool: GPU compute execution"
echo "  • NO SSH needed!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

