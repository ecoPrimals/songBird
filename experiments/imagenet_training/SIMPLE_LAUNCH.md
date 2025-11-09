# 🚀 Simple 3-Tower Launch Guide

**Ready for Distributed Training!**

## ✅ Status Check

```
Tower B (Strandgate): ✅ Squirrel running (port 9011, 3901s uptime)
Tower C (Southgate):  ✅ Squirrel running (port 9012, 3899s uptime)
Network: ✅ < 1ms latency
Data: ✅ 100,000 images sharded
Code: ✅ All scripts ready
```

---

## 🎯 Quick Launch (3 Commands)

### Step 1: Launch Master (Tower A - This machine)

```bash
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

# Create results directory
mkdir -p ../results/distributed_3tower

# Launch master
MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 train_distributed.py \
  --rank 0 \
  --world-size 3 \
  --epochs 2 \
  --batch-size 64 \
  --output-dir ../results/distributed_3tower \
  2>&1 | tee ../results/distributed_3tower/rank_0.log
```

**Wait for:** "Waiting for workers to connect..." message

---

### Step 2: Launch Worker 1 (Tower B - Strandgate)

**On Tower B (physically or via remote terminal):**

```bash
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 train_distributed.py \
  --rank 1 \
  --world-size 3 \
  --epochs 2 \
  --batch-size 64 \
  --output-dir ../results/distributed_3tower \
  2>&1 | tee ../results/distributed_3tower/rank_1.log
```

---

### Step 3: Launch Worker 2 (Tower C - Southgate)

**On Tower C (physically or via remote terminal):**

```bash
cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training

MASTER_ADDR=192.168.1.144 MASTER_PORT=29500 \
python3 train_distributed.py \
  --rank 2 \
  --world-size 3 \
  --epochs 2 \
  --batch-size 64 \
  --output-dir ../results/distributed_3tower \
  2>&1 | tee ../results/distributed_3tower/rank_2.log
```

---

## 📊 What to Expect

### Timeline:
```
0s:    Master starts, waits for workers
5-10s: Worker 1 connects
10-15s: Worker 2 connects
15s:   Training begins!
~90s:  Epoch 1 complete
~180s: Epoch 2 complete
~3min: Training finished!
```

### Output You'll See:

**Master (Rank 0):**
```
======================================================================
  🚀 DISTRIBUTED MULTI-GPU TRAINING
======================================================================

World size: 3 GPUs
...
Epoch [1/2]
  Total Throughput: ~450-550 images/sec  ← MUCH FASTER!
  Speedup vs baseline: 2.7-3.3x
  Epoch time: ~80s
```

**Workers (Rank 1 & 2):**
```
World size: 3 GPUs
Rank: 1 (or 2)
(Training progress - quieter, master shows summaries)
```

---

## 🎯 Expected Performance

```
Baseline (1 GPU):      166.7 images/sec
Distributed (3 GPUs):  ~500 images/sec
Speedup:               3.0x
Time per epoch:        ~80s (vs 220s)
Total training:        ~3 min (vs 7 min)
```

---

## 📈 Monitoring

### Watch Progress:
```bash
# On Tower A
tail -f ../results/distributed_3tower/rank_0.log

# On Tower B  
tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_3tower/rank_1.log

# On Tower C
tail -f /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/results/distributed_3tower/rank_2.log
```

### Watch GPUs:
```bash
# On any tower
watch -n 1 nvidia-smi
```

---

## 🔍 Troubleshooting

### "Connection timeout" or "Workers not connecting"
1. Make sure master started first
2. Check firewall: `sudo ufw allow 29500/tcp`
3. Verify MASTER_ADDR is correct (192.168.1.144)

### "Port already in use"
```bash
# Kill old processes
pkill -f train_distributed
# Wait 5s, restart
```

### One tower fails
- Stop all (Ctrl+C)
- Restart in order: Master → Worker 1 → Worker 2

---

## ✅ Success Indicators

- [ ] Master shows "Waiting for workers..."
- [ ] Worker 1 connects
- [ ] Worker 2 connects  
- [ ] Training begins on all towers
- [ ] Throughput > 400 images/sec
- [ ] Speedup > 2.5x shown
- [ ] Epoch completes successfully

---

## 📊 After Training

### View Results:
```bash
cat ../results/distributed_3tower/results.json

# Show speedup
jq '.final_speedup' ../results/distributed_3tower/results.json

# Compare
echo "Baseline: 166.7 images/sec"
jq '.results[-1].images_per_sec' ../results/distributed_3tower/results.json
```

### Generate Comparison:
```bash
python3 << 'EOF'
import json

baseline = json.load(open('../results/baseline/results.json'))
distributed = json.load(open('../results/distributed_3tower/results.json'))

print("PERFORMANCE COMPARISON")
print("=" * 60)
print(f"Baseline:    {baseline['results'][-1]['images_per_sec']:.1f} img/sec")
print(f"Distributed: {distributed['results'][-1]['images_per_sec']:.1f} img/sec")
print(f"Speedup:     {distributed['final_speedup']:.2f}x")
print(f"Time saved:  {baseline['results'][-1]['epoch_time'] - distributed['results'][-1]['epoch_time']:.1f}s per epoch")
EOF
```

---

## 🚀 Ready to Launch!

**Three simple commands, one on each tower, and you're training!**

The PyTorch DDP handles all the distributed coordination automatically via NCCL.

**Start with Master (Tower A), then Workers (B & C) within 30 seconds.**

Good luck! 🎉

