# Progressive Protocol Enhancement Specification
## Songbird Universal Orchestrator

**Status**: 🎯 Design Phase - Strategic  
**Priority**: P1 - High (Core Architecture)  
**Created**: November 11, 2025  
**Version**: 1.0.0

---

## 📋 Executive Summary

This specification defines **Progressive Protocol Enhancement**, a system where Songbird:
1. **Starts universal** - HTTP/REST for initial connection (works everywhere)
2. **Detects capabilities** - Discovers what protocols the peer supports
3. **Upgrades intelligently** - Transitions to faster protocols (tarpc, JSON-RPC)
4. **Reinforces connection** - Establishes all available protocol channels simultaneously

**Key Benefit**: "Connect once with HTTP, get 10-100x performance with RPC automatically"

---

## 🎯 Goals

### **Primary Goals**
1. **Universal Connectivity**: Any client can connect via HTTP/REST
2. **Automatic Optimization**: Upgrade to fastest protocol without manual configuration
3. **Multi-Protocol Reinforcement**: One connection triggers all available protocol channels
4. **Zero Breaking Changes**: Existing HTTP clients continue working
5. **Progressive Enhancement**: Better protocols enable better features

### **Strategic Goals**
- **Lower barrier to entry** - HTTP/REST works everywhere
- **Maximize performance** - Automatic upgrade to tarpc (10-100x faster)
- **Future-proof** - Easy to add new protocols (QUIC, WebSocket, etc.)
- **Service mesh ready** - Protocol negotiation is fundamental

---

## 🏗️ Architecture

### **Phase Flow**

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROGRESSIVE PROTOCOL ENHANCEMENT              │
└─────────────────────────────────────────────────────────────────┘

Phase 1: INITIAL CONNECTION (HTTP/REST)
  Client                          Songbird
    │                                 │
    │  GET /api/protocols/capabilities│
    │─────────────────────────────────>│
    │                                 │
    │  200 OK                         │
    │  {                              │
    │    "protocols": ["http", "tarpc", "json-rpc"],
    │    "versions": {...},           │
    │    "endpoints": {...}           │
    │  }                              │
    │<─────────────────────────────────│
    │                                 │

Phase 2: CAPABILITY NEGOTIATION
    │                                 │
    │  POST /api/protocols/negotiate  │
    │  {                              │
    │    "client_protocols": ["http", "tarpc"],
    │    "preferred": "tarpc"         │
    │  }                              │
    │─────────────────────────────────>│
    │                                 │
    │  200 OK                         │
    │  {                              │
    │    "upgrade_to": "tarpc",       │
    │    "endpoint": "tarpc://...8081",
    │    "token": "abc123..."         │
    │  }                              │
    │<─────────────────────────────────│
    │                                 │

Phase 3: PROTOCOL UPGRADE (tarpc)
    │                                 │
    │  tarpc.connect("...8081", token)│
    │=================================>│  ← Fast binary RPC
    │                                 │
    │  tarpc.hello("client_id")       │
    │=================================>│
    │                                 │
    │<═════════════════════════════════│
    │  "Hello! Session established"   │
    │                                 │

Phase 4: MULTI-PROTOCOL REINFORCEMENT
    │                                 │
    │  [HTTP still active for health] │
    │  [tarpc for fast RPC calls]     │
    │  [JSON-RPC for universal API]   │
    │  [WebSocket for streaming]      │
    │                                 │
    │  ALL PROTOCOLS ACTIVE           │
    │  CLIENT CHOOSES BEST FOR TASK   │
    └─────────────────────────────────┘
