# 🔐 BTSP Integration Plan - Songbird v3.15.0

**Date**: January 7, 2026  
**Status**: 🎯 **PLANNING** - Ready to Execute  
**Priority**: 🟡 **HIGH** - Port-Free Architecture Goal

---

## 🎯 **Mission**

**Evolve Songbird from HTTPS to BTSP for true port-free, encrypted P2P communication**

> "BearDog and Songbird working close to the metal means encryption overhead is small. This enables secure-by-default P2P with zero port conflicts."

---

## 📊 **Current State Analysis**

### **What Works** ✅
1. **Discovery**: UDP multicast for peer discovery
2. **Trust Evaluation**: Unix socket + JSON-RPC to BearDog
3. **Local IPC**: tarpc (PRIMARY), JSON-RPC (SECONDARY), HTTP (FALLBACK)

### **The Gap** ⚠️
**Tower-to-Tower**: Still using HTTPS (legacy, port-based)

```rust
// Current flow (v3.14.2):
Discover peer → Trust eval → Connect via HTTPS ⚠️
```

**Should Be**:
```rust
// Target flow (v3.15.0):
Discover peer → Trust eval → Request BTSP tunnel → Use tunnel ✅
```

---

## 🏗️ **Architecture**

### **Phase 1: BTSP Tunnel Request** (4-6 hours)

**Goal**: After successful trust evaluation, request BTSP tunnel from BearDog

**Changes**:
1. Create `BearDogClient` for tunnel management
2. Update `discovery_bridge.rs` to request tunnels
3. Store tunnel info in `ConnectionManager`

**Files**:
- `crates/songbird-universal/src/beardog_client.rs` (NEW)
- `crates/songbird-orchestrator/src/app/discovery_bridge.rs` (MODIFY)
- `crates/songbird-orchestrator/src/app/connection_manager.rs` (MODIFY)

---

### **Phase 2: BTSP Transport Layer** (8-12 hours)

**Goal**: Replace HTTPS client with BTSP transport

**Changes**:
1. Create BTSP transport abstraction
2. Replace HTTP calls with BTSP sends
3. Maintain HTTP fallback for debugging

**Files**:
- `crates/songbird-orchestrator/src/transport/btsp.rs` (NEW)
- `crates/songbird-orchestrator/src/transport/mod.rs` (NEW)
- `crates/songbird-orchestrator/src/app/peer_communication.rs` (MODIFY)

---

### **Phase 3: Full Port-Free Federation** (4-6 hours)

**Goal**: Make HTTPS optional, BTSP primary

**Changes**:
1. Update configuration (BTSP by default)
2. Make HTTPS ports optional
3. Comprehensive testing

**Files**:
- `crates/songbird-config/src/lib.rs` (MODIFY)
- `crates/songbird-orchestrator/tests/btsp_integration_tests.rs` (NEW)

---

## 🔧 **Implementation Details**

### **1. BearDogClient for BTSP** (NEW)

```rust
// crates/songbird-universal/src/beardog_client.rs

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// BearDog client for BTSP tunnel management
///
/// This client wraps BearDog's BTSP tunnel APIs, providing a clean
/// interface for Songbird to request and manage encrypted tunnels.
pub struct BearDogClient {
    endpoint: String,
    client: JsonRpcClient, // Reuse existing JSON-RPC client
}

impl BearDogClient {
    /// Create new BearDog client
    pub fn new(endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let client = JsonRpcClient::new(&endpoint)?;
        Ok(Self { endpoint, client })
    }

    /// Establish BTSP tunnel to peer
    ///
    /// This requests BearDog to create an encrypted tunnel to the specified peer.
    /// BearDog handles all encryption, key exchange, and tunnel maintenance.
    pub async fn establish_tunnel(
        &self,
        peer_id: &str,
        peer_endpoint: &str,
    ) -> Result<TunnelInfo> {
        let req = EstablishTunnelRequest {
            peer_id: peer_id.to_string(),
            peer_endpoint: peer_endpoint.to_string(),
        };

        let resp: TunnelInfo = self.client
            .call_method("btsp.establish_tunnel", Some(req))
            .await?;

        Ok(resp)
    }

    /// Get tunnel status
    pub async fn get_tunnel_status(&self, tunnel_id: &str) -> Result<TunnelStatus> {
        let req = serde_json::json!({ "tunnel_id": tunnel_id });
        let resp: TunnelStatus = self.client
            .call_method("btsp.get_tunnel_status", Some(req))
            .await?;
        Ok(resp)
    }

    /// Close tunnel
    pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()> {
        let req = serde_json::json!({ "tunnel_id": tunnel_id });
        self.client
            .call_method::<_, serde_json::Value>("btsp.close_tunnel", Some(req))
            .await?;
        Ok(())
    }
}

/// BTSP tunnel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub tunnel_id: String,
    pub peer_id: String,
    pub status: TunnelStatus,
    pub created_at: u64,
}

/// Tunnel status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelStatus {
    Establishing,
    Active,
    Closed,
    Failed,
}

#[derive(Debug, Serialize)]
struct EstablishTunnelRequest {
    peer_id: String,
    peer_endpoint: String,
}
```

