# 🚀 Songbird Deployment Guide

**Last Updated:** November 8, 2025  
**Status:** Phase 3 Complete (Chunked Upload)

---

## 📖 Quick Links

- **HTTP Deployment:** [HTTP_DEPLOYMENT_GUIDE.md](HTTP_DEPLOYMENT_GUIDE.md)
- **Adaptive System:** [ADAPTIVE_DEPLOYMENT_DESIGN.md](ADAPTIVE_DEPLOYMENT_DESIGN.md)
- **Roadmap:** [ADAPTIVE_DEPLOYMENT_ROADMAP.md](ADAPTIVE_DEPLOYMENT_ROADMAP.md)
- **Phase 3 Status:** [PHASE_3_COMPLETE.md](PHASE_3_COMPLETE.md)
- **Specification:** [specs/ADAPTIVE_DEPLOYMENT_SPECIFICATION.md](specs/ADAPTIVE_DEPLOYMENT_SPECIFICATION.md)

---

## 🎯 Overview

Songbird provides an **adaptive deployment system** that automatically selects the best method for deploying binaries across your federation based on:

- Binary size
- Network capabilities (LAN/VPN/Internet)
- Server resources (CPU, memory, storage)
- Available bandwidth

### Deployment Methods

| Method | Binary Size | Use Case |
|--------|-------------|----------|
| **Single Upload** | < 50MB | Fast, simple, single HTTP POST |
| **Chunked Upload** | 2MB - 1GB | Reliable, negotiated chunks |
| **Streaming** | Unlimited | For huge binaries (Phase 4) |

---

## 🚀 Quick Start

### 1. Deploy a Service via HTTP

```bash
songbird-deploy deploy-http \
  --tower http://192.168.1.144:8080 \
  --binary ./target/release/my-service \
  --service my-service-name \
  --env SERVICE_HOST=0.0.0.0 \
  --env SERVICE_PORT=9000
```

**What Happens:**
1. Client queries server capabilities
2. Binary size analyzed (e.g., 7.7MB)
3. Method selected (single vs chunked)
4. Binary uploaded automatically
5. Service deployed and started

### 2. Deploy Compute Bridge

```bash
# Build the bridge
cargo build --release --bin songbird-compute-bridge

# Deploy to Tower B
songbird-deploy deploy-http \
  --tower http://tower-b:8080 \
  --binary ./target/release/songbird-compute-bridge \
  --service compute-bridge \
  --env COMPUTE_HOST=0.0.0.0 \
  --env COMPUTE_PORT=9000 \
  --env SONGBIRD_FEDERATION_ENDPOINT=http://tower-a:8080
```

### 3. Check Deployment Status

```bash
# Query deployments
curl http://192.168.1.144:8080/api/deployment/list | jq

# Check specific deployment
curl http://192.168.1.144:8080/api/deployment/status/deploy-12345 | jq
```

---

## 📊 How It Works

### Adaptive Selection Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Query Capabilities                                       │
│    GET /api/deployment/capabilities                         │
│    → Network: LAN, Bandwidth: 1Gbps, Limits: 50MB/1000MB  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. Analyze Binary                                           │
│    Size: 7.7MB                                              │
│    Compression: Optional                                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. Select Method                                            │
│    7.7MB < 50MB → Single Upload ✓                          │
│    Method: POST /api/deployment/binary                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Deploy                                                   │
│    Upload → Make executable → Start service                 │
│    Result: deployment_id, status, service_url               │
└─────────────────────────────────────────────────────────────┘
```

### Chunked Upload Flow (60MB binary)

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Negotiate                                                │
│    POST /api/deployment/negotiate                           │
│    { binary_size_mb: 60.0, service_name: "my-service" }    │
│    → { negotiation_id, chunk_size_mb: 10, total_chunks: 6 }│
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. Upload Chunks (6 chunks of 10MB)                        │
│    POST /api/deployment/chunk/:neg_id/0 (10MB)             │
│    POST /api/deployment/chunk/:neg_id/1 (10MB)             │
│    POST /api/deployment/chunk/:neg_id/2 (10MB)             │
│    POST /api/deployment/chunk/:neg_id/3 (10MB)             │
│    POST /api/deployment/chunk/:neg_id/4 (10MB)             │
│    POST /api/deployment/chunk/:neg_id/5 (10MB)             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. Finalize                                                 │
│    POST /api/deployment/finalize/:neg_id                    │
│    Server: Assemble chunks → Make executable → Deploy      │
│    Result: deployment_id, status, service_url               │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 Server Configuration

### Enable HTTP Deployment API

The deployment API is enabled by default when you start the orchestrator:

```bash
export SERVICE_ID=tower-orchestrator
export SERVICE_PORT=8080
export SONGBIRD_HOST=0.0.0.0

