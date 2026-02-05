# 🔄 Pure Rust Relay Server Investigation

**Date**: February 5, 2026  
**Status**: Ready for Implementation  
**Priority**: HIGH (completes sovereign NAT traversal)  
**Complexity**: Medium-High (4-5 days)

---

## 🎯 Executive Summary

**Gap Identified**: Lineage-based relay packet forwarding infrastructure

**Current State**:
- ✅ STUN server: **COMPLETE** (Feb 5, 2026)
- ✅ UDP hole punching: **COMPLETE** (178 lines, working)
- ✅ Relay discovery: **COMPLETE** (session management)
- ✅ Lineage authorization: **COMPLETE** (BearDog integration)
- ❌ **Relay SERVER**: **STUB ONLY** (packet forwarding missing)

**The Problem**: 
- `RelaySession.send()` is a stub (lines 93-105 in relay.rs)
- No actual UDP packet forwarding
- Symmetric NAT traversal requires relay fallback
- coturn still needed for production

**The Solution**:
Implement Pure Rust relay server with lineage-based authorization, completing the sovereign NAT traversal stack.

---

## 📊 Current Infrastructure Analysis

### What Exists (2,910 lines)

| Component | Location | Lines | Status |
|-----------|----------|-------|--------|
| **UDP Hole Punch** | `udp_hole_punch.rs` | 178 | ✅ Complete |
| **Relay Discovery** | `relay.rs` | 371 | ✅ Complete |
| **BirdSong Broadcast** | `birdsong.rs` | ~500 | ✅ Complete |
| **Session Management** | `session.rs` | ~200 | ✅ Complete |
| **Lineage Authority** | `beardog.rs` | ~300 | ✅ Complete |
| **Multi-tier Coordinator** | `multi_tier_coordinator.rs` | ~500 | ✅ Complete |
| **Coordinator** | `coordinator.rs` | ~400 | ✅ Complete |

### The Stub (Critical Gap)

```rust
// relay.rs:93-105 - CURRENT IMPLEMENTATION
pub async fn send(&self, data: &[u8]) -> Result<()> {
    // In real implementation, this would send through UDP socket to relay
    debug!(
        "Sending {} bytes through relay {} (masked: {:?})",
        data.len(),
        self.relay_node,
        self.masking_level
    );

    let mut bytes = self.bytes_relayed.lock().await;
    *bytes += data.len() as u64;

    Ok(())  // ❌ DOES NOTHING - just logs and increments counter
}
```

---

## 🏗️ Proposed Architecture

### Relay Server Design

```rust
/// Pure Rust Relay Server
/// 
/// Forwards UDP packets between peers who cannot establish direct connection
/// (typically due to symmetric NAT on both ends).
pub struct RelayServer {
    /// Bind address for relay service
    bind_addr: SocketAddr,
    
    /// Active relay sessions
    sessions: Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    
    /// Lineage authority for authorization
    authority: Arc<dyn RelayAuthority>,
    
    /// UDP socket for packet forwarding
    socket: Arc<UdpSocket>,
    
    /// Server statistics
    stats: Arc<RwLock<RelayServerStats>>,
}

/// Internal session state for packet forwarding
struct RelaySessionState {
    session_id: Uuid,
    requester_addr: SocketAddr,
    target_addr: SocketAddr,
    masking_level: MaskingLevel,
    created_at: SystemTime,
    last_activity: SystemTime,
    bytes_forwarded: u64,
}

#[derive(Debug, Default)]
pub struct RelayServerStats {
    pub sessions_active: u64,
    pub sessions_total: u64,
    pub bytes_forwarded: u64,
    pub packets_forwarded: u64,
    pub authorization_failures: u64,
}
```

### Packet Forwarding Flow

