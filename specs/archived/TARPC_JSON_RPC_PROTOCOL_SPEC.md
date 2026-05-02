# tarpc + JSON-RPC Protocol Implementation Spec

**Version**: 1.0  
**Date**: November 11, 2025  
**Status**: IMPLEMENTATION SPECIFICATION  
**Priority**: P1 - High Priority Enhancement  
**Companion**: See also `SONGBIRD_NATIVE_RPC_SPECIFICATION.md` for the broader
transport architecture (bidirectional streams, fallback chains, security tunnels).
This document focuses on the concrete dual-server implementation (tarpc binary +
JSON-RPC 2.0 via jsonrpsee), env vars, Cargo deps, and phased rollout.

---

## 📊 EXECUTIVE SUMMARY

**Decision**: Songbird uses **custom Rust-native RPC** (tarpc + JSON-RPC) instead of gRPC.

**Rationale**:
- ✅ Pure Rust (no C++ dependencies like gRPC)
- ✅ No vendor lock-in (no Google protobuf tooling)
- ✅ No non-Rust hardcoded dependencies
- ✅ Full control over protocol evolution
- ✅ Optimized for primal sovereignty architecture
- ✅ Native Rust serialization (serde)

**Current State**:
- ✅ HTTP/REST (dual-stack IPv4 + IPv6)
- ⏳ tarpc infrastructure (partially implemented)
- ⏳ JSON-RPC 2.0 (planned)

**Target State**:
- ✅ HTTP/REST (primary, human-friendly)
- ✅ tarpc (high-performance, primal-to-primal)
- ✅ JSON-RPC 2.0 (universal, language-agnostic)
- ✅ WebSocket (real-time, bidirectional)

---

## 🎯 DESIGN PRINCIPLES

### **1. Pure Rust, No Vendor Lock-in**

**Why Not gRPC**:
```
gRPC Problems:
❌ Requires protoc (C++ compiler)
❌ Requires protobuf (Google tooling)
❌ Non-Rust code generation
❌ Vendor lock-in (Google ecosystem)
❌ Complex build process
❌ Language barrier for contributors
```

**Why tarpc + JSON-RPC**:
```
Our Solution:
✅ Pure Rust (no C/C++ dependencies)
✅ Native serde serialization
✅ Rust macros (procedural generation)
✅ No external tooling required
✅ Full protocol control
✅ Community-driven development
```

---

### **2. Dual Protocol Strategy**

**tarpc**: High-performance, binary RPC for primal-to-primal communication
**JSON-RPC 2.0**: Universal, language-agnostic RPC for external clients

```
┌─────────────────────────────────────────┐
│         Songbird Service Mesh            │
├─────────────────────────────────────────┤
│                                          │
│  tarpc (Binary)    JSON-RPC (Universal) │
│  ↓                 ↓                     │
│  Primal ←→ Primal  External Clients     │
│  • Security Provider         • Python              │
│  • AI provider        • JavaScript          │
│  • Compute provider       • Any language        │
│  • Storage Provider        • curl/httpie         │
│                                          │
│  Both protocols access same service mesh │
└─────────────────────────────────────────┘
```

---

## 🏗️ ARCHITECTURE

### **Protocol Stack**

```
┌──────────────────────────────────────────────┐
│           Application Layer                   │
│  Service Registry | Discovery | Federation   │
└────────────┬──────────────────┬──────────────┘
             │                  │
    ┌────────▼────────┐  ┌─────▼──────────┐
    │  tarpc Server   │  │ JSON-RPC Server│
    │  (Binary RPC)   │  │ (HTTP/JSON)    │
    └────────┬────────┘  └─────┬──────────┘
             │                  │
    ┌────────▼──────────────────▼──────────┐
    │       Transport Layer (TCP/HTTP)      │
    │     IPv6 Dual-Stack [::]:8080         │
    └───────────────────────────────────────┘
```

---

## 🔧 TARPC IMPLEMENTATION

### **Why tarpc**

**Performance**:
- Binary serialization (bincode or JSON)
- Zero-copy where possible
- Async/await native
- 10-100x faster than HTTP/REST

**Developer Experience**:
- Rust macros (no code generation)
- Type-safe RPC definitions
- Automatic client generation
- Full Rust tooling support

**Example**:

