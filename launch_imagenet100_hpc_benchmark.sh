#!/bin/bash
# ImageNet-100 Full HPC Benchmark
# =================================
# 90 epochs, mixed precision, proper data sharding
# Demonstrates HPC-level performance on heterogeneous hardware

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 ImageNet-100 Full HPC Benchmark"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
TOWER_A_IP="192.168.1.144"  # Eastgate (RTX 2070 Super - 8GB)
TOWER_B_IP="192.168.1.134"  # Strandgate (RTX 3070 - 8GB + Dual CPU)
TOWER_C_IP="192.168.1.207"  # Southgate (RTX 3090 - 24GB)

AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29504"

WORLD_SIZE=3
EPOCHS=90  # Full HPC benchmark
BATCH_SIZE_A=32   # RTX 2070 Super
BATCH_SIZE_B=64   # RTX 3070
BATCH_SIZE_C=96   # RTX 3090

SCRIPT_PATH_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_imagenet100_hpc_benchmark.py"
SCRIPT_PATH_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_imagenet100_hpc_benchmark.py"
SCRIPT_PATH_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_imagenet100_hpc_benchmark.py"

OUTPUT_DIR="/tmp/imagenet100_hpc_benchmark"

echo "🔧 HPC Benchmark Configuration:"
echo "   Dataset: ImageNet-100 (250,000 images, 100 classes)"
echo "   Model: ResNet-50 (25.6M parameters)"
echo "   Epochs: $EPOCHS (full training)"
echo "   Mixed Precision: FP16 (faster)"
echo "   Total Batch Size: $(($BATCH_SIZE_A + $BATCH_SIZE_B + $BATCH_SIZE_C))"
echo ""
echo "GPU Allocation (Optimized for HPC):"
echo "   🖥️  Rank 0 (Eastgate):   RTX 2070 Super - Batch $BATCH_SIZE_A"
echo "   💪 Rank 1 (Strandgate): RTX 3070       - Batch $BATCH_SIZE_B"
echo "   🔥 Rank 2 (Southgate):  RTX 3090       - Batch $BATCH_SIZE_C"
echo ""
echo "Expected Performance:"
echo "   • Training Time: ~15-18 hours (distributed)"
echo "   • Target Accuracy: 75-77% (ImageNet-100)"
echo "   • Throughput: 150-200 images/sec"
echo ""
echo "Comparison to Cloud HPC:"
echo "   • AWS 3x p3.2xlarge: $9.18/hour x 18h = $165"
echo "   • Our System: $0 (owned hardware)"
echo "   • Savings: 100%"
echo ""

# Function to launch worker via HTTP
launch_worker() {
    local tower_name=$1
    local tower_ip=$2
    local rank=$3
    local script_path=$4
    local gpu_desc=$5
    local batch_size=$6
    
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
    
    # Command for HPC benchmark
    local command="mkdir -p $output_dir && cd $(dirname $script_path) && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 $(basename $script_path) --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --output-dir $output_dir > $output_dir/training.log 2>&1"
    
    echo "🚀 Launching HPC benchmark worker..."
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
        echo "✅ HPC benchmark worker launched!"
        echo "   Job ID: $JOB_ID"
        echo "   PID: $PID"
        echo "   Batch Size: $batch_size"
        echo "   Log: $output_dir/training.log"
        echo "$JOB_ID" > "/tmp/hpc_benchmark_${tower_name}_rank_${rank}.id"
    else
        echo "❌ Launch failed: $RESPONSE"
        return 1
    fi
    echo ""
}

# Stop any existing training
echo "🧹 Cleaning up previous runs..."
pkill -f "train_imagenet100_hpc_benchmark.py" 2>/dev/null || true
pkill -f "train_distributed.py" 2>/dev/null || true
sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching HPC Benchmark Workers (via HTTP)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Launch in order: Coordinator first, then heavy workers
launch_worker "Eastgate" "$TOWER_A_IP" 0 "$SCRIPT_PATH_A" "🖥️  Coordinator:" "$BATCH_SIZE_A"
sleep 3
launch_worker "Strandgate" "$TOWER_B_IP" 1 "$SCRIPT_PATH_B" "💪 Heavy Worker:" "$BATCH_SIZE_B"
sleep 2
launch_worker "Southgate" "$TOWER_C_IP" 2 "$SCRIPT_PATH_C" "🔥 Heaviest Worker:" "$BATCH_SIZE_C"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ ImageNet-100 HPC Benchmark Launched!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Benchmark Details:"
echo "  • Dataset: ImageNet-100 (250K images, 100 classes)"
echo "  • Model: ResNet-50 (25.6M parameters)"
echo "  • Epochs: $EPOCHS (full HPC benchmark)"
echo "  • Total Batch: $(($BATCH_SIZE_A + $BATCH_SIZE_B + $BATCH_SIZE_C))"
echo "  • Mixed Precision: FP16"
echo "  • Framework: PyTorch DDP with NCCL"
echo ""
echo "Expected Timeline:"
echo "  • Duration: ~15-18 hours"
echo "  • Target Accuracy: 75-77%"
echo "  • Checkpoints: Every 5 epochs"
echo ""
echo "Monitor via HTTP:"
echo "  ./monitor_distributed_training_http.sh"
echo ""
echo "Check logs:"
echo "  tail -f $OUTPUT_DIR/rank_0/training.log"
echo ""
echo "GPU Utilization:"
echo "  • RTX 2070 Super: Coordinator (light load)"
echo "  • RTX 3070:       Heavy worker 💪"
echo "  • RTX 3090:       Heaviest worker (largest batches) 🔥"
echo ""
echo "This is HPC-level distributed ML with ecoPrimals! 🐦🍄🔐"
echo ""

