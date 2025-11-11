# WebSocket Real-Time Communication - Quick Start Guide

**Version**: 0.2.1  
**Last Updated**: November 11, 2025 - Phase 4  
**Status**: ✅ Production Ready

---

## 📖 Overview

Songbird's WebSocket API provides **real-time bidirectional communication** with event subscriptions, status queries, and service discovery. Perfect for monitoring, dashboards, and real-time updates.

### **Key Features**

- ✅ **Real-time bidirectional communication**
- ✅ **Event subscription system** (pub-sub model)
- ✅ **Query federation status and services**
- ✅ **Ping/pong keep-alive**
- ✅ **Automatic reconnection** (client libraries)
- ✅ **Multi-client support** (broadcast to many)
- ✅ **JSON message protocol** (language-agnostic)
- ✅ **Shared port 8080** (with HTTP/REST)

---

## 🚀 Quick Start

### **1. Connect to WebSocket Server**

**Endpoint**: `ws://localhost:8080/api/ws/ws`

**Example (Python)**:
```python
from websocket_client import SongbirdWebSocketClient

client = SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws')
await client.connect()
```

**Example (JavaScript)**:
```javascript
const { SongbirdWebSocketClient } = require('./websocket-client');

const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
await client.connect();
```

### **2. Subscribe to Events**

```python
# Subscribe to service and health updates
await client.subscribe(['service_update', 'health_update'])
```

### **3. Query Status**

```python
# Get federation status
status = await client.queryStatus()
print(f"Services: {status.total_services}, Peers: {status.total_peers}")
```

### **4. Listen for Events**

```python
# Python: async iterator
async for event in client.listen():
    if event['type'] == 'service_update':
        print(f"Service updated: {event['service_name']}")
```

```javascript
// JavaScript: event emitter
client.on('service_update', (event) => {
    console.log('Service updated:', event.service_name);
});
```

---

## 📨 Message Types

### **Client → Server Messages**

#### **1. Subscribe**
Subscribe to event types.

```json
{
    "type": "subscribe",
    "events": ["service_update", "health_update"]
}
```

**Response**:
```json
{
    "type": "ack",
    "message": "Subscribed to 2 event(s)"
}
```

---

#### **2. Unsubscribe**
Unsubscribe from event types.

```json
{
    "type": "unsubscribe",
    "events": ["service_update"]
}
```

**Response**:
```json
{
    "type": "ack",
    "message": "Unsubscribed from 1 event(s)"
}
```

---

#### **3. Ping**
Keep connection alive.

```json
{
    "type": "ping",
    "data": "optional-data"
}
```

**Response**:
```json
{
    "type": "pong",
    "data": "optional-data"
}
```

---

#### **4. Query Status**
Get federation status.

```json
{
    "type": "query_status"
}
```

**Response**:
```json
{
    "type": "federation_status",
    "total_services": 5,
    "total_peers": 2,
    "uptime_seconds": 3600
}
```

---

#### **5. Query Services**
Discover services by capability.

```json
{
    "type": "query_services",
    "capabilities": ["ml", "training"]
}
```

**Response**:
```json
{
    "type": "service_list",
    "services": [
        {
            "name": "ml-service",
            "address": "localhost",
            "port": 8093,
            "capabilities": ["ml", "training"]
        }
    ]
}
```

---

### **Server → Client Events**

#### **1. Service Update**
Service registration or status change.

```json
{
    "type": "service_update",
    "service_name": "ml-service",
    "status": "running",
    "address": "localhost:8093"
}
```

---

#### **2. Health Update**
Service health status change.

```json
{
    "type": "health_update",
    "service_name": "ml-service",
    "healthy": true,
    "message": "All checks passing"
}
```

---

#### **3. Federation Status**
Periodic federation status updates.

```json
{
    "type": "federation_status",
    "total_services": 5,
    "total_peers": 2,
    "uptime_seconds": 3600
}
```

---

#### **4. Error**
Error message.

```json
{
    "type": "error",
    "message": "Invalid message format",
    "code": "INVALID_JSON"
}
```

---

## 🐍 Python Client Usage

### **Installation**

```bash
pip install websockets
```

### **Basic Example**

```python
import asyncio
from websocket_client import SongbirdWebSocketClient

async def main():
    # Connect
    client = SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws')
    await client.connect()
    
    # Query status
    status = await client.query_status()
    print(f"Services: {status.total_services}")
    
    # Subscribe to events
    await client.subscribe(['service_update', 'health_update'])
    
    # Listen for events
    async for event in client.listen():
        print(f"Event: {event['type']}")
    
    # Clean up
    await client.close()

asyncio.run(main())
```

### **Event Subscription Example**