```rust
// Define service interface
#[tarpc::service]
pub trait SongbirdFederation {
    /// Register a service with the mesh
    async fn register_service(service: ServiceInfo) -> Result<String, ServiceError>;
    
    /// Discover services by capability
    async fn discover_services(query: DiscoveryQuery) -> Result<Vec<ServiceInfo>, ServiceError>;
    
    /// Get federation status
    async fn get_federation_status() -> Result<FederationStatus, ServiceError>;
    
    /// Stream service updates (real-time)
    async fn stream_service_updates(
        capabilities: Vec<String>,
    ) -> Result<tokio::sync::mpsc::Receiver<ServiceUpdate>, ServiceError>;
}
```

### **Implementation Structure**

```rust
// File: crates/songbird-orchestrator/src/server/tarpc/mod.rs

use tarpc::{server, context::Context};
use std::sync::Arc;

/// tarpc server implementation
pub struct TarpcServer {
    core: Arc<ServiceMeshCore>,
}

#[tarpc::server]
impl SongbirdFederation for TarpcServer {
    async fn register_service(
        self,
        _ctx: Context,
        service: ServiceInfo,
    ) -> Result<String, ServiceError> {
        // Delegate to service mesh core
        self.core.register_service(service).await
            .map_err(|e| ServiceError::RegistrationFailed(e.to_string()))
    }
    
    async fn discover_services(
        self,
        _ctx: Context,
        query: DiscoveryQuery,
    ) -> Result<Vec<ServiceInfo>, ServiceError> {
        self.core.discover_services(query).await
            .map_err(|e| ServiceError::DiscoveryFailed(e.to_string()))
    }
    
    async fn get_federation_status(
        self,
        _ctx: Context,
    ) -> Result<FederationStatus, ServiceError> {
        self.core.get_federation_status().await
            .map_err(|e| ServiceError::StatusFailed(e.to_string()))
    }
    
    async fn stream_service_updates(
        self,
        _ctx: Context,
        capabilities: Vec<String>,
    ) -> Result<tokio::sync::mpsc::Receiver<ServiceUpdate>, ServiceError> {
        self.core.subscribe_service_updates(capabilities).await
            .map_err(|e| ServiceError::StreamFailed(e.to_string()))
    }
}
```

### **Server Startup**

```rust
// File: crates/songbird-orchestrator/src/app/mod.rs

async fn start_tarpc_server(&self) -> Result<()> {
    let bind_address = SafeEnv::get_or_default("SONGBIRD_TARPC_BIND", "[::]");
    let port = SafeEnv::get_port("SONGBIRD_TARPC_PORT", 
        songbird_config::defaults::ports::tarpc_port()); // e.g., 8081
    
    let addr = parse_bind_address(&bind_address, port)?;
    
    // Create tarpc server
    let server = TarpcServer {
        core: Arc::clone(&self.service_mesh_core),
    };
    
    // Bind and serve
    let listener = tarpc::serde_transport::tcp::listen(&addr, Json::default).await?;
    
    info!("🚀 tarpc server listening on {}", addr);
    
    listener
        .filter_map(|r| async { r.ok() })
        .map(server::BaseChannel::with_defaults)
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        .map(|channel| {
            let server = server.clone();
            channel.execute(server.serve())
        })
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
    
    Ok(())
}
```

### **Client Usage**

```rust
// File: crates/songbird-primal-sdk/src/tarpc_client.rs

use tarpc::{client, context};

/// tarpc client for high-performance RPC
pub struct SongbirdTarpcClient {
    client: SongbirdFederationClient,
}

impl SongbirdTarpcClient {
    pub async fn connect(addr: &str) -> Result<Self> {
        let transport = tarpc::serde_transport::tcp::connect(addr, Json::default).await?;
        let client = SongbirdFederationClient::new(
            client::Config::default(),
            transport,
        ).spawn();
        
        Ok(Self { client })
    }
    
    pub async fn register_service(&self, service: ServiceInfo) -> Result<String> {
        self.client.register_service(context::current(), service).await?
    }
    
    pub async fn discover_services(&self, query: DiscoveryQuery) -> Result<Vec<ServiceInfo>> {
        self.client.discover_services(context::current(), query).await?
    }
}

// Usage example
let client = SongbirdTarpcClient::connect("localhost:8081").await?;
let service_id = client.register_service(ServiceInfo {
    name: "storage_provider".to_string(),
    address: "localhost".to_string(),
    port: 8090,
    capabilities: vec!["gateway".to_string()],
}).await?;
```

---

## 📋 JSON-RPC 2.0 IMPLEMENTATION

