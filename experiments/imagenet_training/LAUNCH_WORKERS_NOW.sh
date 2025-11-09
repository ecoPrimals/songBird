#!/bin/bash
# Worker launch commands for distributed training
# Copy these to Towers B and C

echo "==============================================================================="
echo "  DISTRIBUTED TRAINING WORKER COMMANDS"
echo "==============================================================================="
echo ""
echo "Master is running on Tower A and waiting for workers!"
echo ""
echo "───────────────────────────────────────────────────────────────────────────────"
echo "TOWER B (STRANDGATE) - Execute this:"
echo "───────────────────────────────────────────────────────────────────────────────"
echo ""
cat << 'TOWER_B'
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \
MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 -u train_distributed.py --rank 1 --world-size 3 --epochs 2 --batch-size 64 \
--output-dir ../results/test_distributed
TOWER_B
echo ""
echo "───────────────────────────────────────────────────────────────────────────────"
echo "TOWER C (SOUTHGATE) - Execute this:"
echo "───────────────────────────────────────────────────────────────────────────────"
echo ""
cat << 'TOWER_C'
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \
MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 -u train_distributed.py --rank 2 --world-size 3 --epochs 2 --batch-size 64 \
--output-dir ../results/test_distributed
TOWER_C
echo ""
echo "==============================================================================="
echo ""
echo "After launching both workers, training will start automatically!"
echo "Expected time: ~5 minutes for 2 epochs (test run)"
echo ""

