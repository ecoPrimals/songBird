# 🌉 gRPC Gateway Adapter Specification
## Protocol Translation Layer for External Compatibility

**Version**: 1.0  
**Date**: November 11, 2025  
**Status**: 📋 ARCHITECTURE SPECIFICATION  
**Priority**: P2 - Medium Priority Enhancement

---

## 🎯 Executive Summary

**Strategy**: Accept gRPC connections externally, translate to native Rust RPC internally.

**Key Insight**: We rejected gRPC as our **internal protocol** due to C++ dependencies and vendor lock-in, but we can still **support gRPC clients** through a protocol gateway/adapter.

**Benefits**:
- ✅ **Internal**: Pure Rust (tarpc), no C++ dependencies
- ✅ **External**: gRPC compatibility for clients who need it
- ✅ **Flexibility**: Clients choose their preferred protocol
- ✅ **Performance**: Fast internal communication, universal external access

---

## 🏗️ Architecture: Protocol Gateway Pattern

### **Conceptual Design**

```
┌─────────────────────────────────────────────────────────┐
│                 External Clients                         │
│  (Can use ANY protocol they prefer)                      │
└────┬────────────┬────────────┬────────────┬─────────────┘
     │            │            │            │
     │ gRPC       │ HTTP/REST  │ JSON-RPC   │ WebSocket
     │            │            │            │
┌────▼────────────▼────────────▼────────────▼─────────────┐
│           Songbird Protocol Gateway Layer                │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Protocol Adapters (External → Internal)         │   │
│  │  • gRPC Adapter      (tonic → tarpc/JSON-RPC)   │   │
│  │  • HTTP Adapter      (hyper → tarpc/JSON-RPC)   │   │
│  │  • JSON-RPC Adapter  (jsonrpsee → tarpc)        │   │
│  │  • WebSocket Adapter (tungstenite → tarpc)      │   │
│  └──────────────────────────────────────────────────┘   │
└────┬────────────┬────────────┬────────────┬─────────────┘
     │            │            │            │
     └────────────┴────────────┴────────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Internal Rust RPC   │
            │   (tarpc - Pure Rust)│
            └──────────────────────┘
                       │
                       ▼
            ┌──────────────────────┐
            │  Service Mesh Core   │
            │  • Discovery         │
            │  • Registry          │
            │  • Federation        │
            └──────────────────────┘
```

### **Key Principle: Protocol Adapter Pattern**

Each external protocol has a **thin adapter** that:
1. Listens on external port (e.g., gRPC on 50051)
2. Accepts connections in that protocol
3. Translates to internal tarpc/JSON-RPC calls
4. Returns responses in original protocol

---

## 🔧 gRPC Gateway Implementation

### **Why This Works**

**External (Client-Facing)**:
- gRPC server using `tonic` (optional, only if needed)
- Protocol Buffers definitions for external API
- Standard gRPC tooling compatibility

**Internal (Songbird Core)**:
- Pure Rust tarpc (no C++ dependencies in core)
- Native Rust types and serde
- Full control over internal protocol evolution

**Translation Layer**:
- Thin adapter (~200-300 lines)
- Maps gRPC messages → Internal Rust types
- No core dependency on gRPC

### **Architecture Benefits**

```
External Clients Need gRPC?
  ├─ YES → Use gRPC Gateway (port 50051)
  │         └─> Translates to internal tarpc
  │
  └─ NO  → Use native protocols directly
            ├─> tarpc (binary, fastest)
            ├─> JSON-RPC (universal)
            └─> WebSocket (real-time)
```

---

## 📋 Implementation Design

### **File Structure**

```
crates/songbird-orchestrator/src/
├── server/
│   ├── grpc_gateway/           # NEW: gRPC Gateway
│   │   ├── mod.rs              # Gateway implementation
│   │   ├── service.rs          # gRPC service definitions
│   │   ├── translator.rs       # Protocol translation
│   │   └── proto/              # Protocol Buffer definitions
│   │       └── songbird.proto  # External API schema
│   │
│   ├── tarpc/                  # Internal RPC (core)
│   │   ├── mod.rs
│   │   └── service.rs
│   │
│   ├── jsonrpc/                # Universal RPC
│   │   └── mod.rs
│   │
│   └── mod.rs
```

### **gRPC Gateway Service Definition**

