# 🐻 BearDog Team: NAT Relay via Lineage - Implementation Handoff

**Date**: December 24, 2025  
**Priority**: P1 - Enables Internet-Wide Deployment  
**Timeline**: 7-9 weeks  
**Dependencies**: Genesis Ceremony Implementation

---

## 🎯 What BearDog Needs to Provide

### **Core Responsibility**: Trust & Cryptography for NAT Relay

Songbird handles coordination and networking. BearDog provides the **genetic cryptography** that makes it trustworthy and sovereign.

---

## 📦 Deliverables

### **1. Genesis Lineage Chain (Foundation)**

**What**: Establish cryptographic parent-child relationships during Genesis ceremony

**API Needed**:
```rust
#[async_trait]
pub trait GenesisLineage {
    /// Sign lineage relationship (parent → child)
    async fn sign_lineage(
        &self,
        parent_id: &NodeId,
        child_id: &NodeId,
        child_public_key: &PublicKey,
    ) -> Result<LineageProof>;
    
    /// Verify lineage proof
    async fn verify_lineage_proof(
        &self,
        proof: &LineageProof,
    ) -> Result<bool>;
}
```

**Example Flow**:
```rust
// During Genesis ceremony
let lineage_proof = beardog
    .sign_lineage(
        &parent.node_id,
        &new_node.node_id,
        &new_node.public_key
    )
    .await?;

// New node's identity includes lineage
let identity = NodeIdentity {
    node_id: new_node.node_id,
    public_key: new_node.public_key,
    lineage_proof,  // ← BearDog signed this!
    // ...
};
```

---

### **2. Lineage Graph Queries**

**What**: Allow nodes to query their lineage (ancestors, descendants)

**API Needed**:
```rust
#[async_trait]
pub trait LineageGraph {
    /// Get all ancestors of a node (parent, grandparent, ...)
    async fn get_ancestors(&self, node_id: &NodeId) -> Result<Vec<NodeId>>;
    
    /// Get all descendants of a node (children, grandchildren, ...)
    async fn get_descendants(&self, node_id: &NodeId) -> Result<Vec<NodeId>>;
    
    /// Verify if node_b is a descendant of node_a
    async fn is_descendant_of(
        &self,
        ancestor_id: &NodeId,
        descendant_id: &NodeId,
    ) -> Result<bool>;
    
    /// Get lineage depth (generations between ancestor and descendant)
    async fn get_lineage_depth(
        &self,
        ancestor_id: &NodeId,
        descendant_id: &NodeId,
    ) -> Result<u32>;
}
```

**Example Flow**:
```rust
// When NAT traversal fails, query for potential relays
let my_ancestors = beardog.get_ancestors(&my_node_id).await?;
// Returns: [parent_id, grandparent_id, great_grandparent_id, ...]

// Try each ancestor as potential relay
for ancestor_id in my_ancestors {
    let relay_request = RelayRequest {
        requester: my_node_id,
        target: peer_node_id,
    };
    // Broadcast to ancestor...
}
```

---

### **3. BirdSong Encryption/Decryption**

**What**: Encrypt broadcasts so only lineage can decrypt ("family hears song, others hear noise")

**API Needed**:
```rust
#[async_trait]
pub trait BirdSongCrypto {
    /// Encrypt message for specific lineage hint
    async fn encrypt_for_lineage(
        &self,
        message: &[u8],
        hint: LineageHint,
    ) -> Result<Vec<u8>>;
    
    /// Decrypt BirdSong message (only works if in lineage)
    async fn decrypt_birdsong(
        &self,
        encrypted: &[u8],
        my_identity: &NodeIdentity,
    ) -> Result<Option<Vec<u8>>>;  // None if not in lineage
}

pub enum LineageHint {
    DirectAncestors,      // Only my parent, grandparent, etc.
    DirectDescendants,    // Only my children
    AllDescendants,       // All descendants (any generation)
    SameGeneration,       // Siblings, cousins
}
```

**Example Flow**:
```rust
// Songbird wants to request relay from ancestor
let relay_request = serde_json::to_vec(&RelayRequest {
    requester: my_node_id,
    target: peer_node_id,
})?;

// BearDog encrypts for ancestors only
let encrypted = beardog
    .encrypt_for_lineage(&relay_request, LineageHint::DirectAncestors)
    .await?;

// Songbird broadcasts (only ancestors can decrypt)
birdsong.broadcast(encrypted).await?;

// Ancestor receives broadcast
let decrypted = beardog
    .decrypt_birdsong(&encrypted, &ancestor_identity)
    .await?;

if let Some(message) = decrypted {
    // This ancestor is in the lineage!
    let request: RelayRequest = serde_json::from_slice(&message)?;
    // Offer relay...
}
```

---

### **4. Relay Authority Service**

**What**: Ancestor nodes offer relay service to verified descendants

**API Needed**:
```rust
#[async_trait]
pub trait RelayAuthority {
    /// Check if node can offer relay (has resources, lineage depth, etc.)
    async fn can_offer_relay(&self, requester: &NodeId) -> Result<bool>;
    
    /// Establish relay session with masking level
    async fn offer_relay_session(
        &self,
        requester: &NodeId,
        target: &NodeId,
        masking: MaskingLevel,
    ) -> Result<RelaySession>;
    
    /// Verify relay request is from legitimate descendant
    async fn verify_relay_request(
        &self,
        request: &RelayRequest,
        proof: &LineageProof,
    ) -> Result<bool>;
}

pub enum MaskingLevel {
    Masked,        // Default: minimal metadata
    SubMasked,     // Some metadata visible to lineage
    Unmasked,      // Full visibility (ancestor privilege)
}

pub struct RelaySession {
    pub session_id: Uuid,
    pub requester: NodeId,
    pub target: NodeId,
    pub masking: MaskingLevel,
    pub relay_endpoint: SocketAddr,  // Where to send packets
}
```

