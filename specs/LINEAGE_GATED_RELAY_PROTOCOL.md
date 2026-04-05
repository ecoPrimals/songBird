# Lineage-Gated Relay Protocol (LGRP)

**Date**: December 21, 2025  
**Status**: Architectural Specification  
**Phase**: 3 (NAT Traversal) Design Evolution  
**Integration**: Security Provider + Songbird Federation

---

## 🎯 Core Insight

### Traditional TURN Problem
```
Question: "Do I trust this server?"
Answer: "Yes, because infrastructure/certificates/payment"
Problem: Centralized, observable, monetized, jurisdiction-bound
```

### Lineage-Gated Solution
```
Question: "Does this node descend from me (or my trust root)?"
Answer: Provable via cryptographic lineage
Benefit: Decentralized, self-healing, no permanent observers
```

**Key Innovation:** Replace infrastructure trust with cryptographic lineage.

---

## 🧬 Architecture Overview

### Core Principle

> **Security Provider nodes with genetic lineage can act as conditional relays, where visibility and authority are governed by lineage depth and masking rules rather than network topology.**

### What This Replaces

**Traditional Stack:**
- STUN (discover public IP)
- TURN (relay when direct fails)
- ICE (negotiate optimal path)

**Lineage-Gated Stack:**
- STUN (still needed for discovery)
- **Lineage-Gated Relay** (replaces TURN)
- ICE (enhanced with lineage awareness)

---

## 🔒 Masking & Sub-Masking Layers

### Layer 0: Transport (Always Opaque)

**Relay Sees:**
- Packet size
- Timing

**Relay Never Sees:**
- Payload (encrypted)
- Session keys
- Node identities (unless permitted)

*This is baseline TURN behavior.*

### Layer 1: Masked Identity (Default)

**Relay Sees:**
- Ephemeral relay IDs
- Session-scoped pseudonyms

**Relay Never Sees:**
- Stable node identifiers
- Network topology
- Federation membership

*This prevents graph reconstruction.*

### Layer 2: Sub-Mask Access (Lineage-Gated)

**If Node Proves:**
- Cryptographic descent
- Or delegated authority

**Then Relay May Reveal:**
- Stable node ID
- Network hints
- Topology metadata
- Performance characteristics

*This is NOT decryption — it's selective metadata disclosure.*

### Layer 3: Full Lineage Visibility (Ancestor Privilege)

**Ancestor Node Can:**
- See through all descendant masks
- Audit routing
- Revoke relay privileges
- Enforce policy

**Biological Parallel:**
> Cells don't know the organism.  
> The organism knows the cells.

---

## 👁️ Visibility Matrix

| Relationship          | Visibility   | Metadata Access | Revocation |
|-----------------------|--------------|-----------------|------------|
| Parent → Child        | Full         | Full            | Yes        |
| Ancestor → Descendant | Configurable | Selective       | Yes        |
| Child → Parent        | Minimal      | None            | No         |
| Siblings              | None         | None            | No         |
| Unrelated             | None         | None            | No         |

**Key Property:** Visibility flows downward, never sideways or upward.

**Prevents:**
- Horizontal surveillance
- Federation leakage
- Power concentration

---

## 🔄 Relay Lifecycle

### Phase 1: Discovery

1. Node A and Node B fail direct NAT traversal
2. Query lineage graph via Security Provider
3. Identify suitable ancestor or cousin node
4. Verify lineage proof

### Phase 2: Relay Offer

5. Ancestor offers **temporary relay service**
6. Establishes masked session
7. Negotiates visibility level based on lineage depth

### Phase 3: Active Relay

8. Routes encrypted packets
9. Maintains minimal metadata
10. Monitors session health

### Phase 4: Dissolution

11. Direct connection established (if possible)
12. Or session completes
13. Relay dissolves
14. No persistent state

**Key:** The relay is a *phase*, not a place.

---

## 🧬 Lineage Proof Protocol

### Lineage Claim Structure