---

### **2. Discovery Bridge Integration** (MODIFY)

```rust
// crates/songbird-orchestrator/src/app/discovery_bridge.rs

// After trust evaluation succeeds:

if trust_decision.decision == "auto_accept" {
    info!("✅ Trust evaluation: AUTO-ACCEPT (same family)");

    // NEW: Request BTSP tunnel from BearDog
    if let Some(ref beardog_endpoint) = security_client_endpoint {
        match self.establish_btsp_tunnel(
            &peer.node_id,
            &endpoint,
            beardog_endpoint,
        ).await {
            Ok(tunnel_info) => {
                info!("🔐 BTSP tunnel established: {}", tunnel_info.tunnel_id);
                
                // Store tunnel in connection manager
                self.connection_manager
                    .register_tunnel(&peer.node_id, tunnel_info)
                    .await?;
            }
            Err(e) => {
                warn!("⚠️  BTSP tunnel failed, falling back to HTTPS: {}", e);
                // Fallback to HTTPS (legacy)
            }
        }
    }
}

async fn establish_btsp_tunnel(
    &self,
    peer_id: &str,
    peer_endpoint: &str,
    beardog_endpoint: &str,
) -> Result<TunnelInfo> {
    use songbird_universal::BearDogClient;
    
    let beardog = BearDogClient::new(beardog_endpoint)?;
    let tunnel = beardog.establish_tunnel(peer_id, peer_endpoint).await?;
    
    Ok(tunnel)
}
```

---

### **3. Connection Manager Updates** (MODIFY)

```rust
// crates/songbird-orchestrator/src/app/connection_manager.rs

use std::collections::HashMap;
use songbird_universal::TunnelInfo;

pub struct ConnectionManager {
    // Existing fields...
    
    /// BTSP tunnels to peers (NEW)
    tunnels: Arc<RwLock<HashMap<String, TunnelInfo>>>,
}

impl ConnectionManager {
    /// Register BTSP tunnel for peer
    pub async fn register_tunnel(
        &self,
        peer_id: &str,
        tunnel_info: TunnelInfo,
    ) -> Result<()> {
        let mut tunnels = self.tunnels.write().await;
        tunnels.insert(peer_id.to_string(), tunnel_info);
        info!("🔐 Registered BTSP tunnel for peer: {}", peer_id);
        Ok(())
    }

    /// Get tunnel for peer
    pub async fn get_tunnel(&self, peer_id: &str) -> Option<TunnelInfo> {
        let tunnels = self.tunnels.read().await;
        tunnels.get(peer_id).cloned()
    }

    /// Check if peer has active tunnel
    pub async fn has_tunnel(&self, peer_id: &str) -> bool {
        let tunnels = self.tunnels.read().await;
        tunnels.contains_key(peer_id)
    }
}
```

---

### **4. BTSP Transport Layer** (NEW - Phase 2)

