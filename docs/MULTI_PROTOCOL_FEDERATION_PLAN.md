# 🌐 Multi-Protocol Federation Implementation Plan

**Date:** December 17, 2025  
**Goal:** Internet-ready, protocol-escalating, encrypted federation  
**Status:** 📋 PLANNING → IMPLEMENTATION

---

## 🎯 Vision

**"VPN-free encryption as emergent property of primal interactions"**

Build a federation system where:
1. ✅ **TLS secures initial handshake** (HTTPS)
2. ✅ **Protocol negotiation** allows escalation to high-performance protocols
3. ✅ **Concurrent multi-protocol support** (HTTP, JSON-RPC, tarpc simultaneously)
4. ✅ **Ready for BTSP integration** when BearDog is operational
5. ✅ **Works over internet AND LAN** with sovereign security

---

## 📊 Current State Assessment

### ✅ What We Have (December 17, 2025)

**TLS Foundation:**
- ✅ TLS/HTTPS server operational
- ✅ Self-signed certificate generation
- ✅ CA certificate support
- ✅ Environment-based configuration
- ✅ Internet-ready connections

**Federation:**
- ✅ 2-tower LAN federation working
- ✅ Sub-millisecond latency (0.186ms)
- ✅ Health monitoring
- ✅ Cross-tower communication

**Architecture:**
- ✅ Sovereign security patterns
- ✅ Capability-based discovery
- ✅ Graceful BearDog fallback
- ✅ Universal adapter framework

### 🔴 What We Need

**Protocol Layer:**
- ❌ tarpc server implementation
- ❌ JSON-RPC 2.0 endpoint
- ❌ Protocol negotiation/escalation
- ❌ Concurrent multi-protocol server

**Security Layer:**
- ❌ BTSP interface/mock for testing
- ❌ Protocol-level encryption config
- ❌ Multi-layer security coordination

**Federation:**
- ❌ Internet federation testing
- ❌ TLS on LAN federation
- ❌ Protocol capability advertisement

---

## 🏗️ Implementation Phases

### **Phase 1: Protocol Infrastructure** (Week 1, 5-7 days)

#### Task 1.1: JSON-RPC 2.0 Endpoint (2 days)
**File:** `crates/songbird-orchestrator/src/rpc/jsonrpc.rs` (new)

**Implementation:**
```rust
use jsonrpsee::server::{Server, ServerBuilder};
use jsonrpsee::RpcModule;

/// JSON-RPC 2.0 server for Songbird
pub struct JsonRpcServer {
    addr: SocketAddr,
    module: RpcModule<Arc<SongbirdOrchestrator>>,
}

impl JsonRpcServer {
    pub async fn new(orchestrator: Arc<SongbirdOrchestrator>) -> Result<Self> {
        let mut module = RpcModule::new(orchestrator);
        
        // Register methods
        module.register_async_method("songbird_discover", |params, ctx| async move {
            let capability: String = params.one()?;
            ctx.discover_by_capability(&capability).await
        })?;
        
        module.register_async_method("songbird_register", |params, ctx| async move {
            let service: ServiceRegistration = params.one()?;
            ctx.register_service(service).await
        })?;
        
        // ... more methods
        
        Ok(Self { addr, module })
    }
    
    pub async fn start(self) -> Result<ServerAddr> {
        let server = ServerBuilder::default()
            .build(self.addr)
            .await?;
            
        let handle = server.start(self.module)?;
        Ok(handle.local_addr()?)
    }
}
```

**Success Criteria:**
- ✅ JSON-RPC 2.0 endpoint at `/jsonrpc`
- ✅ All core operations exposed (discover, register, health)
- ✅ Works over HTTPS with existing TLS
- ✅ Client examples (curl, Python)

#### Task 1.2: tarpc Server Implementation (3 days)
**File:** `crates/songbird-orchestrator/src/rpc/tarpc.rs` (new)

