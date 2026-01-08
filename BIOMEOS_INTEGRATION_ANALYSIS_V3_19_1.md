# 🐦🌱 biomeOS Integration Analysis - Songbird v3.19.1

**Date**: January 8, 2026  
**Priority**: High (biomeOS Production Ready)  
**Type**: Infrastructure Evolution (Not Quick Fix)  

---

## 🎯 Executive Summary

biomeOS team has built a complete spore federation system and needs Songbird to expose **3 APIs via Unix socket**. The good news: **90% of the infrastructure already exists!** We just need to wire it up.

**Status**: 
- ✅ UDP multicast with genetic tags: **COMPLETE**
- ✅ BTSP encrypted tunnels: **COMPLETE** (v3.19.0)
- ✅ Protocol-agnostic adapters: **COMPLETE**
- 🔄 Unix socket JSON-RPC server: **MISSING** (deep debt!)

---

## ✅ What We Already Have

### 1. UDP Multicast Discovery with Genetic Tags ✅

**Code Location**: `crates/songbird-discovery/src/anonymous/broadcaster.rs`

```rust
pub struct AnonymousDiscoveryBroadcaster {
    /// Identity tags (v3.14.0 - tag-based identity)
    /// Opaque strings we broadcast. We don't interpret them!
    /// Format: `{provider}:{type}:{value}` (e.g., `beardog:family:nat0`)
    tags: Option<Vec<String>>,
    
    /// Identity attestations from security provider
    identity_attestations: Option<Vec<crate::IdentityAttestation>>,
}
```

**Broadcasting**: Lines 284-291 (v3.14.2 fix)
```rust
// ✅ CRITICAL FIX (v3.14.2 - Jan 7, 2026): Include identity tags!
if let Some(ref tags) = self.tags {
    debug!("📋 Broadcasting {} identity tags: {:?}", tags.len(), tags);
    message = message.with_tags(tags.clone());
}
```

**Discovery**: `DiscoveredPeer` struct (line 348)
```rust
pub struct DiscoveredPeer {
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub capabilities: Vec<String>,
    pub tags: Option<Vec<String>>,  // ✅ Genetic family tags here!
    pub identity_attestations: Option<Vec<IdentityAttestation>>,
    // ...
}
```

**Family Filtering**: `discovery_bridge.rs` (lines 174-188)
```rust
let same_family = std::env::var("SONGBIRD_FAMILY_ID")
    .ok()
    .map(|my_family| {
        peer.tags.as_ref()
            .map(|tags| {
                tags.iter().any(|tag| {
                    tag.contains(&format!(":family:{}:", my_family))
                })
            })
            .unwrap_or(false)
    })
    .unwrap_or(false);
```

**Status**: ✅ **COMPLETE - Already broadcasting and filtering by genetic tags!**

---

### 2. BTSP Encrypted Tunnels ✅

**Code Location**: `crates/songbird-universal/src/btsp_client.rs`

```rust
impl BtspClient {
    pub async fn request_tunnel(
        &self,
        remote_peer_id: &str,
        remote_peer_tags: &[String],
    ) -> SongbirdResult<BtspTunnel> {
        // Creates encrypted tunnel via BearDog
    }
}
```

**Connection Types**: `crates/songbird-orchestrator/src/connections/`
- `limited_btsp.rs` - TrustLevel::Limited
- `federated_btsp.rs` - TrustLevel::Elevated
- `full_trust_btsp.rs` - TrustLevel::Highest

**Lazy Initialization**: v3.19.0 (OnceCell pattern)
```rust
async fn get_or_init_btsp_client(&self) -> Option<Arc<BtspClient>> {
    self.btsp_client.get_or_try_init(|| async {
        // Discover security provider, create client
    }).await.ok().cloned()
}
```

