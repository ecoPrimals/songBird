# Adaptive Deployment Specification

**Version:** 1.0  
**Status:** Draft  
**Date:** November 8, 2025  

---

## 1. Overview

### 1.1 Purpose

This specification defines Songbird's adaptive deployment system, which enables intelligent, negotiated service deployment across heterogeneous nodes without user configuration.

### 1.2 Goals

- **Zero Configuration**: Automatic capability detection and limit selection
- **Environment-Aware**: Adapt to LAN, WAN, or internet conditions
- **Primal-Agnostic**: Support any service type with optimal method
- **Backward Compatible**: Graceful fallback to simple methods
- **Live Adaptive**: Adjust strategies in real-time

### 1.3 Non-Goals

- Static configuration files
- Manual limit tuning
- One-size-fits-all approach
- Breaking existing deployment API

---

## 2. System Architecture

### 2.1 Components

```
┌─────────────────────────────────────────────────────────────┐
│  Deployer Node (Tower A)                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Deployment Client                                   │   │
│  │  • Capability query                                  │   │
│  │  • Method selection                                  │   │
│  │  • Upload execution                                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ 1. Query capabilities
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  Target Node (Tower B)                                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Capability Discovery Module                         │   │
│  │  • Detect bandwidth                                  │   │
│  │  • Measure CPU/memory                                │   │
│  │  • Check storage                                     │   │
│  │  • Assess network type                               │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Deployment Handler                                  │   │
│  │  • Single upload                                     │   │
│  │  • Chunked upload                                    │   │
│  │  • Streaming upload                                  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
1. Discovery Phase
   Deployer → Target: GET /api/deployment/capabilities
   Target → Deployer: Capabilities JSON

2. Selection Phase
   Deployer: Analyze binary size, network, capabilities
   Deployer: Select optimal method

3. Negotiation Phase (optional for chunked/stream)
   Deployer → Target: POST /api/deployment/negotiate
   Target → Deployer: Negotiation response

4. Execution Phase
   Deployer → Target: Upload via selected method
   Target: Receive, assemble, deploy

5. Verification Phase
   Deployer → Target: GET /api/deployment/status/:id
   Target → Deployer: Deployment status
```

---

## 3. Capability Discovery

### 3.1 Auto-Detection Logic

#### 3.1.1 Network Type Detection

```rust
async fn detect_network_type() -> NetworkType {
    // Check if peer is on same subnet
    if is_same_subnet(peer_ip, local_ip) {
        return NetworkType::LAN;
    }
    
    // Check if peer is in private IP range
    if is_private_ip(peer_ip) {
        return NetworkType::VPN;
    }
    
    // Otherwise assume internet
    NetworkType::Internet
}
```

#### 3.1.2 Bandwidth Estimation

```rust
async fn estimate_bandwidth() -> BandwidthEstimate {
    // LAN: Assume gigabit
    if network_type == NetworkType::LAN {
        return BandwidthEstimate {
            download_mbps: 1000,
            upload_mbps: 1000,
            latency_ms: 1,
            confidence: Confidence::High,
        };
    }
    
    // Internet: Run quick test
    let test_result = run_bandwidth_test().await;
    BandwidthEstimate {
        download_mbps: test_result.download,
        upload_mbps: test_result.upload,
        latency_ms: test_result.latency,
        confidence: Confidence::Medium,
    }
}
```

#### 3.1.3 Resource Availability

```rust
async fn detect_resources() -> ResourceConstraints {
    ResourceConstraints {
        available_memory_gb: sysinfo::available_memory() / GB,
        available_storage_gb: sysinfo::available_disk() / GB,
        cpu_cores: num_cpus::get(),
        cpu_load_percent: sysinfo::cpu_load(),
        max_concurrent_deployments: calculate_max_concurrent(),
    }
}

fn calculate_max_concurrent() -> usize {
    // Base on available memory and typical deployment size
    let available_gb = sysinfo::available_memory() / GB;
    let estimated_deployment_size_gb = 1; // Conservative estimate
    (available_gb / estimated_deployment_size_gb).max(1).min(10)
}
```