```

---

## 🔧 Technical Design

### **1. Protocol Capability Discovery**

**Endpoint**: `GET /api/protocols/capabilities`

**Response**:
```json
{
  "songbird_version": "0.2.1",
  "protocols": {
    "http": {
      "version": "1.1",
      "endpoints": {
        "federation": "http://[::]:8080/api/federation",
        "compute": "http://[::]:8080/api/compute",
        "deployment": "http://[::]:8080/api/deployment"
      },
      "features": ["rest", "streaming", "chunked"]
    },
    "tarpc": {
      "version": "0.34",
      "endpoint": "tcp://[::]:8081",
      "transport": "tcp",
      "features": ["rpc", "bidirectional", "multiplexing"],
      "performance": {
        "latency_us": 50,
        "throughput_mbps": 10000
      }
    },
    "json-rpc": {
      "version": "2.0",
      "endpoint": "http://[::]:8082/rpc",
      "transport": "http",
      "features": ["universal", "language-agnostic", "simple"]
    },
    "websocket": {
      "version": "13",
      "endpoint": "ws://[::]:8080/ws",
      "features": ["streaming", "bidirectional", "real-time"]
    }
  },
  "preferred_protocol": "tarpc",
  "fallback_protocol": "http"
}
```

---

### **2. Protocol Negotiation**

**Endpoint**: `POST /api/protocols/negotiate`

**Request**:
```json
{
  "client_id": "nestgate-primal-001",
  "client_protocols": ["http", "tarpc", "websocket"],
  "preferred": "tarpc",
  "capabilities": {
    "max_connections": 100,
    "supports_tls": true,
    "ipv6": true
  }
}
```

**Response**:
```json
{
  "negotiation_id": "nego_abc123",
  "selected_protocol": "tarpc",
  "upgrade_token": "eyJhbGc...",
  "endpoints": {
    "primary": "tcp://[::]:8081",
    "fallback": "http://[::]:8080"
  },
  "session": {
    "expires_at": "2025-11-11T12:00:00Z",
    "max_idle_seconds": 300,
    "keep_alive": true
  },
  "reinforcement": {
    "enabled": true,
    "protocols": ["http", "tarpc", "json-rpc"],
    "strategy": "all_available"
  }
}
```

---

### **3. Protocol Upgrade**

#### **HTTP → tarpc Upgrade**

```rust
// Client-side (example)
async fn connect_to_songbird(addr: &str) -> Result<SongbirdClient> {
    // Phase 1: Initial HTTP connection
    let http_client = HttpClient::new(addr);
    
    // Phase 2: Discover capabilities
    let capabilities = http_client
        .get("/api/protocols/capabilities")
        .await?;
    
    // Phase 3: Negotiate upgrade
    let negotiation = http_client
        .post("/api/protocols/negotiate", NegotiateRequest {
            client_protocols: vec!["http", "tarpc"],
            preferred: "tarpc",
        })
        .await?;
    
    // Phase 4: Upgrade to tarpc
    if negotiation.selected_protocol == "tarpc" {
        let tarpc_client = tarpc::client::connect(
            &negotiation.endpoints.primary,
            negotiation.upgrade_token,
        ).await?;
        
        // Phase 5: Keep HTTP for health checks
        Ok(SongbirdClient {
            http: http_client,
            tarpc: Some(tarpc_client),
            protocol: Protocol::Tarpc,
        })
    } else {
        // Fallback to HTTP only
        Ok(SongbirdClient {
            http: http_client,
            tarpc: None,
            protocol: Protocol::Http,
        })
    }
}
```

#### **Server-side Protocol Router**

```rust
// Server-side (Songbird)
pub struct ProtocolRouter {
    http_server: HttpServer,
    tarpc_server: Option<TarpcServer>,
    json_rpc_server: Option<JsonRpcServer>,
    websocket_server: Option<WebSocketServer>,
    sessions: Arc<RwLock<HashMap<SessionId, ProtocolSession>>>,
}

impl ProtocolRouter {
    /// Handle protocol negotiation
    pub async fn negotiate(&self, request: NegotiateRequest) -> Result<NegotiateResponse> {
        // 1. Validate client capabilities
        let client_caps = self.validate_capabilities(&request)?;
        
        // 2. Select best protocol (client preferred + server available)
        let selected = self.select_protocol(&request.client_protocols, &request.preferred)?;
        
        // 3. Generate upgrade token
        let token = self.generate_upgrade_token(&request.client_id, &selected)?;
        
        // 4. Create session
        let session = ProtocolSession {
            id: SessionId::new(),
            client_id: request.client_id,
            protocol: selected.clone(),
            token: token.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(300),
        };
        
        self.sessions.write().await.insert(session.id, session);
        
        // 5. Return negotiation response
        Ok(NegotiateResponse {
            selected_protocol: selected,
            upgrade_token: token,
            endpoints: self.get_endpoints(&selected),
            reinforcement: ReinforcementConfig {
                enabled: true,
                protocols: vec!["http", "tarpc", "json-rpc"],
                strategy: ReinforcementStrategy::AllAvailable,
            },
        })
    }
    