**Example Flow**:
```rust
// Ancestor receives relay request via BirdSong
let request = decrypt_birdsong(&broadcast)?;

// Verify lineage
if beardog.verify_relay_request(&request, &request.lineage_proof).await? {
    // Check if can relay
    if beardog.can_offer_relay(&request.requester).await? {
        // Offer relay with masked identity
        let session = beardog.offer_relay_session(
            &request.requester,
            &request.target,
            MaskingLevel::Masked  // Default privacy
        ).await?;
        
        // Send relay offer back via BirdSong
        // ...
    }
}
```

---

### **5. Hardware Seeding (Bonus)**

**What**: Seed SoloKey, TPM, or other hardware for root of trust during Genesis

**API Needed** (optional, can come later):
```rust
#[async_trait]
pub trait HardwareSeed {
    /// Derive keys from hardware seed
    async fn derive_keys_from_hardware(
        &self,
        hardware_seed: &[u8],
        node_id: &NodeId,
    ) -> Result<KeyPair>;
    
    /// Verify hardware attestation
    async fn verify_hardware_attestation(
        &self,
        attestation: &[u8],
    ) -> Result<bool>;
}
```

---

## 🔄 Evolution Path

### **Phase 1: Basic Lineage (MVP)**
**Timeline**: 2-3 weeks

**Deliverables**:
1. Genesis lineage signing (parent → child)
2. Lineage proof verification
3. Simple ancestor query (get my parent)

**Goal**: Enable basic relay (parent can relay for child)

---

### **Phase 2: BirdSong Encryption**
**Timeline**: 2-3 weeks

**Deliverables**:
1. Lineage-derived encryption keys
2. Encrypt for DirectAncestors
3. Decrypt if in lineage

**Goal**: Privacy-preserving relay requests

---

### **Phase 3: Relay Authority**
**Timeline**: 2-3 weeks

**Deliverables**:
1. Relay session management
2. Masking level enforcement
3. Resource-aware relay offering

**Goal**: Production-ready relay service

---

### **Phase 4: Advanced Lineage (Future)**
**Timeline**: 3-4 weeks (later)

**Deliverables**:
1. Full lineage graph traversal
2. All LineageHint variants
3. Hardware seeding support
4. Audit and revocation

**Goal**: Enterprise-grade lineage system

---

## 📚 Reference Specifications

**Must Read**:
1. **[specs/LINEAGE_GATED_RELAY_PROTOCOL.md](specs/LINEAGE_GATED_RELAY_PROTOCOL.md)** - Complete LGRP spec
2. **[specs/BIRDSONG_PROTOCOL.md](specs/BIRDSONG_PROTOCOL.md)** - BirdSong encryption
3. **[specs/SONGBIRD_BEARDOG_INTEGRATION.md](specs/SONGBIRD_BEARDOG_INTEGRATION.md)** - Integration architecture
4. **[NAT_TRAVERSAL_VIA_LINEAGE.md](NAT_TRAVERSAL_VIA_LINEAGE.md)** - Complete integration guide

---

## 🎯 Success Criteria

### **Minimum Viable (Phase 1-2)**
- [ ] Parent-child lineage signing works
- [ ] Lineage proof verification works
- [ ] BirdSong encryption for ancestors works
- [ ] Ancestor can decrypt relay request

**Result**: Parent can relay for child behind NAT

---

### **Production Ready (Phase 1-3)**
- [ ] Full ancestor/descendant queries
- [ ] All LineageHint variants work
- [ ] Relay session management operational
- [ ] Masking levels enforced

**Result**: Multi-generation relay, privacy-preserving

---

## 🤝 Coordination with Songbird

**Songbird Will Provide**:
- STUN client (NAT type discovery)
- BirdSong broadcast system (uses your encryption)
- Relay session coordination (uses your authority)
- Genesis ceremony orchestration (calls your APIs)

**BearDog Provides**:
- Genetic cryptography (lineage signing/verification)
- BirdSong encryption/decryption (privacy)
- Relay authority (who can relay for whom)
- Trust model (no external servers)

**Integration Point**: Your APIs above ↑

---

## 💬 Questions for BearDog Team

1. **Timeline**: Can Phase 1-2 (MVP) be ready in 4-6 weeks?
2. **Genesis First**: Is Genesis ceremony implementation prioritized?
3. **API Design**: Any concerns with the API shapes above?
4. **Hardware Seeding**: Do you want this in MVP or later phase?
5. **Testing**: Need mock implementations from us for your testing?

---

## 📞 Contact

**Songbird Team**: Ready to integrate once your APIs are available  
**Slack**: #beardog-songbird-integration  
**Specs**: All in `specs/` directory  
**Handoff**: This document + references above

---

## ✅ Summary for BearDog Team

**What You Provide**: Genetic crypto, lineage, BirdSong encryption, relay authority  
**What Songbird Does**: Networking, coordination, broadcasting, session management  
**Timeline**: 7-9 weeks total (phased)  
**Priority**: High - enables internet-wide deployment  
**Result**: True P2P NAT traversal with zero external trust

**Next Step**: Review specs, confirm API design, start Genesis lineage implementation!

🐻 **BearDog** provides the security. 🎵 **Songbird** provides the coordination.  
Together: **Sovereign P2P networking!** 🌳