```python
async def monitor_services():
    client = SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws')
    await client.connect()
    
    # Subscribe to service updates
    await client.subscribe(['service_update'])
    
    # Listen indefinitely
    async for event in client.listen():
        if event['type'] == 'service_update':
            print(f"Service: {event['service_name']}")
            print(f"Status: {event['status']}")
            print(f"Address: {event['address']}")

asyncio.run(monitor_services())
```

### **Service Discovery Example**

```python
async def find_ml_services():
    client = SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws')
    await client.connect()
    
    # Find services with ML capability
    services = await client.query_services(['ml'])
    
    for service in services:
        print(f"Found: {service.name} at {service.address}:{service.port}")
    
    await client.close()

asyncio.run(find_ml_services())
```

---

## 🟨 JavaScript Client Usage

### **Installation**

```bash
npm install ws
```

### **Basic Example**

```javascript
const { SongbirdWebSocketClient } = require('./websocket-client');

async function main() {
    // Connect
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    await client.connect();
    
    // Query status
    const status = await client.queryStatus();
    console.log(`Services: ${status.totalServices}`);
    
    // Subscribe to events
    await client.subscribe(['service_update', 'health_update']);
    
    // Listen for events
    client.on('service_update', (event) => {
        console.log('Service updated:', event.service_name);
    });
    
    // Keep running
    await new Promise(resolve => setTimeout(resolve, 30000));
    
    // Clean up
    await client.close();
}

main().catch(console.error);
```

### **Event Subscription Example**

```javascript
async function monitorServices() {
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    await client.connect();
    
    // Subscribe to service updates
    await client.subscribe(['service_update']);
    
    // Listen for events
    client.on('service_update', (event) => {
        console.log('Service:', event.service_name);
        console.log('Status:', event.status);
        console.log('Address:', event.address);
    });
    
    // Keep running
    await new Promise(() => {}); // Run forever
}

monitorServices().catch(console.error);
```

### **Service Discovery Example**

```javascript
async function findMLServices() {
    const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws');
    await client.connect();
    
    // Find services with ML capability
    const services = await client.queryServices(['ml']);
    
    services.forEach(service => {
        console.log(`Found: ${service.name} at ${service.address}:${service.port}`);
    });
    
    await client.close();
}

findMLServices().catch(console.error);
```

---

## 🔧 Advanced Features

### **Automatic Reconnection**

Both Python and JavaScript clients support automatic reconnection:

```python
# Python: auto-reconnect enabled by default
client = SongbirdWebSocketClient(
    'ws://localhost:8080/api/ws/ws',
    auto_reconnect=True,
    max_reconnect_attempts=5
)
```

```javascript
// JavaScript: auto-reconnect enabled by default
const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws', {
    autoReconnect: true,
    maxReconnectAttempts: 5
});
```

### **Connection Events**

```javascript
// JavaScript event handling
client.on('connected', () => {
    console.log('Connected to Songbird');
});

client.on('disconnected', () => {
    console.log('Disconnected from Songbird');
});

client.on('error', (error) => {
    console.error('WebSocket error:', error);
});
```

### **Custom Ping Interval**

```python
# Python: custom ping interval
client = SongbirdWebSocketClient(
    'ws://localhost:8080/api/ws/ws',
    ping_interval=30.0,  # seconds
    ping_timeout=10.0
)
```

```javascript
// JavaScript: custom ping interval
const client = new SongbirdWebSocketClient('ws://localhost:8080/api/ws/ws', {
    pingInterval: 30,  // seconds
    pingTimeout: 10
});
```

---

## 📊 Event Types

| Event Type | Description | When Triggered |
|------------|-------------|----------------|
| `service_update` | Service registration/update | Service joins/updates |
| `health_update` | Health status change | Health check fails/recovers |
| `federation_status` | Federation status | Periodic updates |
| `peer_update` | Peer connection change | Peer joins/leaves |
| `task_update` | Task execution update | Task status changes |

---

## 🎯 Use Cases

### **1. Real-Time Monitoring Dashboard**

```python
# Monitor all events for dashboard
await client.subscribe([
    'service_update',
    'health_update',
    'federation_status'
])

async for event in client.listen():
    update_dashboard(event)
```

### **2. Service Health Monitoring**

```python
# Alert on health changes
await client.subscribe(['health_update'])

async for event in client.listen():
    if not event['healthy']:
        send_alert(event['service_name'], event['message'])
```

### **3. Dynamic Service Discovery**

```python
# Discover and track ML services
services = await client.query_services(['ml'])
await client.subscribe(['service_update'])

# Track changes
async for event in client.listen():
    if 'ml' in event.get('capabilities', []):
        update_service_list(event)
```

---

## ⚡ Performance

### **Latency**

| Metric | Value |
|--------|-------|
| Connection time | ~10-50ms |
| Message latency | ~1-2ms |
| Ping/pong RTT | ~1-5ms |
| Event delivery | ~1-10ms |