### 3.2 Capability Advertisement

#### 3.2.1 API Endpoint

**GET /api/deployment/capabilities**

Response:
```json
{
  "node_id": "tower-b-strandgate",
  "timestamp": "2025-11-08T22:00:00Z",
  
  "network": {
    "type": "lan" | "vpn" | "internet",
    "bandwidth_estimate": {
      "download_mbps": 1000,
      "upload_mbps": 1000,
      "latency_ms": 1,
      "confidence": "high" | "medium" | "low"
    }
  },
  
  "deployment_methods": {
    "single": {
      "enabled": true,
      "max_size_mb": 50,
      "compression_supported": ["gzip", "zstd"],
      "recommended_for": "< 10MB"
    },
    "chunked": {
      "enabled": true,
      "max_total_size_mb": 1000,
      "chunk_size_mb": 10,
      "max_chunks": 100,
      "compression_supported": ["gzip", "zstd"],
      "recommended_for": "10MB - 500MB"
    },
    "streaming": {
      "enabled": true,
      "unlimited": true,
      "compression_supported": ["gzip", "zstd"],
      "recommended_for": "> 500MB"
    }
  },
  
  "resources": {
    "available_storage_gb": 450,
    "available_memory_gb": 180,
    "cpu_cores": 104,
    "cpu_load_percent": 15,
    "max_concurrent_deployments": 5,
    "current_deployments": 1
  },
  
  "preferences": {
    "preferred_compression": "zstd",
    "preferred_method": "chunked",
    "encryption_required": false
  }
}
```

### 3.3 Method Selection Algorithm

```rust
fn select_deployment_method(
    binary_size_mb: f64,
    capabilities: &Capabilities,
    primal_profile: Option<&PrimalProfile>,
) -> DeploymentMethod {
    // Apply primal-specific preferences
    if let Some(profile) = primal_profile {
        if profile.preferred_method.is_available(&capabilities) {
            return profile.preferred_method;
        }
    }
    
    // Auto-select based on size and capabilities
    match (binary_size_mb, capabilities.network.type) {
        // Small binaries: always single
        (size, _) if size < 2.0 => DeploymentMethod::Single,
        
        // Medium binaries on LAN: chunked
        (size, NetworkType::LAN) if size < 100.0 => DeploymentMethod::Chunked {
            chunk_size_mb: 10.0,
        },
        
        // Medium binaries on internet: chunked with smaller chunks
        (size, NetworkType::Internet) if size < 100.0 => DeploymentMethod::Chunked {
            chunk_size_mb: 5.0,
        },
        
        // Large binaries: streaming
        (size, _) if size >= 100.0 => DeploymentMethod::Streaming {
            compression: Some(Compression::Zstd),
        },
        
        // Fallback
        _ => DeploymentMethod::Single,
    }
}
```

---

## 4. Deployment Methods

### 4.1 Single Upload (Existing)

**Status**: Implemented ✅  
**Use Case**: Binaries < 2MB  
**Endpoint**: `POST /api/deployment/binary`

No changes required. Current implementation works.

### 4.2 Chunked Upload

**Status**: To be implemented  
**Use Case**: Binaries 2MB - 500MB  

#### 4.2.1 Negotiation

**POST /api/deployment/negotiate**

Request:
```json
{
  "binary_size_mb": 85,
  "binary_hash": "sha256:abc123...",
  "service_name": "Toadstool Compute",
  "preferred_method": "chunked",
  "compression": "zstd"
}
```

Response:
```json
{
  "negotiation_id": "neg-12345",
  "accepted_method": "chunked",
  "chunk_size_mb": 10,
  "total_chunks": 9,
  "chunk_upload_path": "/api/deployment/chunk/neg-12345/{index}",
  "finalize_path": "/api/deployment/finalize/neg-12345",
  "timeout_seconds": 300
}
```

#### 4.2.2 Chunk Upload

**POST /api/deployment/chunk/:negotiation_id/:chunk_index**

