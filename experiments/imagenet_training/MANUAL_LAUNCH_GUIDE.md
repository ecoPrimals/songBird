# 🚀 Manual Multi-Tower Training Launch Guide

**Status:** Ready for Distributed Training  
**Towers:** 3 (Eastgate, Strandgate, Southgate)  
**Dataset:** Sharded and ready (100,000 images)

---

## 📋 Prerequisites

### On All Towers:
```bash
# 1. Verify code is present
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training

# 2. Verify data shards exist
ls -lh ../data/imagenet100/sharded/
# Should see: shard_0, shard_1, shard_2

# 3. Verify Python environment
python3 -c "import torch; print(f'PyTorch: {torch.__version__}, CUDA: {torch.cuda.is_available()}')"

# 4. Check GPU
nvidia-smi
```

---

## 🌐 Launch Commands

### Tower A (Eastgate) - Master - Rank 0
```bash
# Open terminal on Tower A
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

# Set environment variables
export MASTER_ADDR=192.168.1.144
export MASTER_PORT=29500

# Launch training
python3 train_distributed.py \
    --rank 0 \
    --world-size 3 \
    --epochs 2 \
    --batch-size 64 \
    --output-dir ../results/distributed_multi_tower \
    2>&1 | tee ../results/distributed_multi_tower/rank_0.log
```

### Tower B (Strandgate) - Worker - Rank 1
```bash
# SSH or physically access Tower B
ssh 192.168.1.134

cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

# Set environment variables (point to master)
export MASTER_ADDR=192.168.1.144
export MASTER_PORT=29500

# Launch training
python3 train_distributed.py \
    --rank 1 \
    --world-size 3 \
    --epochs 2 \
    --batch-size 64 \
    --output-dir ../results/distributed_multi_tower \
    2>&1 | tee ../results/distributed_multi_tower/rank_1.log
```

### Tower C (Southgate) - Worker - Rank 2
```bash
# SSH or physically access Tower C
ssh 192.168.1.207

cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

# Set environment variables (point to master)
export MASTER_ADDR=192.168.1.144
export MASTER_PORT=29500

# Launch training
python3 train_distributed.py \
    --rank 2 \
    --world-size 3 \
    --epochs 2 \
    --batch-size 64 \
    --output-dir ../results/distributed_multi_tower \
    2>&1 | tee ../results/distributed_multi_tower/rank_2.log
```

---

## ⚡ Quick Launch (All Towers at Once)

**Important:** Launch in this order:
1. **First:** Start Master (Rank 0) on Tower A
2. **Wait 5 seconds** for master to initialize
3. **Then:** Start Workers (Rank 1 & 2) on Towers B & C

**Timing matters!** Workers need the master to be listening.

---

## 📊 Monitoring

### Watch Master Progress:
```bash
# On Tower A
tail -f ../results/distributed_multi_tower/rank_0.log
```

### Watch Worker Progress:
```bash
# On Tower B
tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_multi_tower/rank_1.log

# On Tower C
tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_multi_tower/rank_2.log
```

### Check GPU Utilization:
```bash
# On any tower
watch -n 1 nvidia-smi
```

---

## ✅ Success Indicators

### Master (Rank 0) Log Should Show:
```
======================================================================
  🚀 DISTRIBUTED MULTI-GPU TRAINING
======================================================================

World size: 3 GPUs
Rank: 0
Device: cuda:0
GPU: NVIDIA GeForce RTX 4070
...

Epoch [1/2]
  Batch [0/524] Loss: 5.XXXX Acc: X.XX% Time: X.XXXs/batch
  ...
  Total Throughput: ~450-500 images/sec  ← MUCH HIGHER than 166.7!
  Speedup vs baseline: 2.7-3.0x
```

### Workers (Rank 1 & 2) Should Show:
```
World size: 3 GPUs
Rank: 1 (or 2)
Device: cuda:0
...
(Similar training progress, but quieter - only master prints summaries)
```

---

## 🔍 Troubleshooting

### Error: "Connection refused" or "Timeout"
**Cause:** Master not started or firewall blocking  
**Fix:**
```bash
# On Tower A, check if port is open
sudo ufw allow 29500/tcp
# Or disable firewall temporarily
sudo ufw disable
```

### Error: "Address already in use"
**Cause:** Previous training still running  
**Fix:**
```bash
# Kill old processes
pkill -f train_distributed.py
# Wait 5 seconds, then restart
```

### Error: "NCCL error" or "Distributed timeout"
**Cause:** Network latency or incompatible NCCL versions  
**Fix:**
```bash
# Increase timeout (on all towers)
export NCCL_TIMEOUT=300  # 5 minutes
# Then restart training
```

