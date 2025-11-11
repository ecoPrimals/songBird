# 🌐 Universal Protocol Framework Specification
## Protocol-Agnostic Service Mesh Architecture

**Version**: 1.0  
**Date**: November 10, 2025  
**Status**: 📋 DESIGN SPECIFICATION  
**Priority**: P1 - Strategic Enhancement  
**Vision**: "Any protocol, same semantics, transparent routing"

---

## 📊 EXECUTIVE SUMMARY

**Vision**: Songbird should support multiple protocols (HTTP/REST, gRPC, WebSocket, QUIC) interchangeably, allowing clients to use any transport while accessing the same service mesh capabilities.

**Current State**:
```
✅ HTTP/REST (IPv4 only)
❌ HTTP/REST (IPv6) - CRITICAL FIX IN PROGRESS
❌ gRPC
❌ WebSocket (partial)
❌ QUIC/HTTP3
```

**Target State**:
```
✅ HTTP/REST (dual-stack IPv4 + IPv6)
✅ gRPC (binary RPC with streaming)
✅ WebSocket (real-time bidirectional)
✅ QUIC/HTTP3 (modern, encrypted by default)
```

**Benefits**:
- 🚀 Client choice (use what fits best)
- 🔄 Gradual migration (add protocols incrementally)
- ⚡ Performance optimization (gRPC for speed, WebSocket for real-time)
- 🔮 Future-proof (QUIC/HTTP3 readiness)

---

## 🎯 DESIGN PRINCIPLES

### **1. Protocol Neutrality**

**Principle**: Service mesh semantics should be independent of transport protocol.

```rust
// Same operation, any protocol:

// HTTP/REST
POST http://songbird:8080/api/federation/services
Content-Type: application/json

// gRPC
rpc RegisterService(ServiceRegistration) returns (RegistrationResponse)

// WebSocket
ws://songbird:8080/api/ws
{ "op": "register", "service": {...} }

// QUIC/HTTP3
https://songbird:8080/api/federation/services (over QUIC)
```

**All protocols provide**:
- Same service registry
- Same discovery mechanisms
- Same federation capabilities
- Same security model

---

### **2. Progressive Enhancement**

**Principle**: Add protocols without breaking existing clients.

```
Phase 1: HTTP/REST (IPv4) ✅ EXISTS
Phase 2: HTTP/REST (IPv6) 🔧 IN PROGRESS
Phase 3: gRPC           🔧 NEXT
Phase 4: WebSocket      🔧 PLANNED
Phase 5: QUIC/HTTP3     🔧 FUTURE
```

Each phase is additive - old clients continue working.

---

### **3. Unified API Surface**

**Principle**: Same API semantics across all protocols.

```rust
trait ServiceMeshApi {
    // Core operations (protocol-agnostic)
    async fn register_service(&self, service: ServiceInfo) -> Result<String>;
    async fn discover_services(&self, query: Query) -> Result<Vec<ServiceInfo>>;
    async fn health_check(&self) -> Result<HealthStatus>;
    async fn get_capabilities(&self) -> Result<Vec<Capability>>;
}

// Implemented for each protocol:
impl ServiceMeshApi for HttpAdapter { ... }
impl ServiceMeshApi for GrpcAdapter { ... }
impl ServiceMeshApi for WebSocketAdapter { ... }
impl ServiceMeshApi for QuicAdapter { ... }
```

---

## 🏗️ ARCHITECTURE

### **High-Level Design**

```
┌─────────────────────────────────────────────────────────┐
│                    Client Applications                   │
│  (NestGate, BearDog, Squirrel, Toadstool, etc.)        │
└─────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   ┌────▼───┐      ┌─────▼────┐      ┌────▼────┐
   │  HTTP  │      │   gRPC   │      │   WS    │
   │ Client │      │  Client  │      │ Client  │
   └────┬───┘      └─────┬────┘      └────┬────┘
        │                │                 │
        │                │                 │
   ┌────▼─────────────────▼─────────────────▼────┐
   │         Songbird Protocol Adapter Layer      │
   │  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
   │  │   HTTP   │ │   gRPC   │ │   WS     │    │
   │  │ Adapter  │ │ Adapter  │ │ Adapter  │    │
   │  └────┬─────┘ └────┬─────┘ └────┬─────┘    │
   └───────┼────────────┼────────────┼───────────┘
           │            │            │
   ┌───────▼────────────▼────────────▼───────────┐
   │      Songbird Service Mesh Core              │
   │  • Service Registry                          │
   │  • Discovery Engine                          │
   │  • Capability Routing                        │
   │  • Federation Logic                          │
   │  • Health Monitoring                         │
   └──────────────────────────────────────────────┘
```

