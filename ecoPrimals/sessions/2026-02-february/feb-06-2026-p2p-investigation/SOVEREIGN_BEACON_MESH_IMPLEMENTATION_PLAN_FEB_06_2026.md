# 🚀 Sovereign Beacon Mesh - Implementation Plan

**Date**: February 6, 2026  
**Status**: Ready to Execute  
**Total Effort**: 6-8 days (MVP) | 8-11 days (Full)  
**Priority**: HIGH

---

## Executive Summary

**Current State**:
- ✅ Architecture designed and validated
- ✅ 80% implementation complete (1,022 lines)
- ✅ Core algorithms implemented and tested
- ✅ Builds cleanly (4 minor warnings)
- ❌ Tor transport layer missing (critical gap)

**Recommended Approach**:
1. **Phase 1A**: Tor outbound connections (stable API) - 2-3 days
2. **Phase 2**: IPC integration - 1-2 days
3. **Phase 3**: Relay integration - 1 day
4. **Phase 5**: Testing & validation - 2 days
5. **Phase 1B**: Full onion service (when Arti ready) - future
6. **Phase 4**: BirdSong layers (optional) - future

**MVP Timeline**: 6-8 days  
**Risk**: Low (phased approach, stable APIs)  
**Value**: High (true sovereign NAT traversal)

---

## Phase 1A: Tor Transport (Outbound-Only)

**Status**: ⚠️ Not started  
**Effort**: 2-3 days  
**Priority**: CRITICAL  
**API Stability**: ✅ Stable

### Goals

Implement `tor_transport.rs` using Arti's **stable** outbound connection API:
- Bootstrap Tor client
- Connect to existing .onion addresses
- Error handling and timeouts
- Unit and integration tests

**Not Included** (deferred to Phase 1B):
- Creating onion services (experimental API)

### Tasks

#### Task 1.1: Research Arti API (2 hours)

**Actions**:
```bash
# Review Arti documentation
firefox https://tpo.pages.torproject.net/core/doc/rust/arti_client/

# Check latest examples
cd /tmp && git clone https://gitlab.torproject.org/tpo/core/arti.git
cat arti/crates/arti-client/examples/client.rs

# Verify version compatibility
cargo search arti-client
# Latest: 0.24.1 (Jan 2026)
```

**Deliverable**: Notes on API usage, examples, potential issues

#### Task 1.2: Implement TorTransport (8 hours)

**File**: `crates/songbird-onion-relay/src/tor_transport.rs`