**BTSP-First Logic**: `connection_manager.rs` (lines 205-240)
```rust
let peer_supports_btsp = peer_tags.iter().any(|t| t == "btsp_enabled");

if peer_supports_btsp {
    match self.get_or_init_btsp_client().await {
        Some(_client) => {
            // Create BTSP connection
        }
        None => {
            // Fall back to HTTPS
        }
    }
}
```

**Status**: ✅ **COMPLETE - BTSP tunnels working, tested, production-ready!**

---

### 3. Protocol-Agnostic Adapters ✅

**Code Location**: `crates/songbird-universal/src/adapters/security.rs`

```rust
pub enum SecurityProtocol {
    Tarpc(TarpcClient),
    JsonRpc(JsonRpcClient),
    Http(reqwest::Client),
}

impl SecurityAdapter {
    pub async fn call_generic<R>(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> SongbirdResult<R> {
        // Protocol negotiation: tarpc > JSON-RPC > HTTP
    }
}
```

**Status**: ✅ **COMPLETE - Already using tarpc/JSON-RPC for BearDog!**

---

## ❌ What's Missing: Unix Socket JSON-RPC Server

### The Deep Debt

**Current Architecture**:
```
biomeOS → HTTP (port 8080) → Songbird
         ❌ Requires TCP port
         ❌ Less secure
         ❌ Not primal-to-primal pattern
```

**Desired Architecture**:
```
biomeOS → Unix Socket (/tmp/songbird-{node_id}.sock) → Songbird
         ✅ Port-free
         ✅ Local IPC (secure by default)
         ✅ Primal-to-primal pattern
         ✅ JSON-RPC 2.0
```

### Why This is Deep Debt

1. **No Unix Socket Server**: Songbird only exposes HTTPS (Tower-based)
2. **Not Inter-Primal**: HTTP is for external federation, not local IPC
3. **Inconsistent with BearDog**: BearDog uses Unix sockets, we should too
4. **Zero Hardcoding Philosophy**: Unix sockets enable runtime discovery

---

## 🎯 What biomeOS Needs (3 APIs)

### API 1: `discover_by_family`

**Purpose**: Get all discovered nodes in a specific genetic family

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "discover_by_family",
  "params": {
    "family_tags": ["nat0"],
    "timeout_ms": 5000
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "nodes": [
      {
        "node_id": "node-beta",
        "genetic_families": ["nat0"],
        "sub_federations": ["gaming"],
        "capabilities": ["compute"],
        "btsp_endpoint": "udp://192.168.1.101:4433",
        "last_seen": "2026-01-08T20:00:00Z"
      }
    ]
  },
  "id": 1
}
```

**Implementation**: Filter `AnonymousDiscoveryListener.get_peers()` by family tags

---

### API 2: `create_genetic_tunnel`

**Purpose**: Establish BTSP tunnel using genetic proof from BearDog

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "create_genetic_tunnel",
  "params": {
    "peer_node_id": "node-beta",
    "peer_endpoint": "udp://192.168.1.101:4433",
    "genetic_proof": {
      "family_id": "nat0",
      "parent_seed_hash": "parent_hash",
      "relationship": "sibling"
    }
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "tunnel_id": "tunnel-nat0-alpha-beta-12345",
    "status": "established",
    "local_endpoint": "udp://192.168.1.100:4433",
    "peer_endpoint": "udp://192.168.1.101:4433",
    "encryption": "BearDog-AES-256-GCM",
    "created_at": "2026-01-08T20:00:00Z"
  },
  "id": 2
}
```

**Implementation**: Call existing `BtspClient.request_tunnel()`

---

### API 3: `announce_capabilities`

