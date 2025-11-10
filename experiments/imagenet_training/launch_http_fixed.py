#!/usr/bin/env python3
"""
Launch Distributed ImageNet Training via HTTP with CORRECT paths for each tower
"""

import requests
import json
import time
import subprocess
import os
from pathlib import Path

# Configuration - DIFFERENT PATHS PER TOWER!
TOWER_A_DIR = "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
TOWER_B_DIR = "/home/strandgate/Development/songbird/experiments/imagenet_training/training"
TOWER_C_DIR = "/home/southgate/Development/songbird/experiments/imagenet_training/training"

OUTPUT_DIR = Path(__file__).parent / "results" / "distributed_http_fixed"
MASTER_ADDR = "192.168.1.144"
MASTER_PORT = "29500"
WORLD_SIZE = 3
EPOCHS = 2
BATCH_SIZE = 64

# Tower execution agent endpoints
TOWERS = {
    "tower-b": {
        "endpoint": "http://192.168.1.134:9020",
        "training_dir": TOWER_B_DIR
    },
    "tower-c": {
        "endpoint": "http://192.168.1.207:9020",
        "training_dir": TOWER_C_DIR
    }
}

def launch_remote_worker(tower_name, tower_info, rank):
    """Launch a worker on a remote tower via HTTP"""
    
    print(f"📍 Launching Worker {rank} on {tower_name}...")
    print(f"   Path: {tower_info['training_dir']}")
    
    # Create the command - use bash -c with proper path
    command = f"cd {tower_info['training_dir']} && MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} python3 train_distributed.py --rank {rank} --world-size {WORLD_SIZE} --epochs {EPOCHS} --batch-size {BATCH_SIZE} --output-dir {OUTPUT_DIR}"
    
    request_data = {
        "command": command,
        "env": {},
        "background": True,
        "capture_output": True,
        "timeout_seconds": 1800
    }
    
    try:
        response = requests.post(
            f"{tower_info['endpoint']}/api/v1/execution/command",
            json=request_data,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 200:
            result = response.json()
            job_id = result.get("job_id", "unknown")
            print(f"✅ Worker {rank} launched on {tower_name}")
            print(f"   Job ID: {job_id}")
            return job_id
        else:
            print(f"❌ Failed to launch worker {rank} on {tower_name}")
            print(f"   Status: {response.status_code}")
            print(f"   Response: {response.text}")
            return None
            
    except Exception as e:
        print(f"❌ Error launching worker {rank} on {tower_name}: {e}")
        return None

def launch_local_master():
    """Launch the master process locally"""
    
    print("📍 Launching Master (Rank 0) on Tower A (Local)...")
    print(f"   Path: {TOWER_A_DIR}")
    
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    cmd = [
        "python3", f"{TOWER_A_DIR}/train_distributed.py",
        "--rank", "0",
        "--world-size", str(WORLD_SIZE),
        "--epochs", str(EPOCHS),
        "--batch-size", str(BATCH_SIZE),
        "--output-dir", str(OUTPUT_DIR)
    ]
    
    env = os.environ.copy()
    env["MASTER_ADDR"] = MASTER_ADDR
    env["MASTER_PORT"] = MASTER_PORT
    
    log_file = OUTPUT_DIR / "rank_0.log"
    with open(log_file, "w") as f:
        proc = subprocess.Popen(
            cmd,
            env=env,
            stdout=f,
            stderr=subprocess.STDOUT,
            cwd=TOWER_A_DIR
        )
    
    print(f"✅ Master launched (PID: {proc.pid})")
    print(f"   Log: {log_file}")
    
    return proc

def main():
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("🚀 Launching Distributed ImageNet Training - FIXED PATHS!")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    print("Configuration:")
    print(f"  • World Size: {WORLD_SIZE} towers")
    print(f"  • Epochs: {EPOCHS}")
    print(f"  • Batch Size: {BATCH_SIZE}")
    print(f"  • Master: {MASTER_ADDR}:{MASTER_PORT}")
    print(f"  • Output: {OUTPUT_DIR}")
    print("")
    print("Paths:")
    print(f"  • Tower A: {TOWER_A_DIR}")
    print(f"  • Tower B: {TOWER_B_DIR}")
    print(f"  • Tower C: {TOWER_C_DIR}")
    print("")
    
    # Step 1: Launch master locally
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    master_proc = launch_local_master()
    print("")
    
    # Give master time to initialize
    print("⏳ Waiting 5 seconds for master to initialize...")
    time.sleep(5)
    print("")
    
    # Step 2: Launch worker 1 on Tower B
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    worker1_job = launch_remote_worker("Tower B (Strandgate)", TOWERS["tower-b"], rank=1)
    print("")
    
    # Step 3: Launch worker 2 on Tower C
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    worker2_job = launch_remote_worker("Tower C (Southgate)", TOWERS["tower-c"], rank=2)
    print("")
    
    # Summary
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("🎯 Training Launched with Correct Paths!")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    print("Monitor:")
    print(f"  • Master log: tail -f {OUTPUT_DIR}/rank_0.log")
    print(f"  • GPU usage:  watch -n 2 nvidia-smi")
    print("")
    print("Expected: ~3 minutes for 2 epochs with 3 GPUs")
    print("")

if __name__ == "__main__":
    main()

