#!/bin/bash
# Launch distributed training across multiple physical towers
# Each tower trains on its own data shard with gradient synchronization

echo "========================================================================"
echo "  🌐 MULTI-TOWER DISTRIBUTED TRAINING LAUNCHER"
echo "========================================================================"
echo ""

# Configuration
MASTER_ADDR="192.168.1.144"  # Tower A (Eastgate)
MASTER_PORT="29500"
WORLD_SIZE=3  # 3 towers
EPOCHS=${EPOCHS:-2}
BATCH_SIZE=${BATCH_SIZE:-64}

export MASTER_ADDR
export MASTER_PORT

echo "Configuration:"
echo "  Master: $MASTER_ADDR:$MASTER_PORT"
echo "  World size: $WORLD_SIZE towers"
echo "  Epochs: $EPOCHS"
echo "  Batch size per tower: $BATCH_SIZE"
echo "  Effective batch: $((BATCH_SIZE * WORLD_SIZE))"
echo ""

# Tower configuration
declare -A TOWERS
TOWERS[0]="192.168.1.144:Eastgate:RTX_4070"      # Tower A (Master)
TOWERS[1]="192.168.1.134:Strandgate:RTX_3070"  # Tower B
TOWERS[2]="192.168.1.207:Southgate:RTX_3090"   # Tower C

# Data shard base (must be accessible on all towers)
SHARD_BASE="/home/eastgate/Development/ecoPrimals/songbird/experiments/data/imagenet100/sharded"

echo "Towers:"
for rank in "${!TOWERS[@]}"; do
    IFS=':' read -r ip name gpu <<< "${TOWERS[$rank]}"
    echo "  Rank $rank: $name ($gpu) at $ip"
done
echo ""

echo "========================================================================"
echo "  📋 PRE-FLIGHT CHECKS"
echo "========================================================================"
echo ""

# Check connectivity
echo "Testing connectivity..."
for rank in "${!TOWERS[@]}"; do
    IFS=':' read -r ip name gpu <<< "${TOWERS[$rank]}"
    if ping -c 1 -W 2 "$ip" > /dev/null 2>&1; then
        echo "  ✅ $name ($ip): Reachable"
    else
        echo "  ❌ $name ($ip): Unreachable"
    fi
done
echo ""

# Check SSH access (for remote towers)
echo "Testing SSH access..."
for rank in "${!TOWERS[@]}"; do
    if [ $rank -eq 0 ]; then
        echo "  ✅ Rank 0 (Master): Local"
        continue
    fi
    
    IFS=':' read -r ip name gpu <<< "${TOWERS[$rank]}"
    if ssh -o BatchMode=yes -o ConnectTimeout=5 "$ip" "echo test" > /dev/null 2>&1; then
        echo "  ✅ $name ($ip): SSH OK"
    else
        echo "  ⚠️  $name ($ip): SSH not configured (will need manual launch)"
    fi
done
echo ""

echo "========================================================================"
echo "  🚀 LAUNCHING DISTRIBUTED TRAINING"
echo "========================================================================"
echo ""

# Create results directory
mkdir -p ../results/distributed_multi_tower

# Launch on each tower
for rank in "${!TOWERS[@]}"; do
    IFS=':' read -r ip name gpu <<< "${TOWERS[$rank]}"
    
    echo "Starting rank $rank on $name ($ip)..."
    
    if [ $rank -eq 0 ]; then
        # Launch locally (master)
        python3 train_distributed.py \
            --rank $rank \
            --world-size $WORLD_SIZE \
            --epochs $EPOCHS \
            --batch-size $BATCH_SIZE \
            --output-dir ../results/distributed_multi_tower \
            > ../results/distributed_multi_tower/rank_${rank}.log 2>&1 &
        
        PID=$!
        echo $PID > ../results/distributed_multi_tower/rank_${rank}.pid
        echo "  Started PID: $PID (local)"
    else
        # Launch on remote tower via SSH
        ssh "$ip" "cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \
                   MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
                   python3 train_distributed.py \
                   --rank $rank \
                   --world-size $WORLD_SIZE \
                   --epochs $EPOCHS \
                   --batch-size $BATCH_SIZE \
                   --output-dir ../results/distributed_multi_tower \
                   > ../results/distributed_multi_tower/rank_${rank}.log 2>&1" &
        
        PID=$!
        echo $PID > ../results/distributed_multi_tower/rank_${rank}.pid
        echo "  Started PID: $PID (remote via SSH)"
    fi
    
    sleep 1  # Stagger launches slightly
done

echo ""
echo "✅ Launched training on $WORLD_SIZE towers"
echo ""
echo "Monitor progress:"
echo "  # Master (local):"
echo "  tail -f ../results/distributed_multi_tower/rank_0.log"
echo ""
echo "  # Remote towers:"
echo "  ssh 192.168.1.134 'tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_multi_tower/rank_1.log'"
echo "  ssh 192.168.1.207 'tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_multi_tower/rank_2.log'"
echo ""
echo "Stop training:"
echo "  pkill -P \$(cat ../results/distributed_multi_tower/rank_*.pid)"
echo ""

# Wait for master process
wait $(cat ../results/distributed_multi_tower/rank_0.pid)

echo ""
echo "========================================================================"
echo "  ✅ MASTER PROCESS COMPLETE"
echo "========================================================================"
echo ""
echo "View results:"
echo "  cat ../results/distributed_multi_tower/results.json"
echo ""
echo "Compare with baseline:"
echo "  echo 'Baseline: 166.7 images/sec'"
echo "  jq '.results[-1].images_per_sec' ../results/distributed_multi_tower/results.json"
echo "  jq '.results[-1].speedup' ../results/distributed_multi_tower/results.json"
echo ""