    /// Select best protocol based on client + server capabilities
    fn select_protocol(
        &self,
        client_protocols: &[String],
        preferred: &str,
    ) -> Result<String> {
        // Priority order:
        // 1. Client preferred (if we support it)
        // 2. tarpc (fastest)
        // 3. json-rpc (universal)
        // 4. http (fallback)
        
        if client_protocols.contains(&preferred.to_string()) {
            if self.supports_protocol(preferred) {
                return Ok(preferred.to_string());
            }
        }
        
        for protocol in &["tarpc", "json-rpc", "http"] {
            if client_protocols.contains(&protocol.to_string()) 
                && self.supports_protocol(protocol) {
                return Ok(protocol.to_string());
            }
        }
        
        Ok("http".to_string()) // Always fallback to HTTP
    }
}
```

---

### **4. Multi-Protocol Reinforcement**

#### **Connection Reinforcement Strategy**

When a client successfully connects:

```rust
pub struct ConnectionReinforcement {
    primary: ProtocolConnection,
    reinforced: Vec<ProtocolConnection>,
    strategy: ReinforcementStrategy,
}

pub enum ReinforcementStrategy {
    /// Establish all available protocols
    AllAvailable,
    
    /// Establish only high-performance protocols
    PerformanceOnly,
    
    /// Establish based on task requirements
    TaskBased,
    
    /// Manual selection
    Manual(Vec<String>),
}

impl ConnectionReinforcement {
    /// Reinforce connection with all available protocols
    pub async fn reinforce_all(&mut self) -> Result<()> {
        // Primary already established (e.g., tarpc)
        let primary_protocol = &self.primary.protocol;
        
        // Reinforce with other protocols
        for protocol in &["http", "json-rpc", "websocket"] {
            if protocol != primary_protocol {
                if let Ok(conn) = self.establish_protocol(protocol).await {
                    self.reinforced.push(conn);
                    info!("✅ Reinforced with {}", protocol);
                }
            }
        }
        
        Ok(())
    }
    
    /// Route request to best protocol for the task
    pub fn route_request(&self, request: &Request) -> &ProtocolConnection {
        match request.task_type {
            TaskType::FastRpc => {
                // Use tarpc for fast RPC
                self.reinforced
                    .iter()
                    .find(|c| c.protocol == "tarpc")
                    .unwrap_or(&self.primary)
            }
            TaskType::Streaming => {
                // Use WebSocket for streaming
                self.reinforced
                    .iter()
                    .find(|c| c.protocol == "websocket")
                    .unwrap_or(&self.primary)
            }
            TaskType::Universal => {
                // Use JSON-RPC for universal access
                self.reinforced
                    .iter()
                    .find(|c| c.protocol == "json-rpc")
                    .unwrap_or(&self.primary)
            }
            TaskType::HealthCheck => {
                // Always use HTTP for health checks
                self.reinforced
                    .iter()
                    .find(|c| c.protocol == "http")
                    .unwrap_or(&self.primary)
            }
        }
    }
}
```

---

## 📊 Performance Characteristics

### **Protocol Performance Matrix**

| Protocol | Latency | Throughput | Setup Time | Use Case |
|----------|---------|------------|------------|----------|
| **HTTP/REST** | ~5ms | 100 MB/s | ~50ms | Initial connection, health checks |
| **tarpc** | ~50μs | 10 GB/s | ~100ms | Fast RPC, internal communication |
| **JSON-RPC** | ~2ms | 500 MB/s | ~50ms | Universal API, external clients |
| **WebSocket** | ~1ms | 1 GB/s | ~100ms | Streaming, real-time events |

### **Upgrade Impact**

```
Initial Connection (HTTP):     50ms
Protocol Negotiation:          +10ms
tarpc Upgrade:                 +100ms
────────────────────────────────────
Total First Connection:        160ms

Subsequent RPC calls:
  HTTP/REST:  5,000μs (5ms)
  tarpc:         50μs
  ────────────────────────
  Speedup:     100x faster! 🚀
```

---

## 🔐 Security Considerations

### **1. Upgrade Token Security**

```rust
pub struct UpgradeToken {
    /// Client ID
    client_id: String,
    
    /// Target protocol
    protocol: String,
    
    /// Session ID
    session_id: SessionId,
    
    /// Expiration (short-lived, 5 minutes)
    expires_at: DateTime<Utc>,
    
    /// Signature (HMAC-SHA256)
    signature: Vec<u8>,
}

