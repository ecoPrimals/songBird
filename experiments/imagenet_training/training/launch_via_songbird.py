#!/usr/bin/env python3
"""
Launch distributed training via Songbird HTTP API
Uses Songbird orchestration instead of SSH
"""

import requests
import json
import time
import subprocess
import sys

# Songbird HTTP API endpoints
TOWERS = {
    0: {
        "name": "Eastgate",
        "gpu": "RTX 4070",
        "url": "http://192.168.1.144:8000",
        "is_master": True
    },
    1: {
        "name": "Strandgate", 
        "gpu": "RTX 3070",
        "url": "http://192.168.1.134:8000",
        "is_master": False
    },
    2: {
        "name": "Southgate",
        "gpu": "RTX 3090",
        "url": "http://192.168.1.207:8000",
        "is_master": False
    }
}

MASTER_ADDR = "192.168.1.144"
MASTER_PORT = "29500"
WORLD_SIZE = 3
EPOCHS = 2
BATCH_SIZE = 64

print("=" * 70)
print("  🚀 DISTRIBUTED TRAINING VIA SONGBIRD")
print("=" * 70)
print()

print("Configuration:")
print(f"  Master: {MASTER_ADDR}:{MASTER_PORT}")
print(f"  World size: {WORLD_SIZE} towers")
print(f"  Epochs: {EPOCHS}")
print(f"  Batch size per tower: {BATCH_SIZE}")
print()

# Check Songbird health on all towers
print("🔍 Checking Songbird status on all towers...")
for rank, tower in TOWERS.items():
    try:
        resp = requests.get(f"{tower['url']}/health", timeout=5)
        if resp.status_code == 200:
            data = resp.json()
            print(f"  ✅ {tower['name']}: Songbird running (uptime: {data.get('uptime_seconds', 0)}s)")
        else:
            print(f"  ⚠️  {tower['name']}: HTTP {resp.status_code}")
    except Exception as e:
        print(f"  ❌ {tower['name']}: {e}")
print()

# Prepare training command for each tower
def get_training_command(rank):
    """Generate training command for a specific rank"""
    cmd = f"""cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \
MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} \
python3 train_distributed.py \
--rank {rank} \
--world-size {WORLD_SIZE} \
--epochs {EPOCHS} \
--batch-size {BATCH_SIZE} \
--output-dir ../results/distributed_multi_tower \
> ../results/distributed_multi_tower/rank_{rank}.log 2>&1 &"""
    return cmd

print("=" * 70)
print("  🚀 LAUNCHING TRAINING ON ALL TOWERS")
print("=" * 70)
print()

# Create results directory locally
subprocess.run("mkdir -p ../results/distributed_multi_tower", shell=True)

# Launch master first (locally, since we're on Tower A)
if 0 in TOWERS and TOWERS[0]["is_master"]:
    print(f"Starting Rank 0 (Master) on {TOWERS[0]['name']} - LOCAL")
    
    master_cmd = f"""cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \
MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} \
python3 train_distributed.py \
--rank 0 \
--world-size {WORLD_SIZE} \
--epochs {EPOCHS} \
--batch-size {BATCH_SIZE} \
--output-dir ../results/distributed_multi_tower \
> ../results/distributed_multi_tower/rank_0.log 2>&1"""
    
    # Launch master in background
    proc = subprocess.Popen(master_cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    print(f"  Started PID: {proc.pid}")
    print(f"  Log: ../results/distributed_multi_tower/rank_0.log")
    
    # Save PID
    with open("../results/distributed_multi_tower/rank_0.pid", "w") as f:
        f.write(str(proc.pid))
    
    print("  ⏳ Waiting 5 seconds for master to initialize...")
    time.sleep(5)
    print()

# Launch workers via Songbird HTTP API
for rank in [1, 2]:
    if rank not in TOWERS:
        continue
    
    tower = TOWERS[rank]
    print(f"Starting Rank {rank} (Worker) on {tower['name']} via Songbird...")
    
    # Use Songbird's command execution endpoint
    command = get_training_command(rank)
    
    try:
        # Attempt to execute via Songbird API
        # This assumes Songbird has a command execution endpoint
        # Adjust based on actual Songbird API
        
        payload = {
            "command": command,
            "async": True
        }
        
        resp = requests.post(
            f"{tower['url']}/api/execute",
            json=payload,
            timeout=10
        )
        
        if resp.status_code == 200:
            result = resp.json()
            print(f"  ✅ Command sent to {tower['name']}")
            print(f"  Response: {result}")
        else:
            print(f"  ⚠️  HTTP {resp.status_code}: {resp.text[:100]}")
            print(f"  Attempting direct deployment...")
            
            # Fallback: Try to deploy the training script as a service
            # This would use Songbird's deployment API
            print(f"  Note: May need manual launch on {tower['name']}")
            print(f"  Command: {command}")
    
    except Exception as e:
        print(f"  ❌ Error: {e}")
        print(f"  Manual launch required on {tower['name']}")
        print(f"  Command: MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} \\")
        print(f"           python3 train_distributed.py --rank {rank} --world-size {WORLD_SIZE}")
    
    print()
    time.sleep(1)

print("=" * 70)
print("  📊 MONITORING")
print("=" * 70)
print()

print("Monitor master progress:")
print("  tail -f ../results/distributed_multi_tower/rank_0.log")
print()

print("Check worker logs via Songbird:")
print(f"  curl http://192.168.1.134:8000/api/logs | grep train_distributed")
print(f"  curl http://192.168.1.207:8000/api/logs | grep train_distributed")
print()

print("Stop training:")
print("  kill $(cat ../results/distributed_multi_tower/rank_0.pid)")
print()

print("🎯 Note: If workers didn't start via Songbird API:")
print("   Manual launch commands provided above")
print("   Or see: MANUAL_LAUNCH_GUIDE.md")
print()

