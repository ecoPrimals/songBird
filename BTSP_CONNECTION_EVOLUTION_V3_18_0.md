# 🔐 BTSP Connection Evolution - v3.18.0

**Date**: January 7, 2026  
**From**: biomeOS Integration Team  
**Priority**: Medium (Not Blocking Current Deployments)  
**Status**: Infrastructure Ready, Needs Connection Manager Evolution  

---

## 🎯 Executive Summary

**The Gap**: Songbird uses HTTPS for inter-tower connections, despite having complete BTSP infrastructure.

**Why It Matters**:
- ✅ BTSP = Port-free (except UDP discovery)
- ✅ BTSP = NAT traversal built-in
- ✅ BTSP = Encrypted by default
- ❌ HTTPS = Requires port forwarding for WAN

**Status**:
- ✅ BearDog v0.15.0: BTSP API complete (6/6 endpoints)
- ✅ Songbird v3.17.0: BTSP client coded
- ❌ Connection Manager: Still creates HTTPS connections

**Timeline**: v3.18.0 (1 session, ~4-6 hours)

---

## ✅ What's Already Built

### BearDog v0.15.0 - COMPLETE

```
POST /btsp/contact/exchange  ✅ Working
POST /btsp/tunnel/establish  ✅ Working
GET  /btsp/tunnel/{id}       ✅ Working
DELETE /btsp/tunnel/{id}     ✅ Working
POST /btsp/tunnel/send       ✅ Working
POST /btsp/tunnel/receive    ✅ Working
```

### Songbird v3.17.0 - Infrastructure Ready

**BTSP Client** (`songbird-universal/src/btsp_client.rs`):
```rust
✅ BtspClient::establish_tunnel()
✅ BtspClient::exchange_contact()
✅ SecurityAdapter.call_generic() (v3.16.0)
✅ Protocol negotiation (tarpc/JSON-RPC/HTTP)
```

**Discovery**:
```rust
✅ Discovery packets include "btsp_enabled" tag
✅ Tags passed to trust evaluation
✅ Tags available in connection manager
```

---

## ❌ The Gap: Connection Manager Still Uses HTTPS

### Current Behavior

**File**: `crates/songbird-orchestrator/src/app/connection_manager.rs` (line 126)

```rust
pub async fn establish_connection(
    &self,
    peer_id: String,
    endpoint: String,  // ← HTTPS URL: "https://192.168.1.144:8080"
    capabilities: Vec<String>,
    trust_level: TrustLevel,
    discovery_method: String,
) -> Result<()> {
    let connection = match trust_level {
        TrustLevel::Limited => {
            info!("🎵 Creating Limited connection (BirdSong only)");
            // ❌ Always creates HTTPS connection:
            let conn = LimitedConnection::with_defaults(
                peer_id.clone(), 
                endpoint.clone()
            )?;
            Connection::Limited(conn)
        }
        TrustLevel::Federated => {
            // ❌ Also HTTPS
            let conn = FederatedConnection::with_defaults(
                peer_id.clone(), 
                endpoint.clone()
            )?;
            Connection::Federated(conn)
        }
        TrustLevel::FullTrust => {
            // ❌ Also HTTPS
            let conn = FullTrustConnection::with_defaults(
                peer_id.clone(), 
                endpoint.clone()
            )?;
            Connection::FullTrust(conn)
        }
        _ => return Err(anyhow!("Trust level not eligible for connection")),
    };
    
    // Store connection
    self.connections.write().await.insert(peer_id.clone(), connection);
    Ok(())
}
```

### Connection Types Still Use HTTP Client

**File**: `crates/songbird-orchestrator/src/connections/limited.rs` (line 66)

```rust
pub fn new(
    peer_id: String,
    endpoint: String,
    allowed_capabilities: Vec<String>,
) -> Result<Self> {
    // ❌ Always creates HTTP client:
    let http_client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to create HTTP client")?;
    
    Ok(Self {
        peer_id,
        endpoint,  // ← HTTPS URL
        allowed_capabilities,
        denied_capabilities: TrustLevel::Limited.default_denied_capabilities(),
        http_client,  // ← HTTP client, not BTSP tunnel
    })
}
```