```rust
struct LineageClaim {
    // Claimer identity
    node_id: String,
    
    // Lineage path (root → ... → self)
    lineage_chain: Vec<LineageLink>,
    
    // Cryptographic proof
    proof: LineageProof,
    
    // Requested visibility level
    requested_access: AccessLevel,
}

struct LineageLink {
    parent_id: String,
    child_id: String,
    generation: u64,
    signature: Signature,  // Parent signs child
}

enum AccessLevel {
    Transport,      // Layer 0 only
    Masked,         // Layer 1
    SubMasked,      // Layer 2
    FullLineage,    // Layer 3 (ancestor only)
}
```

### Verification

```rust
fn verify_lineage_claim(claim: &LineageClaim) -> Result<bool> {
    // 1. Verify each link signature
    for link in &claim.lineage_chain {
        verify_signature(link.parent_id, link.signature)?;
    }
    
    // 2. Verify chain continuity
    verify_chain_continuity(&claim.lineage_chain)?;
    
    // 3. Verify generation depth matches access level
    let depth = claim.lineage_chain.len();
    verify_access_level(depth, claim.requested_access)?;
    
    Ok(true)
}
```

### Non-Transferability

**Critical Property:** A node cannot *claim* ancestry — it must **demonstrate it**.

Lineage is:
- Non-transferable
- Provable
- Directional
- Encodes intent and origin

---

## 🚀 Integration with Songbird

### Phase 3 Evolution

**Original Plan:**
- Phase 3.1: Implement STUN client
- Phase 3.2: Implement TURN relay
- Phase 3.3: Implement ICE negotiation

**Enhanced Plan:**
- Phase 3.1: Implement STUN client (unchanged)
- Phase 3.2: Implement **Lineage-Gated Relay** (replaces TURN)
- Phase 3.3: Implement lineage-aware ICE negotiation

### Security Provider Handoff Update

**Security Provider Responsibilities:**
1. Generate and maintain lineage chains
2. Sign child nodes
3. Provide lineage proofs
4. Act as relay when requested by descendants
5. Enforce masking rules

**Songbird Responsibilities:**
1. Query Security Provider for lineage proofs
2. Request relay service when NAT traversal fails
3. Maintain masked session state
4. Fall back to traditional TURN if lineage unavailable

---

## 🎓 Comparison: Traditional vs Lineage-Gated

### Traditional TURN

**Architecture:**
- Centralized relay servers
- Infrastructure trust model
- Monetized service
- Jurisdiction-bound

**Visibility:**
- Server sees all metadata
- Payload encrypted
- Permanent observation point
- Trust based on ownership

**Scalability:**
- Limited by server capacity
- Requires payment/authentication
- Single point of failure
- Concentrated power

### Lineage-Gated Relay

**Architecture:**
- Decentralized (any ancestor can relay)
- Cryptographic trust model
- Voluntary service
- Geography-agnostic

**Visibility:**
- Minimal by default
- Lineage-gated disclosure
- Temporary observation
- Trust based on ancestry

**Scalability:**
- Scales with federation
- No payment required
- Redundant relays
- Distributed authority

---

## ⚠️ Failure Modes & Mitigations

### Failure 1: No Lineage-Capable Relay

**Scenario:** Node needs relay but no ancestor available.

**Fallback:**
1. User-provided relay
2. Paid TURN service
3. Manual bridge via Songbird compute bridge
4. Postpone connection

**Mitigation:** System still works, just without lineage benefits.

### Failure 2: Malicious Descendant Relay

**Scenario:** Compromised descendant offers relay service.

**Limited Damage:**
- Can only see Layer 0/1 (limited metadata)
- Cannot decrypt payload
- Cannot unmask siblings
- Cannot forge lineage proofs

**Mitigation:** Ancestor can revoke relay privileges.

### Failure 3: Performance Cost

**Scenario:** Relay adds latency vs direct connection.

**Acceptance:**
- Only used when direct NAT traversal fails
- Same tradeoff as traditional TURN
- No permanence (dissolves when possible)
- Temporary performance hit acceptable for connectivity

**Mitigation:** Continuously attempt direct connection upgrade.

### Failure 4: Lineage Chain Compromise

**Scenario:** Ancestor key compromised.

