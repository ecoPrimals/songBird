#!/bin/bash
# Large Distributed Training - Optimized for 3070 & 3090
# Eastgate: Minimal coordinator role
# Strandgate: RTX 3070 + Dual CPU - Heavy worker
# Southgate: RTX 3090 - Heaviest worker

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Large Distributed ML Training"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
TOWER_A_IP="192.168.1.144"  # Eastgate (Coordinator, minimal work)
TOWER_B_IP="192.168.1.134"  # Strandgate (RTX 3070, Dual CPU)
TOWER_C_IP="192.168.1.207"  # Southgate (RTX 3090)

AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29503"

# 3-way training: Coordinator + 2 heavy workers
WORLD_SIZE=3
EPOCHS=10  # Larger training run
BATCH_SIZE=128  # Larger batches for 3070 & 3090

SCRIPT_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
SCRIPT_PATH_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

OUTPUT_DIR="/tmp/imagenet_large_distributed"

echo "🔧 Configuration:"
echo "   Model: ResNet-50 (23.9M parameters)"
echo "   Dataset: ImageNet (subset)"
echo "   Epochs: $EPOCHS"
echo "   Batch Size: $BATCH_SIZE (large for powerful GPUs)"
echo "   World Size: $WORLD_SIZE"
echo ""
echo "GPU Allocation:"
echo "   🖥️  Rank 0 (Eastgate):   RTX 2070 SUPER - Coordinator (light)"
echo "   💪 Rank 1 (Strandgate): RTX 3070 - Heavy worker"
echo "   🔥 Rank 2 (Southgate):  RTX 3090 - Heaviest worker"
echo ""

# Function to launch worker via HTTP
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
    echo "✅ HTTP Execution Agent ready"
    
    # Command with optimizations for large training
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    echo "🚀 Launching training worker..."
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
        echo "$JOB_ID" > "/tmp/large_training_${tower_name}_rank_${rank}.id"
    else
        echo "❌ Launch failed: $RESPONSE"
        return 1
    fi
    echo ""
}

# Stop any existing training
echo "🧹 Cleaning up previous runs..."
pkill -f "train_distributed.py" 2>/dev/null || true
sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Distributed Training Workers"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Launch in order: Coordinator first, then heavy workers
launch_worker "Eastgate" "$TOWER_A_IP" 0 "$SCRIPT_PATH_A" "🖥️  Coordinator:"
sleep 3
launch_worker "Strandgate" "$TOWER_B_IP" 1 "$SCRIPT_PATH_B" "💪 Heavy Worker:"
sleep 2
launch_worker "Southgate" "$TOWER_C_IP" 2 "$SCRIPT_PATH_C" "🔥 Heaviest Worker:"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Large Distributed Training Launched!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Training Details:"
echo "  • Model: ResNet-50 (23.9M params)"
echo "  • Epochs: $EPOCHS"
echo "  • Batch Size: $BATCH_SIZE (optimized for 3070/3090)"
echo "  • Distributed: 3 towers via PyTorch DDP"
echo ""
echo "Monitor via HTTP:"
echo "  ./monitor_distributed_training_http.sh"
echo ""
echo "GPU Utilization:"
echo "  • RTX 2070 SUPER (Eastgate): Light coordination work"
echo "  • RTX 3070 (Strandgate):     Heavy training workload 💪"
echo "  • RTX 3090 (Southgate):      Heaviest training workload 🔥"
echo ""
echo "This is proper distributed ecoPrimals ML! 🐦🍄🔐"
echo ""

