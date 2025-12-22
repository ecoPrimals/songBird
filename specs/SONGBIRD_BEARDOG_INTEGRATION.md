# Songbird-BearDog Integration Specification

**Version**: 1.0  
**Date**: December 21, 2025  
**Status**: Active Development  
**Joint Effort**: Songbird (Orchestration) + BearDog (Security)

---

## 🎯 Executive Summary

### The Transformation

**Songbird Alone:**
- ✅ Works for trusted LANs (university campus, research lab)
- ✅ Plaintext discovery (fast, simple)
- ✅ Direct connections (no NAT traversal)
- ❌ Limited to trusted networks
- ❌ No privacy from network observers
- ❌ Cannot work across internet

**Songbird + BearDog:**
- ✅ Works across untrusted networks (internet, mobile, roaming)
- ✅ Encrypted discovery (birdSong protocol)
- ✅ Lineage-gated relays (NAT traversal without TURN servers)
- ✅ Privacy-preserving (non-family sees noise)
- ✅ Sovereign (no central authority)
- ✅ True peer-to-peer

**Key Insight**: BearDog transforms Songbird from "trusted LAN orchestrator" to "sovereign P2P federation platform."

---

## 🔐 Joint Responsibility Model (BTSP)

### BearDog Responsibilities

**Genetics & Lineage:**
- Generate and maintain lineage chains
- Sign parent-child relationships
- Provide cryptographic lineage proofs
- Verify lineage claims
- Maintain lineage graph

**Encryption & Keys:**
- Derive keys from lineage
- Encrypt birdSong payloads
- Decrypt birdSong (if in lineage)
- Distribute keys to descendants
- Rotate and revoke keys

**Relay Authority:**
- Offer relay service to descendants
- Verify relay requests via lineage
- Enforce masking rules (metadata visibility)
- Audit relay usage
- Revoke relay privileges

### Songbird Responsibilities

**Broadcasting (BirdSong):**
- Structure birdSong messages
- Broadcast encrypted UDP messages
- Manage broadcast frequency/timing
- Handle message queuing

**Discovery & Coordination:**
- Request encryption from BearDog
- Decrypt received birdSong (using BearDog keys)
- Maintain discovered nodes
- Coordinate connections
- Handle failover

**Relay Usage:**
- Detect NAT traversal failures
- Request relay from ancestors (via birdSong)
- Manage relay sessions
- Monitor relay performance
- Upgrade to direct connection when possible

---

## 🎵 Core Protocol: BirdSong

### Concept

**Metaphor**: "A broadcast that is obvious to family and noise otherwise"

**Like Nature:**
- Birds sing in patterns their species recognizes
- Others hear sound but not meaning
- Family knows the song, strangers hear noise

**In Practice:**
1. Songbird creates payload (presence, capability, etc.)
2. Songbird requests encryption from BearDog
3. BearDog encrypts with lineage-derived key
4. Songbird broadcasts encrypted birdSong (UDP)
5. Family nodes request key from BearDog
6. BearDog verifies lineage, provides key
7. Family decrypts and processes message
8. Non-family: sees encrypted noise, learns nothing

### Privacy Properties

**Non-Family Cannot See:**
- ❌ Node identities
- ❌ Node names
- ❌ Capabilities
- ❌ Endpoints
- ❌ Message types
- ❌ Federation topology

**Family Can See (with lineage proof):**
- ✅ All of the above
- ✅ Auto-discovery within lineage
- ✅ Zero configuration

### Message Types

```rust
enum BirdSongType {
    Presence,                // I'm here and alive
    CapabilityAnnouncement, // I have these capabilities
    TransportAnnouncement,  // Reach me via these endpoints
    RelayRequest,           // I need relay assistance
    FederationEvent,        // Something changed
    Custom(String),         // Application-specific
}
```

---

## 🧬 Core Protocol: Lineage-Gated Relay

### Concept

**Traditional TURN**: "Do I trust this server?"  
**Lineage-Gated**: "Does this node descend from me?"

**Transformation:**
- ❌ Central TURN servers → ✅ Any ancestor can relay
- ❌ Infrastructure trust → ✅ Cryptographic lineage
- ❌ Monetized service → ✅ Voluntary family service
- ❌ Permanent observation → ✅ Temporary, masked relay

### Visibility Matrix