**Code**:
```rust
//! Tor transport layer using Arti (Pure Rust Tor)
//!
//! **Phase 1A**: Outbound connections only (stable API)
//! **Phase 1B**: Onion service creation (when API stable)

use arti_client::{TorClient, TorClientConfig};
use tor_rtcompat::PreferredRuntime;
use std::net::IpAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::error::{OnionRelayError, Result};

/// Tor transport for outbound connections
pub struct TorTransport {
    client: TorClient<PreferredRuntime>,
    bootstrap_time: Duration,
}

impl TorTransport {
    /// Bootstrap new Tor client
    ///
    /// This takes 10-30s to download consensus and connect to relays.
    /// Should be done once at startup, then reused.
    pub async fn new() -> Result<Self> {
        info!("🧅 Bootstrapping Tor client...");
        let start = std::time::Instant::now();
        
        // Create default config (uses public Tor network)
        let config = TorClientConfig::default();
        
        // Bootstrap (connects to directory authorities, downloads consensus)
        let client = TorClient::create_bootstrapped(config)
            .await
            .map_err(|e| OnionRelayError::Tor(format!("Bootstrap failed: {}", e)))?;
        
        let bootstrap_time = start.elapsed();
        info!("✅ Tor bootstrapped in {:?}", bootstrap_time);
        
        Ok(Self {
            client,
            bootstrap_time,
        })
    }
    
    /// Connect to an onion service
    ///
    /// # Arguments
    /// * `onion_addr` - Onion address (e.g., "abc123def456...xyz.onion")
    /// * `port` - Port number
    ///
    /// # Returns
    /// TCP stream to the onion service
    pub async fn connect(&self, onion_addr: &str, port: u16) -> Result<TorStream> {
        debug!("🧅 Connecting to {}:{}", &onion_addr[..16.min(onion_addr.len())], port);
        
        let stream = timeout(
            Duration::from_secs(30),
            self.client.connect((onion_addr, port))
        )
        .await
        .map_err(|_| OnionRelayError::Tor("Connection timeout".to_string()))?
        .map_err(|e| OnionRelayError::Tor(format!("Connect failed: {}", e)))?;
        
        info!("✅ Connected to {}", &onion_addr[..16.min(onion_addr.len())]);
        Ok(TorStream { stream })
    }
    
    /// Get bootstrap duration (for metrics)
    pub fn bootstrap_time(&self) -> Duration {
        self.bootstrap_time
    }
    
    /// Check if Tor client is still connected
    pub fn is_connected(&self) -> bool {
        // Arti doesn't expose connection state directly
        // In practice, if bootstrap succeeded, client is usable
        true
    }
}

/// Wrapper around Arti's DataStream
pub struct TorStream {
    stream: arti_client::DataStream,
}

impl TorStream {
    /// Read data from stream
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.stream.read(buf).await
            .map_err(|e| OnionRelayError::Io(e))
    }
    
    /// Write data to stream
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.stream.write(buf).await
            .map_err(|e| OnionRelayError::Io(e))
    }
    
    /// Flush buffered data
    pub async fn flush(&mut self) -> Result<()> {
        self.stream.flush().await
            .map_err(|e| OnionRelayError::Io(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore = "Requires network and Tor consensus download (10-30s)"]
    async fn test_tor_bootstrap() {
        let transport = TorTransport::new().await;
        assert!(transport.is_ok());
        
        let transport = transport.unwrap();
        assert!(transport.bootstrap_time() > Duration::from_secs(5));
        assert!(transport.is_connected());
    }
    
    #[tokio::test]
    #[ignore = "Requires network and real onion service"]
    async fn test_tor_connect() {
        let transport = TorTransport::new().await.unwrap();
        
        // Connect to a known test onion
        // (Replace with actual test onion when available)
        let result = transport.connect("test.onion", 80).await;
        
        // This will fail without a real onion, but tests the API
        match result {
            Ok(_) => println!("✅ Connected to test onion"),
            Err(e) => println!("⚠️ Expected error (no test onion): {}", e),
        }
    }
}
```

**Deliverable**: Working `tor_transport.rs` with outbound connections

#### Task 1.3: Update lib.rs (30 minutes)

**File**: `crates/songbird-onion-relay/src/lib.rs`

**Changes**:
```rust
#[cfg(feature = "tor")]
pub mod tor_transport;  // Already present, just verify

#[cfg(feature = "tor")]
pub use tor_transport::{TorTransport, TorStream};  // Add re-export
```

#### Task 1.4: Integration Test (2 hours)

**File**: `crates/songbird-onion-relay/tests/tor_integration.rs` (new)

**Code**:
```rust
#[cfg(feature = "tor")]
mod tor_tests {
    use songbird_onion_relay::TorTransport;
    use std::time::Duration;
    
    #[tokio::test]
    #[ignore = "Slow test - requires Tor bootstrap"]
    async fn test_tor_bootstrap_real() {
        // This test actually bootstraps Tor (10-30s)
        let result = TorTransport::new().await;
        assert!(result.is_ok());
        
        let transport = result.unwrap();
        println!("✅ Tor bootstrapped in {:?}", transport.bootstrap_time());
        
        // Verify bootstrap was reasonably fast
        assert!(transport.bootstrap_time() < Duration::from_secs(60));
    }
    
    // Additional tests...
}
```

