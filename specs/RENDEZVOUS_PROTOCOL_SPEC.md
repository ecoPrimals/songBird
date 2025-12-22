# Rendezvous Protocol Specification - Phase 2

**Version**: 1.0.0  
**Date**: December 21, 2025  
**Status**: 🟡 **DESIGN** - Ready for Implementation  
**Dependencies**: Phase 1 (BTSP) Complete ✅

---

## 🎯 Executive Summary

This specification defines a **privacy-first rendezvous protocol** for Songbird federation across the internet, enabling node discovery and connection coordination without exposing IP addresses publicly.

### Core Principles

1. **Privacy First**: IPs treated as private identifiers, never exposed publicly
2. **Zero Trust**: End-to-end encryption, rendezvous can't read content
3. **Capability-Based**: Nodes discover each other by capability, not name
4. **Signed Communications**: All messages cryptographically signed (via BearDog)
5. **Ephemeral Sessions**: Session IDs rotate frequently
6. **Graceful Degradation**: Works with or without BearDog (different security levels)

---

## 🏗️ Architecture

### Components

```
┌─────────────────────┐         ┌───────────────────┐         ┌─────────────────────┐
│   Mobile Device     │         │  Rendezvous       │         │   Home Network      │
│   (Roaming)         │◄───────►│  Server           │◄───────►│   (Static)          │
│                     │         │  (Coordinator)     │         │                     │
│  Songbird + BearDog │         │                    │         │  Songbird + BearDog │
└─────────────────────┘         └───────────────────┘         └─────────────────────┘
         │                               │                               │
         │                               │                               │
         └───────── Encrypted ───────────┴───────── Encrypted ──────────┘
           (Rendezvous can't read)
```

### Responsibilities

**Rendezvous Server:**
- Register node presence (ephemeral sessions)
- Route connection requests (coordination only)
- Verify signatures (prevent spoofing)
- Rate limiting and abuse prevention
- NO access to IP addresses (nodes connect directly after coordination)
- NO access to message content (end-to-end encrypted)

**Songbird (Orchestration):**
- Register with rendezvous
- Query for peers by capability
- Coordinate connection attempts
- Maintain ephemeral session IDs

**BearDog (Security):**
- Sign all rendezvous messages
- Verify peer signatures
- Establish encrypted tunnels
- TOFU (Trust On First Use) for new peers

---

## 📋 Protocol Messages

### Message 1: Register Presence

**Direction**: Node → Rendezvous  
**Frequency**: Every 30-60 seconds (heartbeat)  
**Purpose**: Announce presence without revealing IP

```json
{
  "message_type": "register_presence",
  "version": "1.0",
  "timestamp": "2025-12-21T23:45:00Z",
  
  "node_identity": {
    "node_id": "550e8400-e29b-41d4-a716-446655440000",
    "ephemeral_session_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "public_key_fingerprint": "sha256:a3b5c7d9...",
    "capabilities": ["orchestration", "federation", "compute"],
    "protocols": ["https", "btsp", "tarpc"]
  },
  
  "network_context": {
    "nat_type": "cone", // or "symmetric", "open", "unknown"
    "reachability": "direct", // or "relayed", "unknown"
    "connection_quality": "excellent" // or "good", "poor", "unknown"
  },
  
  "security": {
    "signature": "..." // BearDog signs entire message
  }
}
```

**Privacy Notes:**
- ❌ NO IP address in message
- ✅ Ephemeral session ID (rotates every 10-15 minutes)
- ✅ Public key fingerprint (not full key)
- ✅ Capabilities (what can do, not who)

**Rendezvous Response:**
```json
{
  "status": "registered",
  "session_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "expires_at": "2025-12-21T23:46:00Z",
  "rendezvous_endpoint": "wss://rendezvous.example.com/session/abc123"
}
```

### Message 2: Query for Peers

**Direction**: Node → Rendezvous  
**Purpose**: Find peers with specific capabilities

```json
{
  "message_type": "query_peers",
  "version": "1.0",
  "timestamp": "2025-12-21T23:45:10Z",
  
  "requester": {
    "session_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "signature": "..." // BearDog signature
  },
  
  "query": {
    "capabilities_required": ["orchestration", "federation"],
    "capabilities_optional": ["compute"],
    "exclude_node_ids": ["550e8400-..."], // Don't return self
    "max_results": 10
  },
  
  "filters": {
    "connection_quality_min": "good",
    "prefer_direct_connections": true
  }
}
```