```
┌──────────────┐                   ┌──────────────┐                   ┌──────────────┐
│  Requester   │                   │ Relay Server │                   │    Target    │
│  (Pixel)     │                   │   (Tower)    │                   │   (Laptop)   │
└──────┬───────┘                   └──────┬───────┘                   └──────┬───────┘
       │                                  │                                  │
       │ 1. Allocate Request              │                                  │
       │ (with lineage proof)             │                                  │
       │─────────────────────────────────>│                                  │
       │                                  │                                  │
       │                                  │ 2. Verify lineage                │
       │                                  │    with BearDog                  │
       │                                  │                                  │
       │ 3. Allocation Response           │                                  │
       │    (session_id, relay_addr)      │                                  │
       │<─────────────────────────────────│                                  │
       │                                  │                                  │
       │ 4. Data Packet                   │                                  │
       │    (to relay_addr)               │                                  │
       │─────────────────────────────────>│                                  │
       │                                  │ 5. Forward Packet                │
       │                                  │    (to target_addr)              │
       │                                  │─────────────────────────────────>│
       │                                  │                                  │
       │                                  │ 6. Response Packet               │
       │                                  │<─────────────────────────────────│
       │ 7. Forward Response              │                                  │
       │    (to requester_addr)           │                                  │
       │<─────────────────────────────────│                                  │
```

---

## 🔧 Implementation Plan

### Phase 1: Relay Server Core (~2 days)

**File**: `crates/songbird-lineage-relay/src/relay_server.rs`