impl UpgradeToken {
    /// Generate secure upgrade token
    pub fn generate(client_id: &str, protocol: &str, secret: &[u8]) -> Result<String> {
        let token = UpgradeToken {
            client_id: client_id.to_string(),
            protocol: protocol.to_string(),
            session_id: SessionId::new(),
            expires_at: Utc::now() + Duration::seconds(300),
            signature: vec![],
        };
        
        // Sign token
        let mut signed = token;
        signed.signature = sign_token(&signed, secret)?;
        
        // Encode as JWT or custom format
        Ok(encode_token(&signed)?)
    }
    
    /// Validate upgrade token
    pub fn validate(&self, secret: &[u8]) -> Result<()> {
        // Check expiration
        if Utc::now() > self.expires_at {
            return Err(anyhow!("Token expired"));
        }
        
        // Verify signature
        let expected = sign_token(self, secret)?;
        if self.signature != expected {
            return Err(anyhow!("Invalid signature"));
        }
        
        Ok(())
    }
}
```

### **2. Protocol-Specific Security**

| Protocol | TLS | Authentication | Authorization |
|----------|-----|----------------|---------------|
| HTTP | Optional | Token/OAuth | Role-based |
| tarpc | Required | mTLS + Token | Capability-based |
| JSON-RPC | Optional | Token | Method-based |
| WebSocket | Required | Token + Session | Connection-based |

---

## 🎯 Implementation Plan

### **Phase 1: Foundation (Week 1)**
```rust
// 1. Add protocol capability discovery endpoint
// File: crates/songbird-orchestrator/src/server/protocol_api.rs

pub fn protocol_routes() -> Router {
    Router::new()
        .route("/capabilities", get(get_capabilities))
        .route("/negotiate", post(negotiate_protocol))
        .route("/upgrade", post(upgrade_connection))
}

async fn get_capabilities() -> Json<CapabilitiesResponse> {
    Json(CapabilitiesResponse {
        protocols: vec!["http", "tarpc", "json-rpc"],
        preferred: "tarpc",
        endpoints: get_all_endpoints(),
    })
}
```

### **Phase 2: Negotiation (Week 2)**
```rust
// 2. Implement protocol negotiation
async fn negotiate_protocol(
    Json(request): Json<NegotiateRequest>,
) -> Result<Json<NegotiateResponse>> {
    let router = ProtocolRouter::global();
    let response = router.negotiate(request).await?;
    Ok(Json(response))
}
```

### **Phase 3: tarpc Integration (Weeks 3-4)**
```rust
// 3. Add tarpc server alongside HTTP
pub struct SongbirdOrchestrator {
    http_server: HttpServer,
    tarpc_server: TarpcServer,  // ← NEW
    protocol_router: ProtocolRouter,
}

impl SongbirdOrchestrator {
    pub async fn start(&self) -> Result<()> {
        // Start HTTP server (port 8080)
        self.http_server.start().await?;
        
        // Start tarpc server (port 8081)
        self.tarpc_server.start().await?;  // ← NEW
        
        Ok(())
    }
}
```

### **Phase 4: Reinforcement (Week 5)**
```rust
// 4. Implement multi-protocol reinforcement
impl ProtocolRouter {
    pub async fn reinforce_connection(
        &self,
        session_id: SessionId,
    ) -> Result<ReinforcementResult> {
        let session = self.get_session(&session_id)?;
        
        // Establish all available protocols
        for protocol in &["http", "tarpc", "json-rpc"] {
            if protocol != &session.primary_protocol {
                self.establish_secondary(session_id, protocol).await?;
            }
        }
        
        Ok(ReinforcementResult::success())
    }
}
```

---

## 📚 Configuration

### **songbird.toml**

```toml
[protocols]
# Enable progressive protocol enhancement
enabled = true

# Available protocols
available = ["http", "tarpc", "json-rpc", "websocket"]

# Preferred protocol for upgrades
preferred = "tarpc"

# Automatic reinforcement
[protocols.reinforcement]
enabled = true
strategy = "all_available"  # or "performance_only", "task_based"

# Protocol-specific configuration
[protocols.http]
port = 8080
bind = "[::]"
tls = false

[protocols.tarpc]
port = 8081
bind = "[::]"
tls = true
transport = "tcp"

[protocols.json_rpc]
port = 8082
bind = "[::]"
tls = false

[protocols.websocket]
port = 8080  # Share with HTTP
path = "/ws"
```

---

## 🔄 Client Integration Examples

### **Python Client**

```python
import songbird

# Connect with HTTP initially
client = songbird.connect("http://localhost:8080")

# Automatic protocol upgrade to tarpc
# (happens transparently)

# Now using tarpc for fast RPC!
result = client.call("compute.schedule_task", {
    "task": "train_model",
    "data": "dataset.parquet"
})

