# 🧬 BearDog Lineage Relay Handoff

**To**: BearDog Security Team  
**From**: Songbird Coordination Team  
**Date**: December 24, 2025  
**Priority**: P1 - Next Major Feature

---

## 🎯 What BearDog Needs to Deliver

### **Core Mission**
Build the **Genetic Lineage Relay System** that replaces outdated NAT/STUN/TURN infrastructure with cryptographic lineage trust.

**Vision**: Nodes relay for their descendants - no external infrastructure needed.

---

## 🧬 Required Capabilities

### **1. Genesis Lineage Establishment** (P0 - Foundation)

**What**: Every node born through Genesis gets cryptographic lineage chain.

**Deliverables**:
```rust
// Genesis creates lineage
pub struct LineageChain {
    node_id: NodeId,
    parent: Option<NodeId>,      // Who performed genesis
    ancestors: Vec<NodeId>,       // Full chain to root
    birth_signature: Signature,   // Signed by parent
    birth_timestamp: SystemTime,
    witnesses: Vec<WitnessProof>,
}

// BearDog API
pub trait GenesisLineageProvider {
    /// Sign parent → child relationship during genesis
    async fn sign_birth(&self, parent: NodeId, child: NodeId) -> Result<Signature>;
    
    /// Get full ancestry chain
    async fn get_ancestors(&self, node: NodeId) -> Result<Vec<NodeId>>;
    
    /// Verify lineage proof
    async fn verify_lineage(&self, 
        claimed_ancestor: NodeId, 
        claimed_descendant: NodeId,
        proof: LineageProof
    ) -> Result<bool>;
}
```

**Integration Point**: Called during `songbird-genesis` ceremony.

---

### **2. Lineage Graph Maintenance** (P0 - Foundation)

**What**: Maintain the full lineage graph for relay discovery.

**Deliverables**:
```rust
pub trait LineageGraph {
    /// Add new birth to graph
    async fn record_birth(&self, parent: NodeId, child: NodeId) -> Result<()>;
    
    /// Query: Who are my ancestors? (potential relays for me)
    async fn get_my_potential_relays(&self, node: NodeId) -> Result<Vec<NodeId>>;
    
    /// Query: Who are my descendants? (who can I relay for?)
    async fn get_my_relay_candidates(&self, node: NodeId) -> Result<Vec<NodeId>>;
    
    /// Check relationship
    async fn verify_relationship(&self,
        ancestor: NodeId,
        descendant: NodeId
    ) -> Result<RelationshipProof>;
}
```

**Storage**: BearDog's responsibility (secure, tamper-proof).

---

### **3. BirdSong Encryption/Decryption** (P0 - Core)

**What**: Encrypt broadcasts so only lineage can decrypt.

**Deliverables**:
```rust
pub trait BirdSongCrypto {
    /// Encrypt message for lineage
    async fn encrypt_for_lineage(&self,
        message: &[u8],
        hint: LineageHint  // DirectAncestors, AllDescendants, etc.
    ) -> Result<EncryptedBirdSong>;
    
    /// Decrypt received BirdSong (if in lineage)
    async fn decrypt_birdsong(&self,
        encrypted: &EncryptedBirdSong,
        my_identity: &NodeIdentity
    ) -> Result<Option<Vec<u8>>>;  // None if not in lineage
    
    /// Derive keys for lineage-based encryption
    async fn derive_lineage_key(&self,
        root: NodeId,
        generation: u32
    ) -> Result<SymmetricKey>;
}

pub enum LineageHint {
    DirectParent,           // Only my parent
    DirectAncestors,        // Parent, grandparent, etc.
    AllDescendants,         // All my children's children...
    SpecificAncestor(NodeId),
}
```

**Privacy**: Non-lineage nodes see noise, lineage sees clear message.

---

### **4. Relay Authority & Verification** (P1 - Relay Core)

**What**: Verify relay requests and authorize relay service.

**Deliverables**:
```rust
pub trait RelayAuthority {
    /// Can this node relay for this requester?
    async fn authorize_relay(&self,
        relay_node: NodeId,
        requester: NodeId,
        proof: LineageProof
    ) -> Result<RelayAuthorization>;
    
    /// Get masking level based on lineage depth
    async fn determine_masking(&self,
        relay_node: NodeId,
        requester: NodeId
    ) -> Result<MaskingLevel>;
}

pub struct RelayAuthorization {
    pub authorized: bool,
    pub masking_level: MaskingLevel,
    pub ttl: Duration,  // How long authorization valid
    pub audit_token: AuditToken,  // For accountability
}

pub enum MaskingLevel {
    Masked,        // Default: minimal metadata
    SubMasked,     // Some metadata revealed
    FullVisibility, // Ancestor sees all
}
```

---

### **5. Hardware Root of Trust Integration** (P1 - Security)

**What**: Seed Genesis identity from hardware (SoloKey, TPM, etc.).

**Deliverables**:
```rust
pub trait HardwareSeedProvider {
    /// Derive keys from hardware seed
    async fn derive_from_hardware(&self,
        hardware_seed: &[u8],
        node_id: NodeId
    ) -> Result<NodeKeys>;
    
    /// Attest hardware-backed identity
    async fn attest_hardware(&self,
        node_id: NodeId,
        hardware_type: HardwareType
    ) -> Result<AttestationProof>;
}

pub enum HardwareType {
    SoloKey,
    YubiKey,
    TpmChip,
    SecureEnclave,
    HsmModule,
}
```

**Integration**: Works with `songbird-genesis` physical channels.

---

## 🚀 Evolution Path (Moving Beyond Legacy Concepts)

### **Phase 1: Foundation** (Weeks 1-3)
- [x] Genesis lineage signing
- [x] Lineage graph storage
- [x] Basic lineage queries