| Relationship | Visibility | Metadata | Revocation |
|-------------|-----------|----------|------------|
| Parent → Child | Full | Full | Yes |
| Ancestor → Descendant | Configurable | Selective | Yes |
| Child → Parent | Minimal | None | No |
| Siblings | None | None | No |

**Key Property**: Visibility flows downward, never sideways

### Masking Layers

**Layer 0: Transport (Always Opaque)**
- Relay sees: Packet size, timing
- Relay never sees: Payload, keys, identities

**Layer 1: Masked Identity (Default)**
- Relay sees: Ephemeral relay IDs
- Relay never sees: Stable node IDs, topology

**Layer 2: Sub-Mask Access (Lineage-Gated)**
- If lineage proven: Stable node ID, network hints
- Selective metadata disclosure
- NOT payload decryption

**Layer 3: Full Visibility (Ancestor Only)**
- Can see through descendant masks
- Can audit routing
- Can revoke privileges
- Can enforce policy

---

## 📡 API Contracts

### BearDog → Songbird APIs

#### 1. Lineage Management

```rust
/// BearDog provides lineage services to Songbird
#[async_trait]
pub trait LineageProvider {
    /// Generate lineage for a new node
    async fn generate_lineage(
        &self,
        node_id: &str,
        parent_id: &str,
    ) -> Result<LineageChain>;
    
    /// Verify a lineage proof
    async fn verify_lineage(
        &self,
        proof: &LineageProof,
    ) -> Result<bool>;
    
    /// Get all descendants of a root
    async fn get_descendants(
        &self,
        root_id: &str,
    ) -> Result<Vec<String>>;
    
    /// Get lineage depth between two nodes
    async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> Result<usize>;
}
```

#### 2. BirdSong Encryption

```rust
/// BearDog provides birdSong encryption/decryption
#[async_trait]
pub trait BirdSongCrypto {
    /// Encrypt payload for a specific lineage
    async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> Result<EncryptedBirdSong>;
    
    /// Decrypt birdSong (if we're in the lineage)
    async fn decrypt_birdsong(
        &self,
        encrypted: &EncryptedBirdSong,
    ) -> Result<Vec<u8>>;
    
    /// Request decryption key for a lineage
    async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> Result<BroadcastKey>;
    
    /// Batch key request (for multiple lineages)
    async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> Result<Vec<BroadcastKey>>;
}
```

#### 3. Lineage-Gated Relay

```rust
/// BearDog provides relay services based on lineage
#[async_trait]
pub trait LineageRelay {
    /// Offer relay service to descendant
    async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> Result<RelaySession>;
    
    /// Get visibility level based on lineage depth
    fn get_visibility_level(
        &self,
        lineage_depth: usize,
    ) -> AccessLevel;
    
    /// Relay packet (with masking enforced)
    async fn relay_packet(
        &self,
        session: &RelaySession,
        packet: &[u8],
    ) -> Result<()>;
    
    /// Revoke relay for a session
    async fn revoke_relay(
        &self,
        session_id: &str,
    ) -> Result<()>;
}
```

### Songbird → BearDog APIs (Callbacks)

```rust
/// Songbird provides feedback to BearDog
#[async_trait]
pub trait SongbirdFeedback {
    /// Notify BearDog of successful decryption
    async fn decryption_success(
        &self,
        lineage_hint: &LineageHint,
    ) -> Result<()>;
    
    /// Notify BearDog of failed decryption
    async fn decryption_failure(
        &self,
        lineage_hint: &LineageHint,
        reason: &str,
    ) -> Result<()>;
    
    /// Report relay performance
    async fn relay_metrics(
        &self,
        session_id: &str,
        metrics: RelayMetrics,
    ) -> Result<()>;
}
```

---

## 🔄 Integration Flows

### Flow 1: BirdSong Discovery

```
┌─────────────┐          ┌──────────┐          ┌─────────────┐
│  Songbird A │          │ BearDog  │          │ Songbird B  │
└─────┬───────┘          └────┬─────┘          └──────┬──────┘
      │                       │                       │
      │ 1. Create payload     │                       │
      │   (presence)          │                       │
      │                       │                       │
      │ 2. Encrypt for lineage│                       │
      ├──────────────────────>│                       │
      │                       │                       │
      │ 3. Encrypted birdSong │                       │
      │<──────────────────────┤                       │
      │                       │                       │
      │ 4. Broadcast (UDP)    │                       │
      ├──────────────────────────────────────────────>│
      │                       │                       │
      │                       │ 5. Request key        │
      │                       │<──────────────────────┤
      │                       │                       │
      │                       │ 6. Verify lineage     │
      │                       │                       │
      │                       │ 7. Provide key        │
      │                       ├──────────────────────>│
      │                       │                       │
      │                       │ 8. Decrypt & process  │
      │                       │                       │
```