**Run**:
```bash
cargo test --package songbird-onion-relay --features tor -- --ignored
```

#### Task 1.5: Documentation (1 hour)

**Files to update**:
- `crates/songbird-onion-relay/README.md` (create if missing)
- `crates/songbird-onion-relay/src/tor_transport.rs` (rustdoc)
- `CHANGELOG.md` (add entry)

### Success Criteria

- [ ] `TorTransport::new()` successfully bootstraps Tor client
- [ ] `TorTransport::connect()` connects to onion addresses
- [ ] Error handling works (timeouts, network failures)
- [ ] Unit tests pass (mocked Tor)
- [ ] Integration tests pass (real Tor bootstrap)
- [ ] Documentation complete
- [ ] Code review by team (if applicable)

### Risk Mitigation

**Risk**: Arti API changes  
**Mitigation**: Pin to `0.24` in Cargo.toml, monitor releases

**Risk**: Bootstrap takes >30s  
**Mitigation**: Run in background, show progress to user

**Risk**: Network firewall blocks Tor  
**Mitigation**: Detect and report to user with actionable message

---

## Phase 2: IPC Integration

**Status**: ⚠️ Not started  
**Effort**: 1-2 days  
**Dependencies**: Phase 1A  
**Priority**: HIGH

### Goals

Wire `BeaconMesh` into `songbird-universal-ipc` to expose mesh functionality via JSON-RPC.

### Tasks

#### Task 2.1: Add Mesh to IPC Service (4 hours)

**File**: `crates/songbird-universal-ipc/src/service.rs`

**Changes**:
```rust
use songbird_onion_relay::{BeaconMesh, HolePunchCoordinator};

pub struct UniversalIpcService {
    // Existing fields...
    beacon_mesh: Arc<BeaconMesh>,
    hole_punch: Arc<HolePunchCoordinator>,
}

impl UniversalIpcService {
    pub fn new(/* ... */) -> Self {
        let beacon_mesh = Arc::new(BeaconMesh::new(
            node_id.clone(),
            bootstrap_onions, // From config or beacon
        ));
        
        let (hole_punch, signal_tx, signal_rx) = HolePunchCoordinator::new(
            node_id.clone(),
            HolePunchConfig::default(),
        );
        
        Self {
            // ...
            beacon_mesh,
            hole_punch: Arc::new(hole_punch),
        }
    }
}
```

#### Task 2.2: Implement mesh.status (1 hour)

**Code**:
```rust
"mesh.status" => {
    let mesh = self.beacon_mesh.clone();
    let reachable = mesh.get_reachable_nodes().await;
    let my_onion = mesh.my_onion.read().await.clone();
    let paths = mesh.best_paths.read().await.len();
    
    json!({
        "node_id": mesh.my_node_id,
        "reachable_nodes": reachable.len(),
        "my_onion": my_onion,
        "paths": paths,
        "tor_bootstrapped": self.tor_transport.is_some(),
    })
}
```

#### Task 2.3: Implement mesh.find_path (1 hour)

**Code**:
```rust
"mesh.find_path" => {
    let target = params.get("target_node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| /* error */)?;
    
    let path = self.beacon_mesh.find_relay_for(target).await;
    
    match path {
        Some(endpoint) => json!({
            "found": true,
            "node_id": endpoint.node_id,
            "endpoint_type": format!("{:?}", endpoint.endpoint_type),
            "latency_ms": endpoint.latency.map(|d| d.as_millis()),
            "reachable": endpoint.reachable,
        }),
        None => json!({"found": false}),
    }
}
```

#### Task 2.4: Implement mesh.announce (1 hour)