**Implementation:**
```rust
use tarpc::{client, context, server};
use tarpc::server::Channel;

/// High-performance RPC service for primal-to-primal communication
#[tarpc::service]
pub trait SongbirdRpc {
    /// Discover services by capability
    async fn discover(capability: String) -> Vec<ServiceInfo>;
    
    /// Register a service
    async fn register(service: ServiceRegistration) -> RegistrationResult;
    
    /// Health check
    async fn health() -> HealthStatus;
    
    /// Stream service updates (bidirectional)
    async fn subscribe_updates() -> impl Stream<Item = ServiceUpdate>;
}

/// tarpc server implementation
pub struct TarpcServer {
    orchestrator: Arc<SongbirdOrchestrator>,
    addr: SocketAddr,
}

impl SongbirdRpc for TarpcServer {
    async fn discover(self, _: context::Context, capability: String) -> Vec<ServiceInfo> {
        self.orchestrator
            .discovery()
            .discover_by_capability(&capability)
            .await
            .unwrap_or_default()
    }
    
    async fn register(self, _: context::Context, service: ServiceRegistration) 
        -> RegistrationResult 
    {
        self.orchestrator
            .register_service(service)
            .await
            .map(|_| RegistrationResult::Success)
            .unwrap_or_else(|e| RegistrationResult::Error(e.to_string()))
    }
    
    // ... other methods
}

pub async fn start_tarpc_server(
    orchestrator: Arc<SongbirdOrchestrator>,
    addr: SocketAddr,
) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("🚀 tarpc server listening on {}", addr);
    
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let server = TarpcServer {
            orchestrator: orchestrator.clone(),
            addr: peer_addr,
        };
        
        tokio::spawn(async move {
            server
                .serve()
                .execute(stream)
                .await;
        });
    }
}
```

**Success Criteria:**
- ✅ tarpc server on port 8081
- ✅ High-performance binary RPC
- ✅ Bidirectional streaming
- ✅ 10x performance vs HTTP

#### Task 1.3: Protocol Negotiation (2 days)
**File:** `crates/songbird-orchestrator/src/rpc/negotiation.rs` (new)

**Implementation:**
```rust
/// Protocol capability negotiation
pub struct ProtocolNegotiator {
    supported_protocols: Vec<Protocol>,
    tls_config: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    Http,
    Https,
    JsonRpc,
    Tarpc,
    Btsp,  // Future: BearDog protocol
}

impl ProtocolNegotiator {
    /// Negotiate best protocol with peer
    pub async fn negotiate(&self, peer: &PeerInfo) -> Protocol {
        // 1. Query peer capabilities
        let peer_protocols = self.query_peer_protocols(peer).await?;
        
        // 2. Find best mutual protocol
        let best = self.find_best_protocol(&peer_protocols)?;
        
        // 3. Initiate protocol escalation
        match best {
            Protocol::Tarpc => self.escalate_to_tarpc(peer).await?,
            Protocol::JsonRpc => self.escalate_to_jsonrpc(peer).await?,
            Protocol::Btsp => self.escalate_to_btsp(peer).await?,
            _ => Protocol::Https,
        }
    }
    
    /// Escalate from HTTPS to tarpc
    async fn escalate_to_tarpc(&self, peer: &PeerInfo) -> Result<Protocol> {
        // 1. Send upgrade request over HTTPS
        let response = self.http_client
            .post(format!("{}/protocol/upgrade", peer.endpoint))
            .json(&UpgradeRequest {
                protocol: Protocol::Tarpc,
                port: 8081,
            })
            .send()
            .await?;
        
        // 2. If accepted, establish tarpc connection
        if response.status().is_success() {
            let tarpc_addr = response.json::<UpgradeResponse>().await?.addr;
            self.establish_tarpc_connection(tarpc_addr).await?;
            Ok(Protocol::Tarpc)
        } else {
            Ok(Protocol::Https)
        }
    }
}
```

**Success Criteria:**
- ✅ Automatic protocol negotiation
- ✅ Graceful fallback to lower protocols
- ✅ HTTP → HTTPS → JSON-RPC → tarpc escalation
- ✅ Protocol capability advertisement

---

### **Phase 2: Multi-Protocol Server** (Week 1, 2-3 days)

#### Task 2.1: Concurrent Protocol Server
**File:** `crates/songbird-orchestrator/src/rpc/multi_protocol.rs` (new)