### **Why JSON-RPC 2.0**

**Universal**:
- Language-agnostic (works with any HTTP client)
- Simple specification (no complex tooling)
- Human-readable (JSON)
- Easy to debug (curl/httpie)

**Standard**:
- Industry standard (JSON-RPC 2.0 spec)
- Well-understood protocol
- Extensive tooling support
- Works over HTTP/WebSocket

### **JSON-RPC Endpoint**

```rust
// File: crates/songbird-orchestrator/src/server/jsonrpc/mod.rs

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    server::{Server, ServerHandle},
};

/// JSON-RPC 2.0 API for Songbird
#[rpc(server)]
pub trait SongbirdRpc {
    /// Register a service
    #[method(name = "songbird.registerService")]
    async fn register_service(&self, service: ServiceInfo) -> RpcResult<String>;
    
    /// Discover services
    #[method(name = "songbird.discoverServices")]
    async fn discover_services(&self, query: DiscoveryQuery) -> RpcResult<Vec<ServiceInfo>>;
    
    /// Get federation status
    #[method(name = "songbird.getFederationStatus")]
    async fn get_federation_status(&self) -> RpcResult<FederationStatus>;
    
    /// Subscribe to service updates
    #[subscription(
        name = "songbird.subscribeServiceUpdates" => "songbird.serviceUpdate",
        unsubscribe = "songbird.unsubscribeServiceUpdates",
        item = ServiceUpdate
    )]
    async fn subscribe_service_updates(
        &self,
        capabilities: Vec<String>,
    ) -> RpcResult<()>;
}

/// Implementation
pub struct SongbirdRpcServer {
    core: Arc<ServiceMeshCore>,
}

#[async_trait]
impl SongbirdRpcServer for SongbirdRpcServerImpl {
    async fn register_service(&self, service: ServiceInfo) -> RpcResult<String> {
        self.core.register_service(service).await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }
    
    async fn discover_services(&self, query: DiscoveryQuery) -> RpcResult<Vec<ServiceInfo>> {
        self.core.discover_services(query).await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }
    
    async fn get_federation_status(&self) -> RpcResult<FederationStatus> {
        self.core.get_federation_status().await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))
    }
    
    async fn subscribe_service_updates(
        &self,
        pending: PendingSubscriptionSink,
        capabilities: Vec<String>,
    ) -> RpcResult<()> {
        let sink = pending.accept().await?;
        let mut updates = self.core.subscribe_service_updates(capabilities).await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;
        
        tokio::spawn(async move {
            while let Some(update) = updates.recv().await {
                if sink.send(&update).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(())
    }
}
```

### **JSON-RPC Client Usage**

```bash
# Register service via JSON-RPC
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.registerService",
    "params": {
      "name": "storage_provider",
      "address": "localhost",
      "port": 8090,
      "capabilities": ["gateway"]
    },
    "id": 1
  }'

# Response
{
  "jsonrpc": "2.0",
  "result": "service-id-12345",
  "id": 1
}

# Discover services
curl -X POST http://localhost:8080/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "songbird.discoverServices",
    "params": {
      "capabilities": ["gateway"]
    },
    "id": 2
  }'

# Response
{
  "jsonrpc": "2.0",
  "result": [
    {
      "name": "storage_provider",
      "address": "localhost",
      "port": 8090,
      "capabilities": ["gateway"]
    }
  ],
  "id": 2
}
```

### **Python Client Example**

```python
# Python JSON-RPC client
import requests

class SongbirdClient:
    def __init__(self, url="http://localhost:8080/jsonrpc"):
        self.url = url
        self.id = 0
    
    def _call(self, method, params):
        self.id += 1
        response = requests.post(self.url, json={
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": self.id
        })
        return response.json()["result"]
    
    def register_service(self, name, address, port, capabilities):
        return self._call("songbird.registerService", {
            "name": name,
            "address": address,
            "port": port,
            "capabilities": capabilities
        })
    
    def discover_services(self, capabilities):
        return self._call("songbird.discoverServices", {
            "capabilities": capabilities
        })

# Usage
client = SongbirdClient()
service_id = client.register_service(
    name="compute_provider",
    address="localhost",
    port=8093,
    capabilities=["ml", "training"]
)
print(f"Registered: {service_id}")

services = client.discover_services(["ml"])
print(f"Found services: {services}")
```

---

## 📊 PERFORMANCE COMPARISON

