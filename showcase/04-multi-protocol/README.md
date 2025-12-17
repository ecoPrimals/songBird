# 🚀 Multi-Protocol Federation Showcase

**Status:** ✅ Ready for Testing  
**Protocols:** HTTP, HTTPS, JSON-RPC, tarpc  
**Scenario:** Local tower-to-tower with protocol escalation

---

## 🎯 Overview

This showcase demonstrates Songbird's multi-protocol federation capabilities with real tower-to-tower communication. You'll see:

1. **Protocol Discovery** - Towers advertise their capabilities
2. **Intelligent Negotiation** - Automatic best protocol selection
3. **Protocol Escalation** - HTTP → JSON-RPC → tarpc
4. **Concurrent Protocols** - Multiple protocols active simultaneously
5. **Performance Comparison** - Real latency measurements

---

## 📋 Prerequisites

- 2+ local towers (or use loopback)
- Ports 8080, 8081, 8443, 9080, 9081, 9443 available
- `curl`, `jq` installed
- Rust 1.70+ for tarpc client

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Tower A (8080/8443/8081)          │
│  ┌──────────┐  ┌───────────┐  ┌────────┐           │
│  │   HTTP   │  │ JSON-RPC  │  │ tarpc  │           │
│  │  :8080   │  │  /jsonrpc │  │ :8081  │           │
│  └────┬─────┘  └─────┬─────┘  └────┬───┘           │
│       │              │              │                │
└───────┼──────────────┼──────────────┼───────────────┘
        │              │              │
        │ Protocol     │ Protocol     │ Protocol
        │ Discovery    │ Negotiation  │ Escalation
        │              │              │
┌───────┼──────────────┼──────────────┼───────────────┐
│       │              │              │                │
│  ┌────▼─────┐  ┌─────▼─────┐  ┌────▼───┐           │
│  │   HTTP   │  │ JSON-RPC  │  │ tarpc  │           │
│  │  :9080   │  │  /jsonrpc │  │ :9081  │           │
│  └──────────┘  └───────────┘  └────────┘           │
│                    Tower B (9080/9443/9081)          │
└─────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Step 1: Start Tower A (Primary)

```bash
# Terminal 1: Tower A with all protocols
export SONGBIRD_PORT=8080
export SONGBIRD_TLS_PORT=8443
export SONGBIRD_TARPC_PORT=8081
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TARPC_ENABLED=true
export SONGBIRD_JSONRPC_ENABLED=true
export SONGBIRD_NODE_NAME="tower-a"

cargo run --release --bin songbird-orchestrator
```

### Step 2: Start Tower B (Secondary)

```bash
# Terminal 2: Tower B with all protocols
export SONGBIRD_PORT=9080
export SONGBIRD_TLS_PORT=9443
export SONGBIRD_TARPC_PORT=9081
export SONGBIRD_TLS_ENABLED=true
export SONGBIRD_TARPC_ENABLED=true
export SONGBIRD_JSONRPC_ENABLED=true
export SONGBIRD_NODE_NAME="tower-b"

cargo run --release --bin songbird-orchestrator
```

### Step 3: Run the Demo

```bash
# Terminal 3: Run the showcase demo
./showcase/04-multi-protocol/demo_protocol_escalation.sh
```

---

## 📝 Manual Testing

### Test 1: Protocol Discovery

```bash
# Discover protocols on Tower A
curl -s http://localhost:8080/api/protocol/capabilities | jq .

# Expected output:
{
  "songbird_version": "0.1.0",
  "protocols": {
    "http": { "version": "1.1", ... },
    "json-rpc": { "version": "2.0", ... },
    "tarpc": { "version": "0.34", ... }
  },
  "preferred_protocol": "tarpc",
  "fallback_protocol": "http"
}
```

### Test 2: Protocol Negotiation

```bash
# Negotiate best protocol
curl -X POST http://localhost:8080/api/protocol/negotiate \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "test-client",
    "client_protocols": ["http", "json-rpc", "tarpc"],
    "preferred": "tarpc"
  }' | jq .

# Expected: Upgrade to tarpc with token
{
  "negotiation_id": "nego_...",
  "selected_protocol": "tarpc",
  "upgrade_available": true,
  "upgrade_token": "upgrade_...",
  "endpoints": {
    "rpc": "tarpc://localhost:8081"
  }
}
```

### Test 3: HTTP API Call

```bash
# Basic HTTP call (slowest)
time curl -s http://localhost:8080/health | jq .

# Expected: ~5-10ms latency
```

### Test 4: JSON-RPC Call

```bash
# JSON-RPC call (faster)
time curl -s -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.version",
    "params": [],
    "id": 1
  }' | jq .

# Expected: ~2-3ms latency
```

