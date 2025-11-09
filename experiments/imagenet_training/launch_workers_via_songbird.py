#!/usr/bin/env python3
"""
Launch distributed training workers via Songbird HTTP API
This allows remote execution without SSH
"""

import requests
import json
import time
import sys
from pathlib import Path

# Tower configurations
TOWERS = {
    'strandgate': {
        'name': 'Tower B (Strandgate)',
        'rank': 1,
        'api_url': 'http://192.168.1.191:9011',  # Squirrel port on Tower B
        'gpu': 'RTX 3070'
    },
    'southgate': {
        'name': 'Tower C (Southgate)', 
        'rank': 2,
        'api_url': 'http://192.168.1.207:9012',  # Squirrel port on Tower C
        'gpu': 'RTX 3090'
    }
}

MASTER_ADDR = "192.168.1.144"
MASTER_PORT = "29500"
WORLD_SIZE = 3
EPOCHS = 2  # Change to 20 for full run

def check_tower_health(tower_name, api_url):
    """Check if Songbird/Squirrel is running on the tower"""
    try:
        response = requests.get(f"{api_url}/health", timeout=5)
        if response.status_code == 200:
            print(f"  ✅ {tower_name}: Health check OK")
            return True
        else:
            print(f"  ❌ {tower_name}: Health check failed (status {response.status_code})")
            return False
    except requests.exceptions.RequestException as e:
        print(f"  ❌ {tower_name}: Cannot connect - {e}")
        return False

def launch_worker_via_api(tower_name, rank, api_url, gpu):
    """Launch training worker via Songbird API"""
    
    # Training command to execute
    training_cmd = (
        f"cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && "
        f"MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} "
        f"python3 -u train_distributed.py "
        f"--rank {rank} --world-size {WORLD_SIZE} --epochs {EPOCHS} --batch-size 64 "
        f"--output-dir ../results/test_distributed "
        f"> ../results/dist_rank{rank}.log 2>&1 &"
    )
    
    print(f"\n🚀 Launching worker on {tower_name} ({gpu})...")
    print(f"   Command: {training_cmd[:80]}...")
    
    # Try different API endpoints
    endpoints_to_try = [
        f"{api_url}/api/v1/execute",
        f"{api_url}/api/execute",
        f"{api_url}/execute",
    ]
    
    for endpoint in endpoints_to_try:
        try:
            payload = {
                "command": training_cmd,
                "async": True
            }
            
            response = requests.post(
                endpoint,
                json=payload,
                timeout=10
            )
            
            if response.status_code in [200, 201, 202]:
                print(f"  ✅ Worker launched successfully via {endpoint}")
                try:
                    result = response.json()
                    print(f"     Response: {json.dumps(result, indent=2)}")
                except:
                    print(f"     Response: {response.text[:200]}")
                return True
                
        except requests.exceptions.RequestException as e:
            print(f"  ⚠️  Failed to connect to {endpoint}: {e}")
            continue
    
    print(f"  ❌ Could not launch worker on {tower_name}")
    return False

def launch_worker_via_shell_command(tower_name, rank, api_url):
    """Alternative: Launch via direct shell execution endpoint"""
    
    print(f"\n🔄 Trying alternative method for {tower_name}...")
    
    # Simple Python script to launch training in background
    script = f"""
import subprocess
import os
os.chdir('/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training')
subprocess.Popen([
    'python3', '-u', 'train_distributed.py',
    '--rank', '{rank}',
    '--world-size', '{WORLD_SIZE}',
    '--epochs', '{EPOCHS}',
    '--batch-size', '64',
    '--output-dir', '../results/test_distributed'
], env={{
    **os.environ,
    'MASTER_ADDR': '{MASTER_ADDR}',
    'MASTER_PORT': '{MASTER_PORT}'
}})
print("Worker launched")
"""
    
    try:
        response = requests.post(
            f"{api_url}/api/python/execute",
            json={"code": script},
            timeout=10
        )
        
        if response.status_code in [200, 201]:
            print(f"  ✅ Worker launched via Python execution")
            return True
    except:
        pass
    
    return False

def main():
    print("=" * 80)
    print("  🚀 DISTRIBUTED TRAINING - WORKER LAUNCHER VIA SONGBIRD")
    print("=" * 80)
    print()
    print("Master Configuration:")
    print(f"  Address: {MASTER_ADDR}:{MASTER_PORT}")
    print(f"  World Size: {WORLD_SIZE} GPUs")
    print(f"  Epochs: {EPOCHS}")
    print()
    
    # Check master is running
    print("🔍 Checking master status...")
    try:
        import socket
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        result = sock.connect_ex((MASTER_ADDR, int(MASTER_PORT)))
        sock.close()
        if result == 0:
            print("  ✅ Master is running and listening on port", MASTER_PORT)
        else:
            print(f"  ❌ Master is NOT listening on port {MASTER_PORT}")
            print("     Start master first!")
            sys.exit(1)
    except Exception as e:
        print(f"  ❌ Cannot check master: {e}")
    
    print()
    print("─" * 80)
    print("🌐 Checking tower availability...")
    print("─" * 80)
    
    # Check health of all towers
    available_towers = {}
    for tower_id, config in TOWERS.items():
        if check_tower_health(config['name'], config['api_url']):
            available_towers[tower_id] = config
    
    if not available_towers:
        print()
        print("❌ No towers available via Songbird API!")
        print()
        print("Manual launch commands:")
        print()
        for tower_id, config in TOWERS.items():
            print(f"{config['name']} (Rank {config['rank']}):")
            print(f"cd /home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training && \\")
            print(f"MASTER_ADDR={MASTER_ADDR} MASTER_PORT={MASTER_PORT} \\")
            print(f"python3 -u train_distributed.py --rank {config['rank']} --world-size {WORLD_SIZE} --epochs {EPOCHS} --batch-size 64 \\")
            print(f"--output-dir ../results/test_distributed")
            print()
        sys.exit(1)
    
    print()
    print("─" * 80)
    print(f"🚀 Launching workers on {len(available_towers)} towers...")
    print("─" * 80)
    
    # Launch workers
    launched = []
    for tower_id, config in available_towers.items():
        success = launch_worker_via_api(
            config['name'],
            config['rank'],
            config['api_url'],
            config['gpu']
        )
        
        if success:
            launched.append(config['name'])
        else:
            # Try alternative method
            if launch_worker_via_shell_command(config['name'], config['rank'], config['api_url']):
                launched.append(config['name'])
    
    print()
    print("=" * 80)
    print("  📊 LAUNCH SUMMARY")
    print("=" * 80)
    print()
    print(f"Workers launched: {len(launched)}/{len(TOWERS)}")
    for tower_name in launched:
        print(f"  ✅ {tower_name}")
    
    if len(launched) == len(TOWERS):
        print()
        print("🎉 All workers launched successfully!")
        print()
        print("What happens next:")
        print("  1. Workers connect to master")
        print("  2. PyTorch DDP synchronizes all 3 GPUs")
        print("  3. Training begins in ~10 seconds")
        print("  4. Each epoch: ~90 seconds")
        print(f"  5. Total time: ~{EPOCHS * 1.5} minutes")
        print()
        print("Monitor progress:")
        print("  Watch logs: tail -f ../results/test_distributed/dist_rank*.log")
        print("  Check GPU: nvidia-smi (on each tower)")
        print()
    else:
        print()
        print(f"⚠️  Only {len(launched)}/{len(TOWERS)} workers launched")
        print("   Launch remaining workers manually (see commands above)")
        print()

if __name__ == '__main__':
    main()

