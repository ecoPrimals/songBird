#!/usr/bin/env python3
"""
Launch Distributed ImageNet Training via HTTP Remote Execution (NO SSH!)
Uses the newly deployed execution agents on Towers B & C
"""

import requests
import json
import time
import subprocess
import os
from pathlib import Path

# Configuration
TRAINING_DIR = "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"
OUTPUT_DIR = Path(__file__).parent / "results" / "distributed_http"
MASTER_ADDR = "192.168.1.144"
MASTER_PORT = "29500"
WORLD_SIZE = 3
EPOCHS = 2
BATCH_SIZE = 64

# Tower execution agent endpoints
TOWERS = {
    "tower-b": "http://192.168.1.134:9020",
    "tower-c": "http://192.168.1.207:9020"
}

def launch_remote_worker(tower_name, endpoint, rank):
    """Launch a worker on a remote tower via HTTP"""
    
    print(f"📍 Launching Worker {rank} on {tower_name}...")
    
    # Construct the Python command
    python_cmd = (
        f"cd {TRAINING_DIR} && "
        f"MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} "
        f"python3 train_distributed.py "
        f"--rank {rank} --world-size {WORLD_SIZE} "
        f"--epochs {EPOCHS} --batch-size {BATCH_SIZE} "
        f"--output-dir {OUTPUT_DIR} "
        f"> {OUTPUT_DIR}/rank_{rank}.log 2>&1"
    )
    
    # Create execution request
    request_data = {
        "command": "bash",
        "env": {
            "MASTER_ADDR": MASTER_ADDR,
            "MASTER_PORT": MASTER_PORT
        },
        "working_dir": TRAINING_DIR,
        "background": True,
        "capture_output": True,
        "timeout_seconds": 1800
    }
    
    # Actually, let's use a simpler approach - send the python command via bash
    request_data = {
        "command": f"bash -c 'cd {TRAINING_DIR} && MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} python3 train_distributed.py --rank {rank} --world-size {WORLD_SIZE} --epochs {EPOCHS} --batch-size {BATCH_SIZE} --output-dir {OUTPUT_DIR}'",
        "env": {},
        "background": True,
        "capture_output": True,
        "timeout_seconds": 1800
    }
    
    try:
        response = requests.post(
            f"{endpoint}/api/v1/execution/command",
            json=request_data,
            headers={"Content-Type": "application/json"},
            timeout=10
        )
        
        if response.status_code == 200:
            result = response.json()
            job_id = result.get("job_id", "unknown")
            print(f"✅ Worker {rank} launched on {tower_name}")
            print(f"   Job ID: {job_id}")
            print(f"   Monitor: curl {endpoint}/api/v1/execution/jobs/{job_id} | jq '.'")
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
    
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    cmd = [
        "python3", f"{TRAINING_DIR}/train_distributed.py",
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
            cwd=TRAINING_DIR
        )
    
    print(f"✅ Master launched (PID: {proc.pid})")
    print(f"   Log: {log_file}")
    
    return proc

def main():
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("🚀 Launching Distributed ImageNet Training - SSH-FREE!")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    print("Configuration:")
    print(f"  • World Size: {WORLD_SIZE} towers")
    print(f"  • Epochs: {EPOCHS}")
    print(f"  • Batch Size: {BATCH_SIZE}")
    print(f"  • Master: {MASTER_ADDR}:{MASTER_PORT}")
    print(f"  • Output: {OUTPUT_DIR}")
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
    print("🎯 Training Launched!")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    print("Monitor progress:")
    print(f"  • Master log:    tail -f {OUTPUT_DIR}/rank_0.log")
    if worker1_job:
        print(f"  • Worker 1 job:  curl {TOWERS['tower-b']}/api/v1/execution/jobs/{worker1_job} | jq '.'")
    if worker2_job:
        print(f"  • Worker 2 job:  curl {TOWERS['tower-c']}/api/v1/execution/jobs/{worker2_job} | jq '.'")
    print("")
    print("Expected completion: ~3 minutes")
    print("")
    print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
    print("")
    print("💡 This is using HTTP-based remote execution (NO SSH!)")
    print("   The execution agents you just deployed are coordinating everything.")
    print("")

if __name__ == "__main__":
    main()