---

### **Protocol Adapter Architecture**

```rust
/// Core abstraction for protocol adapters
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Protocol name (e.g., "http", "grpc", "ws")
    fn name(&self) -> &'static str;
    
    /// Supported IP versions
    fn supports_ipv6(&self) -> bool;
    fn supports_ipv4(&self) -> bool;
    
    /// Bind to address and start serving
    async fn bind(&mut self, addr: SocketAddr) -> Result<()>;
    
    /// Serve requests (runs until shutdown)
    async fn serve(&mut self, core: Arc<ServiceMeshCore>) -> Result<()>;
    
    /// Graceful shutdown
    async fn shutdown(&mut self) -> Result<()>;
    
    /// Health check
    async fn health(&self) -> ProtocolHealth;
}

/// Protocol health status
pub struct ProtocolHealth {
    pub protocol: String,
    pub status: HealthStatus,
    pub connections: usize,
    pub requests_per_sec: f64,
}
```

---

## 📋 IMPLEMENTATION ROADMAP

### **Phase 1: IPv6 Foundation** (WEEK 1) 🔴 CRITICAL

**Goal**: Enable dual-stack (IPv4 + IPv6) for existing HTTP/REST API.

**Tasks**:
1. ✅ Change binding from `0.0.0.0` to `[::]`
2. ✅ Add `parse_bind_address()` helper
3. ✅ Test with NestGate connection
4. ✅ Verify `localhost` works

**Deliverables**:
- Dual-stack HTTP server
- NestGate integration working
- Discovery flow fixed

**Reference**: `SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md`

---

### **Phase 2: Native RPC Support** (WEEKS 2-3) 🟡 HIGH PRIORITY

**Goal**: Add custom JSON-RPC and tarpc protocol adapters for high-performance RPC.

**NOTE**: Songbird uses custom RPC (JSON-RPC + tarpc), not gRPC. This avoids non-Rust dependencies and vendor lock-in.

**Design**:

```rust
// File: crates/songbird-orchestrator/src/server/grpc/mod.rs

use tonic::{transport::Server, Request, Response, Status};

// Proto definition (federation.proto)
service SongbirdFederation {
    rpc RegisterService(ServiceRegistration) returns (RegistrationResponse);
    rpc DiscoverServices(DiscoveryQuery) returns (ServiceList);
    rpc GetHealth(HealthRequest) returns (HealthResponse);
    rpc StreamServices(DiscoveryQuery) returns (stream ServiceUpdate);
}

// Implementation
pub struct GrpcAdapter {
    server: Option<Server>,
    core: Arc<ServiceMeshCore>,
}

#[tonic::async_trait]
impl SongbirdFederation for GrpcAdapter {
    async fn register_service(
        &self,
        request: Request<ServiceRegistration>,
    ) -> Result<Response<RegistrationResponse>, Status> {
        let service = request.into_inner();
        
        // Delegate to service mesh core
        let id = self.core.register_service(service.into()).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(RegistrationResponse { service_id: id }))
    }
    
    async fn discover_services(
        &self,
        request: Request<DiscoveryQuery>,
    ) -> Result<Response<ServiceList>, Status> {
        let query = request.into_inner();
        
        let services = self.core.discover_services(query.into()).await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(ServiceList {
            services: services.into_iter().map(|s| s.into()).collect(),
        }))
    }
    
    // Streaming support for real-time updates!
    async fn stream_services(
        &self,
        request: Request<DiscoveryQuery>,
    ) -> Result<Response<Self::StreamServicesStream>, Status> {
        let query = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // Subscribe to service updates
        let mut updates = self.core.subscribe_service_updates().await;
        
        tokio::spawn(async move {
            while let Some(update) = updates.recv().await {
                if let Err(_) = tx.send(Ok(update.into())).await {
                    break; // Client disconnected
                }
            }
        });
        
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}
```