### Ranks Get Out of Sync
**Cause:** One tower started too late  
**Fix:**
1. Stop all training (Ctrl+C on all towers)
2. Restart IN ORDER: Master first, then workers

---

## 📈 Expected Performance

### Baseline (Single GPU):
```
Throughput: 166.7 images/sec
Epoch Time: 220 seconds
```

### Distributed (3 GPUs):
```
Tower A (RTX 4070 12GB): ~200 images/sec
Tower B (RTX 3070  8GB): ~150 images/sec
Tower C (RTX 3090 24GB): ~300 images/sec
───────────────────────────────────────────
Total Throughput: ~650 images/sec (ideal)
With 15% overhead: ~500 images/sec (realistic)
Speedup: 3.0x (ideal) / 2.7x (realistic)

Epoch Time: ~75-90 seconds (vs 220 seconds)
Training Time (2 epochs): ~3 minutes (vs 7 minutes)
```

---

## 📊 Results Location

After training completes:

```bash
# View final results
cat ../results/distributed_multi_tower/results.json

# Check speedup
jq '.final_speedup' ../results/distributed_multi_tower/results.json

# Compare with baseline
echo "Baseline: 166.7 images/sec"
jq '.results[-1].images_per_sec' ../results/distributed_multi_tower/results.json
```

---

## 🎯 What to Expect

### Timeline:
- **Master starts:** Immediately shows "Waiting for workers..."
- **Workers join:** Within 5-10 seconds
- **Training begins:** All towers show progress
- **Synchronization:** Happens automatically every batch
- **Epoch completes:** Master shows summary
- **Training finishes:** ~3-4 minutes for 2 epochs

### Network Traffic:
- **Gradient sync:** ~100MB every few seconds
- **Bandwidth used:** ~500 Mbps sustained
- **Latency:** < 1ms (verified earlier)

---

## 🚀 After Successful Run

### Generate Comparison:
```python
# Compare baseline vs distributed
python3 << 'EOF'
import json

with open('../results/baseline/results.json') as f:
    baseline = json.load(f)

with open('../results/distributed_multi_tower/results.json') as f:
    distributed = json.load(f)

print("=" * 60)
print("  PERFORMANCE COMPARISON")
print("=" * 60)
print()
print(f"Baseline (1 GPU):")
print(f"  Throughput: {baseline['results'][-1]['images_per_sec']:.1f} img/sec")
print(f"  Epoch time: {baseline['results'][-1]['epoch_time']:.1f}s")
print()
print(f"Distributed (3 GPUs):")
print(f"  Throughput: {distributed['results'][-1]['images_per_sec']:.1f} img/sec")
print(f"  Epoch time: {distributed['results'][-1]['epoch_time']:.1f}s")
print()
print(f"Speedup: {distributed['final_speedup']:.2f}x")
print(f"Time saved: {baseline['results'][-1]['epoch_time'] - distributed['results'][-1]['epoch_time']:.1f}s per epoch")
print()
EOF
```

---

## 📸 Demo Screenshots

### For Prof. Murillo Presentation:

1. **Terminal Split Screen:**
   - Left: Master training progress
   - Right: Worker GPU utilization (nvidia-smi)

2. **Key Metrics to Capture:**
   - Throughput: ~500 images/sec (vs 166.7)
   - Speedup: 2.7-3.0x
   - Epoch time: ~80s (vs 220s)
   - All 3 GPUs showing activity

3. **Network Monitor:**
   - Show gradient sync happening
   - Network bandwidth usage

---

## 🎓 Educational Value

**Students Learn:**
1. **Distributed Systems:** Real multi-node coordination
2. **Network Programming:** NCCL, all-reduce, gradient sync
3. **GPU Computing:** Heterogeneous hardware management
4. **System Design:** Load balancing, fault tolerance
5. **Cost Analysis:** Local vs cloud economics

---

## 📞 Support

**If Issues Arise:**
1. Check `../results/distributed_multi_tower/rank_*.log`
2. Verify network with `ping` and `iperf3`
3. Check GPU with `nvidia-smi`
4. Review this guide's troubleshooting section

**Success Criteria:**
✅ All 3 ranks initialize  
✅ Training starts on all towers  
✅ Throughput > 400 images/sec  
✅ Speedup > 2.5x  
✅ Epoch completes successfully  

---

**Ready to launch!** 🚀

Follow the commands above in order, and you'll have distributed training across your 3-tower HPC!