**Implementation:**
```rust
/// Run multiple protocol servers concurrently
pub struct MultiProtocolServer {
    http_server: HttpServer,
    https_server: Option<HttpsServer>,
    jsonrpc_server: Option<JsonRpcServer>,
    tarpc_server: Option<TarpcServer>,
    negotiator: ProtocolNegotiator,
}

impl MultiProtocolServer {
    pub async fn start(orchestrator: Arc<SongbirdOrchestrator>) -> Result<Self> {
        let config = orchestrator.config();
        
        // Start all enabled protocols concurrently
        let mut handles = vec![];
        
        // 1. HTTP (always enabled)
        let http = HttpServer::new(orchestrator.clone(), config.http_port);
        handles.push(tokio::spawn(http.start()));
        
        // 2. HTTPS (if TLS enabled)
        let https = if config.tls_enabled {
            let server = HttpsServer::new(orchestrator.clone(), config.https_port);
            handles.push(tokio::spawn(server.start()));
            Some(server)
        } else {
            None
        };
        
        // 3. JSON-RPC (if enabled)
        let jsonrpc = if config.jsonrpc_enabled {
            let server = JsonRpcServer::new(orchestrator.clone()).await?;
            handles.push(tokio::spawn(server.start()));
            Some(server)
        } else {
            None
        };
        
        // 4. tarpc (if enabled)
        let tarpc = if config.tarpc_enabled {
            let server = TarpcServer::new(orchestrator.clone());
            handles.push(tokio::spawn(server.start()));
            Some(server)
        } else {
            None
        };
        
        info!("🚀 Multi-protocol server started:");
        info!("   HTTP:     {}", config.http_port);
        if https.is_some() { info!("   HTTPS:    {}", config.https_port); }
        if jsonrpc.is_some() { info!("   JSON-RPC: {}/jsonrpc", config.http_port); }
        if tarpc.is_some() { info!("   tarpc:    {}", config.tarpc_port); }
        
        Ok(Self {
            http_server: http,
            https_server: https,
            jsonrpc_server: jsonrpc,
            tarpc_server: tarpc,
            negotiator: ProtocolNegotiator::new(config),
        })
    }
}
```

**Configuration:**
```bash
# Environment variables
SONGBIRD_HTTP_PORT=8080
SONGBIRD_HTTPS_PORT=8443
SONGBIRD_JSONRPC_ENABLED=true
SONGBIRD_TARPC_ENABLED=true
SONGBIRD_TARPC_PORT=8081
```

**Success Criteria:**
- ✅ All protocols running concurrently
- ✅ Different ports for different protocols
- ✅ Shared orchestrator state
- ✅ Clean shutdown of all servers

---

### **Phase 3: BTSP Interface** (Week 1, 1-2 days)

#### Task 3.1: BTSP Mock/Interface
**File:** `crates/songbird-network-federation/src/btsp.rs` (new)

**Implementation:**
```rust
/// BearDog Secure Tunnel Protocol Interface
/// Mock implementation for testing, ready for real BearDog integration
pub trait BtspProvider: Send + Sync {
    /// Establish secure tunnel
    async fn establish_tunnel(&self, peer: &PeerInfo) -> Result<TunnelHandle>;
    
    /// Encrypt payload
    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>>;
    
    /// Decrypt payload
    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>>;
    
    /// Get tunnel status
    async fn tunnel_status(&self, handle: &TunnelHandle) -> Result<TunnelStatus>;
}

/// Local BTSP implementation (mock for testing)
pub struct LocalBtspProvider {
    key_manager: Arc<LocalKeyManager>,
    tunnels: Arc<RwLock<HashMap<String, Tunnel>>>,
}

impl BtspProvider for LocalBtspProvider {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> Result<TunnelHandle> {
        // Mock implementation using local crypto
        let tunnel_id = Uuid::new_v4().to_string();
        let shared_key = self.key_manager.generate_shared_key(peer).await?;
        
        let tunnel = Tunnel {
            id: tunnel_id.clone(),
            peer: peer.clone(),
            shared_key,
            established_at: Utc::now(),
        };
        
        self.tunnels.write().await.insert(tunnel_id.clone(), tunnel);
        
        Ok(TunnelHandle { id: tunnel_id })
    }
    
    async fn encrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>> {
        // Mock encryption using local crypto
        // TODO: Replace with BearDog integration
        let tunnel = self.tunnels.read().await
            .get(&context.tunnel_id)
            .ok_or(Error::TunnelNotFound)?
            .clone();
        
        // Simple XOR encryption for testing (NOT SECURE)
        let encrypted: Vec<u8> = data.iter()
            .zip(tunnel.shared_key.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();
        
        Ok(encrypted)
    }
    
    async fn decrypt(&self, data: &[u8], context: &SecurityContext) -> Result<Vec<u8>> {
        // Mock decryption (XOR is symmetric)
        self.encrypt(data, context).await
    }
}

/// Future: Real BearDog integration
pub struct BearDogBtspProvider {
    beardog_client: BearDogClient,
}

impl BtspProvider for BearDogBtspProvider {
    async fn establish_tunnel(&self, peer: &PeerInfo) -> Result<TunnelHandle> {
        // Call BearDog API for real genetic crypto
        self.beardog_client
            .create_tunnel(CreateTunnelRequest {
                peer_id: peer.id.clone(),
                genetic_auth: true,
                key_lineage: true,
            })
            .await
    }
    
    // ... real BearDog implementations
}
```

