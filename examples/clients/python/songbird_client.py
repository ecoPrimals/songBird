#!/usr/bin/env python3
"""
Songbird JSON-RPC Python Client
Universal Gateway Client for Songbird Orchestrator

Usage:
    from songbird_client import SongbirdClient
    
    client = SongbirdClient("http://localhost:8080")
    health = client.health()
    print(health)

Version: 0.2.1
Last Updated: April 16, 2026
"""

import requests
from typing import Any, Dict, List, Optional
import time


class SongbirdError(Exception):
    """Base exception for Songbird client errors"""
    def __init__(self, code: int, message: str, data: Any = None):
        self.code = code
        self.message = message
        self.data = data
        super().__init__(f"Error {code}: {message}")


class SongbirdClient:
    """
    Simple, universal client for Songbird JSON-RPC API
    
    Example:
        >>> client = SongbirdClient("http://localhost:8080")
        >>> health = client.health()
        >>> print(health['status'])
        'healthy'
    """
    
    def __init__(self, base_url: str = "http://localhost:8080", timeout: int = 30):
        """
        Initialize Songbird client
        
        Args:
            base_url: Songbird orchestrator URL (default: http://localhost:8080)
            timeout: Request timeout in seconds (default: 30)
        """
        self.base_url = base_url.rstrip('/')
        self.jsonrpc_url = f"{self.base_url}/jsonrpc"
        self.timeout = timeout
        self.session = requests.Session()
        self._request_id = 0
    
    def _get_next_id(self) -> int:
        """Get next request ID"""
        self._request_id += 1
        return self._request_id
    
    def _call(self, method: str, params: Optional[Dict[str, Any]] = None) -> Any:
        """
        Make a JSON-RPC call
        
        Args:
            method: JSON-RPC method name
            params: Method parameters (optional)
            
        Returns:
            Method result
            
        Raises:
            SongbirdError: If the call fails
            requests.RequestException: If HTTP request fails
        """
        payload = {
            "jsonrpc": "2.0",
            "method": method,
            "id": self._get_next_id()
        }
        
        if params is not None:
            payload["params"] = params
        
        response = self.session.post(
            self.jsonrpc_url,
            json=payload,
            timeout=self.timeout
        )
        response.raise_for_status()
        
        data = response.json()
        
        if "error" in data:
            error = data["error"]
            raise SongbirdError(
                code=error.get("code", -1),
                message=error.get("message", "Unknown error"),
                data=error.get("data")
            )
        
        return data.get("result")
    
    def _batch_call(self, requests_list: List[Dict[str, Any]]) -> List[Any]:
        """
        Make multiple JSON-RPC calls in one HTTP request
        
        Args:
            requests_list: List of request dicts with 'method' and optional 'params'
            
        Returns:
            List of results (in same order as requests)
        """
        payload = [
            {
                "jsonrpc": "2.0",
                "method": req["method"],
                "params": req.get("params"),
                "id": self._get_next_id()
            }
            for req in requests_list
        ]
        
        response = self.session.post(
            self.jsonrpc_url,
            json=payload,
            timeout=self.timeout
        )
        response.raise_for_status()
        
        results = response.json()
        
        # Return results in order
        ordered_results = []
        for result in results:
            if "error" in result:
                error = result["error"]
                ordered_results.append(SongbirdError(
                    code=error.get("code", -1),
                    message=error.get("message", "Unknown error"),
                    data=error.get("data")
                ))
            else:
                ordered_results.append(result.get("result"))
        
        return ordered_results
    
    # ============================================
    # Health & Info Methods
    # ============================================
    
    def health(self) -> Dict[str, Any]:
        """
        Check Songbird health
        
        Returns:
            Health status dict with 'status', 'version', 'uptime_seconds'
            
        Example:
            >>> client.health()
            {'status': 'healthy', 'version': '0.2.1', 'uptime_seconds': 1234}
        """
        return self._call("songbird.health")
    
    def version(self) -> Dict[str, Any]:
        """
        Get Songbird version information
        
        Returns:
            Version info dict with 'version', 'name', 'commit', 'build_date'
            
        Example:
            >>> client.version()
            {'version': '0.2.1', 'name': 'Songbird Universal Orchestrator', ...}
        """
        return self._call("songbird.version")
    
    # ============================================
    # Protocol Methods
    # ============================================
    
    def protocol_capabilities(self) -> Dict[str, Any]:
        """
        Get available protocol capabilities
        
        Returns:
            Protocol capabilities dict with 'protocols' listing available protocols
            
        Example:
            >>> caps = client.protocol_capabilities()
            >>> print(caps['protocols'].keys())
            dict_keys(['http', 'jsonrpc', 'tarpc', 'websocket'])
        """
        return self._call("songbird.protocol.capabilities")
    
    # ============================================
    # Service Management Methods
    # ============================================
    
    def list_services(self) -> List[Dict[str, Any]]:
        """
        List all registered services
        
        Returns:
            List of service dicts
            
        Example:
            >>> services = client.list_services()
            >>> for service in services:
            ...     print(f"{service['name']}: {service['endpoint']}")
        """
        result = self._call("songbird.services.list")
        return result.get("services", [])
    
    def get_service(self, service_id: str) -> Dict[str, Any]:
        """
        Get details for a specific service
        
        Args:
            service_id: Service identifier
            
        Returns:
            Service details dict
            
        Example:
            >>> service = client.get_service("my-service-123")
            >>> print(service['endpoint'])
        """
        return self._call("songbird.services.get", {"service_id": service_id})
    
    def register_service(self, service_config: Dict[str, Any]) -> Dict[str, Any]:
        """
        Register a new service with Songbird
        
        Args:
            service_config: Service configuration dict
            
        Returns:
            Registration result
            
        Example:
            >>> result = client.register_service({
            ...     "name": "my-service",
            ...     "endpoint": "http://localhost:3000",
            ...     "capabilities": ["compute", "storage"]
            ... })
        """
        return self._call("songbird.services.register", service_config)
    
    # ============================================
    # Compute Methods
    # ============================================
    
    def schedule_compute(self, task_config: Dict[str, Any]) -> Dict[str, Any]:
        """
        Schedule a compute task (integrates with Toadstool)
        
        Args:
            task_config: Task configuration dict with 'task', 'language', 'code', etc.
            
        Returns:
            Task scheduling result with 'task_id'
            
        Example:
            >>> result = client.schedule_compute({
            ...     "task": "train_model",
            ...     "language": "python",
            ...     "code": "import numpy as np\\nprint('Hello!')"
            ... })
            >>> task_id = result['task_id']
        """
        return self._call("songbird.compute.schedule", task_config)
    
    def get_compute_status(self, task_id: str) -> Dict[str, Any]:
        """
        Get status of a compute task
        
        Args:
            task_id: Task identifier from schedule_compute
            
        Returns:
            Task status dict with 'status', 'progress', 'result', etc.
            
        Example:
            >>> status = client.get_compute_status("task-123")
            >>> print(status['status'])
            'completed'
        """
        return self._call("songbird.compute.status", {"task_id": task_id})
    
    def wait_for_compute(self, task_id: str, poll_interval: float = 1.0, timeout: Optional[float] = None) -> Dict[str, Any]:
        """
        Wait for a compute task to complete
        
        Args:
            task_id: Task identifier
            poll_interval: Seconds between status checks (default: 1.0)
            timeout: Maximum wait time in seconds (default: None = wait forever)
            
        Returns:
            Final task status
            
        Raises:
            TimeoutError: If timeout is exceeded
            
        Example:
            >>> task = client.schedule_compute({...})
            >>> result = client.wait_for_compute(task['task_id'], timeout=300)
            >>> print(result['result'])
        """
        start_time = time.time()
        
        while True:
            status = self.get_compute_status(task_id)
            
            if status["status"] in ["completed", "failed", "cancelled"]:
                return status
            
            if timeout and (time.time() - start_time) > timeout:
                raise TimeoutError(f"Task {task_id} did not complete within {timeout} seconds")
            
            time.sleep(poll_interval)
    
    # ============================================
    # Federation Methods
    # ============================================
    
    def list_federation_peers(self) -> List[Dict[str, Any]]:
        """
        List all federation peers
        
        Returns:
            List of peer dicts
            
        Example:
            >>> peers = client.list_federation_peers()
            >>> for peer in peers:
            ...     print(f"{peer['id']}: {peer['endpoint']}")
        """
        result = self._call("songbird.federation.peers")
        return result.get("peers", [])
    
    def join_federation(self, peer_config: Dict[str, Any]) -> Dict[str, Any]:
        """
        Join a federation by connecting to a peer
        
        Args:
            peer_config: Peer configuration dict with 'peer_id' and 'endpoint'
            
        Returns:
            Join result
            
        Example:
            >>> result = client.join_federation({
            ...     "peer_id": "tower-a",
            ...     "endpoint": "http://tower-a.example.com:8080"
            ... })
        """
        return self._call("songbird.federation.join", peer_config)
    
    # ============================================
    # Batch Operations
    # ============================================
    
    def batch(self, operations: List[Dict[str, Any]]) -> List[Any]:
        """
        Execute multiple operations in one HTTP request
        
        Args:
            operations: List of operation dicts with 'method' and optional 'params'
            
        Returns:
            List of results (may include SongbirdError objects for failed operations)
            
        Example:
            >>> results = client.batch([
            ...     {"method": "songbird.health"},
            ...     {"method": "songbird.version"},
            ...     {"method": "songbird.services.list"}
            ... ])
            >>> health, version, services = results
        """
        return self._batch_call(operations)
    
    # ============================================
    # Utility Methods
    # ============================================
    
    def is_healthy(self) -> bool:
        """
        Quick health check
        
        Returns:
            True if healthy, False otherwise
            
        Example:
            >>> if client.is_healthy():
            ...     print("Songbird is up!")
        """
        try:
            health = self.health()
            return health.get("status") == "healthy"
        except Exception:
            return False
    
    def close(self):
        """Close the HTTP session"""
        self.session.close()
    
    def __enter__(self):
        """Context manager entry"""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit"""
        self.close()


# ============================================
# Convenience Functions
# ============================================

def create_client(base_url: str = "http://localhost:8080", timeout: int = 30) -> SongbirdClient:
    """
    Create a Songbird client
    
    Args:
        base_url: Songbird orchestrator URL
        timeout: Request timeout in seconds
        
    Returns:
        SongbirdClient instance
    """
    return SongbirdClient(base_url, timeout)


# ============================================
# Example Usage
# ============================================

if __name__ == "__main__":
    # Example 1: Basic usage
    print("=" * 60)
    print("Example 1: Basic Health Check")
    print("=" * 60)
    
    client = SongbirdClient("http://localhost:8080")
    
    try:
        health = client.health()
        print(f"✅ Songbird is {health['status']}")
        print(f"   Version: {health['version']}")
        print(f"   Uptime: {health['uptime_seconds']} seconds")
    except Exception as e:
        print(f"❌ Failed to connect: {e}")
    
    print()
    
    # Example 2: Get version info
    print("=" * 60)
    print("Example 2: Version Information")
    print("=" * 60)
    
    try:
        version = client.version()
        print(f"Name: {version['name']}")
        print(f"Version: {version['version']}")
        print(f"Commit: {version.get('commit', 'N/A')}")
        print(f"Build Date: {version.get('build_date', 'N/A')}")
    except Exception as e:
        print(f"❌ Failed: {e}")
    
    print()
    
    # Example 3: Protocol capabilities
    print("=" * 60)
    print("Example 3: Protocol Capabilities")
    print("=" * 60)
    
    try:
        caps = client.protocol_capabilities()
        protocols = caps.get("protocols", {})
        print(f"Available Protocols:")
        for proto_name, proto_info in protocols.items():
            print(f"  • {proto_name}: {proto_info.get('version', 'N/A')}")
    except Exception as e:
        print(f"❌ Failed: {e}")
    
    print()
    
    # Example 4: Batch operations
    print("=" * 60)
    print("Example 4: Batch Operations")
    print("=" * 60)
    
    try:
        results = client.batch([
            {"method": "songbird.health"},
            {"method": "songbird.version"},
            {"method": "songbird.protocol.capabilities"}
        ])
        
        health, version, caps = results
        print(f"✅ Executed 3 operations in 1 HTTP request")
        print(f"   Health: {health['status']}")
        print(f"   Version: {version['version']}")
        print(f"   Protocols: {len(caps['protocols'])} available")
    except Exception as e:
        print(f"❌ Failed: {e}")
    
    print()
    
    # Example 5: Context manager
    print("=" * 60)
    print("Example 5: Using Context Manager")
    print("=" * 60)
    
    with SongbirdClient("http://localhost:8080") as client:
        if client.is_healthy():
            print("✅ Songbird is healthy and ready!")
        else:
            print("❌ Songbird is not responding")
    
    print()
    print("=" * 60)
    print("🎉 All examples complete!")
    print("=" * 60)