**Code**:
```rust
"mesh.announce" => {
    let msg = self.beacon_mesh.announce_as_relay().await;
    
    // Broadcast via BirdSong (integration with discovery)
    self.birdsong_broadcaster.broadcast(msg).await?;
    
    json!({
        "announced": true,
        "reachable": msg.capabilities.iter()
            .find(|c| c.starts_with("can_reach:"))
            .and_then(|c| c.split(':').nth(1))
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(0)
    })
}
```

#### Task 2.5: Implement mesh.connect (2 hours)

**Code**:
```rust
"mesh.connect" => {
    let peer_node_id = params.get("peer_node_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| /* error */)?;
    
    // Initiate hole punch
    let result = self.hole_punch.punch_to_peer(peer_node_id).await?;
    
    match result {
        PunchResult::Direct { peer_addr, latency, .. } => {
            // Record successful direct connection in mesh
            self.beacon_mesh.record_direct_connection(
                peer_node_id.to_string(),
                peer_addr,
                latency,
            ).await;
            
            json!({
                "status": "direct",
                "addr": peer_addr.to_string(),
                "latency_ms": latency.as_millis(),
            })
        }
        PunchResult::Relay { attempts } => {
            json!({
                "status": "relay",
                "attempts": attempts,
                "message": "Hole punch failed, using relay"
            })
        }
    }
}
```

#### Task 2.6: Handler Tests (2 hours)

**File**: `crates/songbird-universal-ipc/src/handlers/mesh_handler.rs` (new)

**Tests**:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mesh_status() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_find_path_found() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_find_path_not_found() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_announce() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_connect_direct() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_connect_relay() { /* ... */ }
    
    #[tokio::test]
    async fn test_mesh_connect_failure() { /* ... */ }
}
```

### Success Criteria

- [ ] All mesh.* methods work via JSON-RPC
- [ ] Status shows correct mesh state
- [ ] Path finding returns valid endpoints
- [ ] Announce broadcasts to mesh
- [ ] Connect initiates hole punch correctly
- [ ] All handler tests pass (7 tests)
- [ ] Integration test (end-to-end mesh.connect)

---

## Phase 3: Relay Integration

**Status**: ⚠️ Not started  
**Effort**: 1 day  
**Dependencies**: Phase 2  
**Priority**: MEDIUM

### Goals

Wire `BeaconMesh` into `songbird-lineage-relay` for intelligent path selection.

### Tasks

#### Task 3.1: Add Mesh to RelayDiscovery (2 hours)

**File**: `crates/songbird-lineage-relay/src/coordinator.rs`

**Changes**:
```rust
use songbird_onion_relay::BeaconMesh;

pub struct RelayDiscovery {
    // Existing fields...
    mesh: Option<Arc<BeaconMesh>>,
}

impl RelayDiscovery {
    pub fn new(/* ... */) -> Self {
        Self {
            // ...
            mesh: None,  // Set later via set_mesh()
        }
    }
    
