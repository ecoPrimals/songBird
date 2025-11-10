#!/bin/bash
# Launch distributed ImageNet training via Execution Agent (SSH-free!)

set -e

# Configuration
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29501"  # New port to avoid conflicts
WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

# Paths
TOWER_A_TRAINING="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
TOWER_A_OUTPUT="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_agent"

TOWER_B_IP="192.168.1.134"
TOWER_B_AGENT="http://$TOWER_B_IP:9020"
TOWER_B_TRAINING="/home/strandgate/Development/songbird/experiments/imagenet_training/training"
TOWER_B_OUTPUT="/home/strandgate/Development/songbird/experiments/imagenet_training/results/distributed_agent"

TOWER_C_IP="192.168.1.207"
TOWER_C_AGENT="http://$TOWER_C_IP:9020"
TOWER_C_TRAINING="/home/southgate/Development/songbird/experiments/imagenet_training/training"
TOWER_C_OUTPUT="/home/southgate/Development/songbird/experiments/imagenet_training/results/distributed_agent"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed ImageNet Training via Execution Agent (SSH-free!)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • World Size: $WORLD_SIZE towers"
echo "  • Epochs: $EPOCHS"
echo "  • Batch Size: $BATCH_SIZE"
echo "  • Master: $MASTER_ADDR:$MASTER_PORT"
echo "  • Method: Execution Agent (no SSH needed!)"
echo ""

# Create output directory
mkdir -p "$TOWER_A_OUTPUT"

# Launch Master (Rank 0) locally
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Launching Master (Rank 0) on Tower A (Local)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

MASTER_LOG="$TOWER_A_OUTPUT/rank_0.log"

cd "$TOWER_A_TRAINING"
MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
  python3 train_distributed.py \
    --rank 0 \
    --world-size $WORLD_SIZE \
    --epochs $EPOCHS \
    --batch-size $BATCH_SIZE \
    --output-dir "$TOWER_A_OUTPUT" \
  > "$MASTER_LOG" 2>&1 &

MASTER_PID=$!
echo "✅ Master launched (PID: $MASTER_PID)"
echo "   Log: $MASTER_LOG"
echo ""

# Wait for master to initialize
echo "⏳ Waiting 8 seconds for master to bind port..."
sleep 8

# Launch Worker 1 on Tower B via Execution Agent
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Launching Worker 1 on Tower B via Execution Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WORKER_B_RESPONSE=$(curl -s -X POST "$TOWER_B_AGENT/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d "{
    \"command\": \"bash\",
    \"args\": [\"-c\", \"mkdir -p $TOWER_B_OUTPUT && cd $TOWER_B_TRAINING && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT python3 train_distributed.py --rank 1 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $TOWER_B_OUTPUT > $TOWER_B_OUTPUT/rank_1.log 2>&1 &\"],
    \"env\": {},
    \"background\": true,
    \"capture_output\": true
  }")

if echo "$WORKER_B_RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
    JOB_B=$(echo "$WORKER_B_RESPONSE" | jq -r '.job_id')
    echo "✅ Worker 1 launched on Tower B"
    echo "   Job ID: $JOB_B"
else
    echo "❌ Worker 1 failed: $WORKER_B_RESPONSE"
fi
echo ""

# Launch Worker 2 on Tower C via Execution Agent
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Launching Worker 2 on Tower C via Execution Agent"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WORKER_C_RESPONSE=$(curl -s -X POST "$TOWER_C_AGENT/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d "{
    \"command\": \"bash\",
    \"args\": [\"-c\", \"mkdir -p $TOWER_C_OUTPUT && cd $TOWER_C_TRAINING && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT python3 train_distributed.py --rank 2 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $TOWER_C_OUTPUT > $TOWER_C_OUTPUT/rank_2.log 2>&1 &\"],
    \"env\": {},
    \"background\": true,
    \"capture_output\": true
  }")

if echo "$WORKER_C_RESPONSE" | jq -e '.job_id' > /dev/null 2>&1; then
    JOB_C=$(echo "$WORKER_C_RESPONSE" | jq -r '.job_id')
    echo "✅ Worker 2 launched on Tower C"
    echo "   Job ID: $JOB_C"
else
    echo "❌ Worker 2 failed: $WORKER_C_RESPONSE"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Training Launched via Execution Agent!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor Training:"
echo "  • Master log:  tail -f $MASTER_LOG"
echo "  • GPU usage:   watch -n 2 nvidia-smi"
echo ""
echo "Check remote workers:"
echo "  • Worker 1:    curl $TOWER_B_AGENT/api/v1/execution/jobs/$JOB_B | jq '.'"
echo "  • Worker 2:    curl $TOWER_C_AGENT/api/v1/execution/jobs/$JOB_C | jq '.'"
echo ""
echo "Expected: ~3 minutes for 2 epochs with 3 GPUs"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

