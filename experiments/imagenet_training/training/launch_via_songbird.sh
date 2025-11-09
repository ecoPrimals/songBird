#!/bin/bash
# Launch distributed training via Songbird HTTP API
# Uses curl to send commands to Songbird on each tower

echo "========================================================================"
echo "  🚀 DISTRIBUTED TRAINING VIA SONGBIRD HTTP API"
echo "========================================================================"
echo ""

# Configuration
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29500"
WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

echo "Configuration:"
echo "  Master: $MASTER_ADDR:$MASTER_PORT"
echo "  World size: $WORLD_SIZE towers"
echo "  Epochs: $EPOCHS"
echo "  Batch size: $BATCH_SIZE"
echo ""

# Tower Songbird endpoints
declare -A SONGBIRD_URLS
SONGBIRD_URLS[0]="http://192.168.1.144:8000"
SONGBIRD_URLS[1]="http://192.168.1.134:8000"
SONGBIRD_URLS[2]="http://192.168.1.207:8000"

declare -A TOWER_NAMES
TOWER_NAMES[0]="Eastgate"
TOWER_NAMES[1]="Strandgate"
TOWER_NAMES[2]="Southgate"

echo "========================================================================"
echo "  🔍 CHECKING SONGBIRD STATUS"
echo "========================================================================"
echo ""

for rank in "${!SONGBIRD_URLS[@]}"; do
    url="${SONGBIRD_URLS[$rank]}"
    name="${TOWER_NAMES[$rank]}"
    
    echo "Checking $name ($url)..."
    response=$(curl -s -w "\n%{http_code}" "$url/health" 2>/dev/null | tail -1)
    
    if [ "$response" = "200" ]; then
        echo "  ✅ Songbird running"
    else
        echo "  ⚠️  Songbird not responding (HTTP $response)"
    fi
done
echo ""

echo "========================================================================"
echo "  🚀 LAUNCHING MASTER (RANK 0) - LOCAL"
echo "========================================================================"
echo ""

# Create results directory
mkdir -p ../results/distributed_multi_tower

# Launch master locally (we're on Tower A)
echo "Starting master on Eastgate (local)..."

MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
python3 train_distributed.py \
    --rank 0 \
    --world-size $WORLD_SIZE \
    --epochs $EPOCHS \
    --batch-size $BATCH_SIZE \
    --output-dir ../results/distributed_multi_tower \
    > ../results/distributed_multi_tower/rank_0.log 2>&1 &

MASTER_PID=$!
echo $MASTER_PID > ../results/distributed_multi_tower/rank_0.pid

echo "  ✅ Master started (PID: $MASTER_PID)"
echo "  Log: ../results/distributed_multi_tower/rank_0.log"
echo ""

echo "⏳ Waiting 5 seconds for master to initialize..."
sleep 5
echo ""

echo "========================================================================"
echo "  🌐 LAUNCHING WORKERS VIA SONGBIRD API"
echo "========================================================================"
echo ""

# Function to launch worker via Songbird
launch_worker() {
    local rank=$1
    local url="${SONGBIRD_URLS[$rank]}"
    local name="${TOWER_NAMES[$rank]}"
    
    echo "Launching worker on $name (Rank $rank)..."
    
    # Training command to execute
    local cmd="cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT python3 train_distributed.py --rank $rank --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir ../results/distributed_multi_tower > ../results/distributed_multi_tower/rank_${rank}.log 2>&1 &"
    
    # Try Songbird's command execution API
    response=$(curl -s -X POST "$url/api/execute" \
        -H "Content-Type: application/json" \
        -d "{\"command\": \"$cmd\", \"async\": true}" \
        2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        echo "  ✅ Command sent to $name"
        echo "  Response: $response"
    else
        echo "  ⚠️  Songbird API call failed"
        echo "  "
        echo "  Manual launch required on $name:"
        echo "  ---"
        echo "  cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
        echo "  MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \\"
        echo "  python3 train_distributed.py \\"
        echo "    --rank $rank \\"
        echo "    --world-size $WORLD_SIZE \\"
        echo "    --epochs $EPOCHS \\"
        echo "    --batch-size $BATCH_SIZE \\"
        echo "    --output-dir ../results/distributed_multi_tower"
        echo "  ---"
    fi
    
    echo ""
}

# Launch workers
launch_worker 1
sleep 1
launch_worker 2

echo "========================================================================"
echo "  📊 MONITORING"
echo "========================================================================"
echo ""

echo "Monitor master progress:"
echo "  tail -f ../results/distributed_multi_tower/rank_0.log"
echo ""

echo "Check Songbird status:"
echo "  curl http://192.168.1.134:8000/health"
echo "  curl http://192.168.1.207:8000/health"
echo ""

echo "View all processes on towers:"
echo "  curl http://192.168.1.134:8000/api/processes | jq '.'"
echo "  curl http://192.168.1.207:8000/api/processes | jq '.'"
echo ""

echo "Stop training:"
echo "  kill \$(cat ../results/distributed_multi_tower/rank_0.pid)"
echo ""

echo "========================================================================"
echo "  ✅ LAUNCH SCRIPT COMPLETE"
echo "========================================================================"
echo ""

echo "If workers didn't start automatically:"
echo "  1. Check Songbird logs on worker towers"
echo "  2. Use manual launch commands provided above"
echo "  3. See MANUAL_LAUNCH_GUIDE.md for detailed instructions"
echo ""