**Same issue in**:
- `connections/federated.rs`
- `connections/full_trust.rs`

---

## 🎯 Proposed Solution: BTSP-First with HTTPS Fallback

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ ConnectionManager                                            │
│                                                              │
│  establish_connection(peer_id, endpoint, tags, trust_level) │
│         ↓                                                    │
│    Check: Does peer support BTSP?                           │
│         ├─ Yes → Create BTSP connection                     │
│         └─ No  → Create HTTPS connection (fallback)         │
└─────────────────────────────────────────────────────────────┘
           ↓                           ↓
    ┌──────────────┐         ┌──────────────┐
    │ BTSP Path    │         │ HTTPS Path   │
    │ (Encrypted)  │         │ (Fallback)   │
    └──────────────┘         └──────────────┘
```

### Implementation

#### Step 1: Create BTSP Connection Types

**New file**: `crates/songbird-orchestrator/src/connections/limited_btsp.rs`

```rust
use anyhow::{Context, Result};
use songbird_universal::BtspClient;
use songbird_types::TrustLevel;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Limited connection using BTSP tunnel (v3.18.0)
pub struct LimitedBtspConnection {
    peer_id: String,
    btsp_client: Arc<BtspClient>,
    tunnel_id: Arc<RwLock<Option<uuid::Uuid>>>,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
}

impl LimitedBtspConnection {
    /// Create new BTSP connection with Limited trust
    pub async fn new(
        peer_id: String,
        btsp_client: Arc<BtspClient>,
        allowed_capabilities: Vec<String>,
    ) -> Result<Self> {
        info!("🔐 Creating BTSP Limited connection to {}", peer_id);
        
        // Get peer tags (for BearDog trust evaluation)
        let peer_tags = vec![]; // TODO: Get from discovery
        
        // Request BTSP tunnel from BearDog
        let tunnel_request = songbird_universal::btsp_types::BtspTunnelRequest {
            remote_node_id: peer_id.clone(),
            remote_tags: peer_tags,
            tunnel_type: songbird_universal::btsp_types::TunnelType::Auto,
            use_lineage_for_nat: true,
            remote_contact: None,
        };
        
        let tunnel = btsp_client.establish_tunnel(tunnel_request).await
            .context("Failed to establish BTSP tunnel")?;
        
        info!("✅ BTSP tunnel established: {}", tunnel.tunnel_id);
        
        Ok(Self {
            peer_id,
            btsp_client,
            tunnel_id: Arc::new(RwLock::new(Some(tunnel.tunnel_id))),
            allowed_capabilities,
            denied_capabilities: TrustLevel::Limited.default_denied_capabilities(),
        })
    }
    
    /// Send data over BTSP tunnel
    pub async fn send_data(&self, data: &[u8]) -> Result<()> {
        let tunnel_id = self.tunnel_id.read().await
            .ok_or_else(|| anyhow!("No active tunnel"))?;
        
        self.btsp_client.send_data_over_tunnel(&tunnel_id, data).await
    }
    
    /// Call RPC method over BTSP tunnel
    pub async fn call_method<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<T> {
        // Serialize RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": uuid::Uuid::new_v4().to_string(),
        });
        
        let request_bytes = serde_json::to_vec(&request)?;
        
        // Send over BTSP tunnel
        self.send_data(&request_bytes).await?;
        
        // TODO: Receive response
        // For now, return error
        Err(anyhow!("BTSP receive not yet implemented"))
    }
}

