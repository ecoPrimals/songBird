#!/bin/bash
# Quick Distributed ML Demo - 5 minutes of REAL training
# Uses existing working train_distributed.py script

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Quick Distributed ML Demo - ImageNet-100"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER_A_IP="192.168.1.144"
TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"

AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29505"

WORLD_SIZE=3
EPOCHS=2  # Quick demo
BATCH_SIZE=64

# Paths to existing working script
SCRIPT_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

# Use sharded ImageNet-100 data
DATA_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded/shard_0"
DATA_PATH_B="/home/strandgate/Development/songbird/experiments/data/imagenet100/sharded/shard_1"
DATA_PATH_C="/home/southgate/Development/songbird/experiments/data/imagenet100/sharded/shard_2"

OUTPUT_DIR="/tmp/distributed_ml_demo"

echo "🔧 Configuration:"
echo "   Dataset: ImageNet-100 (sharded)"
echo "   Model: ResNet-50"
echo "   Epochs: $EPOCHS (quick demo)"
echo "   Batch Size: $BATCH_SIZE per GPU"
echo "   Towers: 3 (RTX 2070 Super, RTX 3070, RTX 3090)"
echo ""

launch_worker() {
    local tower_name=$1
    local tower_ip=$2
    local rank=$3
    local script_path=$4
    local data_path=$5
    
    local agent_url="http://$tower_ip:$AGENT_PORT"
    local output_dir="$OUTPUT_DIR/rank_$rank"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎯 $tower_name (Rank $rank)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if ! curl -s -f "$agent_url/health" > /dev/null; then
        echo "❌ Execution Agent not responding"
        return 1
    fi
    echo "✅ Execution Agent ready"
    
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    echo "🚀 Launching training..."
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
    
    if echo "$RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
        JOB_ID=$(echo "$RESPONSE" | jq -r '.job_id')
        PID=$(echo "$RESPONSE" | jq -r '.pid')
        echo "✅ Worker launched!"
        echo "   Job ID: $JOB_ID"
        echo "   PID: $PID"
        echo "$JOB_ID" > "/tmp/ml_demo_${tower_name}_rank_${rank}.id"
    else
        echo "❌ Launch failed: $RESPONSE"
        return 1
    fi
    echo ""
}

# Clean up previous runs
echo "🧹 Cleaning up..."
pkill -f "train_distributed.py" 2>/dev/null || true
sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Distributed Training"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

launch_worker "Eastgate" "$TOWER_A_IP" 0 "$SCRIPT_PATH_A" "$DATA_PATH_A"
sleep 3
launch_worker "Strandgate" "$TOWER_B_IP" 1 "$SCRIPT_PATH_B" "$DATA_PATH_B"
sleep 2
launch_worker "Southgate" "$TOWER_C_IP" 2 "$SCRIPT_PATH_C" "$DATA_PATH_C"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎊 Distributed Training Launched!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Training will take ~5-10 minutes for 2 epochs"
echo ""
echo "Monitor GPU:"
echo "  nvidia-smi  # Tower A (local)"
echo ""
echo "Check logs:"
echo "  tail -f $OUTPUT_DIR/rank_0/training.log"
echo ""
echo "This is REAL distributed ML on heterogeneous hardware! 🐦🍄🔐"
echo ""

