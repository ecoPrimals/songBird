#!/usr/bin/env python3
"""
Songbird WebSocket Client - Real-Time Communication

Official Python client for Songbird WebSocket API.
Provides real-time bidirectional communication with event subscriptions.

Version: 0.2.1
Last Updated: November 11, 2025 - Phase 4

Features:
- Real-time bidirectional communication
- Event subscription system
- Automatic reconnection
- Ping/pong keep-alive
- Query federation status
- Service discovery
- Type hints and full error handling

Requirements:
    pip install websockets asyncio

Usage:
    # Basic connection
    client = SongbirdWebSocketClient("ws://localhost:8080/api/ws/ws")
    await client.connect()
    
    # Subscribe to events
    await client.subscribe(["service_update", "health_update"])
    
    # Query status
    status = await client.query_status()
    
    # Query services
    services = await client.query_services(["ml", "training"])
    
    # Listen for events
    async for event in client.listen():
        print(f"Event: {event}")
"""

import asyncio
import json
import logging
from typing import Any, Dict, List, Optional, AsyncIterator
from dataclasses import dataclass
from datetime import datetime

try:
    import websockets
    from websockets.client import WebSocketClientProtocol
except ImportError:
    print("Error: websockets not installed. Run: pip install websockets")
    raise

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('songbird.websocket')


@dataclass
class FederationStatus:
    """Federation status information"""
    total_services: int
    total_peers: int
    uptime_seconds: int
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'FederationStatus':
        return cls(
            total_services=data['total_services'],
            total_peers=data['total_peers'],
            uptime_seconds=data['uptime_seconds']
        )


@dataclass
class ServiceSummary:
    """Service summary information"""
    name: str
    address: str
    port: int
    capabilities: List[str]
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ServiceSummary':
        return cls(
            name=data['name'],
            address=data['address'],
            port=data['port'],
            capabilities=data['capabilities']
        )


class SongbirdWebSocketError(Exception):
    """Base exception for Songbird WebSocket errors"""
    pass


class ConnectionError(SongbirdWebSocketError):
    """Connection-related errors"""
    pass


class MessageError(SongbirdWebSocketError):
    """Message parsing/handling errors"""
    pass