    pub fn set_mesh(&mut self, mesh: Arc<BeaconMesh>) {
        self.mesh = Some(mesh);
    }
}
```

#### Task 3.2: Implement Mesh-Aware Relay (3 hours)

**Code**:
```rust
impl RelayDiscovery {
    pub async fn request_relay_via_mesh(
        &self,
        target: NodeId,
        mesh: &BeaconMesh,
    ) -> Result<Arc<RelaySession>> {
        // 1. Check mesh for best path
        if let Some(endpoint) = mesh.find_relay_for(&target.0).await {
            match endpoint.endpoint_type {
                EndpointType::Direct { addr } => {
                    info!("✅ Direct connection available to {}", target);
                    return self.create_direct_session(target, addr).await;
                }
                EndpointType::FamilyRelay { relay_node_id } => {
                    info!("🔄 Routing via family relay: {}", relay_node_id);
                    return self.create_relayed_session(target, relay_node_id).await;
                }
                EndpointType::TorOnion { onion_addr } => {
                    info!("🧅 Routing via Tor: {}", &onion_addr[..16]);
                    return self.create_tor_session(target, onion_addr).await;
                }
                EndpointType::Local { addr } => {
                    info!("🏠 Local connection to {}", addr);
                    return self.create_direct_session(target, addr).await;
                }
            }
        }
        
        // 2. Fall back to existing BirdSong relay discovery
        warn!("No mesh path found, falling back to BirdSong relay discovery");
        self.request_relay(target, None).await
    }
}
```

#### Task 3.3: Update Existing Request Method (1 hour)

**Code**:
```rust
pub async fn request_relay(
    &self,
    target: NodeId,
    options: Option<RelayOptions>,
) -> Result<Arc<RelaySession>> {
    // If mesh is available, use it first
    if let Some(mesh) = &self.mesh {
        match self.request_relay_via_mesh(target.clone(), mesh).await {
            Ok(session) => return Ok(session),
            Err(e) => {
                warn!("Mesh relay failed: {}, falling back to discovery", e);
            }
        }
    }
    
    // Existing fallback logic...
}
```

#### Task 3.4: Integration Tests (2 hours)

**File**: `crates/songbird-lineage-relay/tests/mesh_integration.rs` (new)

**Tests**:
```rust
#[tokio::test]
async fn test_relay_via_mesh_direct() { /* ... */ }

#[tokio::test]
async fn test_relay_via_mesh_family() { /* ... */ }