**Benefits**:
- ⚡ 7-10x faster than HTTP/REST (binary protocol)
- 🔄 Bidirectional streaming
- 📊 Built-in load balancing
- 🛡️ Strong typing (Protocol Buffers)

**Tasks**:
1. Create `federation.proto` definition
2. Add `tonic` and `prost` dependencies
3. Implement `GrpcAdapter`
4. Add gRPC to main server
5. Create gRPC client examples
6. Update NestGate to support gRPC option

---

### **Phase 3: WebSocket Support** (WEEKS 4-5) 🟡 MEDIUM PRIORITY

**Goal**: Real-time bidirectional communication for live updates.

**Use Cases**:
- Live service discovery updates
- Real-time health monitoring
- Capability change notifications
- Federation event streaming

**Design**:

```rust
// File: crates/songbird-orchestrator/src/server/websocket/mod.rs

use axum::{
    extract::{ws::{WebSocket, Message}, WebSocketUpgrade, State},
    response::Response,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(core): State<Arc<ServiceMeshCore>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, core))
}

async fn handle_socket(socket: WebSocket, core: Arc<ServiceMeshCore>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to service updates
    let mut updates = core.subscribe_service_updates().await;
    
    // Spawn update sender
    let send_task = tokio::spawn(async move {
        while let Some(update) = updates.recv().await {
            let msg = serde_json::to_string(&update).unwrap();
            if sender.send(Message::Text(msg)).await.is_err() {
                break; // Connection closed
            }
        }
    });
    
    // Handle incoming messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                // Handle command
                if let Ok(command) = serde_json::from_str::<WsCommand>(&text) {
                    handle_command(&core, command).await;
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    
    send_task.abort();
}

#[derive(Deserialize)]
enum WsCommand {
    Subscribe { capabilities: Vec<String> },
    Unsubscribe { capabilities: Vec<String> },
    Ping,
}
```

**Benefits**:
- 🔄 Real-time updates (no polling)
- ⚡ Low latency
- 📉 Reduced bandwidth (persistent connection)
- 🎯 Event-driven architecture

---

### **Phase 4: QUIC/HTTP3 Support** (MONTHS 2-3) 🔵 FUTURE

**Goal**: Modern protocol with built-in encryption and improved performance.

**Benefits**:
- 🔐 Encryption by default (TLS 1.3)
- ⚡ Faster connection establishment (0-RTT)
- 🌐 Better mobile/unreliable network support
- 🚀 Multiplexing without head-of-line blocking

**Design**:

```rust
// File: crates/songbird-orchestrator/src/server/quic/mod.rs

use quinn::{Endpoint, ServerConfig};

pub struct QuicAdapter {
    endpoint: Option<Endpoint>,
    core: Arc<ServiceMeshCore>,
}

impl QuicAdapter {
    pub async fn bind(&mut self, addr: SocketAddr, tls_config: TlsConfig) -> Result<()> {
        let server_config = ServerConfig::with_crypto(tls_config.into());
        self.endpoint = Some(Endpoint::server(server_config, addr)?);
        Ok(())
    }
    
    pub async fn serve(&mut self) -> Result<()> {
        let endpoint = self.endpoint.as_ref().unwrap();
        
        while let Some(conn) = endpoint.accept().await {
            let core = Arc::clone(&self.core);
            tokio::spawn(async move {
                if let Ok(connection) = conn.await {
                    handle_quic_connection(connection, core).await;
                }
            });
        }
        
        Ok(())
    }
}
```

---

## 🔧 UNIFIED SERVER IMPLEMENTATION

### **Multi-Protocol Server**

```rust
// File: crates/songbird-orchestrator/src/server/mod.rs

pub struct UniversalProtocolServer {
    core: Arc<ServiceMeshCore>,
    adapters: Vec<Box<dyn ProtocolAdapter>>,
}

impl UniversalProtocolServer {
    pub fn new(core: Arc<ServiceMeshCore>) -> Self {
        Self {
            core,
            adapters: Vec::new(),
        }
    }
    
    pub fn with_http(mut self, addr: SocketAddr) -> Self {
        self.adapters.push(Box::new(HttpAdapter::new(addr)));
        self
    }
    
    pub fn with_grpc(mut self, addr: SocketAddr) -> Self {
        self.adapters.push(Box::new(GrpcAdapter::new(addr)));
        self
    }
    
    pub fn with_websocket(mut self, path: &str) -> Self {
        self.adapters.push(Box::new(WebSocketAdapter::new(path)));
        self
    }
    
    pub fn with_quic(mut self, addr: SocketAddr, tls: TlsConfig) -> Self {
        self.adapters.push(Box::new(QuicAdapter::new(addr, tls)));
        self
    }
    
    pub async fn serve(mut self) -> Result<()> {
        // Start all protocol adapters
        let mut tasks = Vec::new();
        
        for mut adapter in self.adapters {
            let core = Arc::clone(&self.core);
            tasks.push(tokio::spawn(async move {
                adapter.serve(core).await
            }));
        }
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        // Graceful shutdown
        for task in tasks {
            task.abort();
        }
        
        Ok(())
    }
}
```