```rust
// crates/songbird-orchestrator/src/transport/btsp.rs

/// BTSP transport for peer-to-peer communication
///
/// This wraps BearDog's BTSP tunnels, providing a clean async interface
/// for sending/receiving data over encrypted tunnels.
pub struct BtspTransport {
    beardog: Arc<BearDogClient>,
}

impl BtspTransport {
    pub fn new(beardog_endpoint: impl Into<String>) -> Result<Self> {
        let beardog = Arc::new(BearDogClient::new(beardog_endpoint)?);
        Ok(Self { beardog })
    }

    /// Send data over BTSP tunnel
    pub async fn send(
        &self,
        tunnel_id: &str,
        data: &[u8],
    ) -> Result<()> {
        // BearDog handles encryption automatically
        let req = serde_json::json!({
            "tunnel_id": tunnel_id,
            "data": base64::encode(data),
        });

        self.beardog.client
            .call_method::<_, serde_json::Value>("btsp.send", Some(req))
            .await?;

        Ok(())
    }

    /// Receive data over BTSP tunnel
    pub async fn receive(
        &self,
        tunnel_id: &str,
    ) -> Result<Vec<u8>> {
        let req = serde_json::json!({ "tunnel_id": tunnel_id });

        let resp: ReceiveResponse = self.beardog.client
            .call_method("btsp.receive", Some(req))
            .await?;

        let data = base64::decode(&resp.data)?;
        Ok(data)
    }
}

#[derive(Deserialize)]
struct ReceiveResponse {
    data: String, // base64 encoded
}
```

---

## 🧪 **Testing Strategy**

### **Unit Tests**
- `BearDogClient` tunnel request/close
- `ConnectionManager` tunnel registration
- `BtspTransport` send/receive

### **Integration Tests**
- End-to-end tunnel establishment
- Fallback to HTTPS if BTSP fails
- Tunnel health monitoring

### **E2E Tests**
- Two towers with BTSP tunnels
- Discovery → Trust → Tunnel → Communication
- Verify no HTTPS ports used

---

## 📊 **Benefits**

### **Security** 🔐
- ✅ End-to-end encryption (BearDog managed)
- ✅ Genetic lineage-based keys
- ✅ Zero port exposure

### **Architecture** 🏗️
- ✅ True port-free federation
- ✅ No n² port conflicts
- ✅ Fractal scalability

### **Performance** ⚡
- ✅ Low overhead (BearDog close to metal)
- ✅ Direct peer-to-peer
- ✅ No HTTP parsing overhead

---

## 🎯 **Success Criteria**

### **v3.15.0**:
- ✅ BTSP tunnel request after trust eval
- ✅ Tunnel storage in ConnectionManager
- ✅ Basic BTSP transport
- ✅ HTTP fallback maintained

### **v3.16.0** (Future):
- ✅ Full BTSP transport layer
- ✅ Replace all HTTPS with BTSP
- ✅ Make HTTPS optional (debug only)

---

## 🔄 **Migration Path**

### **Backward Compatibility**
- HTTPS remains as fallback
- Configuration toggle: `SONGBIRD_USE_BTSP=true`
- Gradual rollout

### **Deployment**
1. Deploy v3.15.0 (BTSP + HTTPS)
2. Test BTSP tunnels
3. Enable `USE_BTSP=true`
4. Monitor for issues
5. Deprecate HTTPS in v3.16.0

---

## 📋 **Implementation Checklist**

### **Phase 1**: (This Session)
- [ ] Create `BearDogClient`
- [ ] Update `discovery_bridge.rs`
- [ ] Update `ConnectionManager`
- [ ] Add basic tests
- [ ] Document changes

### **Phase 2**: (Next Session)
- [ ] Create BTSP transport layer
- [ ] Replace peer communication
- [ ] Add integration tests
- [ ] Update documentation

### **Phase 3**: (Future)
- [ ] Make HTTPS optional
- [ ] Full E2E testing
- [ ] Performance benchmarks
- [ ] Production rollout

---

## 🎊 **Summary**

**Current**: Tower-to-tower uses HTTPS (legacy, port-based)  
**Target**: Tower-to-tower uses BTSP (port-free, encrypted)  
**Gap**: Songbird needs to call BearDog's BTSP APIs  
**Solution**: Integrate BearDog BTSP client into discovery flow  
**Priority**: HIGH - Enables true port-free architecture  
**ETA**: Phase 1 complete in this session (~6 hours)

---

**Status**: ✅ **READY TO IMPLEMENT**  
**Next**: Create `BearDogClient` and integrate into discovery bridge

---

_"BearDog handles encryption, Songbird handles orchestration. Together, they enable secure-by-default P2P communication."_