### **Throughput**

| Metric | Value |
|--------|-------|
| Messages/sec | ~1,000-5,000 |
| Events/sec | ~500-2,000 |
| Concurrent clients | 100+ |

### **Resource Usage**

| Metric | Value |
|--------|-------|
| Memory per connection | ~50-100 KB |
| CPU per connection | <1% |
| Network overhead | Minimal (JSON) |

---

## 🛡️ Best Practices

### **1. Connection Management**

✅ **DO**:
- Use connection pooling for multiple operations
- Implement exponential backoff for reconnection
- Close connections gracefully
- Handle disconnection events

❌ **DON'T**:
- Create new connections for each operation
- Ignore connection errors
- Leave connections open indefinitely

### **2. Event Subscriptions**

✅ **DO**:
- Subscribe only to needed event types
- Unsubscribe when done
- Filter events on the client side
- Handle events asynchronously

❌ **DON'T**:
- Subscribe to all events if not needed
- Block on event handling
- Ignore subscription errors

### **3. Error Handling**

✅ **DO**:
- Catch and log all errors
- Implement retry logic
- Validate messages before sending
- Handle timeouts gracefully

❌ **DON'T**:
- Ignore error responses
- Crash on connection loss
- Send invalid JSON

### **4. Message Format**

✅ **DO**:
- Use proper JSON format
- Include all required fields
- Validate message types
- Handle unknown message types

❌ **DON'T**:
- Send binary messages (not supported)
- Send malformed JSON
- Assume message order

---

## 🐛 Troubleshooting

### **Connection Failed**

**Problem**: Cannot connect to WebSocket server.

**Solutions**:
1. Check if Songbird is running: `curl http://localhost:8080/health`
2. Verify URL: `ws://localhost:8080/api/ws/ws`
3. Check firewall settings
4. Try IPv4 explicitly: `ws://127.0.0.1:8080/api/ws/ws`

### **Connection Drops**

**Problem**: Connection frequently disconnects.

**Solutions**:
1. Check network stability
2. Increase ping interval
3. Enable auto-reconnect
4. Check server logs for errors

### **No Events Received**

**Problem**: Subscribed but not receiving events.

**Solutions**:
1. Verify subscription: Check for `ack` response
2. Check event types: Ensure correct spelling
3. Trigger test event: Manually trigger an event
4. Check server logs: Verify broadcasting

### **High Latency**

**Problem**: Messages take too long.

**Solutions**:
1. Check network latency: `ping localhost`
2. Reduce message size
3. Use binary protocol (tarpc) for critical paths
4. Check server load

---

## 📚 Additional Resources

- **Client Libraries**:
  - Python: `examples/clients/python/websocket_client.py`
  - JavaScript: `examples/clients/javascript/websocket-client.js`

- **Documentation**:
  - [JSONRPC_QUICKSTART.md](JSONRPC_QUICKSTART.md) - JSON-RPC API
  - [TARPC_PERFORMANCE.md](TARPC_PERFORMANCE.md) - High-performance RPC
  - [NEXT_STEPS_HANDOFF.md](../NEXT_STEPS_HANDOFF.md) - Project status

- **Specifications**:
  - `specs/PROGRESSIVE_PROTOCOL_ENHANCEMENT_SPEC.md` - Protocol strategy
  - `specs/ECOPRIMALS_ARCHITECTURE_CLARITY.md` - Architecture overview

---

## 🎯 Protocol Comparison

| Protocol | Port | Latency | Use Case |
|----------|------|---------|----------|
| HTTP/REST | 8080 | ~5ms | Universal baseline |
| JSON-RPC | 8080 | ~2ms | Multi-language RPC |
| **WebSocket** | 8080 | ~1ms | **Real-time events** 🔌 |
| tarpc | 8091 | ~50μs | High-performance Rust ⚡ |

**When to use WebSocket**:
- ✅ Real-time monitoring and dashboards
- ✅ Event-driven architectures
- ✅ Long-lived connections
- ✅ Bidirectional communication
- ✅ Push notifications

**When NOT to use WebSocket**:
- ❌ Simple one-off requests (use HTTP/REST)
- ❌ High-performance Rust-to-Rust (use tarpc)
- ❌ Large file transfers (use HTTP with chunked upload)

---

## 🚀 Next Steps

1. **Try the examples**: Run Python or JavaScript client examples
2. **Build a dashboard**: Create a real-time monitoring dashboard
3. **Integrate**: Add WebSocket support to your application
4. **Monitor**: Use WebSocket for service health monitoring
5. **Scale**: Deploy with load balancing for multiple clients

---

**Songbird v0.2.1 - Real-Time Communication Ready!** 🔌  
*Production-Ready + IPv6 + Multi-Language + Real-Time Events* ✨