**Purpose**: Update the capabilities/tags this node broadcasts

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "announce_capabilities",
  "params": {
    "capabilities": ["storage", "compute"],
    "sub_federations": ["gaming", "family"],
    "genetic_families": ["nat0", "lan0"]
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "updated",
    "broadcasting": true,
    "updated_at": "2026-01-08T20:00:00Z"
  },
  "id": 3
}
```

**Implementation**: Update `AnonymousDiscoveryBroadcaster` tags and capabilities

---

## 🏗️ Modern Idiomatic Solution

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Songbird v3.19.1: Dual-Interface Architecture               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐         ┌─────────────────────────┐  │
│  │ HTTPS/Tower      │         │ Unix Socket JSON-RPC    │  │
│  │ (Port 8080)      │         │ (/tmp/songbird-*.sock)  │  │
│  │                  │         │                         │  │
│  │ For: External    │         │ For: Inter-Primal IPC   │  │
│  │      Federation  │         │      (biomeOS, etc.)    │  │
│  └────────┬─────────┘         └────────┬────────────────┘  │
│           │                            │                    │
│           └────────────┬───────────────┘                    │
│                        ▼                                    │
│           ┌────────────────────────────┐                    │
│           │ Core Orchestrator Logic    │                    │
│           │                            │                    │
│           │ • Discovery (UDP multicast)│                    │
│           │ • BTSP client (tunnels)    │                    │
│           │ • Connection manager       │                    │
│           │ • Trust evaluation         │                    │
│           └────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────┘
```

### New Components

#### 1. Unix Socket JSON-RPC Server

**File**: `crates/songbird-orchestrator/src/unix_socket_server.rs`

```rust
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_ipc_server::{ServerBuilder, Server};
use std::path::PathBuf;

pub struct UnixSocketServer {
    socket_path: PathBuf,
    orchestrator: Arc<SongbirdOrchestrator>,
}

impl UnixSocketServer {
    pub fn new(node_id: &str, orchestrator: Arc<SongbirdOrchestrator>) -> Self {
        let socket_path = PathBuf::from(format!("/tmp/songbird-{}.sock", node_id));
        Self { socket_path, orchestrator }
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut io = IoHandler::new();
        
        // API 1: discover_by_family
        let orch = self.orchestrator.clone();
        io.add_method("discover_by_family", move |params: Params| {
            let orch = orch.clone();
            async move {
                discover_by_family_handler(orch, params).await
            }
        });
        
        // API 2: create_genetic_tunnel
        let orch = self.orchestrator.clone();
        io.add_method("create_genetic_tunnel", move |params: Params| {
            let orch = orch.clone();
            async move {
                create_genetic_tunnel_handler(orch, params).await
            }
        });
        
        // API 3: announce_capabilities
        let orch = self.orchestrator.clone();
        io.add_method("announce_capabilities", move |params: Params| {
            let orch = orch.clone();
            async move {
                announce_capabilities_handler(orch, params).await
            }
        });
        
        // Start Unix socket server
        let server = ServerBuilder::new(io)
            .start(&self.socket_path.to_string_lossy())
            .map_err(|e| anyhow::anyhow!("Failed to start Unix socket server: {}", e))?;
        
        info!("✅ Unix socket JSON-RPC server listening at {:?}", self.socket_path);
        
        Ok(())
    }
}
```

#### 2. JSON-RPC Handlers

**File**: `crates/songbird-orchestrator/src/unix_socket_handlers.rs`