class SongbirdWebSocketClient:
    """
    Songbird WebSocket Client
    
    Provides real-time bidirectional communication with Songbird.
    
    Example:
        client = SongbirdWebSocketClient("ws://localhost:8080/api/ws/ws")
        await client.connect()
        
        # Subscribe to events
        await client.subscribe(["service_update", "health_update"])
        
        # Query status
        status = await client.query_status()
        print(f"Services: {status.total_services}, Peers: {status.total_peers}")
        
        # Listen for events
        async for event in client.listen():
            if event['type'] == 'service_update':
                print(f"Service updated: {event['service_name']}")
        
        await client.close()
    """
    
    def __init__(
        self,
        url: str,
        ping_interval: float = 30.0,
        ping_timeout: float = 10.0,
        auto_reconnect: bool = True,
        max_reconnect_attempts: int = 5
    ):
        """
        Initialize WebSocket client
        
        Args:
            url: WebSocket URL (e.g., "ws://localhost:8080/api/ws/ws")
            ping_interval: Seconds between ping messages
            ping_timeout: Timeout for pong response
            auto_reconnect: Automatically reconnect on connection loss
            max_reconnect_attempts: Maximum reconnection attempts
        """
        self.url = url
        self.ping_interval = ping_interval
        self.ping_timeout = ping_timeout
        self.auto_reconnect = auto_reconnect
        self.max_reconnect_attempts = max_reconnect_attempts
        
        self._ws: Optional[WebSocketClientProtocol] = None
        self._connected = False
        self._ping_task: Optional[asyncio.Task] = None
        self._subscriptions: List[str] = []
        self._message_queue: asyncio.Queue = asyncio.Queue()
        self._reconnect_attempts = 0
        
    async def connect(self) -> None:
        """
        Connect to Songbird WebSocket server
        
        Raises:
            ConnectionError: If connection fails
        """
        try:
            logger.info(f"Connecting to {self.url}")
            self._ws = await websockets.connect(
                self.url,
                ping_interval=self.ping_interval,
                ping_timeout=self.ping_timeout
            )
            self._connected = True
            self._reconnect_attempts = 0
            logger.info("Connected successfully")
            
            # Start ping task
            self._ping_task = asyncio.create_task(self._ping_loop())
            
            # Start message receiver
            asyncio.create_task(self._receive_messages())
            
        except Exception as e:
            logger.error(f"Connection failed: {e}")
            raise ConnectionError(f"Failed to connect: {e}")
    
    async def close(self) -> None:
        """Close WebSocket connection"""
        if self._ping_task:
            self._ping_task.cancel()
        
        if self._ws:
            await self._ws.close()
        
        self._connected = False
        logger.info("Connection closed")
    
    async def _ping_loop(self) -> None:
        """Send periodic ping messages"""
        while self._connected:
            try:
                await asyncio.sleep(self.ping_interval)
                await self.ping()
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.warning(f"Ping failed: {e}")
    
    async def _receive_messages(self) -> None:
        """Receive and queue messages"""
        try:
            async for message in self._ws:
                try:
                    data = json.loads(message)
                    await self._message_queue.put(data)
                except json.JSONDecodeError as e:
                    logger.error(f"Failed to parse message: {e}")
        except websockets.exceptions.ConnectionClosed:
            self._connected = False
            if self.auto_reconnect:
                await self._reconnect()
    
    async def _reconnect(self) -> None:
        """Attempt to reconnect"""
        if self._reconnect_attempts >= self.max_reconnect_attempts:
            logger.error("Max reconnection attempts reached")
            return
        
        self._reconnect_attempts += 1
        delay = min(2 ** self._reconnect_attempts, 30)  # Exponential backoff
        
        logger.info(f"Reconnecting in {delay}s (attempt {self._reconnect_attempts})")
        await asyncio.sleep(delay)
        
        try:
            await self.connect()
            # Re-subscribe to events
            if self._subscriptions:
                await self.subscribe(self._subscriptions)
        except Exception as e:
            logger.error(f"Reconnection failed: {e}")
            await self._reconnect()
    
    async def _send_message(self, message: Dict[str, Any]) -> None:
        """Send message to server"""
        if not self._connected or not self._ws:
            raise ConnectionError("Not connected")
        
        try:
            await self._ws.send(json.dumps(message))
        except Exception as e:
            logger.error(f"Failed to send message: {e}")
            raise MessageError(f"Failed to send message: {e}")
    
    async def subscribe(self, events: List[str]) -> None:
        """
        Subscribe to event types
        
        Args:
            events: List of event types to subscribe to
                   (e.g., ["service_update", "health_update"])
        """
        self._subscriptions.extend(events)
        await self._send_message({
            "type": "subscribe",
            "events": events
        })
        logger.info(f"Subscribed to: {events}")
    
    async def unsubscribe(self, events: List[str]) -> None:
        """
        Unsubscribe from event types
        
        Args:
            events: List of event types to unsubscribe from
        """
        for event in events:
            if event in self._subscriptions:
                self._subscriptions.remove(event)
        
        await self._send_message({
            "type": "unsubscribe",
            "events": events
        })
        logger.info(f"Unsubscribed from: {events}")
    
    async def ping(self, data: Optional[str] = None) -> None:
        """
        Send ping message
        
        Args:
            data: Optional data to include in ping
        """
        message = {"type": "ping"}
        if data:
            message["data"] = data
        await self._send_message(message)
    
    async def query_status(self) -> FederationStatus:
        """
        Query federation status
        
        Returns:
            FederationStatus object
        """
        await self._send_message({"type": "query_status"})
        
        # Wait for response
        while True:
            event = await self._message_queue.get()
            if event.get('type') == 'federation_status':
                return FederationStatus.from_dict(event)
    
    async def query_services(self, capabilities: List[str]) -> List[ServiceSummary]:
        """
        Query services by capability
        
        Args:
            capabilities: Required capabilities
        
        Returns:
            List of ServiceSummary objects
        """
        await self._send_message({
            "type": "query_services",
            "capabilities": capabilities
        })
        
        # Wait for response
        while True:
            event = await self._message_queue.get()
            if event.get('type') == 'service_list':
                return [
                    ServiceSummary.from_dict(s)
                    for s in event.get('services', [])
                ]
    
    async def listen(self) -> AsyncIterator[Dict[str, Any]]:
        """
        Listen for events (async iterator)
        
        Yields:
            Event dictionaries
        
        Example:
            async for event in client.listen():
                if event['type'] == 'service_update':
                    print(f"Service: {event['service_name']}")
        """
        while self._connected:
            try:
                event = await asyncio.wait_for(
                    self._message_queue.get(),
                    timeout=1.0
                )
                yield event
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                logger.error(f"Error receiving event: {e}")
                break
    
    @property
    def connected(self) -> bool:
        """Check if connected"""
        return self._connected
    
    @property
    def subscriptions(self) -> List[str]:
        """Get current subscriptions"""
        return self._subscriptions.copy()