print(f"Using protocol: {client.active_protocol}")  # "tarpc"
print(f"Available protocols: {client.available_protocols}")  # ["http", "tarpc", "json-rpc"]
```

### **JavaScript/Node.js Client**

```javascript
const songbird = require('@songbird/client');

// Connect with HTTP
const client = await songbird.connect('http://localhost:8080');

// Negotiate upgrade
await client.upgradeProtocol('json-rpc');

// Use JSON-RPC for universal compatibility
const result = await client.call('compute.schedule_task', {
  task: 'train_model',
  data: 'dataset.parquet'
});

console.log(`Active: ${client.activeProtocol}`);  // "json-rpc"
```

### **Rust Client (Native)**

```rust
use songbird_client::SongbirdClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect with progressive enhancement
    let mut client = SongbirdClient::connect("http://localhost:8080").await?;
    
    // Automatic upgrade to tarpc (if available)
    client.upgrade_to_best_protocol().await?;
    
    // Fast RPC calls
    let result = client.compute()
        .schedule_task("train_model", "dataset.parquet")
        .await?;
    
    println!("Protocol: {}", client.active_protocol());  // "tarpc"
    
    Ok(())
}
```

---

## 📊 Benefits

### **1. Universal Connectivity**
- ✅ Any client can connect via HTTP/REST (works everywhere)
- ✅ No special requirements for initial connection
- ✅ Gradual enhancement without breaking changes

### **2. Automatic Performance Optimization**
- ✅ 10-100x faster with tarpc upgrade
- ✅ No manual configuration needed
- ✅ Intelligent protocol selection

### **3. Multi-Protocol Reinforcement**
- ✅ One connection triggers all available channels
- ✅ Best protocol for each task
- ✅ Redundancy and failover

### **4. Future-Proof**
- ✅ Easy to add new protocols
- ✅ Backward compatible with HTTP-only clients
- ✅ Standards-based (HTTP/1.1, WebSocket, etc.)

---

## 🎯 Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Initial Connection Time** | < 100ms | Time to first successful HTTP request |
| **Protocol Upgrade Time** | < 200ms | Time from HTTP to tarpc |
| **Upgrade Success Rate** | > 95% | % of clients successfully upgrading |
| **Performance Improvement** | 10-100x | RPC latency vs HTTP latency |
| **Reinforcement Time** | < 500ms | Time to establish all protocols |
| **HTTP Fallback Rate** | < 5% | % of clients staying on HTTP only |

---

## 🚀 Deployment Strategy

### **Week 1: HTTP Baseline**
- Keep existing HTTP/REST API
- Add `/api/protocols/capabilities` endpoint
- No breaking changes

### **Week 2: Negotiation**
- Add `/api/protocols/negotiate` endpoint
- Implement upgrade token generation
- Still HTTP-only (tarpc not yet active)

### **Week 3-4: tarpc Integration**
- Start tarpc server on port 8081
- Implement protocol upgrade
- Test with internal clients

### **Week 5: Reinforcement**
- Enable multi-protocol reinforcement
- Monitor performance improvements
- Roll out to production

---

## ✅ Acceptance Criteria

- [ ] HTTP clients can discover protocol capabilities
- [ ] Protocol negotiation returns valid upgrade tokens
- [ ] tarpc upgrade works for capable clients
- [ ] HTTP-only clients continue working without changes
- [ ] Multi-protocol reinforcement establishes all channels
- [ ] Protocol selection is automatic and intelligent
- [ ] 10x+ performance improvement for upgraded clients
- [ ] < 5% fallback rate to HTTP-only
- [ ] Comprehensive documentation and examples
- [ ] Client libraries for Python, JavaScript, Rust

---

## 📋 References

- **HTTP/2 Upgrade**: RFC 7540 (connection upgrade mechanism)
- **WebSocket Upgrade**: RFC 6455 (Upgrade: websocket header)
- **ALPN**: RFC 7301 (Application-Layer Protocol Negotiation)
- **tarpc**: https://github.com/google/tarpc
- **JSON-RPC 2.0**: https://www.jsonrpc.org/specification

---

**Status**: 🎯 Design Complete - Ready for Implementation  
**Next Steps**: Review with team → Implement Phase 1 → Test → Deploy  
**Timeline**: 5 weeks from approval to production

---

*Progressive Protocol Enhancement - Making fast protocols universally accessible*  
*Start with HTTP, end with tarpc - automatically! 🚀*