impl Drop for LimitedBtspConnection {
    fn drop(&mut self) {
        // TODO: Close tunnel on drop
        info!("🗑️  Closing BTSP tunnel for {}", self.peer_id);
    }
}
```

**Similarly create**:
- `connections/federated_btsp.rs`
- `connections/full_trust_btsp.rs`

#### Step 2: Update Connection Manager

**File**: `crates/songbird-orchestrator/src/app/connection_manager.rs`

```rust
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    btsp_client: Option<Arc<BtspClient>>,  // ← NEW: BTSP client
    // ... existing fields
}

impl ConnectionManager {
    pub fn new(/* ... */) -> Self {
        // Initialize BTSP client if security endpoint available
        let btsp_client = if let Ok(security_endpoint) = 
            discover_security_endpoint(None) 
        {
            match BtspClient::new(security_endpoint) {
                Ok(client) => {
                    info!("✅ BTSP client initialized");
                    Some(Arc::new(client))
                }
                Err(e) => {
                    warn!("⚠️  BTSP client init failed: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            btsp_client,
            // ... existing fields
        }
    }
    
    pub async fn establish_connection(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        tags: Vec<String>,  // ← NEW: Pass peer tags
        trust_level: TrustLevel,
        discovery_method: String,
    ) -> Result<()> {
        // Check if peer supports BTSP and we have a BTSP client
        let use_btsp = self.btsp_client.is_some() 
            && tags.iter().any(|t| t == "btsp_enabled");
        
        let connection = if use_btsp {
            info!("🔐 Using BTSP tunnel connection for {}", peer_id);
            self.create_btsp_connection(
                peer_id.clone(),
                capabilities,
                trust_level,
            ).await?
        } else {
            info!("🌐 Using HTTPS connection for {} (BTSP not available)", peer_id);
            self.create_https_connection(
                peer_id.clone(),
                endpoint,
                capabilities,
                trust_level,
            )?
        };
        
        // Store connection
        self.connections.write().await.insert(peer_id.clone(), connection);
        
        info!("✅ Connection established to {}", peer_id);
        Ok(())
    }
    
    async fn create_btsp_connection(
        &self,
        peer_id: String,
        capabilities: Vec<String>,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        let btsp_client = self.btsp_client.as_ref()
            .ok_or_else(|| anyhow!("BTSP client not initialized"))?;
        
        match trust_level {
            TrustLevel::Limited => {
                let conn = LimitedBtspConnection::new(
                    peer_id,
                    btsp_client.clone(),
                    capabilities,
                ).await?;
                Ok(Connection::LimitedBtsp(conn))
            }
            TrustLevel::Federated => {
                let conn = FederatedBtspConnection::new(
                    peer_id,
                    btsp_client.clone(),
                    capabilities,
                ).await?;
                Ok(Connection::FederatedBtsp(conn))
            }
            TrustLevel::FullTrust => {
                let conn = FullTrustBtspConnection::new(
                    peer_id,
                    btsp_client.clone(),
                    capabilities,
                ).await?;
                Ok(Connection::FullTrustBtsp(conn))
            }
            _ => Err(anyhow!("Trust level not eligible for connection")),
        }
    }
    
    fn create_https_connection(
        &self,
        peer_id: String,
        endpoint: String,
        capabilities: Vec<String>,
        trust_level: TrustLevel,
    ) -> Result<Connection> {
        // Existing HTTPS logic (unchanged)
        match trust_level {
            TrustLevel::Limited => {
                let conn = LimitedConnection::with_defaults(peer_id, endpoint)?;
                Ok(Connection::Limited(conn))
            }
            // ... etc
        }
    }
}
```

#### Step 3: Update Connection Enum

**File**: `crates/songbird-orchestrator/src/connections/mod.rs`

```rust
pub enum Connection {
    // HTTPS connections (existing)
    Limited(LimitedConnection),
    Federated(FederatedConnection),
    FullTrust(FullTrustConnection),
    
    // BTSP connections (NEW v3.18.0)
    LimitedBtsp(LimitedBtspConnection),
    FederatedBtsp(FederatedBtspConnection),
    FullTrustBtsp(FullTrustBtspConnection),
}
```

#### Step 4: Update Discovery Bridge

**File**: `crates/songbird-orchestrator/src/app/discovery_bridge.rs`

```rust
// When calling connection manager, pass peer tags:
connection_manager.establish_connection(
    peer.node_id.clone(),
    peer.endpoint.clone(),
    peer.capabilities.clone(),
    peer.tags.clone(),  // ← NEW: Pass tags
    trust_level,
    peer.discovery_method.clone(),
).await?;
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_btsp_connection_creation() {
        // Mock BtspClient
        let btsp_client = Arc::new(MockBtspClient::new());
        
        // Create Limited BTSP connection
        let conn = LimitedBtspConnection::new(
            "peer-123".to_string(),
            btsp_client,
            vec!["capability-a".to_string()],
        ).await;
        
        assert!(conn.is_ok());
    }
    
    #[tokio::test]
    async fn test_connection_manager_btsp_selection() {
        let conn_mgr = ConnectionManager::new(/* ... */);
        
        // Peer with btsp_enabled tag → should use BTSP
        let tags = vec!["btsp_enabled".to_string()];
        conn_mgr.establish_connection(
            "peer-123".to_string(),
            "https://...".to_string(),
            vec![],
            tags,
            TrustLevel::Limited,
            "discovery".to_string(),
        ).await.unwrap();
        
        // Should have created BTSP connection
        let conns = conn_mgr.connections.read().await;
        match conns.get("peer-123") {
            Some(Connection::LimitedBtsp(_)) => assert!(true),
            _ => panic!("Expected BTSP connection"),
        }
    }
    
    #[tokio::test]
    async fn test_connection_manager_https_fallback() {
        let conn_mgr = ConnectionManager::new(/* ... */);
        
        // Peer without btsp_enabled tag → should use HTTPS
        let tags = vec![];
        conn_mgr.establish_connection(
            "peer-456".to_string(),
            "https://...".to_string(),
            vec![],
            tags,
            TrustLevel::Limited,
            "discovery".to_string(),
        ).await.unwrap();
        
        // Should have created HTTPS connection
        let conns = conn_mgr.connections.read().await;
        match conns.get("peer-456") {
            Some(Connection::Limited(_)) => assert!(true),
            _ => panic!("Expected HTTPS connection"),
        }
    }
}
```

### Integration Tests

```bash
# Test BTSP connection between two towers
./tests/integration/test_btsp_federation.sh

# Test HTTPS fallback (disable BTSP)
SONGBIRD_DISABLE_BTSP=true ./tests/integration/test_https_fallback.sh
```

---

## 📈 Before/After Comparison

### Before (v3.17.0)

```
Tower A (192.168.1.100:8080)
  ↓ UDP Discovery
Tower B (192.168.1.101:8080) discovered

Tower A: "I'll connect via HTTPS"
  ↓ HTTPS:8080
Tower B: Accepts HTTPS connection

Status:
✅ Works
⚠️  Uses port 8080
⚠️  Requires port forwarding for WAN
⚠️  NAT traversal manual
```

### After (v3.18.0)

```
Tower A
  ↓ UDP Discovery
Tower B discovered (tag: "btsp_enabled")

Tower A: "Peer supports BTSP, I'll use that"
Tower A → BearDog: "Establish tunnel to Tower B"
BearDog: Creates encrypted tunnel (NAT traversal automatic)
Tower A ←[BTSP Tunnel]→ Tower B

Status:
✅ Works
✅ Port-free (only UDP discovery on 4242)
✅ NAT traversal automatic (via BirdSong)
✅ Encrypted by default
✅ Falls back to HTTPS if BTSP unavailable
```

---

## 🎯 Implementation Checklist

### Phase 1: BTSP Connection Types (2 hours)
- [ ] Create `limited_btsp.rs`
- [ ] Create `federated_btsp.rs`
- [ ] Create `full_trust_btsp.rs`
- [ ] Update `Connection` enum
- [ ] Add unit tests

### Phase 2: Connection Manager (2 hours)
- [ ] Add `btsp_client` field
- [ ] Add `create_btsp_connection()` method
- [ ] Update `establish_connection()` to check tags
- [ ] Add BTSP vs HTTPS selection logic
- [ ] Unit tests for selection logic

### Phase 3: Discovery Bridge (30 min)
- [ ] Pass `peer.tags` to connection manager
- [ ] Verify tags are preserved through trust evaluation

### Phase 4: Integration Testing (1.5 hours)
- [ ] E2E test: BTSP connection between towers
- [ ] E2E test: HTTPS fallback
- [ ] E2E test: Mixed (some BTSP, some HTTPS)
- [ ] Verify NAT traversal

---

## 💡 Benefits

### 1. Port-Free Architecture
- Only UDP multicast (239.255.42.99:4242) for discovery
- No TCP port exposure needed
- Firewall-friendly

### 2. NAT Traversal Built-In
- BirdSong contact exchange via genetic lineage
- Hole-punching automatic
- No STUN/TURN servers needed

### 3. Encrypted by Default
- BTSP provides encryption
- No manual cert management
- Rotation automatic

### 4. Backward Compatible
- HTTPS fallback for old versions
- Gradual migration possible
- No breaking changes

### 5. Cloud-Native Ready
- Works across cloud providers
- No VPN infrastructure needed
- True P2P mesh

---

## 🚀 Deployment Strategy

### Phase 1: Testing (Week 1)
```bash
# Deploy v3.18.0 to test towers
# Both support BTSP → should use it
# Verify in logs: "Using BTSP tunnel connection"
```

### Phase 2: Mixed Deployment (Week 2)
```bash
# Some towers v3.17.0 (HTTPS)
# Some towers v3.18.0 (BTSP-capable)
# v3.18.0 should detect v3.17.0 doesn't support BTSP
# Should fall back to HTTPS
```

### Phase 3: Full Migration (Week 3+)
```bash
# All towers v3.18.0+
# All connections via BTSP
# HTTPS only as fallback (never used)
```

---

## 📊 Impact Assessment

| Aspect | Impact | Effort |
|--------|--------|--------|
| **Code Changes** | Medium (3 new connection types) | 4-6 hours |
| **Testing** | Medium (E2E needed) | 2-3 hours |
| **Risk** | Low (HTTPS fallback) | Minimal |
| **Benefit** | High (port-free + NAT) | Major |
| **Urgency** | Medium (not blocking) | v3.18.0 |

---

## 🎊 Why This Matters

**User Vision**: "Songbird is a port-free system. It uses UDP. Ports are like phone numbers or social security - very insecure by default."

**Current State**: Still using TCP port 8080 for federation ❌

**Future State**: Only UDP multicast for discovery, BTSP tunnels for communication ✅

**This evolution completes the vision!**

---

## 📚 References

- **BTSP Client**: `crates/songbird-universal/src/btsp_client.rs` (already implemented v3.16.0)
- **Connection Manager**: `crates/songbird-orchestrator/src/app/connection_manager.rs` (needs evolution)
- **Connection Types**: `crates/songbird-orchestrator/src/connections/*.rs` (need BTSP variants)
- **BearDog API**: v0.15.0 (complete, 6/6 endpoints)
- **User Philosophy**: "Security from cryptography, not port obscurity"

---

## 🤝 Handoff

**From**: biomeOS Integration Team  
**To**: Songbird Development Team  
**Timeline**: v3.18.0 (non-urgent)  
**Effort**: 1 session (~6 hours)  
**Blocker**: None (infrastructure complete)  
**Risk**: Low (HTTPS fallback ensures compatibility)  

**Status**: ✅ Ready for implementation when prioritized

---

**Date**: January 7, 2026  
**Document**: BTSP Connection Evolution  
**Version**: v3.18.0 Proposal  
**Confidence**: 💯 100% (infrastructure exists, just needs wiring!)


