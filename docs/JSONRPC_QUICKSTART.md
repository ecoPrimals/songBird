# JSON-RPC 2.0 Quick Start Guide
## Songbird Universal Gateway

**Status**: ✅ Live and Production Ready  
**Version**: 0.2.1  
**Last Updated**: November 11, 2025

---

## 🚀 Getting Started in 5 Minutes

### **What is JSON-RPC?**

JSON-RPC 2.0 is a lightweight, language-agnostic protocol that lets **any** programming language connect to Songbird using simple HTTP requests. No special libraries needed!

**Supported Languages**: Python, JavaScript, Java, Go, Ruby, PHP, C++, Rust, and more!

---

## 📋 Prerequisites

1. Songbird running on `localhost:8080` (or your configured host/port)
2. Any HTTP client library in your language of choice

---

## 💻 Quick Examples

### **Python** (requests library)

```python
import requests

# Make a JSON-RPC call
def call_songbird(method, params=None):
    response = requests.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    return response.json()

# Health check
health = call_songbird("songbird.health")
print(health)
# Output: {"jsonrpc": "2.0", "result": {"status": "healthy", ...}, "id": 1}

# Get version
version = call_songbird("songbird.version")
print(version["result"])
# Output: {"version": "0.2.1", "name": "Songbird Universal Orchestrator", ...}

# List services
services = call_songbird("songbird.services.list")
print(services["result"])
```

---

### **JavaScript** (fetch API)

```javascript
// Make a JSON-RPC call
async function callSongbird(method, params = null) {
    const response = await fetch("http://localhost:8080/jsonrpc", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body: JSON.stringify({
            jsonrpc: "2.0",
            method: method,
            params: params,
            id: 1
        })
    });
    return await response.json();
}

// Health check
const health = await callSongbird("songbird.health");
console.log(health);

// Get version
const version = await callSongbird("songbird.version");
console.log(version.result);

// Get protocol capabilities
const caps = await callSongbird("songbird.protocol.capabilities");
console.log(caps.result);
```

---

### **cURL** (command line)

```bash
# Health check
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.health",
    "id": 1
  }'

# Get version
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.version",
    "id": 2
  }'

# List services
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.services.list",
    "id": 3
  }'
```

---

## 📚 Available Methods

### **Health & Info**

| Method | Parameters | Description |
|--------|------------|-------------|
| `songbird.health` | None | Health check |
| `songbird.version` | None | Version information |

**Example**:
```python
health = call_songbird("songbird.health")
# Returns: {"status": "healthy", "version": "0.2.1", "uptime_seconds": 0}
```

---

### **Protocol Discovery**

| Method | Parameters | Description |
|--------|------------|-------------|
| `songbird.protocol.capabilities` | None | List available protocols |

**Example**:
```python
caps = call_songbird("songbird.protocol.capabilities")
# Returns: {"protocols": {"http": {...}, "jsonrpc": {...}, ...}}
```

---

### **Service Management**

| Method | Parameters | Description |
|--------|------------|-------------|
| `songbird.services.list` | None | List all services |
| `songbird.services.get` | `{"service_id": "..."}` | Get service details |
| `songbird.services.register` | Service config | Register new service |

**Example**:
```python
# List services
services = call_songbird("songbird.services.list")

# Get specific service
service = call_songbird("songbird.services.get", {
    "service_id": "my-service-123"
})
```

---

### **Compute Tasks**

| Method | Parameters | Description |
|--------|------------|-------------|
| `songbird.compute.schedule` | Task config | Schedule compute task |
| `songbird.compute.status` | `{"task_id": "..."}` | Get task status |

**Example**:
```python
# Schedule compute task (integrates with Toadstool)
task = call_songbird("songbird.compute.schedule", {
    "task": "train_model",
    "language": "python",
    "code": "import torch; ..."
})

# Get task status
status = call_songbird("songbird.compute.status", {
    "task_id": task["result"]["task_id"]
})
```

---

### **Federation**

| Method | Parameters | Description |
|--------|------------|-------------|
| `songbird.federation.peers` | None | List federation peers |
| `songbird.federation.join` | Peer config | Join federation |

**Example**:
```python
# List peers
peers = call_songbird("songbird.federation.peers")

# Join federation
join = call_songbird("songbird.federation.join", {
    "peer_id": "peer-123",
    "endpoint": "http://peer.example.com:8080"
})
```