### **Phase 2: BirdSong** (Weeks 4-6)
- [ ] Lineage-based encryption
- [ ] Key derivation from lineage
- [ ] Broadcast decryption (family vs noise)

### **Phase 3: Relay Authority** (Weeks 7-9)
- [ ] Relay authorization
- [ ] Masking level determination
- [ ] Audit token generation

### **Phase 4: Hardware Integration** (Weeks 10-12)
- [ ] SoloKey integration
- [ ] TPM support
- [ ] Hardware attestation

**Total Timeline**: 12 weeks for complete genetic lineage relay system

---

## 🔄 Songbird Will Handle

**What Songbird Does** (so BearDog doesn't need to):
- ✅ Genesis ceremony orchestration
- ✅ Physical proximity verification (BLE)
- ✅ Witness gathering and coordination
- ✅ BirdSong message broadcasting (UDP)
- ✅ Relay session management
- ✅ Connection lifecycle (establish, monitor, upgrade)
- ✅ Direct connection attempts (legacy STUN if needed)
- ✅ Universal Coordinator integration

**BearDog Only Provides**: Security primitives (crypto, lineage, authority).

---

## 📋 API Contract

### **Minimum Viable API** (Phase 1-2)
```rust
// What Songbird needs from BearDog immediately:
pub trait MinimalLineageApi {
    // Genesis
    async fn sign_birth(parent: NodeId, child: NodeId) -> Result<Signature>;
    
    // Query
    async fn get_ancestors(node: NodeId) -> Result<Vec<NodeId>>;
    
    // Encrypt
    async fn encrypt_for_lineage(
        message: &[u8], 
        hint: LineageHint
    ) -> Result<EncryptedBirdSong>;
    
    // Decrypt
    async fn decrypt_birdsong(
        encrypted: &EncryptedBirdSong,
        my_id: NodeId
    ) -> Result<Option<Vec<u8>>>;
}
```

**This is enough to start!** Relay authorization can come in Phase 3.

---

## 🎯 Success Criteria

### **For BearDog Team**
- [ ] Node can prove lineage cryptographically
- [ ] Ancestors can decrypt descendant BirdSong
- [ ] Non-family sees only noise
- [ ] Relay authorization based on lineage
- [ ] Hardware-backed genesis identity

### **Integration Test**
```bash
# Test 1: Genesis establishes lineage
beardog genesis --parent=node1 --child=node2
# → node2 has lineage proof

# Test 2: BirdSong encryption
songbird broadcast --message="relay request" --lineage-hint=ancestors
# → Only ancestors can decrypt

# Test 3: Relay authorization
beardog authorize-relay --relay=node1 --requester=node2
# → Returns: authorized=true, masking=Masked

# Success! ✅
```

---

## 🌟 Why This Matters

### **Legacy Approach (Outdated)**
```
NAT/STUN/TURN Stack:
❌ Trust external TURN servers
❌ Central infrastructure
❌ Observable by third parties
❌ Jurisdiction-bound
❌ Can be monetized/shut down
```

### **Genetic Lineage Approach (Evolution)**
```
Lineage Relay:
✅ Trust cryptographic ancestry
✅ Distributed (any ancestor)
✅ Privacy-preserving (masked)
✅ Sovereign (no external dependency)
✅ Self-healing network
```

**Key Innovation**: Replace infrastructure with cryptography.

---

## 📞 Coordination

### **Questions?**
- **Slack**: #beardog-lineage-relay
- **Design Questions**: Songbird team available for API design
- **Integration Help**: We'll build mocks for testing

### **Songbird Commits To**
- Mock implementations for your testing
- Integration tests as you deliver each phase
- Clear API documentation
- Fast feedback on integration issues

---

## 🎯 Next Steps

### **For BearDog**
1. Review this handoff
2. Confirm API design (or propose changes)
3. Estimate timeline for Phase 1 (Genesis lineage)
4. Start implementation
5. Deliver Phase 1 → Songbird integrates → Phase 2 begins

### **For Songbird**
1. ✅ Document complete architecture
2. ✅ Create this handoff
3. **Next**: Build Songbird-side implementation (relay session management, BirdSong broadcasting)
4. Create mock BearDog implementations for testing
5. Integration testing framework

---

## 📚 References

- **[specs/LINEAGE_GATED_RELAY_PROTOCOL.md](specs/LINEAGE_GATED_RELAY_PROTOCOL.md)** - Complete protocol
- **[specs/BIRDSONG_PROTOCOL.md](specs/BIRDSONG_PROTOCOL.md)** - BirdSong specification
- **[NAT_TRAVERSAL_VIA_LINEAGE.md](NAT_TRAVERSAL_VIA_LINEAGE.md)** - Integration overview
- **[BEARDOG_GENESIS_HANDOFF_DEC_22_2025.md](BEARDOG_GENESIS_HANDOFF_DEC_22_2025.md)** - Original Genesis handoff

---

## ✅ TL;DR for BearDog Team

**Build 5 Things**:
1. ✅ Sign Genesis lineage (parent → child)
2. ✅ Maintain lineage graph (query ancestors/descendants)
3. ✅ Encrypt BirdSong (only lineage can decrypt)
4. ✅ Authorize relays (verify lineage proofs)
5. ✅ Hardware integration (SoloKey, TPM seed)

**Timeline**: 12 weeks (4 phases × 3 weeks)

**Integration**: Songbird handles networking, BearDog handles security

**Result**: Nodes relay for descendants - no external infrastructure, pure cryptographic trust!

---

**Ready to evolve beyond NAT/STUN!** 🧬🚀

🐻 **BearDog** - Genetic cryptography for sovereign networking  
🌳 **Songbird** - Universal coordination without infrastructure dependency