**Rendezvous Response:**
```json
{
  "peers": [
    {
      "ephemeral_session_id": "8d1f7890-...",
      "public_key_fingerprint": "sha256:b4c6d8e0...",
      "capabilities": ["orchestration", "federation", "compute"],
      "protocols": ["https", "btsp"],
      "network_context": {
        "nat_type": "cone",
        "reachability": "direct"
      },
      "last_heartbeat": "2025-12-21T23:45:05Z"
    }
  ],
  "total_matches": 15,
  "returned": 10
}
```

**Privacy Notes:**
- ❌ NO IP addresses disclosed
- ❌ NO node names (only capabilities)
- ✅ Ephemeral session IDs only
- ✅ Public key fingerprints for verification

### Message 3: Request Connection

**Direction**: Node A → Rendezvous → Node B  
**Purpose**: Coordinate peer-to-peer connection

```json
{
  "message_type": "request_connection",
  "version": "1.0",
  "timestamp": "2025-12-21T23:45:20Z",
  
  "requester": {
    "session_id": "7c9e6679-...",
    "signature": "..." // BearDog signature
  },
  
  "target": {
    "session_id": "8d1f7890-..."
  },
  
  "connection_intent": {
    "purpose": "federation_join",
    "required_protocols": ["https", "btsp"],
    "ice_candidates": [ // for NAT traversal
      {
        "type": "host",
        "priority": 126976767,
        "address_encrypted": "..." // Encrypted with target's pubkey
      },
      {
        "type": "srflx", // Server reflexive (STUN)
        "priority": 2130706431,
        "address_encrypted": "..."
      }
    ]
  }
}
```

**Rendezvous Action:**
1. Verify requester signature
2. Check target is online
3. Forward request to target (via WebSocket or long-poll)
4. Return coordination token

**Rendezvous Response (to requester):**
```json
{
  "status": "forwarded",
  "coordination_token": "abc123def456",
  "relay_endpoint": "wss://rendezvous.example.com/relay/abc123" // Fallback
}
```

**Rendezvous Forward (to target):**
```json
{
  "message_type": "connection_request_received",
  "from": {
    "session_id": "7c9e6679-...",
    "public_key_fingerprint": "sha256:a3b5c7d9...",
    "capabilities": ["orchestration", "federation"],
    "signature": "..." // Original signature for verification
  },
  "ice_candidates_encrypted": [...],
  "coordination_token": "abc123def456"
}
```

### Message 4: Accept/Reject Connection

**Direction**: Node B → Rendezvous → Node A

```json
{
  "message_type": "connection_response",
  "version": "1.0",
  "timestamp": "2025-12-21T23:45:25Z",
  
  "responder": {
    "session_id": "8d1f7890-...",
    "signature": "..."
  },
  
  "coordination_token": "abc123def456",
  
  "decision": "accept", // or "reject", "later"
  
  "response_data": {
    "ice_candidates": [...], // Encrypted with requester's pubkey
    "btsp_ready": true,
    "preferred_protocol": "btsp"
  }
}
```

**Rendezvous Action:**
- Forward response to original requester
- Provide relay endpoint if direct connection fails
- Close coordination after 60 seconds

### Message 5: Direct Connection Established

**Direction**: Both nodes → Rendezvous  
**Purpose**: Notify rendezvous that direct connection succeeded

```json
{
  "message_type": "connection_established",
  "coordination_token": "abc123def456",
  "timestamp": "2025-12-21T23:45:30Z",
  "connection_type": "direct", // or "relayed"
  "signature": "..."
}
```

**Rendezvous Action:**
- Mark coordination as complete
- Release relay resources (if any)
- Update statistics

---

## 🔒 Security Model

### Threat Model

**Assumptions:**
- ✅ Rendezvous server is **honest but curious** (follows protocol, but logs everything)
- ✅ Network is **hostile** (MITM, eavesdropping, tampering)
- ✅ Some nodes may be **malicious** (spam, DOS, impersonation)

**Protections:**

| Threat | Mitigation |
|--------|-----------|
| Rendezvous learns IPs | ✅ Nodes connect directly after coordination |
| Rendezvous reads messages | ✅ End-to-end encryption (BTSP) |
| IP exposure in ICE candidates | ✅ Encrypted with target's public key |
| Node impersonation | ✅ All messages signed by BearDog |
| Session hijacking | ✅ Ephemeral session IDs, short TTLs |
| DOS attacks | ✅ Rate limiting, signature verification |
| Traffic analysis | ⚠️ Mitigated but not eliminated (use Tor if needed) |

### Trust Levels

**Level 0: Anonymous Discovery**
- No BearDog available
- Uses ephemeral session IDs only
- Basic capability matching
- ⚠️ Lower security (suitable for public demo nodes)

