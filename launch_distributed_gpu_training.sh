#!/bin/bash
# Launch Distributed GPU Training Across All Towers
# Tower A: Master coordinator (light work)
# Tower B: Worker 
# Tower C: Heavy worker (RTX 3090)

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed GPU Training - 3 Towers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
TOWER_A_IP="192.168.1.144"  # Eastgate (Hub, RTX 2070 SUPER)
TOWER_B_IP="192.168.1.134"  # Strandgate
TOWER_C_IP="192.168.1.207"  # Southgate (RTX 3090)

MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29502"
AGENT_PORT="9020"

WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

# Paths (adjust for each tower's user)
TRAINING_SCRIPT_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
TRAINING_SCRIPT_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
TRAINING_SCRIPT_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

OUTPUT_DIR_A="/tmp/imagenet_distributed/rank_0"
OUTPUT_DIR_B="/tmp/imagenet_distributed/rank_1"
OUTPUT_DIR_C="/tmp/imagenet_distributed/rank_2"

echo "Configuration:"
echo "  Master: $MASTER_ADDR:$MASTER_PORT"
echo "  World Size: $WORLD_SIZE"
echo "  Epochs: $EPOCHS"
echo "  Batch Size: $BATCH_SIZE"
echo ""
echo "Towers:"
echo "  Tower A (Eastgate):   Master/Light - RTX 2070 SUPER"
echo "  Tower B (Strandgate): Worker"
echo "  Tower C (Southgate):  Heavy Worker - RTX 3090 💪"
echo ""

# Create output directories on Tower A
mkdir -p "$OUTPUT_DIR_A"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Workers via Execution Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Function to launch worker via Execution Agent
launch_worker() {
    local tower_name=$1
    local tower_ip=$2
    local rank=$3
    local script_path=$4
    local output_dir=$5
    local agent_url="http://$tower_ip:$AGENT_PORT"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 Launching Rank $rank on $tower_name ($tower_ip)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if Execution Agent is responding
    if ! curl -s -f "$agent_url/health" > /dev/null; then
        echo "❌ Execution Agent not responding on $tower_name"
        echo "   Please ensure agent is running: songbird-execution-agent"
        return 1
    fi
    
    echo "✅ Execution Agent responding on $tower_name"
    
    # Prepare command
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    # Submit via Execution Agent
    RESPONSE=$(curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d "{
        \"command\": \"bash\",
        \"args\": [\"-c\", \"$command\"],
        \"env\": {
          \"MASTER_ADDR\": \"$MASTER_ADDR\",
          \"MASTER_PORT\": \"$MASTER_PORT\",
          \"CUDA_VISIBLE_DEVICES\": \"0\"
        },
        \"background\": true,
        \"capture_output\": true
      }")
    
    if echo "$RESPONSE" | jq -e '.execution_id' > /dev/null 2>&1; then
        EXEC_ID=$(echo "$RESPONSE" | jq -r '.execution_id')
        echo "✅ Worker launched on $tower_name (Execution ID: $EXEC_ID)"
        echo "   Output: $output_dir/training.log"
    else
        echo "❌ Failed to launch worker on $tower_name"
        echo "   Response: $RESPONSE"
        return 1
    fi
    
    echo ""
}

# Launch workers on all towers
echo "Starting distributed training workers..."
echo ""

# Rank 0 - Tower A (Eastgate) - Master/Light
echo "🎯 Rank 0: Tower A (Eastgate) - Master"
nohup bash -c "cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank 0 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $OUTPUT_DIR_A > $OUTPUT_DIR_A/training.log 2>&1" &
RANK_0_PID=$!
echo "✅ Started locally (PID: $RANK_0_PID)"
echo "   Log: $OUTPUT_DIR_A/training.log"
echo ""

sleep 2

# Rank 1 - Tower B (Strandgate)
launch_worker "Tower B (Strandgate)" "$TOWER_B_IP" 1 "$TRAINING_SCRIPT_B" "$OUTPUT_DIR_B"

# Rank 2 - Tower C (Southgate) - RTX 3090 Heavy Worker
launch_worker "Tower C (Southgate) 💪" "$TOWER_C_IP" 2 "$TRAINING_SCRIPT_C" "$OUTPUT_DIR_C"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Training Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "All workers launched! Distributed training in progress..."
echo ""
echo "Monitor logs:"
echo "  Rank 0 (Eastgate):   tail -f $OUTPUT_DIR_A/training.log"
echo "  Rank 1 (Strandgate): ssh strandgate tail -f $OUTPUT_DIR_B/training.log"
echo "  Rank 2 (Southgate):  ssh southgate tail -f $OUTPUT_DIR_C/training.log"
echo ""
echo "Check GPU usage:"
echo "  Tower A: nvidia-smi"
echo "  Tower B: ssh strandgate nvidia-smi"
echo "  Tower C: ssh southgate nvidia-smi"
echo ""
echo "This is proper ecoPrimals distributed sovereign ML! 🐦🍄🔐"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