```rust
impl RelayServer {
    /// Create new relay server
    pub async fn new(
        bind_addr: SocketAddr,
        authority: Arc<dyn RelayAuthority>,
    ) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        
        Ok(Self {
            bind_addr,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            authority,
            socket: Arc::new(socket),
            stats: Arc::new(RwLock::new(RelayServerStats::default())),
        })
    }
    
    /// Run relay server
    pub async fn run(&self) -> Result<()> {
        let mut buf = vec![0u8; 65536]; // Max UDP datagram size
        
        info!("🔄 Relay server listening on {}", self.bind_addr);
        
        loop {
            let (len, src_addr) = self.socket.recv_from(&mut buf).await?;
            
            // Parse relay protocol
            match RelayProtocol::parse(&buf[..len]) {
                Ok(RelayProtocol::AllocateRequest(req)) => {
                    self.handle_allocate(req, src_addr).await?;
                }
                Ok(RelayProtocol::DataPacket(session_id, data)) => {
                    self.forward_packet(session_id, &data, src_addr).await?;
                }
                Ok(RelayProtocol::Refresh(session_id)) => {
                    self.refresh_session(session_id, src_addr).await?;
                }
                Err(e) => {
                    warn!("Invalid relay protocol: {}", e);
                }
            }
        }
    }
    
    /// Handle allocation request
    async fn handle_allocate(
        &self,
        request: AllocationRequest,
        src_addr: SocketAddr,
    ) -> Result<()> {
        // 1. Verify lineage authorization
        let auth = self.authority
            .authorize_relay(&request.relay_node, &request.requester)
            .await?;
        
        if !auth.authorized {
            warn!("Unauthorized relay request from {}", request.requester);
            let mut stats = self.stats.write().await;
            stats.authorization_failures += 1;
            
            // Send rejection
            let response = AllocationResponse::unauthorized();
            self.socket.send_to(&response.encode(), src_addr).await?;
            return Ok(());
        }
        
        // 2. Create session
        let session_id = Uuid::new_v4();
        let session = RelaySessionState {
            session_id,
            requester_addr: src_addr,
            target_addr: request.target_addr,
            masking_level: auth.masking_level,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            bytes_forwarded: 0,
        };
        
        // 3. Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
            
            let mut stats = self.stats.write().await;
            stats.sessions_active = sessions.len() as u64;
            stats.sessions_total += 1;
        }
        
        // 4. Send success response
        let response = AllocationResponse::success(session_id, self.bind_addr);
        self.socket.send_to(&response.encode(), src_addr).await?;
        
        info!("✅ Allocated relay session {} for {}", session_id, request.requester);
        
        Ok(())
    }
    
    /// Forward packet between peers
    async fn forward_packet(
        &self,
        session_id: Uuid,
        data: &[u8],
        src_addr: SocketAddr,
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            // Update activity
            session.last_activity = SystemTime::now();
            session.bytes_forwarded += data.len() as u64;
            
            // Determine destination (forward to the other peer)
            let dest_addr = if src_addr == session.requester_addr {
                // From requester → to target
                session.target_addr
            } else if src_addr == session.target_addr {
                // From target → to requester
                session.requester_addr
            } else {
                // Unknown source - reject
                warn!("Packet from unknown source: {}", src_addr);
                return Ok(());
            };
            
            // Apply masking if needed
            let masked_data = self.apply_masking(data, session.masking_level)?;
            
            // Forward packet
            self.socket.send_to(&masked_data, dest_addr).await?;
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.bytes_forwarded += data.len() as u64;
            stats.packets_forwarded += 1;
            
            debug!(
                "📦 Forwarded {} bytes: {} → {} (session: {})",
                data.len(), src_addr, dest_addr, session_id
            );
        } else {
            warn!("Unknown session ID: {}", session_id);
        }
        
        Ok(())
    }
    
    /// Apply masking based on lineage relationship
    fn apply_masking(&self, data: &[u8], level: MaskingLevel) -> Result<Vec<u8>> {
        match level {
            MaskingLevel::None => Ok(data.to_vec()),
            MaskingLevel::TimingOnly => {
                // Add random delay jitter (implemented elsewhere)
                Ok(data.to_vec())
            }
            MaskingLevel::SizeObfuscation => {
                // Pad to fixed size or add random padding
                let mut padded = data.to_vec();
                // Pad to next 1KB boundary
                let target_size = ((data.len() + 1023) / 1024) * 1024;
                padded.resize(target_size, 0);
                Ok(padded)
            }
            MaskingLevel::Full => {
                // Full encryption + padding + timing obfuscation
                // In production, integrate with BearDog crypto
                Ok(data.to_vec()) // Placeholder
            }
        }
    }
    
    /// Refresh session (extend TTL)
    async fn refresh_session(&self, session_id: Uuid, src_addr: SocketAddr) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(&session_id) {
            // Verify refresh comes from requester
            if src_addr == session.requester_addr {
                session.last_activity = SystemTime::now();
                debug!("🔄 Refreshed session {}", session_id);
            }
        }
        
        Ok(())
    }
    
    /// Cleanup expired sessions (background task)
    pub async fn cleanup_expired(&self, ttl: Duration) {
        let now = SystemTime::now();
        let mut sessions = self.sessions.write().await;
        
        sessions.retain(|id, session| {
            let age = now.duration_since(session.last_activity).unwrap_or_default();
            if age > ttl {
                info!("🧹 Cleaning up expired session {}", id);
                false
            } else {
                true
            }
        });
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.sessions_active = sessions.len() as u64;
    }
}
```

### Phase 2: Relay Protocol (~1 day)

**File**: `crates/songbird-lineage-relay/src/relay_protocol.rs`

