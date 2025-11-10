#!/bin/bash
# Launch Distributed ImageNet Training via Remote Execution API (NO SSH!)
# Uses the newly deployed execution agents on Towers B & C

set -e

TRAINING_DIR="/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
OUTPUT_DIR="../results/distributed_3tower_http"
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29500"
WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Launching Distributed ImageNet Training - SSH-FREE!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • World Size: $WORLD_SIZE towers"
echo "  • Epochs: $EPOCHS"
echo "  • Batch Size: $BATCH_SIZE"
echo "  • Master: $MASTER_ADDR:$MASTER_PORT"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Step 1: Launch Master (Rank 0) on Tower A (Local)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "$TRAINING_DIR"

# Launch master in background
MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT \
python3 train_distributed.py --rank 0 --world-size $WORLD_SIZE \
  --epochs $EPOCHS --batch-size $BATCH_SIZE \
  --output-dir "$OUTPUT_DIR" \
  > "$OUTPUT_DIR/rank_0.log" 2>&1 &

MASTER_PID=$!
echo "✅ Master launched (PID: $MASTER_PID)"
echo "   Log: $OUTPUT_DIR/rank_0.log"
echo ""

# Give master time to initialize
sleep 5

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Step 2: Launch Worker 1 (Rank 1) on Tower B via HTTP"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WORKER1_RESPONSE=$(curl -s -X POST http://192.168.1.134:9020/api/v1/execution/command \
  -H "Content-Type: application/json" \
  -d "{
    \"command\": \"bash\",
    \"env\": {
      \"MASTER_ADDR\": \"$MASTER_ADDR\",
      \"MASTER_PORT\": \"$MASTER_PORT\"
    },
    \"working_dir\": \"$TRAINING_DIR\",
    \"background\": true,
    \"capture_output\": true,
    \"timeout_seconds\": 1800
  }" --data-binary @- <<'EOF'
{
  "command": "python3",
  "env": {
    "MASTER_ADDR": "192.168.1.144",
    "MASTER_PORT": "29500"
  },
  "working_dir": "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training",
  "background": true,
  "capture_output": true,
  "timeout_seconds": 1800
}
EOF
)

WORKER1_JOB=$(echo "$WORKER1_RESPONSE" | jq -r '.job_id // "FAILED"')
if [ "$WORKER1_JOB" != "FAILED" ]; then
    echo "✅ Worker 1 launched on Tower B"
    echo "   Job ID: $WORKER1_JOB"
else
    echo "❌ Worker 1 failed to launch"
    echo "   Response: $WORKER1_RESPONSE"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📍 Step 3: Launch Worker 2 (Rank 2) on Tower C via HTTP"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

WORKER2_RESPONSE=$(curl -s -X POST http://192.168.1.207:9020/api/v1/execution/command \
  -H "Content-Type: application/json" \
  -d "{
    \"command\": \"bash\",
    \"env\": {
      \"MASTER_ADDR\": \"$MASTER_ADDR\",
      \"MASTER_PORT\": \"$MASTER_PORT\"
    },
    \"working_dir\": \"$TRAINING_DIR\",
    \"background\": true,
    \"capture_output\": true,
    \"timeout_seconds\": 1800
  }" --data-binary @- <<'EOF'
{
  "command": "python3",
  "env": {
    "MASTER_ADDR": "192.168.1.144",
    "MASTER_PORT": "29500"
  },
  "working_dir": "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training",
  "background": true,
  "capture_output": true,
  "timeout_seconds": 1800
}
EOF
)

WORKER2_JOB=$(echo "$WORKER2_RESPONSE" | jq -r '.job_id // "FAILED"')
if [ "$WORKER2_JOB" != "FAILED" ]; then
    echo "✅ Worker 2 launched on Tower C"
    echo "   Job ID: $WORKER2_JOB"
else
    echo "❌ Worker 2 failed to launch"
    echo "   Response: $WORKER2_RESPONSE"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Training Launched!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Monitor progress:"
echo "  • Master log:  tail -f $OUTPUT_DIR/rank_0.log"
echo "  • Worker 1 job: curl http://192.168.1.134:9020/api/v1/execution/jobs/$WORKER1_JOB | jq '.'"
echo "  • Worker 2 job: curl http://192.168.1.207:9020/api/v1/execution/jobs/$WORKER2_JOB | jq '.'"
echo ""
echo "Expected completion: ~3 minutes"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