### Flow 2: Lineage-Gated Relay

```
┌─────────────┐     ┌──────────┐     ┌───────────┐     ┌─────────────┐
│ Songbird A  │     │ BearDog  │     │ Ancestor  │     │ Songbird B  │
│ (behind NAT)│     │          │     │  (relay)  │     │ (behind NAT)│
└─────┬───────┘     └────┬─────┘     └─────┬─────┘     └──────┬──────┘
      │                  │                  │                  │
      │ 1. NAT traversal fails              │                  │
      │                  │                  │                  │
      │ 2. Create relay request             │                  │
      │    (birdSong)    │                  │                  │
      │                  │                  │                  │
      │ 3. Encrypt for lineage              │                  │
      ├─────────────────>│                  │                  │
      │                  │                  │                  │
      │ 4. Encrypted request                │                  │
      │<─────────────────┤                  │                  │
      │                  │                  │                  │
      │ 5. Broadcast relay request          │                  │
      ├─────────────────────────────────────>│                  │
      │                  │                  │                  │
      │                  │ 6. Decrypt (ancestor can hear)      │
      │                  │<─────────────────┤                  │
      │                  │                  │                  │
      │                  │ 7. Verify lineage│                  │
      │                  ├─────────────────>│                  │
      │                  │                  │                  │
      │                  │ 8. Offer relay   │                  │
      │                  │<─────────────────┤                  │
      │                  │                  │                  │
      │ 9. Relay established                │                  │
      │<────────────────────────────────────┤──────────────────>│
      │                  │                  │                  │
      │ 10. Communication via relay         │                  │
      │<────────────────────────────────────┼──────────────────>│
      │                  │                  │                  │
```

---

## 🎭 Showcase Demonstrations

### Demo 1: Privacy Comparison
**Location**: `showcase/13-beardog-integration/01-privacy-comparison.sh`

**Scenario**: Same federation, with and without BearDog

**Without BearDog:**
```bash
# Plaintext UDP broadcast
# Outside observer can see:
# - Node IDs
# - Capabilities
# - Endpoints
# - Full topology
```

**With BearDog:**
```bash
# Encrypted birdSong
# Outside observer sees:
# - Encrypted UDP traffic
# - Packet sizes and timing
# - Cannot decrypt content
# - Learns nothing
```

**Result**: Demonstrates privacy gain

### Demo 2: Private Federation Discovery
**Location**: `showcase/13-beardog-integration/02-private-discovery.sh`

**Scenario**: University federation, outsider eavesdropping

**Steps:**
1. Start 3 university nodes (same lineage)
2. Start 1 outsider node (different lineage)
3. University nodes discover each other via birdSong
4. Outsider sees encrypted traffic, cannot decrypt
5. University federation forms, outsider excluded

**Result**: Privacy-preserving discovery

### Demo 3: Lineage-Gated Relay
**Location**: `showcase/13-beardog-integration/03-lineage-relay.sh`

**Scenario**: Two nodes behind NAT, ancestor helps

**Steps:**
1. Start ancestor node (public IP)
2. Start two descendant nodes (behind NAT)
3. Descendants broadcast relay request via birdSong
4. Ancestor hears request (lineage verified)
5. Ancestor offers relay service
6. Descendants connect through ancestor
7. Minimal metadata exposed (masking enforced)

**Result**: NAT traversal without central TURN server

### Demo 4: Roaming Device
**Location**: `showcase/13-beardog-integration/04-roaming-device.sh`

**Scenario**: Laptop moves from WiFi to cellular

**Steps:**
1. Laptop joins federation on university WiFi
2. Laptop switches to cellular network
3. BearDog preserves lineage & trust
4. Songbird detects network change
5. Songbird re-establishes connections
6. Federation membership maintained

**Result**: Connection migration with trust persistence

---

## 📊 Implementation Timeline

### Phase 1: BearDog Lineage Foundation (4-6 weeks)
**BearDog Team Delivers:**
- Lineage chain structure
- Parent-child signing
- Lineage proof protocol
- Verification logic
- `LineageProvider` trait implementation