### Test 5: Tower-to-Tower Communication

```bash
# Register Tower B on Tower A
curl -X POST http://localhost:8080/api/federation/register \
  -H "Content-Type: application/json" \
  -d '{
    "node_id": "tower-b",
    "address": "localhost:9080",
    "capabilities": ["orchestration", "compute"],
    "metadata": {
      "protocols": ["http", "json-rpc", "tarpc"]
    }
  }' | jq .

# List federated towers
curl -s http://localhost:8080/api/federation/towers | jq .
```

---

## 🔬 Performance Comparison

Run the performance benchmark:

```bash
./showcase/04-multi-protocol/benchmark_protocols.sh
```

Expected results:

| Protocol  | Avg Latency | Requests/sec | Use Case              |
|-----------|-------------|--------------|------------------------|
| HTTP      | ~5-10ms     | ~100-200     | Web dashboards        |
| JSON-RPC  | ~2-3ms      | ~300-500     | Universal RPC         |
| tarpc     | ~50-100μs   | ~10,000+     | High-perf Rust-to-Rust|

**Performance Gain:** tarpc is 40-100x faster than HTTP!

---

## 🎭 Demo Scenarios

### Scenario 1: Progressive Enhancement

1. Client starts with basic HTTP
2. Discovers JSON-RPC is available
3. Upgrades to JSON-RPC (2-3x speedup)
4. Discovers tarpc is available
5. Upgrades to tarpc (40-100x speedup)

**Script:** `./showcase/04-multi-protocol/demo_progressive_enhancement.sh`

### Scenario 2: Multi-Client Concurrent

1. Client A uses HTTP (legacy)
2. Client B uses JSON-RPC (modern)
3. Client C uses tarpc (high-performance)
4. All communicate simultaneously

**Script:** `./showcase/04-multi-protocol/demo_concurrent_clients.sh`

### Scenario 3: Protocol Fallback

1. Client prefers tarpc
2. tarpc not available on target
3. Falls back to JSON-RPC
4. Still works (graceful degradation)

**Script:** `./showcase/04-multi-protocol/demo_fallback.sh`

---

## 📊 Monitoring

### Watch Protocol Usage

```bash
# Monitor active protocols
watch -n 1 'curl -s http://localhost:8080/api/protocol/capabilities | jq .protocols'
```

### Monitor Federation

```bash
# Watch tower list
watch -n 2 'curl -s http://localhost:8080/api/federation/towers | jq .'
```

### Monitor Performance

```bash
# Real-time latency
./showcase/04-multi-protocol/monitor_latency.sh
```

---

## 🐛 Troubleshooting

### Protocol Not Available

**Problem:** `tarpc` not in capabilities list

**Solution:**
```bash
# Ensure tarpc is enabled
export SONGBIRD_TARPC_ENABLED=true
# Restart Songbird
```

### Connection Refused

**Problem:** Can't connect to port 8081 (tarpc)

**Solution:**
```bash
# Check if port is listening
lsof -i :8081

# Verify tarpc server started
grep "tarpc" <songbird-log>
```

### TLS Certificate Errors

**Problem:** `certificate verify failed`

**Solution:**
```bash
# Use -k for self-signed certs
curl -k https://localhost:8443/...

# Or generate proper certs
./scripts/generate_certs.sh
```

---

## 📚 Related Documentation

- [Multi-Protocol Federation Plan](../../docs/MULTI_PROTOCOL_FEDERATION_PLAN.md)
- [JSON-RPC API Guide](../../docs/JSONRPC_GUIDE.md)
- [Deployment Guide](../../docs/DEPLOYMENT_GUIDE_MULTI_PROTOCOL.md)
- [Protocol API Reference](../../crates/songbird-orchestrator/src/server/protocol_api.rs)

---

## ✅ Success Criteria

- [ ] Tower A and Tower B both start successfully
- [ ] All 7 protocols are advertised (HTTP, HTTPS, JSON-RPC, tarpc, WS, WSS, BTSP)
- [ ] Protocol negotiation returns correct selected_protocol
- [ ] Upgrade tokens are generated
- [ ] Tower-to-tower federation works
- [ ] Performance: tarpc < 1ms, JSON-RPC < 5ms, HTTP < 10ms
- [ ] Multiple clients can use different protocols simultaneously
- [ ] Graceful fallback when preferred protocol unavailable

---

## 🎯 Next Steps

1. **BearDog Integration** - Add BTSP genetic crypto
2. **Protocol Upgrade Handshake** - Live protocol switching
3. **Load Balancing** - Distribute across protocols
4. **Cross-Internet Federation** - Test over WAN

---

**Status:** ✅ Ready for Testing  
**Updated:** December 17, 2025  
**Maintainer:** ecoPrimals Team