```rust
// File: crates/songbird-orchestrator/src/server/grpc_gateway/service.rs

use tonic::{Request, Response, Status};
use crate::server::tarpc::SongbirdTarpcClient;

/// External gRPC service definition
#[tonic::async_trait]
impl songbird_proto::songbird_service_server::SongbirdService for GrpcGateway {
    /// Register a service (external gRPC → internal tarpc)
    async fn register_service(
        &self,
        request: Request<songbird_proto::ServiceRegistrationRequest>,
    ) -> Result<Response<songbird_proto::ServiceRegistrationResponse>, Status> {
        // 1. Extract gRPC request
        let grpc_req = request.into_inner();
        
        // 2. Translate to internal Rust type
        let internal_req = ServiceInfo {
            name: grpc_req.name,
            address: grpc_req.address,
            port: grpc_req.port as u16,
            capabilities: grpc_req.capabilities,
        };
        
        // 3. Call internal tarpc service
        let result = self.internal_tarpc_client
            .register_service(internal_req)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        // 4. Translate response back to gRPC
        let grpc_response = songbird_proto::ServiceRegistrationResponse {
            service_id: result,
            success: true,
        };
        
        Ok(Response::new(grpc_response))
    }
    
    /// Discover services (external gRPC → internal tarpc)
    async fn discover_services(
        &self,
        request: Request<songbird_proto::DiscoveryRequest>,
    ) -> Result<Response<songbird_proto::DiscoveryResponse>, Status> {
        let grpc_req = request.into_inner();
        
        // Translate to internal query
        let internal_query = DiscoveryQuery {
            capabilities: grpc_req.capabilities,
        };
        
        // Call internal tarpc
        let services = self.internal_tarpc_client
            .discover_services(internal_query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        // Translate response
        let grpc_services = services.into_iter().map(|s| {
            songbird_proto::ServiceInfo {
                name: s.name,
                address: s.address,
                port: s.port as i32,
                capabilities: s.capabilities,
            }
        }).collect();
        
        Ok(Response::new(songbird_proto::DiscoveryResponse {
            services: grpc_services,
        }))
    }
}
```

### **Protocol Buffer Definition (External API Only)**

```protobuf
// File: crates/songbird-orchestrator/src/server/grpc_gateway/proto/songbird.proto

syntax = "proto3";

package songbird;

// External gRPC service (for clients who need gRPC)
service SongbirdService {
  rpc RegisterService(ServiceRegistrationRequest) returns (ServiceRegistrationResponse);
  rpc DiscoverServices(DiscoveryRequest) returns (DiscoveryResponse);
  rpc GetFederationStatus(FederationStatusRequest) returns (FederationStatusResponse);
  rpc StreamServiceUpdates(stream UpdateSubscription) returns (stream ServiceUpdate);
}

// External message types (protocol buffer format)
message ServiceRegistrationRequest {
  string name = 1;
  string address = 2;
  int32 port = 3;
  repeated string capabilities = 4;
}

message ServiceRegistrationResponse {
  string service_id = 1;
  bool success = 2;
}

message DiscoveryRequest {
  repeated string capabilities = 1;
}

message DiscoveryResponse {
  repeated ServiceInfo services = 1;
}

message ServiceInfo {
  string name = 1;
  string address = 2;
  int32 port = 3;
  repeated string capabilities = 4;
}

// ... other message definitions
```

### **Gateway Server Initialization**

```rust
// File: crates/songbird-orchestrator/src/server/grpc_gateway/mod.rs

use tonic::transport::Server;
use songbird_proto::songbird_service_server::SongbirdServiceServer;

/// gRPC Gateway - Optional protocol adapter
pub struct GrpcGateway {
    /// Connection to internal tarpc service (pure Rust)
    internal_tarpc_client: Arc<SongbirdTarpcClient>,
}

impl GrpcGateway {
    pub async fn new(tarpc_addr: &str) -> Result<Self> {
        // Connect to internal tarpc service
        let internal_tarpc_client = Arc::new(
            SongbirdTarpcClient::connect(tarpc_addr).await?
        );
        
        Ok(Self { internal_tarpc_client })
    }
    
    /// Start gRPC gateway server (optional, only if gRPC support needed)
    pub async fn serve(self, grpc_addr: SocketAddr) -> Result<()> {
        info!("🌉 Starting gRPC gateway on {}", grpc_addr);
        info!("   (Translating to internal tarpc)");
        
        Server::builder()
            .add_service(SongbirdServiceServer::new(self))
            .serve(grpc_addr)
            .await?;
        
        Ok(())
    }
}
```

### **Main Server with Optional gRPC Gateway**