**Success Criteria:**
- ✅ BTSP interface defined
- ✅ Local mock for testing
- ✅ Ready for BearDog integration
- ✅ Tests pass with mock

---

### **Phase 4: Federation Testing** (Week 2, 3-5 days)

#### Task 4.1: Internet Federation Test
**File:** `tests/integration/internet_federation_test.rs` (new)

**Test Setup:**
```rust
#[tokio::test]
async fn test_internet_federation_https() {
    // Setup two towers on different networks
    let tower1 = setup_tower_with_tls("tower1.example.com", 8443).await;
    let tower2 = setup_tower_with_tls("tower2.example.com", 8443).await;
    
    // Register tower1 on tower2
    let result = tower2
        .register_peer(&tower1.public_info())
        .await
        .expect("Failed to register peer");
    
    // Test cross-tower discovery
    let services = tower2
        .discover_services("compute")
        .await
        .expect("Discovery failed");
    
    assert!(services.iter().any(|s| s.tower_id == tower1.id()));
}

#[tokio::test]
async fn test_protocol_escalation() {
    let tower1 = setup_multi_protocol_tower().await;
    let tower2 = setup_multi_protocol_tower().await;
    
    // Start with HTTPS
    let conn = tower1.connect(&tower2, Protocol::Https).await?;
    assert_eq!(conn.protocol(), Protocol::Https);
    
    // Negotiate upgrade to tarpc
    let conn = tower1.escalate_protocol(&conn, Protocol::Tarpc).await?;
    assert_eq!(conn.protocol(), Protocol::Tarpc);
    
    // Verify high-performance RPC
    let latency = measure_latency(&conn).await?;
    assert!(latency < Duration::from_micros(500)); // <0.5ms
}
```

#### Task 4.2: LAN Encrypted Federation
**File:** `tests/integration/lan_encrypted_federation_test.rs` (new)

```rust
#[tokio::test]
async fn test_lan_tls_federation() {
    // Even on LAN, use TLS for encryption
    let tower1 = setup_tower_with_tls("192.168.1.10", 8443).await;
    let tower2 = setup_tower_with_tls("192.168.1.11", 8443).await;
    
    // Test encrypted communication on LAN
    let result = tower1
        .send_encrypted_message(&tower2, "test payload")
        .await?;
    
    assert!(result.encrypted);
    assert_eq!(result.protocol, Protocol::Https);
}

#[tokio::test]
async fn test_all_protocols_lan() {
    let tower = setup_multi_protocol_tower().await;
    
    // Test all protocols work on LAN
    assert!(tower.protocol_available(Protocol::Http));
    assert!(tower.protocol_available(Protocol::Https));
    assert!(tower.protocol_available(Protocol::JsonRpc));
    assert!(tower.protocol_available(Protocol::Tarpc));
    assert!(tower.protocol_available(Protocol::Btsp)); // Mock
}
```

---

## 📋 Implementation Checklist

### Week 1: Core Protocols (5-7 days)

- [ ] **Day 1-2: JSON-RPC Implementation**
  - [ ] Create `crates/songbird-orchestrator/src/rpc/jsonrpc.rs`
  - [ ] Add `jsonrpsee` dependency
  - [ ] Implement JSON-RPC 2.0 server
  - [ ] Register core methods (discover, register, health)
  - [ ] Test with curl/Python client
  - [ ] Documentation

- [ ] **Day 3-5: tarpc Implementation**
  - [ ] Create `crates/songbird-orchestrator/src/rpc/tarpc.rs`
  - [ ] Add `tarpc` dependency
  - [ ] Define `SongbirdRpc` trait
  - [ ] Implement tarpc server
  - [ ] Implement tarpc client
  - [ ] Benchmark vs HTTP
  - [ ] Documentation

- [ ] **Day 6-7: Protocol Negotiation & Multi-Protocol Server**
  - [ ] Create `crates/songbird-orchestrator/src/rpc/negotiation.rs`
  - [ ] Create `crates/songbird-orchestrator/src/rpc/multi_protocol.rs`
  - [ ] Implement protocol negotiation
  - [ ] Implement concurrent server
  - [ ] Configuration system
  - [ ] Integration tests

### Week 2: BTSP & Testing (3-5 days)

- [ ] **Day 1-2: BTSP Interface**
  - [ ] Create `crates/songbird-network-federation/src/btsp.rs`
  - [ ] Define `BtspProvider` trait
  - [ ] Implement `LocalBtspProvider` (mock)
  - [ ] Stub `BearDogBtspProvider` for future
  - [ ] Unit tests with mock

