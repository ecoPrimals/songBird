#!/bin/bash
# Launch training workers on Towers B and C

TOWER_B_IP="192.168.1.134"
TOWER_C_IP="192.168.1.207"
MASTER_ADDR="192.168.1.144"
MASTER_PORT="29507"

echo "🚀 Launching Workers on Towers B & C..."
echo ""

# Tower B
echo "Tower B (RTX 3070 - Rank 1)..."
ssh strandgate "cd /home/strandgate/Development/songbird/experiments/imagenet_training/training && mkdir -p /tmp/imagenet100_full_training/rank_1 && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 nohup python3 train_distributed.py --rank 1 --world-size 3 --epochs 90 --batch-size 64 --output-dir /tmp/imagenet100_full_training/rank_1 > /tmp/imagenet100_full_training/rank_1/training.log 2>&1 &"
echo "✅ Tower B launched"

# Tower C  
echo "Tower C (RTX 3090 - Rank 2)..."
ssh southgate "cd /home/southgate/Development/songbird/experiments/imagenet_training/training && mkdir -p /tmp/imagenet100_full_training/rank_2 && MASTER_ADDR=$MASTER_ADDR MASTER_PORT=$MASTER_PORT CUDA_VISIBLE_DEVICES=0 nohup python3 train_distributed.py --rank 2 --world-size 3 --epochs 90 --batch-size 64 --output-dir /tmp/imagenet100_full_training/rank_2 > /tmp/imagenet100_full_training/rank_2/training.log 2>&1 &"
echo "✅ Tower C launched"

echo ""
echo "Workers launched! Check with: tail -f /tmp/imagenet100_full_training/rank_0/training.log"

