#!/usr/bin/env python3
"""
Demo: Songbird Remote Execution API

Demonstrates the three-tier sovereignty model:
- Tier 1: Sovereign security (Songbird alone)
- Tier 2: Network effect (Songbird + BearDog)
- Tier 3: Full federation (all primals)

Usage:
    python demos/remote_execution_demo.py
"""

import json
import requests
import time
from typing import Dict, List, Optional


class SongbirdExecutionClient:
    """Client for interacting with Songbird execution agent"""
    
    def __init__(self, base_url: str = "http://localhost:9020", auth_token: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.auth_token = auth_token
        self.headers = {
            "Content-Type": "application/json"
        }
        if auth_token:
            self.headers["Authorization"] = f"Bearer {auth_token}"
    
    def health_check(self) -> Dict:
        """Check agent health"""
        response = requests.get(f"{self.base_url}/health")
        response.raise_for_status()
        return response.json()
    
    def execute_command(
        self,
        command: str,
        background: bool = False,
        working_dir: Optional[str] = None,
        env: Optional[Dict[str, str]] = None,
        timeout_seconds: Optional[int] = None
    ) -> Dict:
        """Execute a command on the remote agent"""
        payload = {
            "command": command,
            "background": background,
            "capture_output": True,
        }
        
        if working_dir:
            payload["working_dir"] = working_dir
        if env:
            payload["env"] = env
        if timeout_seconds:
            payload["timeout_seconds"] = timeout_seconds
        
        response = requests.post(
            f"{self.base_url}/api/v1/execution/command",
            headers=self.headers,
            json=payload
        )
        response.raise_for_status()
        return response.json()
    
    def get_job(self, job_id: str) -> Dict:
        """Get job status"""
        response = requests.get(
            f"{self.base_url}/api/v1/execution/jobs/{job_id}",
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def list_jobs(self) -> List[Dict]:
        """List all jobs"""
        response = requests.get(
            f"{self.base_url}/api/v1/execution/jobs",
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()
    
    def stop_job(self, job_id: str, signal: str = "SIGTERM") -> Dict:
        """Stop a running job"""
        payload = {"signal": signal}
        response = requests.post(
            f"{self.base_url}/api/v1/execution/jobs/{job_id}/stop",
            headers=self.headers,
            json=payload
        )
        response.raise_for_status()
        return response.json()
    
    def get_stats(self) -> Dict:
        """Get agent statistics"""
        response = requests.get(
            f"{self.base_url}/api/v1/execution/stats",
            headers=self.headers
        )
        response.raise_for_status()
        return response.json()


def print_section(title: str):
    """Print a section header"""
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}\n")


def demo_basic_commands():
    """Demo: Basic command execution (Tier 1 - Sovereign)"""
    print_section("Demo 1: Basic Command Execution (Tier 1 - Sovereign)")
    
    client = SongbirdExecutionClient()
    
    # Check health
    print("Checking agent health...")
    health = client.health_check()
    print(f"✅ Agent is {health['status']}")
    print(f"   Service: {health['service']}")
    print(f"   Version: {health['version']}")
    
    # Execute simple command
    print("\nExecuting: echo 'Hello from Songbird!'")
    result = client.execute_command("echo 'Hello from Songbird!'")
    print(f"✅ Exit code: {result['exit_code']}")
    print(f"   Output: {result['stdout'].strip()}")
    print(f"   Duration: {result['duration_ms']}ms")
    
    # Execute command with working directory
    print("\nExecuting: pwd (in /tmp)")
    result = client.execute_command("pwd", working_dir="/tmp")
    print(f"✅ Exit code: {result['exit_code']}")
    print(f"   Output: {result['stdout'].strip()}")
    
    # Execute command with environment variables
    print("\nExecuting: printenv with custom var")
    result = client.execute_command(
        "printenv CUSTOM_VAR",
        env={"CUSTOM_VAR": "custom_value"}
    )
    print(f"✅ Exit code: {result['exit_code']}")
    print(f"   Output: {result['stdout'].strip()}")


def demo_background_jobs():
    """Demo: Background job management"""
    print_section("Demo 2: Background Job Management")
    
    client = SongbirdExecutionClient()
    
    # Start background job
    print("Starting background job: sleep 5")
    result = client.execute_command("sleep 5", background=True)
    job_id = result['job_id']
    print(f"✅ Job started: {job_id}")
    print(f"   Status: {result['status']}")
    print(f"   PID: {result['pid']}")
    
    # Check job status
    print("\nChecking job status after 1 second...")
    time.sleep(1)
    job = client.get_job(job_id)
    print(f"✅ Job {job_id}")
    print(f"   Status: {job['status']}")
    print(f"   Running for: {(time.time() - job['started_at'])} seconds")
    
    # List all jobs
    print("\nListing all jobs...")
    jobs = client.list_jobs()
    print(f"✅ Found {len(jobs)} jobs")
    for job in jobs[:3]:  # Show first 3
        print(f"   - {job['id']}: {job['status']}")


def demo_security_validation():
    """Demo: Security validation (Tier 1 - Sovereign)"""
    print_section("Demo 3: Security Validation (Tier 1 - Sovereign)")
    
    client = SongbirdExecutionClient()
    
    # Safe commands (should succeed)
    safe_commands = [
        "echo hello",
        "ls -la",
        "pwd",
        "date",
    ]
    
    print("Testing safe commands...")
    for cmd in safe_commands:
        try:
            result = client.execute_command(cmd)
            print(f"✅ Allowed: {cmd} (exit: {result['exit_code']})")
        except requests.exceptions.HTTPError as e:
            print(f"❌ Blocked: {cmd} ({e.response.status_code})")
    
    # Dangerous commands (should be blocked)
    dangerous_commands = [
        "rm -rf /",
        ":(){ :|:& };:",
        "mkfs.ext4 /dev/sda",
    ]
    
    print("\nTesting dangerous commands (should be blocked)...")
    for cmd in dangerous_commands:
        try:
            result = client.execute_command(cmd)
            print(f"⚠️  UNEXPECTED: {cmd} was allowed!")
        except requests.exceptions.HTTPError as e:
            print(f"✅ Blocked: {cmd} ({e.response.status_code})")


def demo_resource_limits():
    """Demo: Resource limit enforcement"""
    print_section("Demo 4: Resource Limit Enforcement")
    
    client = SongbirdExecutionClient()
    
    # Test timeout enforcement
    print("Testing timeout enforcement...")
    try:
        result = client.execute_command("sleep 1", timeout_seconds=2)
        print(f"✅ Short sleep completed (timeout=2s): exit {result['exit_code']}")
    except requests.exceptions.HTTPError as e:
        print(f"❌ Unexpected error: {e}")
    
    # Test command with output
    print("\nTesting output capture...")
    result = client.execute_command("echo 'Line 1' && echo 'Line 2' && echo 'Line 3'")
    lines = result['stdout'].strip().split('\n')
    print(f"✅ Captured {len(lines)} lines of output:")
    for line in lines:
        print(f"   {line}")


def demo_agent_stats():
    """Demo: Agent statistics"""
    print_section("Demo 5: Agent Statistics")
    
    client = SongbirdExecutionClient()
    
    print("Getting agent statistics...")
    stats = client.get_stats()
    print(f"✅ Agent Statistics:")
    print(f"   Total jobs: {stats.get('total_jobs', 0)}")
    print(f"   Running jobs: {stats.get('running_jobs', 0)}")
    print(f"   Completed jobs: {stats.get('completed_jobs', 0)}")
    print(f"   Failed jobs: {stats.get('failed_jobs', 0)}")


def demo_ml_training_workflow():
    """Demo: ML training workflow simulation"""
    print_section("Demo 6: ML Training Workflow (Simulated)")
    
    client = SongbirdExecutionClient()
    
    print("Simulating distributed ML training workflow...")
    
    # Step 1: Prepare environment
    print("\n1. Preparing environment...")
    result = client.execute_command("echo 'Setting up training environment'")
    print(f"✅ Environment prepared: {result['stdout'].strip()}")
    
    # Step 2: Download data (simulated)
    print("\n2. Downloading training data...")
    result = client.execute_command(
        "echo 'Downloading imagenet data...' && sleep 1 && echo 'Downloaded 50GB'",
        timeout_seconds=30
    )
    print(f"✅ Data downloaded: {result['stdout'].strip()}")
    
    # Step 3: Start training (background)
    print("\n3. Starting training job...")
    result = client.execute_command(
        "echo 'Training model...' && sleep 3",
        background=True
    )
    job_id = result['job_id']
    print(f"✅ Training started: {job_id}")
    
    # Step 4: Monitor training
    print("\n4. Monitoring training...")
    for i in range(3):
        time.sleep(1)
        job = client.get_job(job_id)
        print(f"   Epoch {i+1}/3: status={job['status']}")
    
    print("\n✅ ML training workflow completed!")


def main():
    """Main demo function"""
    print("╔════════════════════════════════════════════════════════════╗")
    print("║   Songbird Remote Execution API Demo                      ║")
    print("║   Primal Sovereignty with Network Effects                 ║")
    print("╚════════════════════════════════════════════════════════════╝")
    
    print("\n🏛️  Current Mode: Tier 1 (Sovereign Security)")
    print("   - Songbird's own authentication")
    print("   - Command validation")
    print("   - Resource limits")
    print("   - Works offline, zero dependencies")
    
    try:
        # Run demos
        demo_basic_commands()
        demo_background_jobs()
        demo_security_validation()
        demo_resource_limits()
        demo_agent_stats()
        demo_ml_training_workflow()
        
        print_section("All Demos Complete! ✅")
        print("\n📚 Next Steps:")
        print("   1. Add auth token for enhanced security")
        print("   2. Deploy with BearDog for Tier 2 (Network Effect)")
        print("   3. Full primal federation for Tier 3")
        print("\n   See: docs/SOVEREIGNTY_QUICK_START.md")
        
    except requests.exceptions.ConnectionError:
        print("\n❌ Error: Could not connect to Songbird execution agent")
        print("   Please start the agent first:")
        print("   $ cargo run -p songbird-execution-agent")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()