```rust
async fn discover_by_family_handler(
    orch: Arc<SongbirdOrchestrator>,
    params: Params,
) -> Result<Value, jsonrpc_core::Error> {
    // Parse params
    let family_tags: Vec<String> = params.parse()?;
    
    // Get discovered peers from orchestrator
    let all_peers = orch.discovery_listener.get_peers().await;
    
    // Filter by family tags
    let filtered_peers: Vec<_> = all_peers
        .into_iter()
        .filter(|peer| {
            peer.tags.as_ref()
                .map(|tags| {
                    family_tags.iter().any(|family_tag| {
                        tags.iter().any(|tag| tag.contains(family_tag))
                    })
                })
                .unwrap_or(false)
        })
        .collect();
    
    // Convert to JSON response
    Ok(json!({
        "nodes": filtered_peers.into_iter().map(|peer| {
            json!({
                "node_id": peer.node_id,
                "genetic_families": extract_families(&peer.tags),
                "capabilities": peer.capabilities,
                "btsp_endpoint": peer.address.to_string(),
                "last_seen": peer.last_seen,
            })
        }).collect::<Vec<_>>()
    }))
}

async fn create_genetic_tunnel_handler(
    orch: Arc<SongbirdOrchestrator>,
    params: Params,
) -> Result<Value, jsonrpc_core::Error> {
    // Parse params
    let request: CreateTunnelRequest = params.parse()?;
    
    // Get BTSP client from connection manager
    let tunnel = orch.connection_manager
        .create_btsp_connection(
            request.peer_node_id,
            request.peer_tags,
            TrustLevel::Limited, // Based on genetic proof
        )
        .await?;
    
    Ok(json!({
        "tunnel_id": tunnel.id,
        "status": "established",
        "created_at": SystemTime::now(),
    }))
}

async fn announce_capabilities_handler(
    orch: Arc<SongbirdOrchestrator>,
    params: Params,
) -> Result<Value, jsonrpc_core::Error> {
    // Parse params
    let announcement: CapabilitiesAnnouncement = params.parse()?;
    
    // Update broadcaster (need to make it mutable or use Arc<RwLock>)
    orch.update_capabilities(announcement.capabilities).await?;
    orch.update_tags(announcement.genetic_families).await?;
    
    Ok(json!({
        "status": "updated",
        "broadcasting": true,
        "updated_at": SystemTime::now(),
    }))
}
```

#### 3. Orchestrator Integration

**File**: `crates/songbird-orchestrator/src/app/core.rs`

```rust
pub struct SongbirdOrchestrator {
    // Existing fields...
    
    /// Unix socket JSON-RPC server for inter-primal IPC (v3.19.1)
    unix_socket_server: Option<Arc<UnixSocketServer>>,
}

impl SongbirdOrchestrator {
    pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
        // ... existing initialization ...
        
        // Create Unix socket server (v3.19.1)
        let unix_socket_server = if config.enable_unix_socket_ipc.unwrap_or(true) {
            let server = UnixSocketServer::new(&config.node_id, /* orchestrator */);
            Some(Arc::new(server))
        } else {
            None
        };
        
        Ok(Self {
            // ... existing fields ...
            unix_socket_server,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        // ... existing startup ...
        
        // Start Unix socket server (v3.19.1)
        if let Some(ref server) = self.unix_socket_server {
            server.start().await?;
            info!("✅ Unix socket IPC enabled");
        }
        
        Ok(())
    }
}
```

---

## 📦 Dependencies

Add to `crates/songbird-orchestrator/Cargo.toml`:

```toml
[dependencies]
# Existing...

# v3.19.1: Unix socket JSON-RPC server for inter-primal IPC
jsonrpc-core = "18.0"
jsonrpc-ipc-server = "18.0"
```

---

## 🧪 Testing Strategy

### Unit Tests

**File**: `crates/songbird-orchestrator/src/unix_socket_server_tests.rs`