---

## 🔧 Advanced Usage

### **Error Handling**

JSON-RPC 2.0 uses standard error codes:

```python
def call_songbird_safe(method, params=None):
    response = requests.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    
    data = response.json()
    
    if "error" in data:
        error = data["error"]
        print(f"Error {error['code']}: {error['message']}")
        return None
    
    return data["result"]

# Use it
result = call_songbird_safe("songbird.health")
if result:
    print("Healthy:", result)
```

**Standard Error Codes**:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

---

### **Batch Requests** (Multiple calls in one HTTP request)

```python
# Send multiple requests at once
batch = [
    {"jsonrpc": "2.0", "method": "songbird.health", "id": 1},
    {"jsonrpc": "2.0", "method": "songbird.version", "id": 2},
    {"jsonrpc": "2.0", "method": "songbird.services.list", "id": 3}
]

response = requests.post("http://localhost:8080/jsonrpc", json=batch)
results = response.json()

for result in results:
    print(f"ID {result['id']}: {result.get('result', result.get('error'))}")
```

---

### **Notifications** (Fire and forget, no response)

```python
# Send notification (no id = no response expected)
requests.post("http://localhost:8080/jsonrpc", json={
    "jsonrpc": "2.0",
    "method": "songbird.services.register",
    "params": {"service": "my-service"}
    # Note: No "id" field means this is a notification
})
```

---

## 🌟 Real-World Examples

### **Example 1: Service Discovery**

```python
import requests

def discover_services():
    """Discover all available services in Songbird"""
    response = requests.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": "songbird.services.list",
        "id": 1
    })
    
    data = response.json()
    services = data["result"]["services"]
    
    print(f"Found {len(services)} services:")
    for service in services:
        print(f"  - {service['name']}: {service['endpoint']}")
    
    return services

# Use it
services = discover_services()
```

---

### **Example 2: Distributed Compute**

```python
import requests
import time

def run_distributed_task(code, language="python"):
    """Run code on Toadstool via Songbird"""
    
    # Schedule task
    response = requests.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": "songbird.compute.schedule",
        "params": {
            "task": "execute_code",
            "language": language,
            "code": code
        },
        "id": 1
    })
    
    task_data = response.json()
    task_id = task_data["result"]["task_id"]
    print(f"Task scheduled: {task_id}")
    
    # Poll for status
    while True:
        status_response = requests.post("http://localhost:8080/jsonrpc", json={
            "jsonrpc": "2.0",
            "method": "songbird.compute.status",
            "params": {"task_id": task_id},
            "id": 2
        })
        
        status = status_response.json()["result"]
        print(f"Status: {status['status']}")
        
        if status["status"] in ["completed", "failed"]:
            return status
        
        time.sleep(1)

# Use it
python_code = """
import numpy as np
result = np.mean([1, 2, 3, 4, 5])
print(f"Mean: {result}")
"""

result = run_distributed_task(python_code)
print("Result:", result)
```

---

### **Example 3: Federation Setup**

```python
import requests

def setup_federation(peers):
    """Connect to multiple Songbird peers"""
    
    results = []
    
    for peer in peers:
        response = requests.post("http://localhost:8080/jsonrpc", json={
            "jsonrpc": "2.0",
            "method": "songbird.federation.join",
            "params": {
                "peer_id": peer["id"],
                "endpoint": peer["endpoint"]
            },
            "id": 1
        })
        
        result = response.json()
        results.append(result)
        print(f"Connected to {peer['id']}: {result['result']['status']}")
    
    return results

# Use it
peers = [
    {"id": "tower-a", "endpoint": "http://tower-a:8080"},
    {"id": "tower-b", "endpoint": "http://tower-b:8080"},
    {"id": "tower-c", "endpoint": "http://tower-c:8080"}
]

setup_federation(peers)
```

---

## 🔐 Security Best Practices

### **1. Use HTTPS in Production**

```python
# Always use HTTPS in production
SONGBIRD_URL = "https://songbird.example.com/jsonrpc"  # Not HTTP!
```

### **2. Add Authentication**