```rust
/// Relay protocol messages
#[derive(Debug, Clone)]
pub enum RelayProtocol {
    /// Request relay allocation
    AllocateRequest(AllocationRequest),
    
    /// Allocation response (success or failure)
    AllocateResponse(AllocationResponse),
    
    /// Data packet to forward
    DataPacket(Uuid, Vec<u8>),
    
    /// Refresh session TTL
    Refresh(Uuid),
}

impl RelayProtocol {
    /// Parse from bytes
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        // Simple protocol:
        // [1 byte: message type][remaining: payload]
        if bytes.is_empty() {
            return Err(LineageRelayError::InvalidProtocol);
        }
        
        match bytes[0] {
            0x01 => {
                // AllocateRequest
                let req = serde_json::from_slice(&bytes[1..])?;
                Ok(Self::AllocateRequest(req))
            }
            0x02 => {
                // AllocateResponse
                let resp = serde_json::from_slice(&bytes[1..])?;
                Ok(Self::AllocateResponse(resp))
            }
            0x10 => {
                // DataPacket
                // [1 byte type][16 bytes session_id][remaining: data]
                if bytes.len() < 17 {
                    return Err(LineageRelayError::InvalidProtocol);
                }
                let session_id = Uuid::from_slice(&bytes[1..17])?;
                let data = bytes[17..].to_vec();
                Ok(Self::DataPacket(session_id, data))
            }
            0x20 => {
                // Refresh
                let session_id = Uuid::from_slice(&bytes[1..17])?;
                Ok(Self::Refresh(session_id))
            }
            _ => Err(LineageRelayError::InvalidProtocol),
        }
    }
    
    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        
        match self {
            Self::AllocateRequest(req) => {
                buf.push(0x01);
                buf.extend_from_slice(&serde_json::to_vec(req).unwrap());
            }
            Self::AllocateResponse(resp) => {
                buf.push(0x02);
                buf.extend_from_slice(&serde_json::to_vec(resp).unwrap());
            }
            Self::DataPacket(session_id, data) => {
                buf.push(0x10);
                buf.extend_from_slice(session_id.as_bytes());
                buf.extend_from_slice(data);
            }
            Self::Refresh(session_id) => {
                buf.push(0x20);
                buf.extend_from_slice(session_id.as_bytes());
            }
        }
        
        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub relay_node: NodeId,
    pub requester: NodeId,
    pub target_addr: SocketAddr,
    pub lineage_proof: Vec<u8>, // BearDog lineage verification
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResponse {
    pub success: bool,
    pub session_id: Option<Uuid>,
    pub relay_addr: Option<SocketAddr>,
    pub ttl_seconds: u32,
    pub error: Option<String>,
}
```

### Phase 3: Update RelaySession.send() (~4 hours)

```rust
// relay.rs - UPDATE EXISTING METHOD
pub async fn send(&self, data: &[u8]) -> Result<()> {
    // Connect to relay server
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(self.relay_address).await?;
    
    // Encode data packet with session ID
    let packet = RelayProtocol::DataPacket(self.session_id, data.to_vec());
    let encoded = packet.encode();
    
    // Send to relay server
    socket.send(&encoded).await?;
    
    // Update stats
    let mut bytes = self.bytes_relayed.lock().await;
    *bytes += data.len() as u64;
    
    debug!(
        "Sent {} bytes through relay {} (session: {})",
        data.len(),
        self.relay_node,
        self.session_id
    );
    
    Ok(())
}
```

### Phase 4: JSON-RPC Integration (~4 hours)

**Add to** `songbird-universal-ipc/src/service.rs`:

```rust
// New method: relay.serve
"relay.serve" => {
    let bind_addr = params.get("bind_addr")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0.0:3479");
    
    self.start_relay_server(bind_addr).await
}

// New method: relay.allocate
"relay.allocate" => {
    let target = params.get("target")
        .and_then(|v| v.as_str())
        .ok_or("Missing target")?;
    
    self.allocate_relay(target).await
}

// New method: relay.status
"relay.status" => {
    self.relay_server_status().await
}

// Implementation
async fn start_relay_server(&self, bind_addr: &str) -> Result<Value, String> {
    let addr: SocketAddr = bind_addr.parse()
        .map_err(|e| format!("Invalid bind address: {e}"))?;
    
    let authority = self.get_relay_authority();
    let mut server = RelayServer::new(addr, authority).await
        .map_err(|e| format!("Failed to create relay server: {e}"))?;
    
    // Spawn server in background
    tokio::spawn(async move {
        if let Err(e) = server.run().await {
            tracing::error!("Relay server error: {}", e);
        }
    });
    
    Ok(json!({
        "status": "started",
        "bind_addr": bind_addr,
        "comment": "Relay server running in background"
    }))
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_relay_server_allocation() {
    let authority = Arc::new(MockRelayAuthority::new());
    let server = RelayServer::new("127.0.0.1:0".parse().unwrap(), authority)
        .await
        .unwrap();
    
    // Test allocation request
    let request = AllocationRequest {
        relay_node: "tower".into(),
        requester: "pixel".into(),
        target_addr: "192.168.1.100:12345".parse().unwrap(),
        lineage_proof: vec![],
    };
    
    // Should succeed
    // ...
}

#[tokio::test]
async fn test_packet_forwarding() {
    // Create relay server
    let server = RelayServer::new(/* ... */).await.unwrap();
    
    // Allocate session
    // ...
    
    // Send packet through relay
    let data = b"Hello, World!";
    // ...
    
    // Verify forwarded to target
    // ...
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_relay_flow() {
    // 1. Start relay server
    let server = RelayServer::new("127.0.0.1:3479".parse().unwrap(), authority)
        .await
        .unwrap();
    
    tokio::spawn(async move { server.run().await });
    
    // 2. Create two peers (requester and target)
    let requester_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let target_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // 3. Requester allocates relay
    // ...
    
    // 4. Send data requester → target via relay
    let data = b"Test message";
    // ...
    
    // 5. Verify target receives data
    // ...
    
    // 6. Send response target → requester via relay
    // ...
    
    // 7. Verify round-trip works
    // ...
}
```