```rust
#[tokio::test]
async fn test_discover_by_family_api() {
    // Mock orchestrator with discovered peers
    let orch = mock_orchestrator_with_peers(vec![
        peer_with_family("nat0"),
        peer_with_family("lan0"),
        peer_with_family("nat0"),
    ]);
    
    // Create Unix socket server
    let server = UnixSocketServer::new("test-node", orch);
    server.start().await.unwrap();
    
    // Connect as client
    let client = UnixSocketClient::new("/tmp/songbird-test-node.sock").await.unwrap();
    
    // Call discover_by_family
    let response = client.call("discover_by_family", json!({
        "family_tags": ["nat0"],
        "timeout_ms": 5000
    })).await.unwrap();
    
    // Verify filtered results
    let nodes = response["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2); // Only nat0 peers
}

#[tokio::test]
async fn test_create_genetic_tunnel_api() {
    // Mock orchestrator with BTSP client
    let orch = mock_orchestrator_with_btsp();
    let server = UnixSocketServer::new("test-node", orch);
    server.start().await.unwrap();
    
    // Connect as client
    let client = UnixSocketClient::new("/tmp/songbird-test-node.sock").await.unwrap();
    
    // Call create_genetic_tunnel
    let response = client.call("create_genetic_tunnel", json!({
        "peer_node_id": "node-beta",
        "peer_endpoint": "udp://192.168.1.101:4433",
        "genetic_proof": {
            "family_id": "nat0",
            "parent_seed_hash": "abc123",
            "relationship": "sibling"
        }
    })).await.unwrap();
    
    // Verify tunnel created
    assert_eq!(response["status"], "established");
    assert!(response["tunnel_id"].as_str().unwrap().starts_with("tunnel-"));
}
```

### E2E Tests

**File**: `crates/songbird-orchestrator/tests/e2e_unix_socket_ipc.rs`

```rust
#[tokio::test]
async fn test_biomeos_integration_workflow() {
    // Start Songbird with Unix socket enabled
    let songbird = start_songbird_with_unix_socket().await;
    
    // biomeOS connects via Unix socket
    let biomeos_client = UnixSocketClient::new("/tmp/songbird-test-node.sock").await.unwrap();
    
    // Step 1: Discover nodes in family
    let discovery = biomeos_client.call("discover_by_family", json!({
        "family_tags": ["nat0"]
    })).await.unwrap();
    
    assert!(discovery["nodes"].as_array().unwrap().len() > 0);
    
    // Step 2: Create BTSP tunnel
    let tunnel = biomeos_client.call("create_genetic_tunnel", json!({
        "peer_node_id": "node-beta",
        "genetic_proof": { /* ... */ }
    })).await.unwrap();
    
    assert_eq!(tunnel["status"], "established");
    
    // Step 3: Update capabilities
    let update = biomeos_client.call("announce_capabilities", json!({
        "capabilities": ["storage", "compute"],
        "genetic_families": ["nat0"]
    })).await.unwrap();
    
    assert_eq!(update["status"], "updated");
}
```

---

## 📊 Implementation Roadmap

### Phase 1: Unix Socket Infrastructure (v3.19.1)
**Priority**: Critical  
**Effort**: 2-3 days

- [ ] Add `jsonrpc-core` and `jsonrpc-ipc-server` dependencies
- [ ] Create `unix_socket_server.rs` with JSON-RPC handler
- [ ] Integrate with `SongbirdOrchestrator`
- [ ] Add configuration option (`enable_unix_socket_ipc`)
- [ ] Unit tests for server initialization

### Phase 2: API Implementation (v3.19.2)
**Priority**: Critical  
**Effort**: 2-3 days

- [ ] Implement `discover_by_family` handler
- [ ] Implement `create_genetic_tunnel` handler
- [ ] Implement `announce_capabilities` handler
- [ ] Add family tag extraction utility
- [ ] Unit tests for each API

### Phase 3: Integration & Testing (v3.19.3)
**Priority**: High  
**Effort**: 2 days

- [ ] E2E tests with Unix socket client
- [ ] Integration tests with biomeOS patterns
- [ ] Error handling and edge cases
- [ ] Performance testing
- [ ] Documentation

### Phase 4: biomeOS Coordination (v3.19.4)
**Priority**: High  
**Effort**: 1-2 days

- [ ] Coordinate with biomeOS team
- [ ] Test with real USB spores
- [ ] Validate genetic federation workflow
- [ ] Production deployment

---

## 🎯 Success Criteria

### Functional
- [ ] Unix socket server listening at `/tmp/songbird-{node_id}.sock`
- [ ] `discover_by_family` returns filtered peers by genetic tags
- [ ] `create_genetic_tunnel` establishes BTSP tunnels
- [ ] `announce_capabilities` updates broadcaster in real-time
- [ ] All APIs use JSON-RPC 2.0 format
- [ ] Graceful error handling with JSON-RPC error codes