```python
import requests

def call_songbird_authenticated(method, params=None, api_key=None):
    headers = {
        "Content-Type": "application/json"
    }
    
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"
    
    response = requests.post(
        "https://songbird.example.com/jsonrpc",
        headers=headers,
        json={
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        }
    )
    
    return response.json()

# Use it
result = call_songbird_authenticated(
    "songbird.health",
    api_key="your-api-key-here"
)
```

### **3. Handle Timeouts**

```python
import requests

def call_songbird_with_timeout(method, params=None, timeout=10):
    try:
        response = requests.post(
            "http://localhost:8080/jsonrpc",
            json={
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": 1
            },
            timeout=timeout  # 10 second timeout
        )
        return response.json()
    except requests.Timeout:
        print(f"Request timed out after {timeout} seconds")
        return None
```

---

## 📊 Performance Tips

### **1. Reuse HTTP Connections**

```python
import requests

# Create a session to reuse connections
session = requests.Session()

def call_songbird_fast(method, params=None):
    return session.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    }).json()

# Much faster for multiple calls
for i in range(100):
    result = call_songbird_fast("songbird.health")
```

### **2. Use Batch Requests**

```python
# Instead of 10 separate HTTP requests:
# DON'T DO THIS
for i in range(10):
    call_songbird(f"songbird.services.get", {"service_id": f"service-{i}"})

# DO THIS (1 HTTP request)
batch = [
    {
        "jsonrpc": "2.0",
        "method": "songbird.services.get",
        "params": {"service_id": f"service-{i}"},
        "id": i
    }
    for i in range(10)
]

response = requests.post("http://localhost:8080/jsonrpc", json=batch)
results = response.json()
```

### **3. Use Async for High Throughput**

```python
import asyncio
import aiohttp

async def call_songbird_async(session, method, params=None):
    async with session.post("http://localhost:8080/jsonrpc", json={
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    }) as response:
        return await response.json()

async def main():
    async with aiohttp.ClientSession() as session:
        # Make 100 concurrent requests
        tasks = [
            call_songbird_async(session, "songbird.health")
            for _ in range(100)
        ]
        results = await asyncio.gather(*tasks)
        print(f"Completed {len(results)} requests")

asyncio.run(main())
```

---

## 🐛 Troubleshooting

### **Problem: Connection Refused**

```bash
# Check if Songbird is running
curl http://localhost:8080/health

# Check the correct port
ss -tlnp | grep 8080
```

### **Problem: Method Not Found**

```python
# Make sure method name is correct (case-sensitive!)
# WRONG: "songbird.Health"
# RIGHT: "songbird.health"

response = call_songbird("songbird.health")  # Correct!
```

### **Problem: Invalid Params**

```python
# Always pass params as a dict (object) or array
# WRONG:
call_songbird("songbird.services.get", "service-123")

# RIGHT:
call_songbird("songbird.services.get", {"service_id": "service-123"})
```

---

## 📖 Additional Resources

- **Full Specification**: `specs/PROGRESSIVE_PROTOCOL_ENHANCEMENT_SPEC.md`
- **Architecture**: `specs/ECOPRIMALS_ARCHITECTURE_CLARITY.md`
- **API Reference**: `crates/songbird-orchestrator/src/server/jsonrpc_api.rs`
- **JSON-RPC 2.0 Spec**: https://www.jsonrpc.org/specification

---

## 🚀 Next Steps

1. **Try the examples above** - Start with health check
2. **Explore available methods** - Use `songbird.protocol.capabilities`
3. **Build your integration** - Use your favorite language
4. **Scale up** - Use batch requests and async for performance
5. **Deploy to production** - Use HTTPS and authentication

---

## 💡 Pro Tips

1. **Use a JSON-RPC library** if available in your language (e.g., `jsonrpcclient` for Python)
2. **Cache protocol capabilities** - They don't change often
3. **Implement retries** for transient failures
4. **Monitor latency** - JSON-RPC typically < 2ms
5. **Use WebSocket** for real-time updates (coming in Phase 4!)

---

## 🎉 You're Ready!

You now have universal access to Songbird from **any programming language**. The JSON-RPC gateway handles the complexity, you just send simple HTTP requests!

**Happy coding!** 🚀

---

*Songbird v0.2.1 - Universal Language Access*  
*100% Rust Core + Universal Compatibility = Best of Both Worlds!* ✨