```rust
// File: crates/songbird-orchestrator/src/app/mod.rs

pub async fn start_all_servers(&self) -> Result<()> {
    // 1. Start CORE internal tarpc server (always)
    let tarpc_addr = SocketAddr::new(
        IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        SafeEnv::get_port("SONGBIRD_TARPC_PORT", 8081)
    );
    tokio::spawn(start_tarpc_server(tarpc_addr));
    info!("✅ Internal tarpc server started (pure Rust)");
    
    // 2. Start JSON-RPC server (always, for universal access)
    let jsonrpc_addr = SocketAddr::new(
        IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        SafeEnv::get_port("SONGBIRD_JSONRPC_PORT", 8080)
    );
    tokio::spawn(start_jsonrpc_server(jsonrpc_addr));
    info!("✅ JSON-RPC server started (universal)");
    
    // 3. Start gRPC gateway (OPTIONAL, only if enabled)
    if SafeEnv::get_bool("SONGBIRD_GRPC_GATEWAY_ENABLED", false) {
        let grpc_addr = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            SafeEnv::get_port("SONGBIRD_GRPC_PORT", 50051)
        );
        
        let gateway = GrpcGateway::new("localhost:8081").await?;
        tokio::spawn(gateway.serve(grpc_addr));
        info!("🌉 gRPC gateway started (optional adapter)");
    } else {
        info!("ℹ️  gRPC gateway disabled (use SONGBIRD_GRPC_GATEWAY_ENABLED=true to enable)");
    }
    
    Ok(())
}
```

---

## 🎯 Key Advantages of This Approach

### **1. Core Stays Pure Rust**
```
✅ No C++ dependencies in core codebase
✅ No protoc required for core development
✅ No vendor lock-in in core architecture
✅ Full control over internal protocol
```

### **2. External Compatibility**
```
✅ Clients can use gRPC if they need it
✅ No forcing gRPC on everyone
✅ Multiple protocols supported
✅ Client choice preserved
```

### **3. Optional Deployment**
```
✅ gRPC gateway is OPTIONAL
✅ Only enable if clients need it
✅ No performance penalty if disabled
✅ Configuration-driven
```

### **4. Performance Optimization**
```
✅ Internal: tarpc (10-100x faster than HTTP)
✅ External: Client's preferred protocol
✅ Translation layer is thin (~1ms overhead)
✅ Best of both worlds
```

---

## 📊 Performance Comparison

| Scenario | Internal Protocol | External Protocol | Total Latency |
|----------|-------------------|-------------------|---------------|
| **Rust Client (Optimal)** | tarpc | tarpc | <1ms (direct) |
| **Python Client (Universal)** | tarpc | JSON-RPC | ~5ms (translation) |
| **Go Client (gRPC)** | tarpc | gRPC | ~6ms (translation) |
| **Web Client (WebSocket)** | tarpc | WebSocket | <5ms (translation) |

**Key Insight**: Translation overhead is minimal (~1-2ms), and core stays fast.

---

## 🔧 Configuration

### **Environment Variables**

```bash
# Core (always enabled)
SONGBIRD_TARPC_PORT=8081              # Internal tarpc (pure Rust)
SONGBIRD_JSONRPC_PORT=8080            # Universal JSON-RPC

# Optional gateways (enable as needed)
SONGBIRD_GRPC_GATEWAY_ENABLED=true    # Enable gRPC gateway (default: false)
SONGBIRD_GRPC_PORT=50051              # gRPC gateway port

SONGBIRD_WEBSOCKET_ENABLED=true       # Enable WebSocket (default: true)
SONGBIRD_WEBSOCKET_PORT=8082          # WebSocket port
```

### **Cargo.toml (Optional gRPC Dependencies)**

```toml
[dependencies]
# Core (always included)
tarpc = { version = "0.34", features = ["full"] }
jsonrpsee = { version = "0.21", features = ["server"] }

# Optional: gRPC gateway (feature-gated)
tonic = { version = "0.11", optional = true }
prost = { version = "0.12", optional = true }

[build-dependencies]
# Only if gRPC gateway feature is enabled
tonic-build = { version = "0.11", optional = true }

[features]
default = ["tarpc", "jsonrpc"]
grpc-gateway = ["tonic", "prost", "tonic-build"]  # Optional feature
```

### **Building with gRPC Gateway (Optional)**

```bash
# Build without gRPC (default, pure Rust)
cargo build --workspace

# Build with gRPC gateway (optional)
cargo build --workspace --features grpc-gateway

# Run with gRPC gateway enabled
SONGBIRD_GRPC_GATEWAY_ENABLED=true cargo run --features grpc-gateway
```

---

## 📋 Implementation Roadmap

### **Phase 1: Core (Current Priority)**
1. ✅ IPv6 dual-stack (complete)
2. 🔧 tarpc internal server (Week 1-2)
3. 🔧 JSON-RPC universal endpoint (Week 3)

### **Phase 2: Optional Gateways**
1. gRPC Gateway (if needed by clients)
   - Estimate: 1 week
   - Thin adapter pattern
   - Feature-gated (optional)

2. WebSocket Gateway (real-time)
   - Estimate: 1 week
   - Bidirectional streaming
   - Always enabled by default

### **Phase 3: Advanced**
1. Custom protocol adapters
2. Performance tuning
3. Load balancing across gateways

---

## 🎓 Design Principles