Request (multipart):
- `chunk`: Binary data (compressed if negotiated)
- `chunk_hash`: SHA-256 of chunk
- `chunk_index`: Index in sequence

Response:
```json
{
  "chunk_index": 0,
  "received": true,
  "verified": true
}
```

#### 4.2.3 Finalization

**POST /api/deployment/finalize/:negotiation_id**

Request:
```json
{
  "service_name": "Toadstool Compute",
  "env_vars": {...},
  "auto_start": true
}
```

Response: Same as single upload

### 4.3 Streaming Upload

**Status**: Future (Phase 4)  
**Use Case**: Binaries > 500MB  

**POST /api/deployment/stream/:negotiation_id**

Uses HTTP chunked transfer encoding for continuous upload.

---

## 5. Primal Profiles

### 5.1 Profile Definition

```rust
pub struct PrimalProfile {
    pub primal_type: String,
    pub typical_size_range_mb: (f64, f64),
    pub preferred_method: DeploymentMethod,
    pub preferred_compression: Option<Compression>,
    pub priority: Priority,
}
```

### 5.2 Built-in Profiles

#### 5.2.1 Toadstool (Compute)

```rust
PrimalProfile {
    primal_type: "toadstool",
    typical_size_range_mb: (10.0, 200.0),
    preferred_method: DeploymentMethod::Chunked { chunk_size_mb: 10.0 },
    preferred_compression: Some(Compression::Gzip),
    priority: Priority::Speed,
}
```

#### 5.2.2 NestGate (Storage)

```rust
PrimalProfile {
    primal_type: "nestgate",
    typical_size_range_mb: (100.0, 10000.0),
    preferred_method: DeploymentMethod::Streaming,
    preferred_compression: Some(Compression::Zstd),
    priority: Priority::Reliability,
}
```

#### 5.2.3 BearDog (Security)

```rust
PrimalProfile {
    primal_type: "beardog",
    typical_size_range_mb: (5.0, 50.0),
    preferred_method: DeploymentMethod::Single,
    preferred_compression: None, // Don't compress encrypted data
    priority: Priority::Security,
}
```

### 5.3 Profile Discovery

Profiles can be:
1. **Built-in**: Hardcoded in Songbird for known primals
2. **Advertised**: Primals can advertise their preferences via capability endpoint
3. **Learned**: Songbird learns optimal methods over time (future)

---

## 6. Error Handling & Fallback

### 6.1 Fallback Strategy

```
1st Attempt: Optimal method (e.g., chunked)
    ↓ Failed
2nd Attempt: Fallback method (e.g., single)
    ↓ Failed
3rd Attempt: Manual intervention (log error, notify operator)
```

### 6.2 Timeout Handling

```rust
const UPLOAD_TIMEOUT_PER_MB: Duration = Duration::from_secs(5);

fn calculate_timeout(size_mb: f64) -> Duration {
    UPLOAD_TIMEOUT_PER_MB * (size_mb.ceil() as u32)
}
```

### 6.3 Retry Logic

