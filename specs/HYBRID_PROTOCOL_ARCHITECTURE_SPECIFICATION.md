# 🌐 Hybrid Protocol Architecture Specification

**Date**: January 2025  
**Status**: **CURRENT IMPLEMENTATION**  
**Priority**: **P0 CRITICAL** - Foundation transport layer  
**Scope**: Ecosystem-wide protocol standardization using tarpc + custom JSON RPC  

---

## 🎯 **Actual Implementation Architecture**

Based on review of beardog and songbird implementations, the ecoPrimals ecosystem uses a **hybrid protocol approach** with different protocols optimized for different use cases.

### **🏆 Protocol Selection Matrix (CORRECTED)**

```
Use Case                    | Protocol Choice              | Implementation Status | Rationale
---------------------------|------------------------------|---------------------|---------------------------
Web UI ↔ Songbird          | WebSocket + Custom JSON      | ✅ Implemented      | Browser compatibility, real-time
External API ↔ Songbird     | HTTP/REST + JSON             | ✅ Implemented      | Universal compatibility  
Songbird ↔ BearDog          | tarpc + HTTP fallback        | 🚧 Tunnel module    | High performance + security
Songbird ↔ NestGate         | Custom JSON RPC              | 🚧 Planning         | Storage-optimized
Songbird ↔ ToadStool        | Custom JSON RPC              | 🚧 Planning         | Resource coordination
Songbird ↔ Squirrel         | MCP Protocol Extensions       | ✅ Implemented      | AI agent streaming
Cross-Primal Events         | Event System + JSON          | ✅ Implemented      | Reactive, pub/sub pattern
Internal Service Mesh       | tarpc (where supported)      | 🚧 Transitioning    | Zero-copy, type safety
```

### **🔧 Transport Layer Implementation**

#### **External Communication (Client-Facing)**
```rust
// Current Songbird implementation
pub struct ExternalCommunicationLayer {
    /// WebSocket server for real-time web clients
    websocket_server: Arc<WebSocketServer>,
    
    /// HTTP/REST API for traditional clients  
    http_api_server: Arc<HttpApiServer>,
    
    /// Custom JSON RPC over WebSocket for advanced clients
    json_rpc_websocket: Arc<JsonRpcWebSocketServer>,
}
```

#### **Internal Communication (Service-to-Service)**
```rust
// Hybrid approach - tarpc where possible, JSON RPC fallback
pub struct InternalCommunicationLayer {
    /// tarpc connections for high-performance primals
    tarpc_connections: HashMap<PrimalType, TarpcConnection>,
    
    /// Custom JSON RPC for compatibility
    json_rpc_connections: HashMap<PrimalType, JsonRpcConnection>,
    
    /// Universal adapter layer for protocol translation
    universal_adapters: UniversalAdapterPool,
}
```

### **🚨 Key Corrections to Previous Assumptions**

1. **NO gRPC ANYWHERE** - Pure Rust ecosystem using tarpc + JSON
2. **Gradual tarpc adoption** - Not universal yet, HTTP/JSON still primary
3. **BearDog uses HTTP/REST primarily** - tarpc only in tunnel module
4. **MCP protocol for AI** - Specialized for Squirrel integration
5. **Custom JSON RPC** - Not standard JSON-RPC, optimized for primals

---

## 🏗️ **Protocol Implementation Details**

### **tarpc Implementation (BearDog Tunnel Example)**
```rust
// Based on ../beardog/crates/beardog-tunnel/
use tarpc::{client, context, server};

#[tarpc::service]
pub trait SecureTunnel {
    /// Establish encrypted tunnel
    async fn establish_tunnel(tunnel_config: TunnelConfig) -> TunnelHandle;
    
    /// Send data through tunnel
    async fn send_data(tunnel_id: String, data: Vec<u8>) -> Result<(), TunnelError>;
    
    /// Bidirectional streaming
    async fn stream_data(tunnel_id: String) -> Stream<TunnelData>;
}
```

### **Custom JSON RPC Implementation**
```rust
// Songbird's current universal adapter approach
#[derive(Serialize, Deserialize)]
pub struct CustomRpcRequest {
    /// Unique request ID
    pub id: uuid::Uuid,
    
    /// Target primal capability
    pub capability: String,
    
    /// Request payload (JSON)
    pub payload: serde_json::Value,
    
    /// Routing hints for load balancing
    pub routing_hints: RoutingHints,
}

#[derive(Serialize, Deserialize)]  
pub struct CustomRpcResponse {
    /// Request ID for correlation
    pub request_id: uuid::Uuid,
    
    /// Success flag
    pub success: bool,
    
    /// Response data (JSON)
    pub data: Option<serde_json::Value>,
    
    /// Error information
    pub error: Option<String>,
    
    /// Performance metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
```

---

## 🎯 **Migration Strategy**

### **Phase 1: Current State (Completed)**
- ✅ HTTP/REST APIs operational
- ✅ WebSocket connections for real-time 
- ✅ Custom JSON RPC in universal adapters
- ✅ MCP protocol for AI integration

### **Phase 2: tarpc Integration (In Progress)**
- 🚧 BearDog tunnel module using tarpc
- 🚧 Universal adapters support tarpc fallback
- 🚧 Service discovery includes protocol capabilities

### **Phase 3: Ecosystem Standardization (Planned)**
- 📋 All primals support tarpc option
- 📋 Automatic protocol negotiation
- 📋 Performance monitoring per protocol
- 📋 Migration tools for community primals

---

## 🚀 **Performance Characteristics**

### **Protocol Benchmarks (Estimated)**
| Protocol | Latency | Throughput | Memory | Use Case |
|----------|---------|------------|--------|-----------|
| **tarpc** | <1ms | 100K+ msg/s | Zero-copy | Internal high-perf |
| **Custom JSON RPC** | 2-5ms | 50K+ msg/s | Moderate | Universal compatibility |
| **WebSocket + JSON** | 5-10ms | 20K+ msg/s | Higher | Real-time web clients |
| **HTTP/REST + JSON** | 10-50ms | 10K+ req/s | Highest | Traditional APIs |

### **Protocol Selection Logic**
```rust
impl ProtocolSelector {
    pub fn select_optimal_protocol(&self, target: &PrimalType, requirements: &Requirements) -> Protocol {
        // 1. Check if target supports tarpc
        if self.supports_tarpc(target) && requirements.needs_high_performance() {
            return Protocol::Tarpc;
        }
        
        // 2. Check if streaming is needed
        if requirements.needs_streaming() {
            return Protocol::WebSocketJsonRpc;
        }
        
        // 3. Fall back to HTTP for compatibility
        Protocol::HttpRestJson
    }
}
```

This specification reflects the **actual implementation** found in the codebase rather than aspirational architecture. 