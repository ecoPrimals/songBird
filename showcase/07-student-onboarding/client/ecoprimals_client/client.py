"""
Songbird Client - Connect to EcoPrimals Federation

Simple async client for students to submit ML tasks.
"""

import asyncio
import json
import os
from pathlib import Path
from typing import Optional, Dict, Any
from datetime import datetime

try:
    import websockets
except ImportError:
    raise ImportError("Please install: pip install websockets")

from rich.console import Console
from rich.progress import Progress, SpinnerColumn, TextColumn

console = Console()


class SongbirdClient:
    """
    Client for connecting to EcoPrimals Songbird federation.
    
    Example:
        >>> client = SongbirdClient("ws://192.168.1.144:8080")
        >>> await client.connect()
        >>> task_id = await client.submit_task("train.py", dataset="mnist")
        >>> result = await client.wait_for_result(task_id)
    """
    
    def __init__(self, url: Optional[str] = None):
        """
        Initialize client.
        
        Args:
            url: Songbird WebSocket URL (or set SONGBIRD_URL env var)
        """
        self.url = url or os.getenv("SONGBIRD_URL")
        if not self.url:
            raise ValueError(
                "No Songbird URL provided. Set SONGBIRD_URL env var or pass to constructor."
            )
        
        # Ensure we're using WebSocket protocol
        if self.url.startswith("http://"):
            self.url = self.url.replace("http://", "ws://")
        elif self.url.startswith("https://"):
            self.url = self.url.replace("https://", "wss://")
        
        # Add /ws/tasks endpoint if not specified
        if not "/ws/" in self.url:
            self.url = f"{self.url}/ws/tasks"
            
        self.ws = None
        self.connected = False
        
    async def connect(self) -> Dict[str, Any]:
        """
        Connect to Songbird federation.
        
        Returns:
            Federation info dict
        """
        console.print(f"🎵 Connecting to Songbird at {self.url}...")
        
        try:
            self.ws = await websockets.connect(self.url)
            
            # Send hello message
            await self.ws.send(json.dumps({
                "type": "hello",
                "client": "student",
                "version": "0.1.0"
            }))
            
            # Wait for welcome response (with timeout)
            try:
                response = await asyncio.wait_for(self.ws.recv(), timeout=5.0)
                data = json.loads(response)
                
                if data.get("type") == "welcome" or data.get("status") == "connected":
                    self.connected = True
                    console.print(f"✅ [green]Connected to Federation![/green]")
                    
                    # Display federation info if available
                    if "federation_name" in data:
                        console.print(f"   Federation: {data['federation_name']}")
                    if "node_count" in data:
                        console.print(f"   Available nodes: {data['node_count']}")
                    if "gpu_count" in data:
                        console.print(f"   Available GPUs: {data['gpu_count']}")
                    
                    return data
                else:
                    raise ConnectionError(f"Unexpected response: {data}")
                    
            except asyncio.TimeoutError:
                # If we don't get a welcome message, assume we're connected
                # (Songbird might not send welcome yet)
                self.connected = True
                console.print(f"✅ [green]Connected to Songbird![/green]")
                console.print(f"   (Federation discovery in progress...)")
                return {"status": "connected"}
                
        except Exception as e:
            console.print(f"❌ [red]Connection failed:[/red] {e}")
            raise
            
    async def submit_task(
        self,
        script_path: str,
        dataset: Optional[str] = None,
        gpu_required: bool = True,
        timeout_minutes: int = 30,
        **kwargs
    ) -> str:
        """
        Submit a training task to the federation.
        
        Args:
            script_path: Path to your Python training script
            dataset: Optional dataset name (e.g., "mnist", "cifar10")
            gpu_required: Whether task requires GPU
            timeout_minutes: Max execution time
            **kwargs: Additional task parameters
            
        Returns:
            task_id: Unique task identifier
        """
        if not self.connected:
            raise RuntimeError("Not connected. Call connect() first.")
            
        script_path = Path(script_path)
        if not script_path.exists():
            raise FileNotFoundError(f"Script not found: {script_path}")
            
        # Read script
        script_code = script_path.read_text()
        
        # Build task payload
        task = {
            "type": "submit_task",
            "script": script_code,
            "script_name": script_path.name,
            "dataset": dataset,
            "gpu_required": gpu_required,
            "timeout_minutes": timeout_minutes,
            "submitted_at": datetime.utcnow().isoformat(),
            "parameters": kwargs
        }
        
        console.print(f"\n🚀 [bold]Submitting task:[/bold] {script_path.name}")
        if dataset:
            console.print(f"   Dataset: {dataset}")
        if gpu_required:
            console.print(f"   GPU: Required")
            
        await self.ws.send(json.dumps(task))
        
        # Wait for acceptance
        response = await self.ws.recv()
        data = json.loads(response)
        
        if data.get("type") == "task_accepted" or data.get("status") == "accepted":
            task_id = data.get("task_id")
            console.print(f"\n✅ [green]Task accepted![/green]")
            console.print(f"   Task ID: [cyan]{task_id}[/cyan]")
            
            if "node" in data:
                console.print(f"   Allocated to: {data['node']}")
            if "gpu" in data:
                console.print(f"   GPU: {data['gpu']}")
                
            return task_id
        else:
            error = data.get("error", "Unknown error")
            console.print(f"❌ [red]Task rejected:[/red] {error}")
            raise RuntimeError(f"Task rejected: {error}")
            
    async def get_status(self, task_id: str) -> Dict[str, Any]:
        """Get current status of a task."""
        await self.ws.send(json.dumps({
            "type": "get_status",
            "task_id": task_id
        }))
        
        response = await self.ws.recv()
        return json.loads(response)
        
    async def wait_for_result(
        self,
        task_id: str,
        poll_interval: float = 2.0
    ) -> Dict[str, Any]:
        """
        Wait for task to complete and return results.
        
        Args:
            task_id: Task identifier
            poll_interval: Seconds between status checks
            
        Returns:
            Result dict with metrics and cryptographic receipt
        """
        console.print(f"\n⏳ Waiting for task {task_id} to complete...")
        console.print("   (This may take several minutes)\n")
        
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console
        ) as progress:
            task = progress.add_task("Training in progress...", total=None)
            
            while True:
                status = await self.get_status(task_id)
                
                state = status.get("status", "unknown")
                
                if state == "completed":
                    progress.update(task, description="✅ Training complete!")
                    break
                elif state == "failed":
                    error = status.get("error", "Unknown error")
                    progress.update(task, description=f"❌ Training failed: {error}")
                    raise RuntimeError(f"Task failed: {error}")
                elif state == "running":
                    # Update progress if available
                    if "progress" in status:
                        progress.update(task, description=f"Training... {status['progress']}")
                else:
                    progress.update(task, description=f"Status: {state}")
                    
                await asyncio.sleep(poll_interval)
                
        # Get final results
        await self.ws.send(json.dumps({
            "type": "get_result",
            "task_id": task_id
        }))
        
        response = await self.ws.recv()
        result = json.loads(response)
        
        # Display results
        console.print("\n" + "="*60)
        console.print("📊 [bold]RESULTS[/bold]")
        console.print("="*60)
        
        if "metrics" in result:
            metrics = result["metrics"]
            for key, value in metrics.items():
                if isinstance(value, float):
                    console.print(f"   {key}: {value:.4f}")
                else:
                    console.print(f"   {key}: {value}")
                    
        if "receipt" in result:
            receipt_file = f"receipt_{task_id}.json"
            Path(receipt_file).write_text(json.dumps(result["receipt"], indent=2))
            console.print(f"\n📜 Cryptographic receipt saved to: [cyan]{receipt_file}[/cyan]")
            
        console.print("="*60 + "\n")
        
        return result
        
    async def disconnect(self):
        """Disconnect from federation."""
        if self.ws:
            await self.ws.close()
            self.connected = False
            console.print("👋 Disconnected from federation")
            

async def quick_submit(script_path: str, **kwargs) -> Dict[str, Any]:
    """
    Quick helper: Connect, submit, wait, disconnect.
    
    Example:
        >>> result = await quick_submit("train.py", dataset="mnist")
    """
    url = os.getenv("SONGBIRD_URL")
    if not url:
        raise ValueError("Set SONGBIRD_URL environment variable")
        
    client = SongbirdClient(url)
    
    try:
        await client.connect()
        task_id = await client.submit_task(script_path, **kwargs)
        result = await client.wait_for_result(task_id)
        return result
    finally:
        await client.disconnect()

