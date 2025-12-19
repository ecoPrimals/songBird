"""
Submit MNIST training to EcoPrimals federation.

This helper script makes it easy to submit your training task
without manually using the client API.

Usage:
    python submit.py
"""

import asyncio
import os
import sys
from pathlib import Path

# Add client to path
client_path = Path(__file__).parent.parent.parent / "client"
sys.path.insert(0, str(client_path))

from ecoprimals_client.client import SongbirdClient


async def main():
    """Submit MNIST training task."""
    
    print("\n" + "=" * 60)
    print("  MNIST TRAINING SUBMISSION")
    print("  EcoPrimals Federation")
    print("=" * 60 + "\n")
    
    # Check for Songbird URL
    url = os.getenv("SONGBIRD_URL")
    if not url:
        print("❌ No SONGBIRD_URL environment variable set.\n")
        print("Ask your instructor for the federation URL, then:")
        print("  export SONGBIRD_URL=ws://192.168.1.144:8080")
        print("\nOr run:")
        print("  SONGBIRD_URL=ws://192.168.1.144:8080 python submit.py")
        print()
        return
    
    try:
        # Connect to federation
        client = SongbirdClient(url)
        await client.connect()
        
        # Submit task
        task_id = await client.submit_task(
            script_path="train.py",
            dataset="mnist",
            gpu_required=True,
            timeout_minutes=30
        )
        
        # Wait for results
        result = await client.wait_for_result(task_id)
        
        print("\n🎉 [bold green]Success![/bold green]")
        print("\nYou just trained a neural network on distributed GPUs!")
        print("Check your cryptographic receipt for verification.\n")
        
        await client.disconnect()
        
    except KeyboardInterrupt:
        print("\n\n⚠️  Interrupted by user")
        print("Your task may still be running on the federation.")
        print(f"To check status later, use task ID: {task_id if 'task_id' in locals() else 'N/A'}\n")
        
    except Exception as e:
        print(f"\n❌ Error: {e}\n")
        print("Troubleshooting:")
        print("  1. Check SONGBIRD_URL is correct")
        print("  2. Verify you're on the same network")
        print("  3. Ensure Songbird is running")
        print("  4. Ask your instructor for help\n")


if __name__ == "__main__":
    asyncio.run(main())

