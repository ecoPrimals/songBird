#!/bin/bash
# Quick launcher for distributed training on THIS tower (Master)
# Workers must be launched manually on Towers B and C

cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

echo "================================================================================"
echo "  🚀 LAUNCHING DISTRIBUTED TRAINING - MASTER NODE"
echo "================================================================================"
echo ""
echo "Tower A (Eastgate): Master (Rank 0)"
echo "GPU: NVIDIA RTX 2070 SUPER (8GB)"
echo ""
echo "Waiting for workers to connect from:"
echo "  - Tower B (Strandgate): Rank 1"
echo "  - Tower C (Southgate): Rank 2"
echo ""
echo "Master will wait up to 10 minutes for workers..."
echo "Launch workers NOW using commands from LAUNCH_DISTRIBUTED_NOW.md"
echo ""
echo "───────────────────────────────────────────────────────────────────────────────"
echo ""
echo "Starting in 5 seconds..."
sleep 5

echo "🚀 MASTER STARTING..."
echo ""

MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 train_distributed.py \
  --rank 0 \
  --world-size 3 \
  --epochs 20 \
  --batch-size 64 \
  --output-dir ../results/distributed_20epochs \
  2>&1 | tee ../results/dist_rank0.log