### **1. Protocol Agnostic Core**
```
Core doesn't know about external protocols
↓
Protocol adapters translate to/from core
↓
Core evolves independently
```

### **2. Thin Translation Layer**
```
Minimal overhead (1-2ms)
↓
Direct type mapping where possible
↓
No business logic in adapters
```

### **3. Optional Everything**
```
Only enable what clients need
↓
No forced dependencies
↓
Configuration-driven deployment
```

### **4. Performance First**
```
Internal: Fastest (tarpc)
↓
External: Client's choice
↓
Translation: Optimized
```

---

## 🔐 Security Considerations

### **Gateway-Level Security**

```rust
// Apply security at gateway, not core
impl GrpcGateway {
    async fn register_service(&self, request: Request<...>) -> Result<...> {
        // 1. Authentication (gateway level)
        let token = request.metadata().get("authorization")
            .ok_or(Status::unauthenticated("Missing token"))?;
        
        validate_token(token)?;
        
        // 2. Authorization (gateway level)
        if !has_permission(token, "service.register") {
            return Err(Status::permission_denied("Not authorized"));
        }
        
        // 3. Call internal (already authenticated)
        let result = self.internal_tarpc_client
            .register_service(...)
            .await?;
        
        Ok(Response::new(result))
    }
}
```

---

## ✅ Success Criteria

### **gRPC Gateway Complete When**:
- [ ] gRPC server running on port 50051 (optional)
- [ ] Translates to internal tarpc correctly
- [ ] Go/Java/Python clients can connect via gRPC
- [ ] Performance overhead < 2ms
- [ ] Feature-gated (doesn't bloat core)
- [ ] Documentation complete

### **Overall Protocol Strategy Complete When**:
- [ ] Internal tarpc server (pure Rust, always on)
- [ ] JSON-RPC endpoint (universal, always on)
- [ ] gRPC gateway (optional, feature-gated)
- [ ] WebSocket endpoint (real-time, configurable)
- [ ] All protocols access same core
- [ ] Clients can choose preferred protocol

---

## 🎯 Strategic Summary

### **Our Decision**

**Core Internal Protocol**: tarpc (pure Rust, no C++ dependencies)  
**External Compatibility**: Protocol gateways/adapters (optional)

### **Why This is Better Than "Just Use gRPC"**

| Aspect | Our Approach | Pure gRPC |
|--------|--------------|-----------|
| **Core Dependencies** | ✅ Pure Rust | ❌ C++ protoc |
| **Vendor Lock-in** | ✅ None | ❌ Google ecosystem |
| **Internal Performance** | ✅ 10-100x faster | ❌ Slower |
| **External Compatibility** | ✅ gRPC + others | ✅ gRPC only |
| **Deployment Flexibility** | ✅ Enable as needed | ❌ Always required |
| **Future Control** | ✅ Full control | ❌ Limited |

### **Client Experience**

```
Rust Clients:
  → Use tarpc directly (fastest)

Python/JS Clients:
  → Use JSON-RPC (universal)

Go/Java Clients (if they need gRPC):
  → Use gRPC gateway (optional)

Web Clients:
  → Use WebSocket (real-time)

All clients access the same service mesh!
```

---

## 📚 References

**Design Patterns**:
- Gateway Pattern (Martin Fowler)
- Protocol Adapter Pattern
- Anti-Corruption Layer (DDD)

**Similar Implementations**:
- Envoy Proxy (protocol translation)
- NGINX (protocol bridging)
- Kong Gateway (multi-protocol)

**Related Specs**:
- `TARPC_JSON_RPC_PROTOCOL_SPEC.md` - Core strategy
- `UNIVERSAL_PROTOCOL_FRAMEWORK_SPECIFICATION.md` - Overall vision
- `SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md` - Network foundation

---

## 🎉 Conclusion

**Answer to "Can we interact with gRPC connections and make them Rust RPC on our side?"**

**YES!** ✅ 

We can absolutely create a thin gRPC gateway that:
1. Accepts external gRPC connections (for clients who need it)
2. Translates to internal tarpc (pure Rust, fast)
3. Keeps our core clean (no C++ dependencies)
4. Provides universal compatibility (client choice)

**Best of Both Worlds**:
- Internal: Pure Rust, fast, maintainable
- External: Whatever protocol clients prefer

**This is the right architecture**: Core stays pure, gateways provide compatibility.

---

**Status**: 📋 **ARCHITECTURE SPECIFICATION COMPLETE**  
**Priority**: P2 - Implement after core tarpc (Phase 2, optional)  
**Recommendation**: Start with tarpc + JSON-RPC, add gRPC gateway only if clients specifically need it

**Key Insight**: We're not rejecting gRPC clients, we're rejecting gRPC as our internal protocol. The gateway pattern lets us support gRPC without the downsides.