**Level 1: Capability-Verified** (BearDog Available)
- All messages signed
- Public key fingerprints verified
- Capability-based authorization
- ✅ Recommended minimum

**Level 2: Trust-On-First-Use (TOFU)**
- First connection pins peer's public key
- Future connections verify against pin
- Prevents MITM on subsequent connections
- ✅ Recommended for personal networks

**Level 3: Out-of-Band Verified**
- Public keys exchanged via QR code, secure channel, etc.
- Highest assurance
- ✅ Recommended for sensitive deployments

### Signature Verification

**Without BearDog (Graceful Degradation):**
```rust
// No signatures, rely on TLS only
// Suitable for testing and public networks
// Lower security, but still functional
```

**With BearDog (Full Security):**
```rust
// Message signing
let message = serde_json::to_vec(&register_msg)?;
let signature = beardog.sign_message(&message).await?;

// Message verification (rendezvous side)
let public_key = get_public_key_for_session(session_id).await?;
beardog.verify_signature(&message, &signature, &public_key).await?;
```

---

## 🚀 Implementation Plan

### Phase 2.1: Rendezvous Server (Standalone)

**Technology:** Rust + Axum + WebSockets  
**Timeline:** 1 week

```
rendezvous/
├── src/
│   ├── main.rs
│   ├── registry.rs      // Session management
│   ├── coordination.rs  // Connection coordination
│   ├── relay.rs         // TURN-like relay (fallback)
│   └── security.rs      // Signature verification
├── Cargo.toml
└── README.md
```

**Key Features:**
- Session registry (ephemeral IDs → metadata)
- WebSocket server for real-time coordination
- HTTP API for queries
- Rate limiting (per IP, per session)
- Relay fallback for failed direct connections

### Phase 2.2: Songbird Integration

**Timeline:** 1 week

**File**: `crates/songbird-network-federation/src/rendezvous/mod.rs` (new)

```rust
pub mod client;
pub mod messages;
pub mod coordination;

use crate::btsp::BtspProvider;

pub struct RendezvousClient {
    server_url: String,
    btsp_provider: Option<Arc<dyn BtspProvider>>,
    session_id: Option<String>,
}

impl RendezvousClient {
    /// Register presence with rendezvous server
    pub async fn register_presence(
        &mut self,
        node_info: &NodeRegistration,
    ) -> Result<RendezvousSession> {
        // Build registration message
        let msg = RegisterPresenceMessage {
            node_identity: NodeIdentity {
                node_id: node_info.node_id.clone(),
                ephemeral_session_id: Uuid::new_v4().to_string(),
                public_key_fingerprint: self.get_pubkey_fingerprint().await?,
                capabilities: node_info.capabilities.clone(),
                protocols: vec!["https".to_string(), "btsp".to_string()],
            },
            network_context: self.detect_network_context().await?,
            security: self.sign_message(&msg).await?,
        };
        
        // Send to rendezvous
        let response = self.http_client
            .post(&format!("{}/api/v1/register", self.server_url))
            .json(&msg)
            .send()
            .await?;
        
        let session: RendezvousSession = response.json().await?;
        self.session_id = Some(session.session_id.clone());
        
        Ok(session)
    }
    
    /// Query for peers with capabilities
    pub async fn query_peers(
        &self,
        capabilities: Vec<String>,
    ) -> Result<Vec<PeerInfo>> {
        // Build query
        let query = QueryPeersMessage {
            requester: self.session_info().await?,
            query: PeerQuery {
                capabilities_required: capabilities,
                max_results: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        
        // Send query
        let response = self.http_client
            .post(&format!("{}/api/v1/query", self.server_url))
            .json(&query)
            .send()
            .await?;
        
        let peers: PeerQueryResponse = response.json().await?;
        Ok(peers.peers)
    }
    
    /// Request connection to peer
    pub async fn request_connection(
        &self,
        target_session_id: &str,
    ) -> Result<ConnectionCoordination> {
        // Collect ICE candidates
        let ice_candidates = self.gather_ice_candidates().await?;
        
        // Encrypt candidates with target's public key
        let target_pubkey = self.get_peer_pubkey(target_session_id).await?;
        let encrypted_candidates = self.encrypt_for_peer(
            &ice_candidates,
            &target_pubkey,
        ).await?;
        
        // Build request
        let request = RequestConnectionMessage {
            requester: self.session_info().await?,
            target: TargetInfo {
                session_id: target_session_id.to_string(),
            },
            connection_intent: ConnectionIntent {
                purpose: "federation_join".to_string(),
                required_protocols: vec!["btsp".to_string()],
                ice_candidates: encrypted_candidates,
            },
        };
        
        // Send request
        let response = self.http_client
            .post(&format!("{}/api/v1/connect", self.server_url))
            .json(&request)
            .send()
            .await?;
        
        Ok(response.json().await?)
    }
}
```

