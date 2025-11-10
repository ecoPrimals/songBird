#!/bin/bash
# Launch Full 90 Epoch Training NOW - All 3 GPUs

TOWER_A_IP="192.168.1.144"
TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
AGENT_PORT="9020"
MASTER_ADDR="$TOWER_A_IP"
MASTER_PORT="29507"
WORLD_SIZE=3
EPOCHS=90
BATCH_SIZE=64
OUTPUT_DIR="/tmp/imagenet100_full_training"

echo "🚀 Launching 90 Epoch Training on 3 GPUs..."
echo ""

# Tower A (Local)
echo "Tower A (RTX 2070 Super - Rank 0)..."
mkdir -p "$OUTPUT_DIR/rank_0"
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training
MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 \
  python3 train_distributed.py --rank 0 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir "$OUTPUT_DIR/rank_0" > "$OUTPUT_DIR/rank_0/training.log" 2>&1 &
echo "✅ Rank 0 launched (PID: $!)"
sleep 3

# Tower B
echo "Tower B (RTX 3070 - Rank 1)..."
curl -s -X POST "http://$TOWER_B_IP:$AGENT_PORT/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d "{\"command\":\"bash\",\"args\":[\"-c\",\"mkdir -p $OUTPUT_DIR/rank_1 && cd /home/strandgate/Development/songbird/experiments/imagenet_training/training && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank 1 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $OUTPUT_DIR/rank_1 > $OUTPUT_DIR/rank_1/training.log 2>&1 &\"],\"env\":{},\"background\":true}" > /dev/null
echo "✅ Rank 1 launched"
sleep 2

# Tower C
echo "Tower C (RTX 3090 - Rank 2)..."
curl -s -X POST "http://$TOWER_C_IP:$AGENT_PORT/api/v1/execution/command" \
  -H "Content-Type: application/json" \
  -d "{\"command\":\"bash\",\"args\":[\"-c\",\"mkdir -p $OUTPUT_DIR/rank_2 && cd /home/southgate/Development/songbird/experiments/imagenet_training/training && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 python3 train_distributed.py --rank 2 --world-size $WORLD_SIZE --epochs $EPOCHS --batch-size $BATCH_SIZE --output-dir $OUTPUT_DIR/rank_2 > $OUTPUT_DIR/rank_2/training.log 2>&1 &\"],\"env\":{},\"background\":true}" > /dev/null
echo "✅ Rank 2 launched"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ FULL 90 EPOCH TRAINING LAUNCHED ON ALL 3 GPUS!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • RTX 2070 Super + RTX 3070 + RTX 3090"
echo "  • 90 epochs (~15-18 hours)"
echo "  • ImageNet-100 (250K images)"
echo "  • ResNet-50 (25.6M params)"
echo ""
echo "Monitor: tail -f $OUTPUT_DIR/rank_0/training.log"
echo "GPU: nvidia-smi"
echo ""

