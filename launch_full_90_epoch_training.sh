#!/bin/bash
# Full 90 Epoch ImageNet-100 Training - All 3 GPUs
# RTX 2070 Super, RTX 3070, RTX 3090

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Full 90 Epoch Distributed Training - ImageNet-100"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER_A_IP="192.168.1.144"  # Eastgate - RTX 2070 Super (8GB)
TOWER_B_IP="192.168.1.134"  # Strandgate - RTX 3070 (8GB)
TOWER_C_IP="192.168.1.207"  # Southgate - RTX 3090 (24GB)

AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29506"

WORLD_SIZE=3
EPOCHS=90  # FULL HPC TRAINING
BATCH_SIZE=64  # Per GPU

SCRIPT_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

OUTPUT_DIR="/tmp/imagenet100_full_90epochs"

echo "🔧 Configuration:"
echo "   Dataset: ImageNet-100 (250,000 images)"
echo "   Model: ResNet-50 (25.6M parameters)"
echo "   Epochs: $EPOCHS (FULL HPC training)"
echo "   Batch Size: $BATCH_SIZE per GPU"
echo "   Total Batch: $(($BATCH_SIZE * $WORLD_SIZE))"
echo ""
echo "GPU Configuration:"
echo "   🖥️  Tower A: RTX 2070 Super (8GB)  - Rank 0 (Coordinator)"
echo "   💪 Tower B: RTX 3070 (8GB)        - Rank 1 (Heavy Worker)"
echo "   🔥 Tower C: RTX 3090 (24GB)       - Rank 2 (Heaviest Worker)"
echo ""
echo "Expected Duration: ~15-18 hours"
echo "Cost Equivalent: $165 on AWS (we're doing it for $0!)"
echo ""

launch_worker() {
    local tower_name=$1
    local tower_ip=$2
    local rank=$3
    local script_path=$4
    local gpu_desc=$5
    
    local agent_url="http://$tower_ip:$AGENT_PORT"
    local output_dir="$OUTPUT_DIR/rank_$rank"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$gpu_desc $tower_name (Rank $rank)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if ! curl -s -f "$agent_url/health" > /dev/null; then
        echo "❌ Execution Agent not responding"
        return 1
    fi
    echo "✅ Execution Agent ready"
    
    # Verify GPU is available
    GPU_CHECK=$(curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d '{"command":"nvidia-smi","args":["--query-gpu=name"],"env":{},"background":false}' | jq -r '.stdout')
    echo "✅ GPU: $(echo "$GPU_CHECK" | head -1)"
    
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    echo "🚀 Launching full 90 epoch training..."
    RESPONSE=$(curl -s -X POST "$agent_url/api/v1/execution/command" \
      -H "Content-Type: application/json" \
      -d "{
        \"command\": \"bash\",
        \"args\": [\"-c\", \"$command\"],
        \"env\": {
          \"MASTER_ADDR\": \"$MASTER_ADDR\",
          \"MASTER_PORT\": \"$MASTER_PORT\",
          \"CUDA_VISIBLE_DEVICES\": \"0\",
          \"OMP_NUM_THREADS\": \"4\"
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
        echo "   Log: $output_dir/training.log"
        echo "$JOB_ID" > "/tmp/full_training_${tower_name}_rank_${rank}.id"
    else
        echo "❌ Launch failed: $RESPONSE"
        return 1
    fi
    echo ""
}

# Clean up any previous training
echo "🧹 Cleaning up previous runs..."
pkill -f "train_distributed.py" 2>/dev/null || true
curl -s -X POST "http://192.168.1.134:9020/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d '{"command":"pkill","args":["-f","train_distributed"],"env":{},"background":false}' > /dev/null 2>&1 || true
curl -s -X POST "http://192.168.1.207:9020/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d '{"command":"pkill","args":["-f","train_distributed"],"env":{},"background":false}' > /dev/null 2>&1 || true
sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Full 90 Epoch Training on All Towers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Launch in order: Master first, then workers
launch_worker "Eastgate" "$TOWER_A_IP" 0 "$SCRIPT_PATH_A" "🖥️  Coordinator:"
sleep 3
launch_worker "Strandgate" "$TOWER_B_IP" 1 "$SCRIPT_PATH_B" "💪 Heavy Worker:"
sleep 2
launch_worker "Southgate" "$TOWER_C_IP" 2 "$SCRIPT_PATH_C" "🔥 Heaviest Worker:"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎊 Full 90 Epoch Training Launched on All 3 GPUs!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Training Details:"
echo "  • Duration: ~15-18 hours"
echo "  • Model: ResNet-50 (25.6M params)"
echo "  • Dataset: ImageNet-100 (250K images)"
echo "  • Epochs: 90 (full HPC benchmark)"
echo "  • Total Batch Size: $(($BATCH_SIZE * $WORLD_SIZE))"
echo ""
echo "GPU Workload Distribution:"
echo "  🖥️  RTX 2070 Super: Batch $BATCH_SIZE (Coordinator)"
echo "  💪 RTX 3070:       Batch $BATCH_SIZE (Heavy Worker)"
echo "  🔥 RTX 3090:       Batch $BATCH_SIZE (Heaviest Worker)"
echo ""
echo "Monitor:"
echo "  • Local GPU: nvidia-smi"
echo "  • Local log: tail -f $OUTPUT_DIR/rank_0/training.log"
echo "  • All towers: ./monitor_distributed_training_http.sh"
echo ""
echo "This is HPC-level distributed ML on heterogeneous hardware!"
echo "Proving sovereign distributed == cloud HPC performance! 🐦🍄🔐"
echo ""