### **Usage Example**

```rust
// Start Songbird with multiple protocols
let server = UniversalProtocolServer::new(core)
    .with_http("[::]:8080".parse()?)        // HTTP/REST (dual-stack)
    .with_grpc("[::]:50051".parse()?)       // gRPC
    .with_websocket("/api/ws")              // WebSocket
    .with_quic("[::]:4433".parse()?, tls)   // QUIC/HTTP3
    .serve()
    .await?;
```

---

## ✅ SUCCESS CRITERIA

### **Phase 1 (IPv6)**: ✅ Complete When
- [ ] NestGate connects via `localhost`
- [ ] Both IPv4 and IPv6 work
- [ ] Discovery flow works universally
- [ ] No breaking changes

### **Phase 2 (gRPC)**: ✅ Complete When
- [ ] gRPC server running on port 50051
- [ ] Service registration works via gRPC
- [ ] Discovery works via gRPC
- [ ] Streaming updates working
- [ ] 7-10x performance improvement demonstrated
- [ ] NestGate has gRPC client option

### **Phase 3 (WebSocket)**: ✅ Complete When
- [ ] WebSocket endpoint at `/api/ws`
- [ ] Real-time service updates streaming
- [ ] Bidirectional command/response working
- [ ] Automatic reconnection handling
- [ ] Dashboard can use WebSocket for live updates

### **Phase 4 (QUIC)**: ✅ Complete When
- [ ] QUIC server running on port 4433
- [ ] TLS 1.3 encryption by default
- [ ] 0-RTT connections working
- [ ] Performance benchmarks show improvement
- [ ] Mobile clients benefit from connection migration

---

## 📊 PERFORMANCE TARGETS

| Protocol | Latency Target | Throughput Target | Use Case |
|----------|---------------|-------------------|----------|
| HTTP/REST | <10ms | 10K req/s | General API |
| gRPC | <1ms | 100K req/s | High-performance RPC |
| WebSocket | <5ms | 50K msg/s | Real-time updates |
| QUIC/HTTP3 | <5ms | 50K req/s | Modern web, mobile |

---

## 🔐 SECURITY CONSIDERATIONS

1. **TLS/Encryption**:
   - HTTP: Optional TLS 1.2+
   - gRPC: Optional TLS 1.2+
   - WebSocket: Optional WSS
   - QUIC: Mandatory TLS 1.3

2. **Authentication**:
   - Unified auth across all protocols
   - Bearer tokens (HTTP)
   - Metadata (gRPC)
   - Connection auth (WebSocket, QUIC)

3. **Authorization**:
   - Same RBAC model for all protocols
   - Capability-based access control
   - Service-level permissions

---

## 📚 REFERENCES

- **Related Specs**:
  - `SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md` (Phase 1 foundation)
  - `SONGBIRD_NATIVE_RPC_SPECIFICATION.md` (existing gRPC concepts)
  - `HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md` (existing hybrid patterns)
  - `TRANSPORT_SYSTEM_EVOLUTION_SPEC.md` (transport evolution)

- **External Standards**:
  - RFC 9114 (HTTP/3)
  - RFC 9000 (QUIC)
  - RFC 6455 (WebSocket)
  - gRPC specification (grpc.io)

---

**Status**: 📋 **DESIGN SPECIFICATION** - Ready for Implementation  
**Next Action**: Complete Phase 1 (IPv6) to unlock protocol expansion  
**Long-term Vision**: Protocol-agnostic service mesh with client choice

**This specification provides a clear path from the current IPv6 shortfall to a fully protocol-agnostic service mesh architecture.**

