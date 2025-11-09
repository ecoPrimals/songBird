#!/bin/bash
# Launch distributed training across all available local GPUs
# For testing on single machine with multiple shards

echo "========================================================================"
echo "  🚀 LAUNCHING DISTRIBUTED TRAINING"
echo "========================================================================"
echo ""

# Configuration
WORLD_SIZE=${WORLD_SIZE:-1}  # Number of GPUs to use
MASTER_ADDR=${MASTER_ADDR:-"127.0.0.1"}
MASTER_PORT=${MASTER_PORT:-"29500"}
EPOCHS=${EPOCHS:-2}
BATCH_SIZE=${BATCH_SIZE:-64}

export MASTER_ADDR
export MASTER_PORT

echo "Configuration:"
echo "  World size: $WORLD_SIZE GPUs"
echo "  Master: $MASTER_ADDR:$MASTER_PORT"
echo "  Epochs: $EPOCHS"
echo "  Batch size per GPU: $BATCH_SIZE"
echo "  Effective batch: $((BATCH_SIZE * WORLD_SIZE))"
echo ""

# Check available GPUs
NUM_GPUS=$(nvidia-smi -L 2>/dev/null | wc -l)
echo "Available GPUs: $NUM_GPUS"
nvidia-smi -L
echo ""

if [ "$NUM_GPUS" -lt "$WORLD_SIZE" ]; then
    echo "⚠️  Warning: Requested $WORLD_SIZE GPUs but only $NUM_GPUS available"
    echo "   Setting WORLD_SIZE=$NUM_GPUS"
    WORLD_SIZE=$NUM_GPUS
fi

echo "========================================================================"
echo "  🔥 STARTING TRAINING"
echo "========================================================================"
echo ""

# Launch training processes
for ((rank=0; rank<WORLD_SIZE; rank++)); do
    echo "Starting rank $rank..."
    python3 train_distributed.py \
        --rank $rank \
        --world-size $WORLD_SIZE \
        --epochs $EPOCHS \
        --batch-size $BATCH_SIZE \
        > ../results/distributed/rank_${rank}.log 2>&1 &
    
    # Save PID
    echo $! > ../results/distributed/rank_${rank}.pid
done

echo ""
echo "✅ Launched $WORLD_SIZE training processes"
echo ""
echo "Monitor progress:"
echo "  tail -f ../results/distributed/rank_0.log"
echo ""
echo "View all logs:"
echo "  tail -f ../results/distributed/rank_*.log"
echo ""
echo "Stop training:"
echo "  pkill -P \$(cat ../results/distributed/rank_*.pid)"
echo ""

# Wait for all processes
wait

echo ""
echo "========================================================================"
echo "  ✅ ALL PROCESSES COMPLETE"
echo "========================================================================"
echo ""
echo "View results:"
echo "  cat ../results/distributed/results.json"
echo ""
echo "Compare with baseline:"
echo "  diff ../results/baseline/results.json ../results/distributed/results.json"
echo ""

