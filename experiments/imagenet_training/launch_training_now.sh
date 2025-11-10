#!/bin/bash
# PRAGMATIC APPROACH: Launch distributed ImageNet training via SSH
# This gets us results NOW while we work on proper Toadstool integration

set -e

# Configuration
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29500"
WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

# Paths (CORRECTED for each tower)
TOWER_A_TRAINING="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
TOWER_A_OUTPUT="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_now"

TOWER_B_IP="192.168.1.134"
TOWER_B_TRAINING="/home/strandgate/Development/songbird/experiments/imagenet_training/training"
TOWER_B_OUTPUT="/home/strandgate/Development/songbird/experiments/imagenet_training/results/distributed_now"

TOWER_C_IP="192.168.1.207"
TOWER_C_TRAINING="/home/southgate/Development/songbird/experiments/imagenet_training/training"
TOWER_C_OUTPUT="/home/southgate/Development/songbird/experiments/imagenet_training/results/distributed_now"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Launching Distributed ImageNet Training - PRAGMATIC APPROACH"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • World Size: $WORLD_SIZE towers"
echo "  • Epochs: $EPOCHS"
echo "  • Batch Size: $BATCH_SIZE"
echo "  • Master: $MASTER_ADDR:$MASTER_PORT"
echo ""
echo "This uses SSH for now (fast, works) while we perfect Toadstool integration."
echo ""

# Create output directory locally
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
echo "⏳ Waiting 5 seconds for master to initialize..."
sleep 5

# Launch Worker 1 on Tower B via SSH
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Launching Worker 1 on Tower B (Strandgate)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

ssh -f $TOWER_B_IP "mkdir -p $TOWER_B_OUTPUT && cd $TOWER_B_TRAINING && \
  MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
  python3 train_distributed.py \
    --rank 1 \
    --world-size $WORLD_SIZE \
    --epochs $EPOCHS \
    --batch-size $BATCH_SIZE \
    --output-dir $TOWER_B_OUTPUT \
  > $TOWER_B_OUTPUT/rank_1.log 2>&1 &"

echo "✅ Worker 1 launched on Tower B"
echo "   Log: ssh $TOWER_B_IP 'tail -f $TOWER_B_OUTPUT/rank_1.log'"
echo ""

# Launch Worker 2 on Tower C via SSH
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Launching Worker 2 on Tower C (Southgate)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

ssh -f $TOWER_C_IP "mkdir -p $TOWER_C_OUTPUT && cd $TOWER_C_TRAINING && \
  MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
  python3 train_distributed.py \
    --rank 2 \
    --world-size $WORLD_SIZE \
    --epochs $EPOCHS \
    --batch-size $BATCH_SIZE \
    --output-dir $TOWER_C_OUTPUT \
  > $TOWER_C_OUTPUT/rank_2.log 2>&1 &"

echo "✅ Worker 2 launched on Tower C"
echo "   Log: ssh $TOWER_C_IP 'tail -f $TOWER_C_OUTPUT/rank_2.log'"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Training Launched Across 3 Towers!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor Training:"
echo "  • Master log:  tail -f $MASTER_LOG"
echo "  • Worker 1:    ssh $TOWER_B_IP 'tail -f $TOWER_B_OUTPUT/rank_1.log'"
echo "  • Worker 2:    ssh $TOWER_C_IP 'tail -f $TOWER_C_OUTPUT/rank_2.log'"
echo "  • GPU usage:   watch -n 2 nvidia-smi"
echo ""
echo "Expected: ~3 minutes for 2 epochs with 3 GPUs"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

