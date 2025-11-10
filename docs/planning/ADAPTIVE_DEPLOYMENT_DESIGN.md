# 🎯 Adaptive Deployment System Design

**Songbird: Negotiable, Configurable, Live-Adaptive Deployment**

---

## 🌟 Vision

Instead of hardcoded limits, Songbird should **negotiate** deployment capabilities between nodes based on:
- **Service needs** (NestGate = huge data, Toadstool = fast access)
- **Node capabilities** (bandwidth, storage, memory)
- **Network conditions** (LAN vs internet, latency)
- **Real-time adaptation** (start small, scale up if needed)

---

## 🏗️ Architecture

### Current State (Static Limits)
```
Tower A ────[HTTP POST 50MB max]────> Tower B
          ❌ Fixed limit
          ❌ No negotiation
          ❌ One-size-fits-all
```

### Future State (Adaptive Negotiation)
```
Tower A: "I need to deploy 100MB binary"
Tower B: "I can accept 50MB at a time, or stream unlimited"
Tower A: "Let's use streaming"
Tower B: "Ready. Send chunks."
Tower A: ────[Chunks: 10MB each]────> Tower B
Tower B: "Chunk 1 ✅, Chunk 2 ✅, ..."
Tower B: "Assembly complete ✅"
```

---

## 📊 Capability Negotiation Protocol

### 1. Discovery Phase

**GET /api/deployment/capabilities**

Response:
```json
{
  "node_id": "tower-b-strandgate",
  "deployment_capabilities": {
    "methods": ["http-single", "http-chunked", "http-stream", "ssh"],
    "max_single_upload_mb": 50,
    "max_chunked_upload_mb": 1000,
    "streaming_supported": true,
    "compression_supported": ["gzip", "zstd"],
    "bandwidth_estimate_mbps": 1000,
    "available_storage_gb": 500,
    "preferred_method": "http-chunked"
  },
  "resource_constraints": {
    "max_concurrent_deployments": 5,
    "current_deployments": 1,
    "cpu_load_percent": 15,
    "memory_available_gb": 180
  }
}
```

### 2. Negotiation Phase

**POST /api/deployment/negotiate**

Request:
```json
{
  "service_name": "Toadstool Compute Engine",
  "binary_size_mb": 85,
  "preferred_methods": ["http-chunked", "http-stream"],
  "compression": "zstd",
  "priority": "high",
  "deployment_strategy": "fast" | "reliable" | "balanced"
}
```

Response:
```json
{
  "negotiation_id": "neg-12345",
  "accepted_method": "http-chunked",
  "chunk_size_mb": 10,
  "total_chunks": 9,
  "upload_endpoints": [
    "/api/deployment/chunk/neg-12345/0",
    "/api/deployment/chunk/neg-12345/1",
    ...
  ],
  "finalize_endpoint": "/api/deployment/finalize/neg-12345",
  "timeout_seconds": 300,
  "compression_used": "zstd"
}
```

### 3. Execution Phase

**Chunked Upload:**
```bash
# Upload each chunk
for i in {0..8}; do
  POST /api/deployment/chunk/neg-12345/$i
  -F "chunk=@binary.part$i"
done

# Finalize
POST /api/deployment/finalize/neg-12345
{
  "auto_start": true,
  "env_vars": {...}
}
```

**Streaming Upload:**
```bash
# Single streaming connection
POST /api/deployment/stream/neg-12345
Transfer-Encoding: chunked
Content-Type: application/octet-stream
```

---

## 🎯 Primal-Specific Profiles

### NestGate (Large Data Storage)
```json
{
  "primal_type": "nestgate",
  "deployment_profile": {
    "typical_size_range_mb": [100, 10000],
    "preferred_method": "http-stream",
    "compression": "zstd",
    "storage_priority": "high",
    "speed_priority": "medium"
  }
}
```

### Toadstool (Fast Compute)
```json
{
  "primal_type": "toadstool",
  "deployment_profile": {
    "typical_size_range_mb": [10, 200],
    "preferred_method": "http-chunked",
    "compression": "gzip",
    "storage_priority": "medium",
    "speed_priority": "critical",
    "startup_latency_target_ms": 100
  }
}
```

### BearDog (Security)
```json
{
  "primal_type": "beardog",
  "deployment_profile": {
    "typical_size_range_mb": [5, 50],
    "preferred_method": "http-single",
    "compression": "none",
    "encryption_required": true,
    "signature_verification": true,
    "security_priority": "critical"
  }
}
```

---

## 🚀 Implementation Phases

### Phase 1: Current (Static) ✅
- [x] HTTP deployment API
- [x] 50MB body limit
- [x] Single-shot upload
- [x] Works for small binaries (<2MB currently due to multipart limit)