### Phase 2: BearDog BirdSong Encryption (2-3 weeks)
**BearDog Team Delivers:**
- Key derivation from lineage
- `BirdSongCrypto` trait implementation
- Key distribution mechanism
- Key caching and rotation

### Phase 3: Songbird BirdSong Integration (2-3 weeks)
**Songbird Team Delivers:**
- BirdSong message structures
- Replace plaintext discovery with birdSong
- Key request flow
- Backward compatibility (parallel mode)

### Phase 4: Lineage-Gated Relay (4-6 weeks)
**Both Teams Deliver:**
- BearDog: `LineageRelay` trait implementation
- Songbird: Relay request via birdSong
- Masking layer enforcement
- End-to-end relay tests

### Phase 5: Joint Testing & Deployment (2-3 weeks)
**Both Teams Deliver:**
- Integration test suite
- Privacy verification tests
- Performance benchmarks
- Showcase demonstrations
- Production deployment

**Total Timeline**: 14-20 weeks (~3.5-5 months)

---

## 🎯 Success Criteria

### Functional Requirements
- ✅ BearDog generates lineage chains
- ✅ BearDog encrypts/decrypts birdSong
- ✅ Songbird broadcasts encrypted birdSong
- ✅ Family nodes can decrypt (with lineage proof)
- ✅ Non-family nodes cannot decrypt
- ✅ Lineage-gated relay operational
- ✅ Backward compatibility maintained

### Privacy Requirements
- ✅ No plaintext broadcasts on untrusted networks
- ✅ No topology leakage to non-family
- ✅ No node identity exposure
- ✅ Minimal metadata in relay (masking enforced)
- ✅ Cannot forge lineage proofs
- ✅ Cannot unmask siblings

### Performance Requirements
- ✅ Key caching: < 10ms lookup
- ✅ Encryption overhead: < 5ms per message
- ✅ Decryption: Batch processing support
- ✅ Relay latency: < 50ms overhead
- ✅ Scales to 1000+ node federation

### Security Requirements
- ✅ Cryptographically secure lineage proofs
- ✅ Key rotation every 30 days
- ✅ Ancestor revocation functional
- ✅ Replay attack prevention (timestamps)
- ✅ No forward secrecy compromise

---

## 📚 Reference Specifications

### Core Documents
1. `specs/BIRDSONG_PROTOCOL.md` - Complete birdSong specification
2. `specs/LINEAGE_GATED_RELAY_PROTOCOL.md` - Relay protocol
3. `specs/PRIMAL_RESPONSIBILITY_SEPARATION_SPEC.md` - Role boundaries
4. `BEARDOG_BTSP_HANDOFF.md` - Integration guide
5. `BEARDOG_TEAM_BLURB.md` - High-level overview

### Related Specs
6. `specs/RENDEZVOUS_PROTOCOL_SPEC.md` - Internet discovery
7. `docs/PRIVACY_FIRST_FEDERATION.md` - Privacy architecture
8. `INTERNET_DEPLOYMENT_ROADMAP.md` - Overall timeline

---

## 🤝 Coordination

### Communication Channels
- **Primary**: Joint Slack/Discord channel
- **Documentation**: Shared specs/ directory
- **Code**: Cross-repo integration tests

### Meeting Cadence
- **Weekly**: Integration sync (1 hour)
- **Bi-weekly**: Technical deep dive (2 hours)
- **Monthly**: Roadmap review (1 hour)

### Decision Making
- **API Design**: Joint approval required
- **Protocol Changes**: Joint approval required
- **Internal Implementation**: Team autonomy
- **Showcase Demos**: Joint development

---

## 🎉 The Vision

**Songbird Today**: Trusted LAN orchestrator  
**Songbird + BearDog**: True P2P platform

**Capabilities Unlocked:**
- ✅ Internet deployment (not just LAN)
- ✅ Privacy-preserving (encrypted discovery)
- ✅ Sovereign (no central servers)
- ✅ Scalable (lineage-based trust)
- ✅ Mobile-ready (roaming support)
- ✅ Anti-capture (biological trust model)

**Together, we build the future of sovereign, privacy-preserving, peer-to-peer federation.** 🎵🐻🔒🧬✨

---

**Document Version**: 1.0  
**Date**: December 21, 2025  
**Status**: Active Specification  
**Next Review**: After Phase 1 completion

