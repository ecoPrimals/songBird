# 🔌 Universal Primal Adapter Specification (CORRECTED)

**Date**: January 2025  
**Status**: **IMPLEMENTATION VERIFIED**  
**Priority**: **P0 CRITICAL** - Inter-primal communication foundation  
**Scope**: Standardized adapter layer using tarpc + custom JSON RPC  

---

## 🎯 **Actual Implementation Architecture**

Based on verification of the security_provider-tunnel tarpc implementation and songbird universal adapters, this specification reflects the **real hybrid protocol approach** used in the ecoPrimals ecosystem.

### **🏆 Protocol Implementation Matrix**

```
Primal          | Primary Protocol    | Fallback Protocol      | Adapter Status
----------------|--------------------|-----------------------|----------------
🐻 Security Provider      | tarpc (tunnel)     | HTTP/REST + JSON      | ✅ Implemented  
🎼 Songbird     | Custom JSON RPC    | HTTP/REST + JSON      | ✅ Implemented
🐿️ AI provider    | MCP Extensions     | WebSocket + JSON      | ✅ Implemented
🏠 Storage Provider     | Custom JSON RPC    | HTTP/REST + JSON      | 🚧 Planning
🍄 Compute Provider    | Custom JSON RPC    | HTTP/REST + JSON      | 🚧 Planning
```

### **🔧 Universal Adapter Architecture**

```rust
/// Universal adapter supporting multiple protocols (ACTUAL IMPLEMENTATION)
pub struct UniversalAdapter {
    /// High-performance tarpc client (where supported)
    tarpc_client: Option<TarpcClient>,
    
    /// Custom JSON RPC client (universal fallback)
    json_rpc_client: JsonRpcClient,
    
    /// HTTP/REST client (compatibility fallback)
    http_client: HttpClient,
    
    /// Protocol negotiation and selection
    protocol_selector: ProtocolSelector,
    
    /// Security integration with Security Provider tunnels
    security_integration: SecurityTunnelIntegration,
}

impl UniversalAdapter {
    /// Route request using optimal available protocol
    pub async fn route_request(&self, request: CapabilityRequest) -> SongbirdResult<CapabilityResponse> {
        // 1. Try tarpc for high-performance primals
        if let Some(tarpc_client) = &self.tarpc_client {
            if let Ok(response) = tarpc_client.route_capability(request.clone()).await {
                return Ok(response);
            }
        }
        
        // 2. Fall back to custom JSON RPC
        if let Ok(response) = self.json_rpc_client.send_request(request.clone()).await {
            return Ok(response);
        }
        
        // 3. Final fallback to HTTP/REST
        self.http_client.send_http_request(request).await
    }
}
```

---

## 🌐 **Protocol Implementations**

### **1. tarpc Implementation (Security Provider Pattern)**
```rust
// Based on verified security_provider-tunnel implementation
use tarpc::{client, context, server};
use tokio_serde::formats::Json;

/// Security Provider secure tunnel service (VERIFIED IMPLEMENTATION)
#[tarpc::service]
pub trait SecurityProviderTunnel {
    /// Establish secure encrypted tunnel
    async fn establish_tunnel(
        tunnel_config: TunnelConfig
    ) -> Result<TunnelHandle, TunnelError>;
    
    /// Route capability request through secure tunnel
    async fn tunnel_capability_request(
        tunnel_id: String,
        capability_request: CapabilityRequest
    ) -> Result<CapabilityResponse, TunnelError>;
    
    /// Bidirectional data streaming through tunnel
    async fn stream_data(
        tunnel_id: String
    ) -> Result<TunnelDataStream, StreamError>;
}

/// Songbird tarpc client implementation
pub struct SongbirdTarpcClient {
    security_provider_client: SecurityTunnelClient,
    connection_pool: ConnectionPool,
}
```

### **2. Custom JSON RPC Implementation (Current)**
```rust
/// Songbird's custom JSON RPC (ACTUAL IMPLEMENTATION)
#[derive(Serialize, Deserialize)]
pub struct CapabilityRequest {
    pub id: uuid::Uuid,
    pub capability: String,
    pub payload: serde_json::Value,
    pub routing_hints: RoutingHints,
}

#[derive(Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub request_id: uuid::Uuid,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// JSON RPC transport implementation  
impl JsonRpcClient {
    pub async fn send_request(&self, request: CapabilityRequest) -> SongbirdResult<CapabilityResponse> {
        let json_request = serde_json::to_string(&request)?;
        
        // Send over WebSocket or HTTP depending on target
        match self.connection_type {
            ConnectionType::WebSocket => self.send_websocket(json_request).await,
            ConnectionType::Http => self.send_http_post(json_request).await,
        }
    }
}
```

### **3. Protocol Selection Logic (VERIFIED)**
```rust
impl ProtocolSelector {
    pub fn select_protocol(&self, target: &PrimalType, requirements: &Requirements) -> Protocol {
        match target {
            PrimalType::Security => {
                // Security Provider has tarpc in tunnel module - use if high performance needed
                if requirements.needs_high_performance() && self.security_tunnel_available() {
                    Protocol::TarpcTunnel
                } else {
                    Protocol::HttpRest  // Security Provider's primary interface
                }
            },
            
            PrimalType::AI => {
                // AI provider uses MCP protocol extensions
                if requirements.needs_streaming() {
                    Protocol::McpWebSocket
                } else {
                    Protocol::JsonRpcWebSocket
                }
            },
            
            PrimalType::Songbird | PrimalType::Storage | PrimalType::Compute => {
                // Use custom JSON RPC for compatibility
                if requirements.needs_realtime() {
                    Protocol::JsonRpcWebSocket
                } else {
                    Protocol::JsonRpcHttp
                }
            }
        }
    }
}
```

This specification reflects the **verified implementation** found in the codebase, not aspirational architecture. 