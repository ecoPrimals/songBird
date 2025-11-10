#!/bin/bash
# Distributed ML Training via Songbird + Toadstool

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Distributed ML Training via Songbird Intelligent Routing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
TOWER_A_IP="192.168.1.144"
TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"

MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29501"

TOADSTOOL_ROOT="/home/eastgate/Development/ecoPrimals/toadstool"
SONGBIRD_ROOT="/home/eastgate/Development/ecoPrimals/songbird"
TRAINING_SCRIPT="$SONGBIRD_ROOT/experiments/imagenet_training/training/train_distributed.py"
OUTPUT_DIR="/tmp/imagenet_distributed_results"

WORLD_SIZE=3
EPOCHS=2
BATCH_SIZE=64

echo "🔧 Configuration:"
echo "   Master: $MASTER_ADDR:$MASTER_PORT"
echo "   World Size: $WORLD_SIZE"
echo "   Epochs: $EPOCHS"
echo "   Batch Size: $BATCH_SIZE"
echo ""

# Ensure output directories exist
mkdir -p "$OUTPUT_DIR/rank_0"
mkdir -p "$OUTPUT_DIR/rank_1"
mkdir -p "$OUTPUT_DIR/rank_2"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🍄 Starting Toadstool Workers via CLI"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create workload specs for each rank
cat > /tmp/workload_rank_0.toml << WORKLOAD0
[metadata]
name = "imagenet-training-rank-0"
description = "Distributed ImageNet training - Master (Rank 0)"
version = "0.1.0"

[execution]
type = "python"
script = "$TRAINING_SCRIPT"
args = ["--rank", "0", "--world-size", "$WORLD_SIZE", "--epochs", "$EPOCHS", "--batch-size", "$BATCH_SIZE", "--output-dir", "$OUTPUT_DIR/rank_0"]
env = { MASTER_ADDR = "$MASTER_ADDR", MASTER_PORT = "$MASTER_PORT" }

[resources]
gpu = true
cpu_cores = 4.0
memory_mb = 8192
WORKLOAD0

cat > /tmp/workload_rank_1.toml << WORKLOAD1
[metadata]
name = "imagenet-training-rank-1"
description = "Distributed ImageNet training - Worker 1 (Rank 1)"
version = "0.1.0"

[execution]
type = "python"
script = "$TRAINING_SCRIPT"
args = ["--rank", "1", "--world-size", "$WORLD_SIZE", "--epochs", "$EPOCHS", "--batch-size", "$BATCH_SIZE", "--output-dir", "$OUTPUT_DIR/rank_1"]
env = { MASTER_ADDR = "$MASTER_ADDR", MASTER_PORT = "$MASTER_PORT" }

[resources]
gpu = true
cpu_cores = 4.0
memory_mb = 8192
WORKLOAD1

cat > /tmp/workload_rank_2.toml << WORKLOAD2
[metadata]
name = "imagenet-training-rank-2"
description = "Distributed ImageNet training - Worker 2 (Rank 2)"
version = "0.1.0"

[execution]
type = "python"
script = "$TRAINING_SCRIPT"
args = ["--rank", "2", "--world-size", "$WORLD_SIZE", "--epochs", "$EPOCHS", "--batch-size", "$BATCH_SIZE", "--output-dir", "$OUTPUT_DIR/rank_2"]
env = { MASTER_ADDR = "$MASTER_ADDR", MASTER_PORT = "$MASTER_PORT" }

[resources]
gpu = true
cpu_cores = 4.0
memory_mb = 8192
WORKLOAD2

echo "✅ Workload specs created"
echo ""

# Launch Master (Rank 0) on Tower A
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Launching Master (Rank 0) on Tower A"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "$TOADSTOOL_ROOT"
nohup cargo run --release --bin toadstool-cli -- execute /tmp/workload_rank_0.toml > "$OUTPUT_DIR/rank_0/toadstool.log" 2>&1 &
RANK_0_PID=$!
echo "✅ Rank 0 started (PID: $RANK_0_PID)"
echo "   Log: $OUTPUT_DIR/rank_0/toadstool.log"
echo ""

sleep 3

# Note: For Ranks 1 and 2, we'd need SSH or Execution Agent to start on remote towers
# For now, let's test with just Rank 0 locally
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Monitoring Training Progress"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Rank 0 Log:"
tail -f "$OUTPUT_DIR/rank_0/toadstool.log" &
TAIL_PID=$!

echo ""
echo "Press Ctrl+C to stop monitoring..."
echo ""

# Wait for user interrupt
trap "kill $TAIL_PID 2>/dev/null; echo ''; echo '🛑 Monitoring stopped'; exit 0" INT

wait $TAIL_PID
