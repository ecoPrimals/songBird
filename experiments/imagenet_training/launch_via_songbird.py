#!/usr/bin/env python3
"""
Automated Distributed ML Training Launch via Songbird Remote Execution API

This script demonstrates launching PyTorch distributed training across
multiple towers using the Songbird remote execution API.

Usage:
    python3 launch_via_songbird.py --epochs 20 --batch-size 64
"""

import argparse
import requests
import time
import json
from typing import Dict, List

# Configuration
ORCHESTRATOR_URL = "http://192.168.1.144:8080/api/v1/execution"
MASTER_TOWER = "http://192.168.1.144:9020"
WORKER_TOWERS = [
    "http://192.168.1.191:9020",  # strandgate
    "http://192.168.1.193:9020",  # southgate
]
TRAINING_DIR = "/home/eastgate/Development/ecoPrimals/songbird/experiments/imagenet_training/training"

def parse_args():
    """Parse command line arguments"""
    parser = argparse.ArgumentParser(description="Launch distributed training via Songbird")
    parser.add_argument("--epochs", type=int, default=20, help="Number of epochs")
    parser.add_argument("--batch-size", type=int, default=64, help="Batch size")
    parser.add_argument("--master-addr", default="192.168.1.144", help="Master address")
    parser.add_argument("--master-port", default="29500", help="Master port")
    return parser.parse_args()

def launch_master(args, world_size: int) -> Dict:
    """Launch master training process on tower A"""
    print(f"🚀 Launching master (rank 0) on {MASTER_TOWER}...")
    
    request = {
        "tower_endpoint": MASTER_TOWER,
        "request": {
            "command": f"python3 -u train_distributed.py --rank 0 --world-size {world_size} --epochs {args.epochs} --batch-size {args.batch_size}",
            "working_dir": TRAINING_DIR,
            "env": {
                "MASTER_ADDR": args.master_addr,
                "MASTER_PORT": args.master_port,
                "CUDA_VISIBLE_DEVICES": "0",
            },
            "background": True,
            "timeout_seconds": 7200,  # 2 hours
            "capture_output": True
        }
    }
    
    try:
        response = requests.post(f"{ORCHESTRATOR_URL}/execute", json=request, timeout=30)
        response.raise_for_status()
        result = response.json()
        print(f"✅ Master launched: job_id={result['job_id']}, pid={result.get('pid')}")
        return result
    except Exception as e:
        print(f"❌ Failed to launch master: {e}")
        raise

def launch_workers(args, world_size: int, start_rank: int = 1) -> List[Dict]:
    """Launch worker processes on towers B and C"""
    print(f"🚀 Launching {len(WORKER_TOWERS)} workers...")
    
    results = []
    for i, tower_endpoint in enumerate(WORKER_TOWERS):
        rank = start_rank + i
        print(f"  Launching worker rank {rank} on {tower_endpoint}...")
        
        request = {
            "tower_endpoint": tower_endpoint,
            "request": {
                "command": f"python3 -u train_distributed.py --rank {rank} --world-size {world_size} --epochs {args.epochs} --batch-size {args.batch_size}",
                "working_dir": TRAINING_DIR,
                "env": {
                    "MASTER_ADDR": args.master_addr,
                    "MASTER_PORT": args.master_port,
                    "CUDA_VISIBLE_DEVICES": "0",
                },
                "background": True,
                "timeout_seconds": 7200,
                "capture_output": True
            }
        }
        
        try:
            response = requests.post(f"{ORCHESTRATOR_URL}/execute", json=request, timeout=30)
            response.raise_for_status()
            result = response.json()
            print(f"  ✅ Worker rank {rank} launched: job_id={result['job_id']}, pid={result.get('pid')}")
            results.append(result)
        except Exception as e:
            print(f"  ❌ Failed to launch worker rank {rank}: {e}")
            # Continue with other workers
    
    return results

def monitor_jobs(master_job: Dict, worker_jobs: List[Dict]) -> None:
    """Monitor training progress"""
    print(f"\n📊 Monitoring training progress...")
    print(f"Master job: {master_job['job_id']}")
    for i, job in enumerate(worker_jobs):
        print(f"Worker {i+1} job: {job['job_id']}")
    
    all_jobs = [master_job] + worker_jobs
    
    # Monitor until all complete or fail
    max_iterations = 720  # 2 hours at 10s intervals
    for iteration in range(max_iterations):
        time.sleep(10)
        
        statuses = []
        for job in all_jobs:
            try:
                # Query job status
                # Note: This requires the orchestrator to proxy status queries
                # For now, we'll just wait and check logs
                statuses.append("running")
            except Exception as e:
                print(f"⚠️  Error checking job {job['job_id']}: {e}")
                statuses.append("unknown")
        
        running_count = statuses.count("running")
        if running_count == 0:
            print(f"\n🎉 All jobs completed!")
            break
        
        if iteration % 6 == 0:  # Print every minute
            elapsed_mins = (iteration * 10) // 60
            print(f"  [{elapsed_mins:3d}m] {running_count} jobs still running...")

def main():
    """Main execution function"""
    args = parse_args()
    world_size = 1 + len(WORKER_TOWERS)  # Master + workers
    
    print("=" * 70)
    print("🎯 Songbird Distributed ML Training Launcher")
    print("=" * 70)
    print(f"Configuration:")
    print(f"  Epochs: {args.epochs}")
    print(f"  Batch size: {args.batch_size}")
    print(f"  World size: {world_size} (1 master + {len(WORKER_TOWERS)} workers)")
    print(f"  Master: {args.master_addr}:{args.master_port}")
    print("=" * 70)
    
    try:
        # Launch master
        master_job = launch_master(args, world_size)
        
        # Wait a bit for master to initialize
        print(f"\n⏳ Waiting 5 seconds for master to initialize...")
        time.sleep(5)
        
        # Launch workers
        worker_jobs = launch_workers(args, world_size)
        
        if not worker_jobs:
            print("\n❌ No workers launched successfully. Aborting.")
            return 1
        
        print(f"\n✅ Successfully launched {len(worker_jobs)} workers!")
        
        # Monitor progress
        # monitor_jobs(master_job, worker_jobs)
        
        print("\n" + "=" * 70)
        print("🎊 Distributed training launched successfully!")
        print("=" * 70)
        print("\nTo monitor progress, check logs on each tower:")
        print(f"  Master: ssh eastgate@192.168.1.144 'tail -f {TRAINING_DIR}/../results/dist_rank0.log'")
        print(f"  Worker 1: ssh eastgate@192.168.1.191 'tail -f {TRAINING_DIR}/../results/dist_rank1.log'")
        print(f"  Worker 2: ssh eastgate@192.168.1.193 'tail -f {TRAINING_DIR}/../results/dist_rank2.log'")
        print("\nOr check GPU usage:")
        print(f"  watch -n 1 nvidia-smi")
        
        return 0
        
    except Exception as e:
        print(f"\n❌ Error during launch: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())