### Phase 2.3: BearDog Integration

**Timeline:** 3 days (after BearDog BTSP implementation)

```rust
// Message signing
impl RendezvousClient {
    async fn sign_message(&self, msg: &impl Serialize) -> Result<Signature> {
        if let Some(btsp) = &self.btsp_provider {
            let data = serde_json::to_vec(msg)?;
            Ok(btsp.sign(&data).await?)
        } else {
            // Graceful degradation: no signature
            Ok(Signature::None)
        }
    }
    
    async fn verify_peer_signature(
        &self,
        msg: &[u8],
        signature: &Signature,
        peer_pubkey_fingerprint: &str,
    ) -> Result<bool> {
        if let Some(btsp) = &self.btsp_provider {
            btsp.verify(msg, signature, peer_pubkey_fingerprint).await
        } else {
            // No verification without BearDog
            Ok(true)
        }
    }
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```bash
# Rendezvous server
cd rendezvous
cargo test

# Songbird integration
cd crates/songbird-network-federation
cargo test rendezvous
```

### Integration Tests

**Test 1: Registration**
```bash
./showcase/12-internet-deployment/01-rendezvous-registration-test.sh
```

**Test 2: Peer Discovery**
```bash
./showcase/12-internet-deployment/02-peer-discovery-test.sh
```

**Test 3: Connection Coordination**
```bash
./showcase/12-internet-deployment/03-connection-coordination-test.sh
```

### E2E Tests

**Scenario**: Mobile device discovers and connects to home network
```bash
./showcase/12-internet-deployment/04-mobile-to-home-test.sh
```

---

## 📊 Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Registration latency | < 100ms | Single HTTP request |
| Query latency | < 200ms | Database lookup + filtering |
| Connection coordination | < 500ms | WebSocket or long-poll |
| Session TTL | 60 seconds | Heartbeat every 30s |
| Max concurrent sessions | 10,000+ | Per rendezvous instance |
| Relay bandwidth | 10 Mbps | Per session (fallback only) |

---

## 🔗 Related Specifications

- `specs/PRIMAL_RESPONSIBILITY_SEPARATION_SPEC.md` - Songbird vs BearDog
- `INTERNET_DEPLOYMENT_ROADMAP.md` - Overall roadmap
- `docs/PRIVACY_FIRST_FEDERATION.md` - Privacy principles
- `docs/BTSP_INTERFACE_GUIDE.md` - BTSP details

---

## 📝 Open Questions

1. **Rendezvous Server Hosting**:
   - Self-hosted (recommended for sovereignty)?
   - Public instance for convenience?
   - Both (self-host preferred, public fallback)?

2. **Session ID Rotation**:
   - 10 minutes (more privacy)?
   - 15 minutes (less overhead)?
   - User-configurable?

3. **Relay Bandwidth Limits**:
   - Free tier: 10 MB/day?
   - Paid tier: unlimited?
   - No relay (direct only)?

4. **Rate Limiting**:
   - Per IP: 100 requests/minute?
   - Per session: 10 connections/minute?
   - Adaptive based on behavior?

---

## ✅ Success Criteria

### Phase 2.1: Rendezvous Server
- [ ] HTTP API for registration, query, connection
- [ ] WebSocket server for real-time coordination
- [ ] Session management (register, heartbeat, expire)
- [ ] Signature verification (with BearDog support)
- [ ] Rate limiting
- [ ] Relay fallback (basic TURN)

### Phase 2.2: Songbird Integration
- [ ] `RendezvousClient` implementation
- [ ] Registration with rendezvous
- [ ] Peer discovery by capability
- [ ] Connection coordination
- [ ] ICE candidate handling
- [ ] Integration with existing federation

### Phase 2.3: E2E Testing
- [ ] Mobile device discovers home network
- [ ] Direct P2P connection established
- [ ] Fallback to relay if direct fails
- [ ] Connection migrates on network change

---

**Status**: Design Complete, Ready for Implementation  
**Dependencies**: Phase 1 (BTSP) ✅  
**Timeline**: ~2-3 weeks for Phase 2.1-2.2  
**Blocker**: None (can start immediately)

*Privacy-first, zero-trust, internet-ready! 🌍🔒*