**Impact:**
- Affects descendants only
- Cannot affect ancestors or siblings
- Damage is bounded by lineage depth

**Mitigation:**
- Key rotation protocols
- Lineage chain revocation
- Time-bounded lineage proofs
- Multi-ancestor verification

---

## 🌐 IPv4 vs IPv6 Considerations

### IPv4 Challenges

**NAT Prevalence:**
- Most nodes behind NAT
- Symmetric NAT especially problematic
- Relay requirement: HIGH

**Lineage-Gated Benefit:**
- More frequent relay usage
- Lineage model shines here
- Reduces TURN dependency

### IPv6 Advantages

**Direct Connectivity:**
- Every device potentially addressable
- Less NAT traversal needed
- Relay requirement: LOW

**Lineage-Gated Benefit:**
- Still useful for:
  - Firewall traversal
  - Privacy (mask public IPs)
  - Access control
- Graceful enhancement vs necessity

### Dual-Stack Strategy

**Prefer IPv6 Direct:**
1. Attempt direct IPv6 connection
2. Fall back to IPv4 with lineage relay

**Best of Both:**
- Performance (IPv6 direct)
- Privacy (lineage masking)
- Reliability (relay fallback)

---

## 📊 Lineage Depth & Access Levels

### Access Level Rules

```rust
fn determine_access_level(lineage_depth: usize) -> AccessLevel {
    match lineage_depth {
        0 => AccessLevel::FullLineage,      // Direct parent
        1..=3 => AccessLevel::SubMasked,     // Close ancestor
        4..=10 => AccessLevel::Masked,       // Distant ancestor
        _ => AccessLevel::Transport,         // Very distant or unrelated
    }
}
```

### Example Lineage Tree

```
Root (Generation 0)
└── Node A (Generation 1)
    ├── Node B (Generation 2)
    │   └── Node C (Generation 3)
    │       └── Node D (Generation 4)
    └── Node E (Generation 2)
```

**Visibility from Root:**
- Root → A: FullLineage
- Root → B: SubMasked
- Root → C: SubMasked
- Root → D: Masked

**Visibility from Node A:**
- A → B: FullLineage
- A → C: SubMasked
- A → D: Masked

**Visibility between Siblings:**
- B → E: None (must relay through A)
- C → E: None

---

## 🔐 Security Properties

### What Lineage Provides

**Authenticity:**
- Cryptographic proof of descent
- Non-repudiable ancestry
- Time-stamped generation events

**Authorization:**
- Ancestor privilege is derived, not assigned
- Cannot be transferred or forged
- Automatically inherits trust from parent

**Accountability:**
- All relay actions auditable by ancestors
- Revocation cascades downward
- Misbehavior is traceable

**Privacy:**
- Default masking
- Selective disclosure
- No horizontal visibility

### What Lineage Does NOT Provide

**Confidentiality of Payload:**
- Still requires E2E encryption (BTSP)
- Relay cannot decrypt

**Anonymity from Ancestors:**
- Ancestors can always de-mask descendants
- Intentional design for accountability

**Immunity from Traffic Analysis:**
- Timing and size still observable
- Separate countermeasures needed

---

## 🎯 Implementation Phases

### Phase 3.1: STUN (Unchanged)

**Deliverables:**
- STUN client for discovering public IP
- Integration with discovery protocol
- IPv4/IPv6 dual-stack support

**Timeline:** 1 week

### Phase 3.2: Lineage-Gated Relay

**Deliverables:**
1. Lineage proof structures
2. Lineage verification protocol
3. Relay offering mechanism
4. Masking layer implementation
5. Security Provider relay service
6. Songbird relay client

**Timeline:** 2-3 weeks

### Phase 3.3: Lineage-Aware ICE

**Deliverables:**
- ICE negotiation with lineage hints
- Prefer direct, fall back to lineage relay
- Continuous upgrade attempts
- Session migration support

**Timeline:** 1-2 weeks

**Total Phase 3:** 4-6 weeks (vs 1-2 weeks for traditional STUN/TURN/ICE)