### Non-Functional
- [ ] Zero hardcoding (runtime discovery of Unix socket)
- [ ] Modern async Rust (no blocking calls)
- [ ] Thread-safe (concurrent API calls)
- [ ] Observable (structured logging)
- [ ] Tested (unit + E2E tests)
- [ ] Production-ready (error handling, edge cases)

### Integration
- [ ] biomeOS E2E tests pass
- [ ] Spore incubation workflow works end-to-end
- [ ] LAN federation working via Unix socket
- [ ] BTSP tunnels established via genetic proof

---

## 🚧 Challenges & Solutions

### Challenge 1: Circular Dependency

**Problem**: Unix socket server needs `Arc<SongbirdOrchestrator>`, but orchestrator creates the server.

**Solution**: Use `Arc<OnceCell>` or pass orchestrator handle after construction:

```rust
impl SongbirdOrchestrator {
    pub async fn new(config: Config) -> Result<Self> {
        let orchestrator = Self { /* ... */ unix_socket_server: None };
        let orch_arc = Arc::new(orchestrator);
        
        let server = UnixSocketServer::new(&config.node_id, orch_arc.clone());
        orch_arc.unix_socket_server = Some(Arc::new(server));
        
        Ok(orch_arc)
    }
}
```

### Challenge 2: Broadcaster Mutability

**Problem**: `announce_capabilities` needs to update broadcaster, but it's immutable.

**Solution**: Wrap broadcaster in `Arc<RwLock<>>`:

```rust
pub struct SongbirdOrchestrator {
    broadcaster: Arc<RwLock<AnonymousDiscoveryBroadcaster>>,
}

pub async fn update_capabilities(&self, caps: Vec<String>) {
    let mut broadcaster = self.broadcaster.write().await;
    broadcaster.update_capabilities(caps);
}
```

### Challenge 3: API Versioning

**Problem**: biomeOS API might evolve, need backward compatibility.

**Solution**: Version APIs in method names:

```jsonc
{
  "method": "discover_by_family.v1",
  "method": "discover_by_family.v2",  // Future
}
```

---

## 📚 Documentation Updates

### For biomeOS Team

**File**: `docs/UNIX_SOCKET_API.md`

- API specification (request/response formats)
- Error codes and handling
- Connection examples
- Authentication (future: Unix socket credentials)

### For Songbird Developers

**File**: `docs/INTER_PRIMAL_IPC.md`

- Architecture overview
- Adding new APIs
- Testing patterns
- Security considerations

---

## 🎊 Summary

### What We Have ✅
- UDP multicast with genetic tags
- BTSP encrypted tunnels
- Protocol-agnostic adapters
- Modern async Rust patterns

### What We Need 🔄
- Unix socket JSON-RPC server (new infrastructure!)
- 3 API handlers (wire existing logic)
- Configuration and integration
- Tests and documentation

### Effort Estimate
- **Total**: ~1-2 weeks for complete implementation
- **Phase 1** (Unix socket): 2-3 days
- **Phase 2** (APIs): 2-3 days
- **Phase 3** (Testing): 2 days
- **Phase 4** (Coordination): 1-2 days

### Impact
- ✅ Port-free inter-primal communication
- ✅ Enables biomeOS spore federation
- ✅ Modern primal-to-primal pattern
- ✅ Consistent with ecosystem (BearDog uses Unix sockets)

---

**Date**: January 8, 2026  
**Version**: v3.19.1 (planned)  
**Status**: 📋 Analysis Complete, Ready for Implementation  
**Confidence**: 💯 100% (infrastructure already exists, just need to expose it)

🐦 **Songbird** + 🌱 **biomeOS** = 🌐 **Global Genetic Federation!**