./target/release/songbird-orchestrator
```

### Endpoints

```
GET  /api/deployment/capabilities           - Query deployment capabilities
POST /api/deployment/binary                 - Single upload
POST /api/deployment/negotiate              - Start chunked upload
POST /api/deployment/chunk/:neg_id/:index   - Upload chunk
POST /api/deployment/finalize/:neg_id       - Finalize chunked
GET  /api/deployment/status/:id             - Check status
GET  /api/deployment/list                   - List deployments
DELETE /api/deployment/:id                  - Stop deployment
```

### Capabilities Response

```json
{
  "node_id": "tower-a",
  "timestamp": "2025-11-08T...",
  "network": {
    "type": "lan",
    "bandwidth_estimate": {
      "download_mbps": 1000,
      "upload_mbps": 1000,
      "latency_ms": 1
    }
  },
  "deployment_methods": {
    "single": {
      "enabled": true,
      "max_size_mb": 50,
      "recommended_for": "< 50MB"
    },
    "chunked": {
      "enabled": true,
      "max_total_size_mb": 1000,
      "chunk_size_mb": 10,
      "recommended_for": "2MB - 1GB"
    }
  },
  "resources": {
    "available_storage_gb": 1000,
    "available_memory_gb": 32,
    "cpu_cores": 16,
    "max_concurrent_deployments": 32
  }
}
```

---

## 🎯 Use Cases

### 1. Deploy Toadstool Compute Service

```bash
# On Tower A, deploy Toadstool to Tower B
songbird-deploy deploy-http \
  --tower http://tower-b:8080 \
  --binary ./toadstool/target/release/toadstool-compute \
  --service toadstool-compute \
  --env TOADSTOOL_PORT=9001 \
  --env SONGBIRD_ENDPOINT=http://tower-a:8080
```

### 2. Deploy BearDog Security Service

```bash
songbird-deploy deploy-http \
  --tower http://tower-b:8080 \
  --binary ./beardog/target/release/beardog-validator \
  --service beardog-validator \
  --env BEARDOG_PORT=8443
```

### 3. Deploy NestGate Data Service

```bash
songbird-deploy deploy-http \
  --tower http://tower-b:8080 \
  --binary ./nestgate/target/release/nestgate-storage \
  --service nestgate-storage \
  --env NESTGATE_PORT=9002
```

---

## 🐛 Troubleshooting

### Deployment Failed: Binary Too Large

**Error:** `Binary exceeds max size (50MB for single upload)`

**Solution:** Binary will automatically use chunked upload. If still failing:
```bash
# Check server capabilities
curl http://tower:8080/api/deployment/capabilities | jq '.deployment_methods'

# Verify chunked is enabled
# chunked.enabled should be true
```

### Connection Refused

**Error:** `Connection refused (os error 111)`

**Solution:**
```bash
# Check if orchestrator is running
ps aux | grep songbird-orchestrator

# Restart orchestrator
./target/release/songbird-orchestrator
```

### Multipart Parse Error

**Error:** `Error parsing multipart/form-data request`

**Solution:** This is a known issue (Phase 3 debugging). Workaround:
```bash
# Use smaller binary for testing
# Or wait for multipart fix (in progress)
```

---

## 📚 Related Documentation

- **HTTP API Details:** [HTTP_DEPLOYMENT_GUIDE.md](HTTP_DEPLOYMENT_GUIDE.md)
- **Adaptive System Design:** [ADAPTIVE_DEPLOYMENT_DESIGN.md](ADAPTIVE_DEPLOYMENT_DESIGN.md)
- **Implementation Roadmap:** [ADAPTIVE_DEPLOYMENT_ROADMAP.md](ADAPTIVE_DEPLOYMENT_ROADMAP.md)
- **Full Specification:** [specs/ADAPTIVE_DEPLOYMENT_SPECIFICATION.md](specs/ADAPTIVE_DEPLOYMENT_SPECIFICATION.md)
- **Tower Setup:** [TOWER_SETUP_QUICK.md](TOWER_SETUP_QUICK.md)
- **Toadstool Integration:** [TOADSTOOL_INTEGRATION_PLAN.md](TOADSTOOL_INTEGRATION_PLAN.md)

---

## 🚀 Next Steps

1. **Deploy compute-bridge** to Tower B
2. **Test chunked upload** with 60MB binary
3. **Integrate Toadstool** for GPU compute
4. **Add BearDog** for security
5. **Scale to N towers** for distributed HPC

---

**Status:** Phase 3 complete, multipart debugging in progress  
**Supported:** Single upload (< 50MB), Chunked upload (2MB-1GB)  
**Coming:** Streaming upload (Phase 4), parallel chunk upload (Phase 3.5)