| Protocol | Latency | Throughput | Use Case |
|----------|---------|------------|----------|
| **HTTP/REST** | ~10ms | 10K req/s | Human APIs, debugging |
| **tarpc (binary)** | <1ms | 100K req/s | Primal-to-primal |
| **JSON-RPC** | ~5ms | 20K req/s | External clients |
| **WebSocket** | <5ms | 50K msg/s | Real-time updates |

---

## 🔧 IMPLEMENTATION ROADMAP

### **Phase 1: Foundation** (Complete)
- ✅ IPv6 dual-stack binding
- ✅ HTTP/REST API
- ✅ Service registry core

### **Phase 2: tarpc Integration** (2 weeks)
1. Add tarpc dependency
2. Define service traits
3. Implement tarpc server
4. Create client library
5. Add to multi-protocol server
6. Performance benchmarks

### **Phase 3: JSON-RPC Support** (1 week)
1. Add jsonrpsee dependency
2. Define RPC methods
3. Implement JSON-RPC server
4. Create client examples (Python, JS)
5. Documentation
6. Integration tests

### **Phase 4: WebSocket Real-time** (1 week)
1. WebSocket endpoint
2. Subscription system
3. Real-time updates
4. Client examples
5. Performance tuning

---

## 📋 DEPENDENCIES

```toml
# Cargo.toml additions

[dependencies]
# tarpc for high-performance RPC
tarpc = { version = "0.34", features = ["full"] }
bincode = "1.3"  # Binary serialization

# JSON-RPC 2.0
jsonrpsee = { version = "0.21", features = ["server", "client", "ws-client"] }

# Existing (already have)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

---

## ✅ SUCCESS CRITERIA

### **tarpc Implementation Complete When**:
- [ ] tarpc server running on port 8081
- [ ] Service registration works via tarpc
- [ ] Discovery works via tarpc
- [ ] Client library published
- [ ] 10x performance improvement demonstrated
- [ ] Primal-to-primal communication working

### **JSON-RPC Implementation Complete When**:
- [ ] JSON-RPC endpoint at `/jsonrpc`
- [ ] All service mesh operations exposed
- [ ] Python client library working
- [ ] JavaScript client library working
- [ ] curl examples documented
- [ ] Integration tests passing

### **Full Implementation Complete When**:
- [ ] All three protocols working (HTTP/REST, tarpc, JSON-RPC)
- [ ] Same service mesh semantics across all
- [ ] Client choice enabled
- [ ] Performance benchmarks published
- [ ] Documentation complete
- [ ] Storage Provider using optimal protocol

---

## 🔐 SECURITY CONSIDERATIONS

### **Authentication**
- HTTP/REST: Bearer tokens
- tarpc: Client certificates (TLS)
- JSON-RPC: API keys in request

### **Authorization**
- Unified RBAC across all protocols
- Capability-based access control
- Service-level permissions

### **Encryption**
- HTTP/REST: Optional TLS 1.3
- tarpc: Optional TLS 1.3
- JSON-RPC: TLS when over HTTPS

---

## 📚 REFERENCES

**Standards**:
- JSON-RPC 2.0 Specification: https://www.jsonrpc.org/specification
- tarpc Documentation: https://docs.rs/tarpc/

**Related Specs**:
- `SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md` (IPv6 foundation)
- `UNIVERSAL_PROTOCOL_FRAMEWORK_SPECIFICATION.md` (overall vision)
- `SONGBIRD_NATIVE_RPC_SPECIFICATION.md` (existing, may need update)

---

## 🎯 DESIGN DECISIONS

### **Why Not Protocol Buffers (protobuf)**
- ❌ Requires `protoc` compiler (C++)
- ❌ Code generation complexity
- ❌ Vendor lock-in (Google)
- ✅ Serde is native Rust, more flexible

### **Why tarpc Over Cap'n Proto**
- ✅ Better Rust integration
- ✅ Simpler API
- ✅ Active maintenance
- ✅ async/await native

### **Why JSON-RPC Over GraphQL**
- ✅ Simpler specification
- ✅ Lower overhead
- ✅ Better for RPC semantics
- ✅ Easier client implementation

---

**Status**: 📋 **SPECIFICATION READY** - Implementation can begin  
**Priority**: P1 - High Priority  
**Owner**: Songbird Protocol Team

**This specification provides a clear path for implementing high-performance, pure-Rust RPC without gRPC dependencies or vendor lock-in, while maintaining universal access via JSON-RPC 2.0.**

