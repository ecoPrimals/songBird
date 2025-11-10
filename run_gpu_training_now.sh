#!/bin/bash
# Run Distributed GPU Training via Toadstool (NOW!)

set -e

# Configuration
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29502"
WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

# Toadstool endpoints
TOADSTOOL_A="http://localhost:8084"
TOADSTOOL_B="http://192.168.1.134:8084"
TOADSTOOL_C="http://192.168.1.207:8084"

# Training paths (adjusted per tower)
TRAINING_SCRIPT_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training/train_distributed.py"
TRAINING_SCRIPT_B="/home/strandgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"
TRAINING_SCRIPT_C="/home/southgate/Development/songbird/experiments/imagenet_training/training/train_distributed.py"

OUTPUT_DIR_A="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/toadstool_final"
OUTPUT_DIR_B="/home/strandgate/Development/songbird/experiments/imagenet_training/results/toadstool_final"
OUTPUT_DIR_C="/home/southgate/Development/songbird/experiments/imagenet_training/results/toadstool_final"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed GPU Training via Toadstool"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • World Size: $WORLD_SIZE towers"
echo "  • Epochs: $EPOCHS"
echo "  • Batch Size: $BATCH_SIZE"
echo "  • Master: $MASTER_ADDR:$MASTER_PORT"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR_A"

# Function to submit training to Toadstool
submit_training_task() {
    local tower_name=$1
    local toadstool_url=$2
    local rank=$3
    local script_path=$4
    local output_dir=$5
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📡 Submitting Rank $rank to $tower_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if Toadstool is responding
    if ! curl -s -f "$toadstool_url/health" > /dev/null 2>&1; then
        echo "⚠️  Toadstool not responding on $tower_name ($toadstool_url)"
        echo "   Trying to execute directly via Python..."
        
        # Fallback: Direct Python execution
        if [ "$rank" -eq 0 ]; then
            # Master - run locally
            echo "   Running master locally..."
            cd "$(dirname "$script_path")"
            MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
              nohup python3 train_distributed.py \
                --rank $rank --world-size $WORLD_SIZE \
                --epochs $EPOCHS --batch-size $BATCH_SIZE \
                --output-dir "$output_dir" \
              > "$output_dir/rank_${rank}.log" 2>&1 &
            echo "✅ Master launched (PID: $!)"
        fi
        return 1
    fi
    
    echo "✅ Toadstool responding on $tower_name"
    
    # Submit workload to Toadstool
    WORKLOAD_REQUEST=$(cat <<EOF
{
  "biome_config": {
    "version": "1.0",
    "name": "pytorch-training-rank-$rank",
    "runtime": {
      "type": "python",
      "version": "3.10"
    },
    "command": "python3",
    "args": [
      "$script_path",
      "--rank", "$rank",
      "--world-size", "$WORLD_SIZE",
      "--epochs", "$EPOCHS",
      "--batch-size", "$BATCH_SIZE",
      "--output-dir", "$output_dir"
    ],
    "environment": {
      "MASTER_ADDR": "$MASTER_ADDR",
      "MASTER_PORT": "$MASTER_PORT",
      "CUDA_VISIBLE_DEVICES": "0"
    },
    "resources": {
      "gpu": true,
      "memory": "8GB"
    }
  }
}
EOF
)
    
    echo "Submitting to $toadstool_url/api/v1/biomes..."
    RESPONSE=$(curl -s -X POST "$toadstool_url/api/v1/biomes" \
      -H "Content-Type: application/json" \
      -d "$WORKLOAD_REQUEST" 2>&1)
    
    if echo "$RESPONSE" | jq -e '.biome_id' > /dev/null 2>&1; then
        BIOME_ID=$(echo "$RESPONSE" | jq -r '.biome_id')
        echo "✅ Training task submitted to $tower_name"
        echo "   Biome ID: $BIOME_ID"
        echo "   Monitor: curl $toadstool_url/api/v1/biomes/$BIOME_ID | jq '.'"
        return 0
    else
        echo "⚠️  Submission response: $RESPONSE"
        echo ""
        echo "   Trying direct execution fallback..."
        
        # Fallback for rank 0
        if [ "$rank" -eq 0 ]; then
            cd "$(dirname "$script_path")"
            MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
              nohup python3 train_distributed.py \
                --rank $rank --world-size $WORLD_SIZE \
                --epochs $EPOCHS --batch-size $BATCH_SIZE \
                --output-dir "$output_dir" \
              > "$output_dir/rank_${rank}.log" 2>&1 &
            echo "✅ Master launched locally (PID: $!)"
        fi
        return 1
    fi
}

# Kill any old training processes
pkill -f "train_distributed.py" 2>/dev/null || true
sleep 2

# Submit to each tower
echo ""
submit_training_task "Tower A (Master)" "$TOADSTOOL_A" 0 "$TRAINING_SCRIPT_A" "$OUTPUT_DIR_A"
TOWER_A_STATUS=$?

echo ""
echo "⏳ Waiting 8 seconds for master to initialize..."
sleep 8

echo ""
submit_training_task "Tower B (Worker 1)" "$TOADSTOOL_B" 1 "$TRAINING_SCRIPT_B" "$OUTPUT_DIR_B"
TOWER_B_STATUS=$?

echo ""
submit_training_task "Tower C (Worker 2)" "$TOADSTOOL_C" 2 "$TRAINING_SCRIPT_C" "$OUTPUT_DIR_C"
TOWER_C_STATUS=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Submission Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $TOWER_A_STATUS -eq 0 ]; then
    echo "✅ Tower A: Training task submitted"
else
    echo "⚠️  Tower A: Direct execution (fallback)"
fi

if [ $TOWER_B_STATUS -eq 0 ]; then
    echo "✅ Tower B: Training task submitted"
else
    echo "⚠️  Tower B: Check Toadstool status"
fi

if [ $TOWER_C_STATUS -eq 0 ]; then
    echo "✅ Tower C: Training task submitted"
else
    echo "⚠️  Tower C: Check Toadstool status"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 TRAINING LAUNCHED!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor:"
echo "  • GPU Usage:   watch -n 2 nvidia-smi"
echo "  • Master Log:  tail -f $OUTPUT_DIR_A/rank_0.log"
echo "  • Toadstool A: curl $TOADSTOOL_A/health | jq '.'"
echo ""
echo "Expected: ~3 minutes for 2 epochs"
echo ""
echo "Results will be in: $OUTPUT_DIR_A"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

