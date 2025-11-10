#!/usr/bin/env python3
"""Real-time monitoring of distributed training"""

import subprocess
import time
import requests

def get_gpu_usage():
    """Get GPU usage on local machine"""
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=utilization.gpu,memory.used", 
             "--format=csv,noheader,nounits"],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            gpu, mem = result.stdout.strip().split(',')
            return f"GPU: {gpu.strip()}% | VRAM: {mem.strip()}MB"
    except:
        pass
    return "GPU: N/A"

def get_worker_status(tower_name, endpoint, job_id):
    """Get worker status from remote tower"""
    try:
        response = requests.get(
            f"{endpoint}/api/v1/execution/jobs/{job_id}",
            timeout=3
        )
        if response.status_code == 200:
            data = response.json()
            status = data.get('status', 'unknown')
            exit_code = data.get('exit_code', 'running')
            return f"{tower_name}: {status} (exit: {exit_code})"
    except:
        pass
    return f"{tower_name}: checking..."

# Job IDs from launch
WORKER1_JOB = "fbd5f025-02f8-46ef-a555-f8d4093ed8cc"
WORKER2_JOB = "53b2d86f-4118-4463-917b-5fa0ade5b769"

print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("🔍 Monitoring Distributed Training - Press Ctrl+C to stop")
print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")
print("")

try:
    while True:
        print(f"\r[{time.strftime('%H:%M:%S')}] ", end="")
        print(f"Tower A: {get_gpu_usage()} | ", end="")
        print(f"{get_worker_status('Tower B', 'http://192.168.1.134:9020', WORKER1_JOB)} | ", end="")
        print(f"{get_worker_status('Tower C', 'http://192.168.1.207:9020', WORKER2_JOB)}", end="", flush=True)
        time.sleep(2)
except KeyboardInterrupt:
    print("\n\n✅ Monitoring stopped")