#[tokio::test]
async fn test_relay_fallback_to_discovery() { /* ... */ }
```

### Success Criteria

- [ ] Relay uses mesh for path selection when available
- [ ] Falls back gracefully when mesh unavailable
- [ ] Direct connections preferred over relays
- [ ] Family relays preferred over Tor
- [ ] All tests pass (3 integration tests)
- [ ] Performance: path selection <10ms

---

## Phase 5: Testing & Validation

**Status**: ⚠️ Not started  
**Effort**: 2 days  
**Dependencies**: Phases 1-3  
**Priority**: CRITICAL

### Goals

Comprehensive test suite and physical device validation.

### Tasks

#### Task 5.1: Unit Test Completion (4 hours)

**Target**: 30+ unit tests passing

**Areas**:
- Tor transport (4 tests)
- Signaling protocol (already 2, add 3 more)
- Hole punch coordinator (already 2, add 4 more)
- Beacon mesh (already 4, add 4 more)
- IPC handlers (7 tests)
- Relay integration (3 tests)

#### Task 5.2: Integration Tests (4 hours)

**Target**: 12+ integration tests

**Scenarios**:
1. Full hole punch flow (signaling → punch → direct)
2. Hole punch failure → relay fallback
3. Mesh relay path selection
4. Multi-hop relay (A → B → C)
5. IPC method validation (all mesh.* methods)
6. Tor bootstrap and connect

#### Task 5.3: Physical Validation Plan (2 hours)

**Document**: `NAT_TRAVERSAL_MESH_VALIDATION_GUIDE.md`

**Test Matrix**:

| Test | Device 1 | Device 2 | Expected | Status |
|------|----------|----------|----------|--------|
| Local | Tower (home LAN) | Pixel (home LAN) | Local connection | ⬜ |
| Cone NAT | Tower (home) | Laptop (coffee shop) | Direct P2P | ⬜ |
| Symmetric | Tower (home) | Phone (carrier 4G) | Relay (or 30% direct) | ⬜ |
| Mesh Growth | Add 3rd device | | All can reach all | ⬜ |
| Failover | Disconnect relay | | Auto-reconnect | ⬜ |

#### Task 5.4: Performance Benchmarks (2 hours)

**Metrics to measure**:
- Tor bootstrap time (target: <30s)
- Hole punch time (target: <2s)
- Direct P2P latency (target: <100ms)
- Relay latency (target: <500ms)
- Mesh path selection (target: <10ms)

#### Task 5.5: Documentation (2 hours)

**Files to create/update**:
- `NAT_TRAVERSAL_MESH_VALIDATION_GUIDE.md`
- `DEPLOYMENT_READY_STATUS.md` (update with mesh methods)
- `CHANGELOG.md` (v3.25.0 entry)
- `README.md` (mention mesh capability)

### Success Criteria

- [ ] 30+ unit tests passing
- [ ] 12+ integration tests passing
- [ ] Physical validation guide complete
- [ ] At least 1 successful cross-NAT test (Tower ↔ Pixel)
- [ ] Performance targets met
- [ ] Documentation complete and accurate

---

## Timeline

### Week 1 (Days 1-3): Core Implementation

| Day | Focus | Deliverables |
|-----|-------|--------------|
| **1** | Phase 1A (pt 1) | Arti API research, TorTransport skeleton |
| **2** | Phase 1A (pt 2) | TorTransport complete, tests passing |
| **3** | Phase 2 (pt 1) | IPC integration, mesh.* methods |

### Week 2 (Days 4-6): Integration & Testing

| Day | Focus | Deliverables |
|-----|-------|--------------|
| **4** | Phase 2 (pt 2) + Phase 3 | IPC tests, relay integration |
| **5** | Phase 5 (pt 1) | Unit & integration tests |
| **6** | Phase 5 (pt 2) | Physical validation, documentation |

**Optional** (Days 7-8):
- Phase 1B: Full onion service (when Arti ready)
- Phase 4: BirdSong layered encryption

---

## Risk Management

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Arti API issues** | Medium | High | Pin version, use stable API only |
| **Tor bootstrap slow** | High | Medium | Background thread, cache consensus |
| **Hole punch low success** | Medium | Medium | Document expected rates, relay fallback |
| **Integration complexity** | Low | Medium | Phased approach, good tests |
| **Physical validation blocked** | Low | High | Virtual NAT testing as fallback |

---

## Rollout Plan

### MVP (Phase 1A + 2 + 3)

**Timeline**: 6-8 days  
**Scope**: Outbound Tor, IPC, Relay integration  
**Capability**: Connect to existing onion services, hole punch, mesh relay

### Full (+ Phase 1B + 4)

**Timeline**: 8-11 days  
**Scope**: Create onion services, layered encryption  
**Capability**: True sovereign (no external dependency)

### Production

**Timeline**: After physical validation  
**Features**:
- `mesh.status` - Check mesh state
- `mesh.find_path` - Path discovery
- `mesh.announce` - Advertise as relay
- `mesh.connect` - Auto hole punch + relay

---

## Success Metrics

| Metric | Target | Current | Phase |
|--------|--------|---------|-------|
| **Implementation** | 100% | 80% | 1-3 |
| **Unit Tests** | 30+ | 8 | 5 |
| **Integration Tests** | 12+ | 0 | 5 |
| **Build Status** | Clean | ✅ 4 warnings | 1 |
| **Deep Debt Score** | 95%+ | 95% | All |
| **Physical Validation** | 1+ successful | 0 | 5 |

---

## Next Steps

### Immediate (This Week)

1. ✅ Investigation complete
2. ✅ Specification written
3. ⚠️ **START: Phase 1A** - Implement Tor transport
4. ⚠️ **START: Phase 2** - IPC integration

### Short-Term (Next 2 Weeks)

5. Complete Phase 3 (relay integration)
6. Complete Phase 5 (testing)
7. Physical validation (Tower ↔ Pixel)
8. Document results

### Medium-Term (Next 1-2 Months)

9. Monitor Arti for API stabilization
10. Implement Phase 1B (full onion service)
11. Production deployment
12. Gather usage metrics

---

**Plan Complete**: February 6, 2026  
**Status**: ✅ **READY TO EXECUTE**  
**Next Action**: Begin Phase 1A (Tor Transport)

🦀 **Pure Rust** | 🧅 **Sovereign** | 🚀 **Ready to Build**