```rust
async fn deploy_with_retry(
    target: &NodeAddress,
    binary: &Binary,
    max_attempts: u32,
) -> Result<DeploymentId> {
    let mut attempts = 0;
    let mut last_error = None;
    
    while attempts < max_attempts {
        match try_deploy(target, binary).await {
            Ok(deployment_id) => return Ok(deployment_id),
            Err(e) => {
                warn!("Deployment attempt {} failed: {}", attempts + 1, e);
                last_error = Some(e);
                attempts += 1;
                
                // Exponential backoff
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

---

## 7. Performance Optimization

### 7.1 Compression

**Auto-Selection Logic:**
```rust
fn select_compression(
    binary_type: &str,
    network_type: NetworkType,
) -> Option<Compression> {
    // Don't compress already-compressed formats
    if ["jpg", "png", "mp4", "zip"].contains(&binary_type) {
        return None;
    }
    
    // LAN: Prioritize speed (gzip)
    if network_type == NetworkType::LAN {
        return Some(Compression::Gzip);
    }
    
    // Internet: Prioritize compression ratio (zstd)
    Some(Compression::Zstd)
}
```

### 7.2 Parallel Chunk Upload

For chunked uploads, upload chunks in parallel (up to 3 concurrent):

```rust
async fn upload_chunks_parallel(
    chunks: Vec<Chunk>,
    target: &NodeAddress,
) -> Result<()> {
    let mut tasks = Vec::new();
    let semaphore = Arc::new(Semaphore::new(3)); // Max 3 concurrent
    
    for chunk in chunks {
        let sem = semaphore.clone();
        let target = target.clone();
        
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await;
            upload_chunk(&target, chunk).await
        }));
    }
    
    futures::future::try_join_all(tasks).await?;
    Ok(())
}
```

---

## 8. Security Considerations

### 8.1 Validation

- **Binary hash verification**: SHA-256 checksum
- **Size limits**: Enforce advertised limits
- **Rate limiting**: Max deployments per minute
- **Storage quotas**: Prevent disk exhaustion

### 8.2 Authentication

For internet deployments with BearDog:
- Bearer token authentication
- mTLS certificate verification
- Signature validation

---

## 9. Implementation Phases

### Phase 1: Foundation ✅
- [x] HTTP deployment API
- [x] Single upload method
- [x] Basic error handling

### Phase 2: Capability Discovery 🎯
- [ ] Add `/api/deployment/capabilities` endpoint
- [ ] Implement auto-detection (network, resources)
- [ ] Method selection algorithm
- [ ] Update client to query capabilities

### Phase 3: Chunked Upload
- [ ] Negotiation protocol
- [ ] Chunk upload endpoints
- [ ] Assembly logic
- [ ] Parallel upload

### Phase 4: Streaming Upload
- [ ] Streaming endpoint
- [ ] Progress tracking
- [ ] Resume capability

### Phase 5: Optimization
- [ ] Compression auto-selection
- [ ] Adaptive chunk sizing
- [ ] Bandwidth testing

### Phase 6: Intelligence
- [ ] Learn optimal methods over time
- [ ] Predict deployment time
- [ ] Proactive optimization

---

## 10. Configuration

**Zero required** - all auto-detected.

Optional overrides:
```toml
[deployment]
# Override auto-detection
force_method = "chunked" # optional
max_body_size_mb = 50     # optional
enable_compression = true # default

# Manual bandwidth override (for testing)
# bandwidth_override_mbps = 100
```

---

## 11. Monitoring & Metrics

### 11.1 Metrics to Track

- Deployment success rate by method
- Average upload time by size
- Bandwidth utilization
- Fallback frequency
- Method selection accuracy

### 11.2 Logging

```rust
info!("Deploying {} ({}MB) to {} via {} method",
    service_name, size_mb, target_node, method);

info!("Deployment complete: {} in {:.2}s ({:.2} MB/s)",
    deployment_id, duration.as_secs_f64(), throughput);
```

---

## 12. Testing Strategy

### 12.1 Unit Tests

- Capability detection
- Method selection algorithm
- Chunk assembly

### 12.2 Integration Tests

- Single upload (< 2MB)
- Chunked upload (10MB)
- Large binary (100MB+)
- Network failure scenarios
- Fallback logic

### 12.3 Performance Tests

- LAN throughput
- Internet throughput
- Compression overhead
- Parallel upload speedup

---

## 13. Future Enhancements

### 13.1 Phase 7+

- **Predictive deployment**: Pre-stage binaries before needed
- **P2P deployment**: Deploy from peer to peer
- **Delta updates**: Only upload changed parts
- **Multi-source**: Download from multiple nodes simultaneously
- **Smart caching**: Cache frequently deployed binaries

---

## 14. References

- HTTP/1.1 Chunked Transfer Encoding (RFC 7230)
- Multipart Form Data (RFC 7578)
- Content Negotiation (RFC 7231)
- SHA-256 Hashing (FIPS 180-4)

---

**Status**: Specification Complete  
**Next**: Implementation (Phase 2)  
**Owner**: Songbird Core Team