# ============================================================================
# Example Usage
# ============================================================================

async def example_basic():
    """Basic usage example"""
    print("=== Basic WebSocket Example ===\n")
    
    client = SongbirdWebSocketClient("ws://localhost:8080/api/ws/ws")
    
    try:
        # Connect
        await client.connect()
        print("✅ Connected to Songbird\n")
        
        # Query status
        status = await client.query_status()
        print(f"📊 Federation Status:")
        print(f"   Services: {status.total_services}")
        print(f"   Peers: {status.total_peers}")
        print(f"   Uptime: {status.uptime_seconds}s\n")
        
        # Close
        await client.close()
        print("✅ Connection closed")
        
    except Exception as e:
        print(f"❌ Error: {e}")


async def example_subscriptions():
    """Event subscription example"""
    print("=== Event Subscription Example ===\n")
    
    client = SongbirdWebSocketClient("ws://localhost:8080/api/ws/ws")
    
    try:
        await client.connect()
        print("✅ Connected to Songbird\n")
        
        # Subscribe to events
        await client.subscribe(["service_update", "health_update"])
        print("✅ Subscribed to events\n")
        
        # Listen for 5 events
        print("🔊 Listening for events (Ctrl+C to stop)...\n")
        count = 0
        async for event in client.listen():
            print(f"📨 Event: {event.get('type')}")
            print(f"   Data: {event}")
            count += 1
            if count >= 5:
                break
        
        await client.close()
        
    except KeyboardInterrupt:
        print("\n⏹️  Stopped by user")
        await client.close()
    except Exception as e:
        print(f"❌ Error: {e}")


async def example_service_discovery():
    """Service discovery example"""
    print("=== Service Discovery Example ===\n")
    
    client = SongbirdWebSocketClient("ws://localhost:8080/api/ws/ws")
    
    try:
        await client.connect()
        print("✅ Connected to Songbird\n")
        
        # Query services with ML capability
        print("🔍 Querying services with 'ml' capability...")
        services = await client.query_services(["ml"])
        
        print(f"📋 Found {len(services)} service(s):\n")
        for service in services:
            print(f"   • {service.name}")
            print(f"     Address: {service.address}:{service.port}")
            print(f"     Capabilities: {', '.join(service.capabilities)}\n")
        
        await client.close()
        
    except Exception as e:
        print(f"❌ Error: {e}")


async def main():
    """Run examples"""
    print("\n" + "="*80)
    print("Songbird WebSocket Client - Python Examples")
    print("="*80 + "\n")
    
    # Run examples
    await example_basic()
    print("\n" + "-"*80 + "\n")
    
    await example_service_discovery()
    print("\n" + "-"*80 + "\n")
    
    # Uncomment to test event subscriptions (requires active events)
    # await example_subscriptions()


if __name__ == "__main__":
    asyncio.run(main())