*Worth it for sovereignty and privacy.*

---

## 📚 Related Specifications

**Security Provider:**
- Security provider BTSP handoff (historical filename `BEARDOG_BTSP_HANDOFF.md`) — BTSP integration
- `specs/PRIMAL_RESPONSIBILITY_SEPARATION_SPEC.md` - Security Provider security role
- `specs/BIRDSONG_PROTOCOL.md` - ✨ **NEW**: Lineage-gated broadcasts

**Federation:**
- `specs/RENDEZVOUS_PROTOCOL_SPEC.md` - Internet discovery
- `docs/PRIVACY_FIRST_FEDERATION.md` - Privacy architecture
- `specs/BIRDSONG_PROTOCOL.md` - Encrypted discovery broadcasts

**Roadmap:**
- `INTERNET_DEPLOYMENT_ROADMAP.md` - Overall plan (update Phase 3)

---

## 🏆 Why This Is Better

### Architectural

**Decentralized:**
- No central authority
- Self-healing network
- Scales with federation

**Sovereign:**
- Authority follows trust, not hardware
- No vendor dependency
- Community-owned relays

**Privacy-Preserving:**
- Minimal metadata by default
- Selective disclosure
- No permanent observers

### Philosophical

**Biological Model:**
- Mirrors natural lineage
- Intuitive trust model
- Scalable ethics

**Anti-Capture:**
- Cannot be monopolized
- Cannot be censored
- Cannot be monetized (without consent)

**Graduated Trust:**
- Aligns with existing 5-level trust model
- Progressive disclosure
- Consent-based visibility

---

## 🚀 Next Steps

### Immediate

1. **Update INTERNET_DEPLOYMENT_ROADMAP.md**
   - Revise Phase 3 with lineage-gated approach
   - Adjust timelines (4-6 weeks vs 1-2 weeks)

2. **Create Formal Specification**
   - `specs/LINEAGE_GATED_RELAY_PROTOCOL.md`
   - Wire protocol details
   - Lineage proof format

3. **Update Security Provider Handoff**
   - Add relay service responsibilities
   - Define lineage proof API

### Phase 3 Implementation

4. **Implement Lineage Structures** (Week 1)
   - Rust types for lineage chains
   - Verification functions
   - Test suite

5. **Security Provider Relay Service** (Week 2-3)
   - Relay offering mechanism
   - Masking implementation
   - Descendant discovery

6. **Songbird Relay Client** (Week 3-4)
   - Lineage proof requests
   - Relay negotiation
   - Session management

7. **Integration Testing** (Week 5-6)
   - End-to-end relay tests
   - Failure mode validation
   - Performance benchmarks

---

## 📊 Success Criteria

### Functional

- ✅ Nodes can request relay from ancestors
- ✅ Lineage proofs verify correctly
- ✅ Masking levels enforce properly
- ✅ Relay dissolves when not needed
- ✅ Fallback to traditional TURN works

### Security

- ✅ Cannot forge lineage proofs
- ✅ Cannot unmask siblings
- ✅ Ancestor revocation works
- ✅ No metadata leakage at Layer 1

### Performance

- ✅ Relay adds < 50ms latency
- ✅ Lineage verification < 10ms
- ✅ Scales to 1000+ node federation
- ✅ Graceful degradation under load

---

## 🎉 Conclusion

### One Sentence Verdict

> **Security Provider lineage can safely and correctly act as a TURN-like startup and fallback relay system, with masking and sub-masking governed by cryptographic ancestry rather than central authority.**

### Deep Insight

> You've turned **connectivity assistance** into a **biological role**, not an infrastructure service.

**This is:**
- Rare
- Powerful
- Ethically scalable

**You're no longer asking:**
- "Who runs the server?"

**You're asking:**
- "Who do I descend from — and who may act on my behalf?"

---

**Status:** Architectural specification complete  
**Next:** Formal protocol specification and implementation  
**Impact:** Transforms Phase 3 from infrastructure dependency to sovereign, lineage-based connectivity

*This aligns perfectly with Songbird's privacy-first, sovereign architecture.* 🧬🔒✨