- [ ] **Day 3-5: Federation Testing**
  - [ ] Internet federation tests
  - [ ] LAN encrypted federation tests
  - [ ] Protocol escalation tests
  - [ ] Multi-protocol concurrent tests
  - [ ] Performance benchmarks
  - [ ] Documentation

---

## 🎯 Success Criteria

### **Phase 1 Complete When:**
- ✅ JSON-RPC 2.0 endpoint operational
- ✅ tarpc server operational
- ✅ Protocol negotiation working
- ✅ All three protocols can run concurrently

### **Phase 2 Complete When:**
- ✅ Multi-protocol server working
- ✅ Can start all protocols with one command
- ✅ Configuration via environment variables
- ✅ Clean logs showing all active protocols

### **Phase 3 Complete When:**
- ✅ BTSP interface defined
- ✅ Local mock working for tests
- ✅ Ready for BearDog integration
- ✅ Tests pass with mock BTSP

### **Phase 4 Complete When:**
- ✅ Internet federation working with TLS
- ✅ LAN federation with encryption
- ✅ Protocol escalation demonstrated
- ✅ All tests passing

---

## 📊 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   Songbird Tower (Multi-Protocol)            │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │   HTTP   │  │  HTTPS   │  │ JSON-RPC │  │  tarpc   │   │
│  │  :8080   │  │  :8443   │  │  :8080   │  │  :8081   │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │             │             │           │
│       └─────────────┴─────────────┴─────────────┘           │
│                           ↓                                  │
│              ┌─────────────────────────┐                     │
│              │  Protocol Negotiator    │                     │
│              │  (Escalation Manager)   │                     │
│              └────────────┬────────────┘                     │
│                           ↓                                  │
│              ┌─────────────────────────┐                     │
│              │   Orchestrator Core     │                     │
│              │  (Shared State)         │                     │
│              └────────────┬────────────┘                     │
│                           ↓                                  │
│              ┌─────────────────────────┐                     │
│              │    Security Layer       │                     │
│              │  TLS + BTSP (future)    │                     │
│              └─────────────────────────┘                     │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                           ↓
        ┌──────────────────┴──────────────────┐
        ↓                                      ↓
  ┌───────────┐                          ┌───────────┐
  │  Tower 2  │  ←─── Internet/LAN ───→  │  Tower 3  │
  │  (HTTPS)  │      (Encrypted)         │  (tarpc)  │
  └───────────┘                          └───────────┘
```

---

## 🔐 Security Layers

```
Layer 4: Application  ┌──────────────────────┐
                      │  BTSP (BearDog)      │ ← Future
                      │  Genetic Crypto      │
                      └──────────────────────┘
                                ↓
Layer 3: Protocol     ┌──────────────────────┐
                      │  tarpc/JSON-RPC      │ ← Phase 1
                      │  (encrypted)         │
                      └──────────────────────┘
                                ↓
Layer 2: Transport    ┌──────────────────────┐
                      │  TLS 1.3             │ ← Active Today
                      │  (HTTPS)             │
                      └──────────────────────┘
                                ↓
Layer 1: Network      ┌──────────────────────┐
                      │  TCP/IP              │
                      │  IPv4 + IPv6         │
                      └──────────────────────┘
```

---

## 🌟 Emergent Properties

**"VPN-free encryption as emergent property of primal interactions"**

When complete, the system will have:

1. **Self-Healing Security**: TLS + BTSP + protocol negotiation
2. **Adaptive Performance**: Automatically use fastest protocol
3. **Sovereign Federation**: Each tower self-determines protocols
4. **Zero Configuration**: Protocols discovered and negotiated
5. **BearDog Ready**: Drop-in BTSP when BearDog integrated

---

## 📈 Timeline

```
Week 1:
├─ JSON-RPC (2 days)
├─ tarpc (3 days)
└─ Multi-protocol server (2 days)

Week 2:
├─ BTSP interface (2 days)
└─ Testing & validation (3 days)

Total: 10-12 days to complete
```

---

## 🚀 Getting Started

**Start with Phase 1, Task 1.1:**

```bash
# Add dependencies
cd crates/songbird-orchestrator
cargo add jsonrpsee --features server

# Create file structure
mkdir -p src/rpc
touch src/rpc/mod.rs
touch src/rpc/jsonrpc.rs

# Begin implementation
```

---

**Status**: 📋 **PLAN COMPLETE** - Ready to begin implementation  
**Priority**: P0 - Critical for BearDog integration readiness  
**Owner**: Songbird Protocol Team

**Let's build the internet-ready, multi-protocol, encrypted federation system!** 🚀

