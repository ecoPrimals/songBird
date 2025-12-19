# 🎵 Songbird Evolution - Sovereign Coordination Primal

**Status:** Active Development  
**Started:** December 2025  
**Current Version:** 0.2.0 (Access Control Design)

---

## Vision

**Songbird is a sovereign primal for federated compute coordination** - capable of operating independently with built-in security, enhanced by integration with other primals (BearDog, RhizoCrypt, ToadStool).

---

## Core Identity

### What Songbird IS

- ✅ **Coordination layer** for distributed compute
- ✅ **Service discovery** via capability-based registry
- ✅ **Task routing** to federated nodes
- ✅ **Access control** with graduated information disclosure
- ✅ **Sovereign primal** - functions independently
- ✅ **Federation protocol** - connects multiple coordinators

### What Songbird is NOT

- ❌ Just a "thin API layer" (it's a full primal)
- ❌ Dependent on BearDog (enhanced by it, not required)
- ❌ Monolithic orchestrator (it coordinates, nodes execute)
- ❌ Single point of failure (can federate multiple Songbirds)

---

## Evolution Milestones

### ✅ Milestone 1: Foundation (December 2025)

**Achievements:**
- Basic task submission and routing
- Service registry integration
- Capability-based discovery
- Zero production mocks
- ToadStool federation working
- 95.19% ML accuracy with cryptographic receipts

**Files:**
- `crates/songbird-orchestrator/src/orchestrator.rs`
- `crates/songbird-orchestrator/src/rpc/tarpc_server.rs`
- `crates/songbird-orchestrator/src/rpc/jsonrpc.rs`
- `crates/songbird-network-federation/src/service_registry.rs`

**Status:** ✅ Complete

---

### 🔄 Milestone 2: Access Control (Q1 2025)

**Goals:**
- Graduated information disclosure (Public → Educational → Operational → Administrative → Infrastructure)
- Role-based access (Student, TA, Professor, Admin, Remote Admin)
- Capability-based authorization
- Standalone security (JWT tokens, audit logs)
- Hardware key support (SoloKey for admin)

**Specifications:**
- `specs/SONGBIRD_ACCESS_CONTROL.md` (✅ Design Complete)

**Implementation Plan:**
```
Phase 1: Core Access Control (2 weeks)
├── Token validation system
├── Role definitions
├── Capability checking
└── Audit logging

Phase 2: Information Layers (2 weeks)
├── Public info builder
├── Educational info builder
├── Operational info builder
├── Administrative info builder
└── Infrastructure info builder

Phase 3: Security Hardening (1 week)
├── Hardware key integration (SoloKey)
├── 2FA for admin
├── VPN requirement for infrastructure access
└── Session management
```

**Status:** 🔄 Design Complete, Implementation Planned

---

### 📋 Milestone 3: Student Onboarding (Q1 2025)

**Goals:**
- Simple Python client for students
- Local network deployment (campus WiFi)
- Example projects (MNIST, CIFAR-10)
- Documentation for students and instructors
- Testing with real class (Prof. Murillo)

**Showcase:**
- `showcase/07-student-onboarding/` (✅ Built)

**Status:** ✅ Ready for Testing

---

### 📋 Milestone 4: BearDog Integration (Q2 2025)

**Goals:**
- Genetic identity verification
- Hardware-bound tokens
- Zero-knowledge capability proofs
- Instant revocation via genetic identity
- Enhanced security for all roles

**Dependencies:**
- BearDog genetic encryption system
- SoloKey hardware key support
- Pixel 8a / GrapheneOS spore gateway

**Specifications:**
- `specs/SONGBIRD_BEARDOG_INTEGRATION.md` (Planned)

**Benefits:**
- Identity theft impossible (genetic encryption)
- Token theft doesn't compromise identity
- Hardware-backed operations
- Cryptographic audit trail

**Status:** 📋 Planned (After BearDog spec complete)

---

### 📋 Milestone 5: Multi-Coordinator Federation (Q2-Q3 2025)

**Goals:**
- Multiple Songbird coordinators
- Gossip protocol for state sync
- Load balancing across coordinators
- No single point of failure
- Cross-campus deployment

**Architecture:**
```
Campus A: Songbird-A
Campus B: Songbird-B
    ↕ (State sync)
Compute Nodes: ToadStool federation
    ↕ (Task routing)
Students: Connect to any coordinator
```

**Status:** 📋 Planned

---

### 📋 Milestone 6: Internet Deployment (Q3 2025)

**Goals:**
- Public endpoint (WSS, BearDog auth)
- Remote student access (from anywhere)
- Spore gateway (Pixel 8a / GrapheneOS)
- VPN tunnel to home infrastructure
- Zero IP exposure to students

**Security Model:**
```
Internet (Untrusted)
    ↓ WSS + BearDog
Spore Gateway (Pixel 8a)
    ↓ WireGuard + SoloKey
Home Infrastructure (Hidden)
```

**Status:** 📋 Planned

---

## Technical Architecture

### Current (December 2025)

```
Student Laptop
    ↓ WebSocket
Songbird Coordinator (Windows laptop, local network)
    ↓ Service Registry Query
FederatedServiceRegistry (capability-based discovery)
    ↓ Task Routing
ToadStool Nodes (Eastgate, Strandgate)
    ↓ Results + Receipt
Student Laptop
```

**Characteristics:**
- Zero hardcoded IPs (capability-based discovery)
- Real distributed ML working (95.19% accuracy)
- Cryptographic receipts for all tasks
- No production mocks

---

### Target (Q3 2025)

```
Internet (Students anywhere)
    ↓ WSS + BearDog Auth
Multiple Songbird Coordinators (federated, no SPOF)
    ↓ Gossip Protocol Sync
    ↓ Service Registry Query
FederatedServiceRegistry (capability-based discovery)
    ↓ Task Routing
ToadStool Nodes (Your towers + Friends/Family)
    ↓ Results + Receipt
Students (with graduated information disclosure)
```

**Characteristics:**
- Multi-coordinator (no single point of failure)
- Internet access (students from anywhere)
- BearDog genetic identity (theft-proof)
- Hardware-backed security (SoloKey, Titan M2)
- Information privacy (graduated disclosure)
- Geographic distribution (multiple sites)

---

## Security Evolution

### Phase 1: Standalone (Current → Q1 2025)

**Authentication:**
- JWT tokens (Songbird-signed)
- API keys for basic auth
- Token expiry enforcement

**Authorization:**
- Role-based access control
- Capability checking
- Information layer separation

**Audit:**
- Access logs
- Task history
- Security events

**Limitations:**
- Token theft = compromise until expiry
- No hardware binding
- Revocation requires distributed state

---

### Phase 2: BearDog-Enhanced (Q2 2025)

**Authentication:**
- Genetic identity (BearDog)
- Hardware-bound tokens (SoloKey)
- Zero-knowledge proofs

**Authorization:**
- Cryptographic capability delegation
- Instant revocation (genetic identity)
- Multi-party authorization

**Audit:**
- Cryptographic audit trail
- Immutable logs (RhizoCrypt)
- Provenance chain (LoamSpine)

**Advantages:**
- Identity theft impossible
- Token theft doesn't compromise identity
- Instant revocation
- Hardware-backed operations

---

## Integration with Other Primals

### 🍄 ToadStool (Compute)

**Status:** ✅ Working (December 2025)

**Integration:**
- Songbird discovers ToadStool nodes via registry
- Routes tasks based on capabilities
- ToadStool executes, Songbird coordinates

**Proven:**
- 95.19% distributed ML accuracy
- Cryptographic receipts
- Zero hardcoded knowledge

---

### 🐻 BearDog (Security)

**Status:** 📋 Planned (Q2 2025)

**Integration:**
- Genetic identity for users
- Hardware-bound tokens
- Cryptographic capability delegation
- Instant revocation

**Benefits:**
- Enhanced security
- Identity theft impossible
- Hardware-backed operations

---

### 🔐 RhizoCrypt (Cryptography)

**Status:** 📋 Planned (Q2-Q3 2025)

**Integration:**
- DAG-based provenance
- Immutable audit logs
- Cryptographic receipts enhanced by DAG
- Distributed trust

**Benefits:**
- Tamper-proof audit trail
- Distributed verification
- Long-term provenance

---

### 🦴 LoamSpine (Linear Crypto)

**Status:** 📋 Planned (Q3 2025)

**Integration:**
- Linear event ordering
- Causal consistency
- Provenance chains

**Benefits:**
- Ordered audit logs
- Causal reasoning
- Trust without centralization

---

### 🌾 SweetGrass (Attribution)

**Status:** 📋 Planned (Q3 2025)

**Integration:**
- Semantic metadata for tasks
- Attribution for educational content
- Provenance for research data

**Benefits:**
- Rich task metadata
- Educational attribution
- Research provenance

---

### 🌍 Gaia (Knowledge Commons)

**Status:** 📋 Planned (2026)

**Integration:**
- Task results → Gaia shards
- Self-owning scientific data
- Distributed knowledge commons

**Benefits:**
- Results become part of commons
- Cryptographic provenance
- Can't be paywalled or captured

---

## Key Design Decisions

### Decision 1: Standalone Security First

**Choice:** Build access control into Songbird, enhance with BearDog later

**Rationale:**
- ✅ Songbird must function independently
- ✅ No circular dependencies
- ✅ Fail-safe sovereignty
- ✅ Immediate deployment possible

**Alternative Considered:** Require BearDog from start
**Why Rejected:** Creates dependency, delays deployment, reduces sovereignty

---

### Decision 2: Graduated Information Disclosure

**Choice:** Multiple information layers, not binary show/hide

**Rationale:**
- ✅ Educational value preserved (students see sharding)
- ✅ Operational support enabled (TAs see debug info)
- ✅ Security maintained (IPs hidden from students)
- ✅ Flexible for different use cases

**Alternative Considered:** Binary access (all or nothing)
**Why Rejected:** Loses educational value, reduces operational flexibility

---

### Decision 3: Capability-Based Access

**Choice:** Capabilities (what you can do) not permissions (who you are)

**Rationale:**
- ✅ More flexible (capabilities compose)
- ✅ Easier delegation
- ✅ Principle of least privilege
- ✅ Aligns with federated model

**Alternative Considered:** Traditional RBAC (role = fixed permissions)
**Why Rejected:** Too rigid for federated, collaborative environments

---

### Decision 4: Zero Hardcoded Knowledge

**Choice:** Capability-based discovery, no hardcoded IPs

**Rationale:**
- ✅ Nodes self-register
- ✅ Dynamic federation
- ✅ No config updates when adding nodes
- ✅ Aligns with sovereignty principles

**Alternative Considered:** Config file with node list
**Why Rejected:** Technical debt, violates sovereignty, hard to scale

---

## Deployment Models

### Model 1: Campus Classroom (Current)

```
Instructor's Laptop (Songbird)
    ↓ Campus WiFi
Students (same network)
    ↓ VPN Tunnel
Instructor's Home Towers (ToadStool)
```

**Use Case:** MSU class with Prof. Murillo  
**Security:** Moderate (trusted environment, VPN to home)  
**Status:** ✅ Ready for testing

---

### Model 2: Multi-Campus Federation (Q2 2025)

```
Campus A: Songbird-A + Local Compute
Campus B: Songbird-B + Local Compute
    ↕ Federation Protocol
Students: Connect to nearest coordinator
```

**Use Case:** Multiple universities collaborating  
**Security:** High (BearDog auth, federated trust)  
**Status:** 📋 Planned

---

### Model 3: Internet + Spore Gateway (Q3 2025)

```
Internet (Students anywhere)
    ↓ WSS + BearDog
Pixel 8a (Spore Gateway, mobile)
    ↓ WireGuard + SoloKey
Home Infrastructure (Hidden, secure)
```

**Use Case:** Remote learning, distributed research  
**Security:** Maximum (hardware-backed, zero IP exposure)  
**Status:** 📋 Planned

---

### Model 4: Friends/Family Federation (Q3-Q4 2025)

```
Your Towers (Eastgate, Strandgate, etc.)
Friend's Tower (Remote location)
Family Tower (Another state)
    ↕ Federated Discovery + BearDog Trust
Songbird Coordinators (multiple, no SPOF)
    ↓
Students/Researchers (global access)
```

**Use Case:** Distributed sovereign compute network  
**Security:** Maximum (multi-party, distributed)  
**Status:** 📋 Planned

---

## Success Metrics

### Technical Metrics

- ✅ Zero production mocks (Achieved December 2025)
- ✅ Cryptographic receipts (Achieved December 2025)
- ✅ 95%+ ML accuracy (Achieved: 95.19%)
- 🔄 Student onboarding < 10 minutes (Built, pending test)
- 📋 Multi-coordinator federation (Planned Q2)
- 📋 BearDog genetic identity (Planned Q2)

### Educational Metrics

- 🔄 First class deployment (Q1 2025 with Murillo)
- 📋 Student satisfaction > 80% (Measure after deployment)
- 📋 Learning outcomes improved (Compare to traditional)
- 📋 10+ universities using (Long-term goal)

### Research Metrics

- 🔄 Academic paper with Murillo (Draft Q1, publish Q2)
- 📋 Open source release (Q3 2025)
- 📋 Conference presentation (Q3-Q4 2025)
- 📋 Grant applications (NSF, NIH) (2026)

### Sovereignty Metrics

- ✅ No cloud dependency (Achieved)
- ✅ Zero hardcoded IPs (Achieved)
- 🔄 Hardware-backed security (Q2 2025)
- 📋 Multi-party federation (Q2-Q3 2025)
- 📋 Gaia integration (2026)

---

## Next Actions

### Immediate (This Week)

- [ ] Test student onboarding on Windows laptop
- [ ] Submit MNIST task from client, verify full workflow
- [ ] Document any issues encountered
- [ ] Prepare demo materials for Prof. Murillo

### Short Term (Q1 2025)

- [ ] Implement core access control system
- [ ] Build information layer builders
- [ ] Integrate SoloKey for admin access
- [ ] Deploy in Prof. Murillo's class
- [ ] Collect student feedback

### Medium Term (Q2 2025)

- [ ] Integrate BearDog genetic identity
- [ ] Implement hardware-bound tokens
- [ ] Build multi-coordinator federation
- [ ] Deploy across multiple campuses
- [ ] Publish academic paper

### Long Term (Q3-Q4 2025)

- [ ] Internet deployment with spore gateway
- [ ] Friends/family federation
- [ ] RhizoCrypt audit trail integration
- [ ] Gaia knowledge commons integration
- [ ] Open source release

---

## Documentation

### Specifications

- ✅ `specs/SONGBIRD_ACCESS_CONTROL.md` - Access control design
- 📋 `specs/SONGBIRD_BEARDOG_INTEGRATION.md` - BearDog integration (Planned)
- 📋 `specs/SONGBIRD_FEDERATION_PROTOCOL.md` - Multi-coordinator federation (Planned)

### Showcases

- ✅ `showcase/06-toadstool-ml-orchestration/` - Distributed ML validation
- ✅ `showcase/07-student-onboarding/` - Student client and examples

### Root Documentation

- ✅ `SONGBIRD_EVOLUTION.md` - This document
- ✅ `README.md` - Project overview
- ✅ `STATUS.md` - Current status

---

## Reflections

### What We've Learned

**Technical:**
- Capability-based discovery works (95.19% accuracy proves it)
- Zero hardcoded knowledge is achievable (and necessary)
- Graduated information disclosure is the right model
- Standalone security enables fail-safe sovereignty

**Educational:**
- Students should see sharding (educational value)
- TAs need operational info (debugging support)
- Professors need administrative info (research)
- Admins need infrastructure access (maintenance)

**Security:**
- IP privacy is important (students shouldn't see home network)
- Hardware keys are essential (SoloKey for admin)
- Genetic identity prevents theft (BearDog integration)
- Graduated disclosure balances security and utility

### What's Working

- Service registry and capability-based discovery
- Distributed ML training with cryptographic receipts
- Student onboarding design (pending testing)
- Alignment with sovereignty principles

### What's Next

- Test with real students (Prof. Murillo's class)
- Implement access control system
- Integrate BearDog when available
- Scale to multi-campus federation

---

**Songbird is evolving from a simple coordinator to a sovereign primal for federated compute** - with built-in security, graduated information disclosure, and capability-based access control. Enhanced by BearDog, RhizoCrypt, and other primals, but functional independently.

**The nervous system that connects computation across sovereign nodes.** 🎵✨