---

## 📊 Estimated Effort

| Phase | Component | Effort | Complexity |
|-------|-----------|--------|------------|
| 1 | Relay Server Core | 2 days | High |
| 2 | Relay Protocol | 1 day | Medium |
| 3 | Update `RelaySession.send()` | 4 hours | Low |
| 4 | JSON-RPC Integration | 4 hours | Low |
| 5 | Unit Tests | 1 day | Medium |
| 6 | Integration Tests | 0.5 day | Medium |
| 7 | Documentation | 0.5 day | Low |
| **Total** | **~5 days** | **Medium-High** |

---

## 🎯 Success Criteria

| Criteria | Target | Verification |
|----------|--------|--------------|
| **Packet Forwarding** | <10ms latency | Integration tests |
| **Throughput** | >10 MB/s | Load tests |
| **Memory** | <1MB per 1000 sessions | Resource tests |
| **Binary Size** | <150KB impact | `cargo build --release` |
| **Test Coverage** | >80% | `cargo tarpaulin` |
| **Zero Unsafe** | 0 unsafe blocks | Clippy + manual audit |
| **Symmetric NAT** | Works | Real-world testing |

---

## 🔮 Future Enhancements

### Phase 5: ICE (Interactive Connectivity Establishment)

**Effort**: 1-2 weeks  
**Value**: High (industry-standard NAT traversal)

Implement full ICE protocol:
- Candidate gathering (host, server-reflexive, relayed)
- Connectivity checks
- Nomination
- Standards compliance (RFC 8445)

### Phase 6: Genetic Lineage Optimization

**Effort**: 1 week  
**Value**: Medium (sovereignty enhancement)

- Prefer ancestor relays (family-based routing)
- Reputation system for relay quality
- Load balancing across family members
- Relay failover

---

## 📚 References

1. **RFC 5766**: TURN - https://datatracker.ietf.org/doc/html/rfc5766
2. **RFC 8445**: ICE - https://datatracker.ietf.org/doc/html/rfc8445
3. **Existing Code**: `crates/songbird-lineage-relay/src/`
4. **STUN Implementation**: `crates/songbird-stun/src/server.rs`

---

## 🎊 Unique Differentiation

**Songbird's Lineage Relay vs Traditional TURN**:

| Feature | Traditional TURN | Songbird Lineage Relay |
|---------|-----------------|------------------------|
| Authorization | Username/password | Genetic lineage proof |
| Trust Model | Centralized server | Distributed family network |
| Privacy | Relay sees all traffic | Masking based on relationship |
| Deployment | External infrastructure | Any family member can relay |
| Cost | Server hosting | Mutual aid (family helps family) |
| Sovereignty | Infrastructure dependency | Self-sovereign |

---

**Status**: Ready for implementation  
**Next Step**: Create detailed specification and begin Phase 1

**Investigation Complete**: February 5, 2026