### Phase 2: Capability Discovery 🎯
- [ ] Add `/api/deployment/capabilities` endpoint
- [ ] Node advertises its upload limits
- [ ] Deployer queries before sending
- [ ] Graceful fallback to smaller methods

### Phase 3: Chunked Upload 📦
- [ ] Negotiation protocol
- [ ] Chunk-by-chunk upload
- [ ] Assembly on target node
- [ ] Checksum verification

### Phase 4: Streaming Upload 🌊
- [ ] HTTP streaming support
- [ ] Unlimited size support
- [ ] Progress tracking
- [ ] Resume capability

### Phase 5: Adaptive Selection 🧠
- [ ] Auto-select best method based on:
  - Binary size
  - Network conditions
  - Node capabilities
  - Primal profile
- [ ] Real-time bandwidth testing
- [ ] Fallback on failure

### Phase 6: Live Negotiation 🤝
- [ ] Renegotiate mid-transfer if conditions change
- [ ] Adapt chunk size based on performance
- [ ] Switch methods if one fails
- [ ] Multi-path deployment (parallel uploads)

---

## 💡 Example Scenarios

### Scenario 1: Small Service (< 2MB)
```
Deployer: Query capabilities
Target: "Single upload OK, 50MB limit"
Deployer: POST /api/deployment/binary (single shot)
Target: ✅ Deployed
```

### Scenario 2: Medium Service (10MB)
```
Deployer: Query capabilities
Target: "Chunked preferred, 10MB chunks"
Deployer: Negotiate
Target: "Send 1 chunk"
Deployer: POST chunk
Target: ✅ Deployed
```

### Scenario 3: Large Service (500MB NestGate)
```
Deployer: Query capabilities
Target: "Streaming supported, zstd compression"
Deployer: Negotiate streaming
Target: "Ready for stream"
Deployer: Stream compressed binary
Target: Decompress, verify, ✅ Deployed
```

### Scenario 4: Internet Deployment (with BearDog)
```
Deployer: Query capabilities through BearDog
Target: "Chunked only, encrypted, 5MB chunks"
Deployer: Negotiate encrypted chunked
BearDog: Establish TLS tunnel
Deployer: Send encrypted chunks through BearDog
Target: Decrypt, verify signatures, ✅ Deployed
```

---

## 🎛️ Configuration (Proposed)

### Songbird Configuration
```toml
[deployment]
# Static limits (fallback)
max_body_size_mb = 50
max_multipart_field_mb = 10

# Dynamic capabilities
enable_chunked_upload = true
enable_streaming_upload = true
enable_compression = true
supported_compression = ["gzip", "zstd", "lz4"]

# Adaptive settings
auto_negotiate = true
bandwidth_test_enabled = true
adaptive_chunk_size = true

# Primal profiles
[deployment.primal_profiles.nestgate]
preferred_method = "stream"
compression = "zstd"

[deployment.primal_profiles.toadstool]
preferred_method = "chunked"
chunk_size_mb = 10

[deployment.primal_profiles.beardog]
preferred_method = "single"
encryption_required = true
```

---

## 🔄 Backward Compatibility

All new capabilities are **optional enhancements**:
- Nodes without negotiation support fall back to single upload
- Existing deployments continue to work
- New features are discovered dynamically
- No breaking changes to existing API

---

## 📈 Benefits

### For Operators
- ✅ No manual configuration of limits
- ✅ Automatic adaptation to network conditions
- ✅ Optimal performance without tuning

### For Primals
- ✅ Each primal gets optimal deployment method
- ✅ NestGate can send huge files
- ✅ Toadstool gets fast deployment
- ✅ BearDog enforces security

### For Songbird
- ✅ True agnostic architecture
- ✅ Scales from tiny to massive binaries
- ✅ Works on LAN and internet
- ✅ Self-optimizing system

---

## 🎯 Next Steps

1. **Validate Current System** (< 2MB binaries)
   - Test HTTP deployment with mini compute bridge
   - Verify federation integration
   - Confirm capability-based routing

2. **Design Negotiation Protocol**
   - Define capability discovery API
   - Spec out chunked upload protocol
   - Document streaming approach

3. **Implement Phase 2** (Capability Discovery)
   - Add capabilities endpoint
   - Query before deployment
   - Graceful fallback

4. **Implement Phase 3** (Chunked Upload)
   - Chunk assembly logic
   - Progress tracking
   - Verification

5. **Implement Phase 4+** (Streaming, Adaptive)
   - Full streaming support
   - Live adaptation
   - Multi-path deployment

---

## 🎵 The Songbird Way

> "Don't hardcode limits. Negotiate them. Don't fix methods. Adapt them. 
> Let the nodes tell you what they can do, and orchestrate accordingly."

**Zero assumptions. Maximum flexibility. Pure adaptation.**

---

**Status:** Design Phase  
**Target:** Songbird v0.2.0  
**Priority:** High (enables true cross-primal deployment at any scale)

