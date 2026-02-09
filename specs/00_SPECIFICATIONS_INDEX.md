# Songbird Specifications Index

**Last Updated:** February 8, 2026  
**Version:** v3.36.0  
**Status:** ✅ **WORLD-CLASS** - Multi-Protocol + IGD Router Evolution

---

## 🎉 LATEST: IGD ROUTER CONFIGURATION (February 8, 2026)

### 🌐 Sovereign Router Configuration — #1 Blocker for Cross-Network

1. **[IGD_ROUTER_CONFIGURATION_SPECIFICATION.md](IGD_ROUTER_CONFIGURATION_SPECIFICATION.md)** ⭐ **COMPLETE** ⭐
   - **Pure Rust UPnP IGD + NAT-PMP** - Automatic router port forwarding
   - ✅ **COMPLETE** (Feb 8, 2026) - SSDP + SOAP + NAT-PMP from scratch
   - Zero external protocol crates (SSDP/SOAP implemented directly)
   - Protocols: UPnP IGD (RFC 6970) + NAT-PMP (RFC 6886)
   - JSON-RPC: 6 methods (discover, map_port, unmap_port, status, external_ip, auto_configure)
   - Enables zero-touch deployment (router auto-configured on startup)
   - Cross-architecture: Tower (x86_64) + Pixel (aarch64) validated
   - 19 unit tests + 1 doc-test passing

**Status**: ✅ **COMPLETE** - All modules implemented and tested  
**Validated**: Cross-network crypto chain proven (Tower encrypts → Pixel decrypts ✅)  
**Blocker**: AT&T gateway at 192.168.1.254 has no UPnP — IGD will auto-detect and guide

**Related Documents**:
- Crate: `crates/songbird-igd/` - Implementation
- Session: `docs/sessions/2026-02-february/IGD_IMPLEMENTATION_PROGRESS_FEB_08_2026.md`
- Spec: `SOVEREIGN_MULTIPATH_PROTOCOL.md` - Multi-path tier integration

---

## 🎉 LATEST: QUIC + NFC PROTOCOLS (February 8, 2026)

### 🚀 New Protocol Crates

2. **QUIC Protocol** (`crates/songbird-quic/`) ⭐ **COMPLETE** ⭐
   - Pure Rust QUIC via quinn v0.11
   - 0-RTT, connection migration, stream multiplexing
   - BearDog crypto integration points ready

3. **NFC Genesis** (`crates/songbird-nfc/`) ⭐ **COMPLETE** ⭐
   - Dark Forest compliant mobile pairing
   - Ephemeral keys, timing protection
   - Platform abstraction (Android/iOS/Linux)

4. **WireGuard Beacon Extension** ✅ COMPLETE
   - External tunnel advertising in encrypted beacons
   - Extends `dark_forest_beacon.rs`

**Status**: ✅ **COMPLETE** - All three protocols implemented and tested

---

## 🎉 PREVIOUS: TOR PROTOCOL EVOLUTION (February 7, 2026)

### 🧅 Pure Rust Tor Protocol (Phase 2)

1. **[TOR_PROTOCOL_PURE_RUST.md](TOR_PROTOCOL_PURE_RUST.md)** ⭐ **SPECIFICATION** ⭐
   - **Pure Rust Tor Protocol** - Minimal implementation for .onion services
   - ✅ **PHASE 2A COMPLETE** (Directory protocol)
   - Zero external dependencies (no Tor daemon, no Arti, no C)
   - 100% TRUE PRIMAL (BearDog crypto delegation)
   - ~2,600 lines total (vs. Tor's 220k+ lines)
   - Components: Directory, Circuit, Onion Service, Stream protocols

2. **[NTOR_HANDSHAKE.md](NTOR_HANDSHAKE.md)** ⭐ **NEW** ⭐
   - **ntor Handshake Protocol** - Detailed CREATE2/CREATED2 specification
   - Key derivation function (KDF) via SHA3-256
   - BearDog integration patterns
   - Test vectors for validation
   - Ready for Phase 2B implementation

**Status**: ✅ **PHASE 2A COMPLETE** - Directory protocol implemented, tested, pushed  
**Blocked**: Phase 2B awaiting BearDog AES-128-CTR + SHA3-256 extensions

**Related Documents**:
- Root: `TOR_INTEGRATION_ROADMAP_FEB_07_2026.md` - Overall roadmap
- Root: `TOR_PHASE2_EVOLUTION_TRACKER.md` - Daily progress tracking
- Root: `PHASE_2A_COMPLETE_FEB_07_2026.md` - Phase 2A completion report
- Root: `PHASE_2B_PREPARATION.md` - Phase 2B design (ready for implementation)

---

## 🎉 LATEST: SOVEREIGN MULTI-PATH PROTOCOL (February 8, 2026)

### 🌐 7-Tier Connection Strategy ACTIVE

1. **[SOVEREIGN_MULTIPATH_PROTOCOL.md](SOVEREIGN_MULTIPATH_PROTOCOL.md)** ⭐ **NEW MASTER SPEC** ⭐
   - **Comprehensive Multi-Path Protocol** - 7-tier resilient connectivity
   - ✅ **IPv6 Dual-Stack** (Feb 8, 2026) - Global reachability without port forwarding
   - ✅ **Sovereign Onion** (Feb 8, 2026) - X25519 + ChaCha20Poly1305 active
   - 🔨 **IGD/UPnP** - Router evolution for sovereign port forwarding
   - 🔨 **STUN Coordinator** - Hole-punch coordinator wiring needed
   - 🔨 **Family Relay** - Mesh peer connection integration needed
   - Complete architecture: IPv6 → Onion → IPv4 → LAN → STUN → Relay → Beacon

**Status**: ✅ **IPv6 + Onion WORKING** - IGD + Coordinator + Relay wiring next

**Related Documents**:
- Root: Session handoff with complete multi-path design
- Specs: `SOVEREIGN_BEACON_MESH_SPECIFICATION.md` - Mesh topology
- Specs: `SOVEREIGN_ONION_PROTOCOL.md` - Onion protocol details
- Specs: `STUN_SERVER_CAPABILITY_SPECIFICATION.md` - STUN implementation
- Specs: `RELAY_SERVER_SPECIFICATION.md` - Relay server
- Specs: `SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md` - IPv6 binding

---

## 🎉 RECENT: P2P SOVEREIGN ONION (February 6, 2026)

### 🌐 P2P Service + Connector Complete

2. **[SOVEREIGN_ONION_PROTOCOL.md](SOVEREIGN_ONION_PROTOCOL.md)** ⭐ **COMPLETE** ⭐
   - **Custom Onion Service Protocol** - P2P encrypted communication
   - ✅ **IMPLEMENTED** (Feb 6, 2026) - OnionService + OnionConnector
   - 100% BearDog crypto delegation (Ed25519, X25519, ChaCha20Poly1305)
   - TCP listener + handshake + encrypted data transfer
   - Protocol: KeyExchange → Data (encrypted) → Close
   - 199 lines (service) + 160 lines (connector) + OnionConnection
   - Zero direct crypto, production ready

**Status**: ✅ **COMPLETE** - Production ready, integrated with Multi-Path Protocol

**Related Documents**:
- Root: `P2P_IMPLEMENTATION_COMPLETE_FEB_06_2026.md` - Completion report
- Root: `P2P_IMPLEMENTATION_ROADMAP_FEB_06_2026.md` - Implementation guide
- Crate: `crates/songbird-sovereign-onion/` - Source code

---

## 🎉 PREVIOUS: NAT TRAVERSAL STACK (February 5, 2026)

### 🌐 NEW: Complete Sovereign NAT Traversal

3. **[STUN_SERVER_CAPABILITY_SPECIFICATION.md](STUN_SERVER_CAPABILITY_SPECIFICATION.md)** ⭐ **COMPLETE** ⭐
   - **Pure Rust STUN Server** - RFC 5389 compliant NAT traversal
   - ✅ **IMPLEMENTED** (Feb 5, 2026) - 958 lines, 24 tests passing
   - Eliminates coturn C dependency (ecoBin compliance)
   - JSON-RPC integration (`stun.serve`, `stun.stop`, `stun.status`)
   - Performance: <1ms response (~0.2ms measured)
   - Zero unsafe code, zero new dependencies

**Status**: ✅ **COMPLETE** - Production ready, coturn STUN eliminated

4. **[RELAY_SERVER_SPECIFICATION.md](RELAY_SERVER_SPECIFICATION.md)** ⭐ **NEW** ⭐
   - **Lineage Relay Server** - Evolution of TURN RFC 5766
   - Packet forwarding for symmetric NAT (30% of connections)
   - Genetic lineage authorization (not username/password)
   - Privacy masking based on family relationship
   - Distributed relay network (any ancestor can help)
   - 80% infrastructure exists (2,910 lines of relay code)
   - Phase 1-5: 5 days implementation (~1,250 new lines)
   - Completes sovereign NAT traversal stack
   - Eliminates coturn completely

**Status**: 📋 **PLANNED** - Ready for Implementation (investigation complete, spec approved)

---

## 🎉 PREVIOUS: SECURE COMMUNICATIONS PROTOCOL (January 27, 2026)

### 🔒 BearDog Policy Delegation Architecture

5. **[SECURE_COMMUNICATIONS_PROTOCOL.md](SECURE_COMMUNICATIONS_PROTOCOL.md)**
   - **"BearDog Decides What Goes Where"** - Security policy delegation
   - Tower Atomic pattern for transport security
   - 5-level data classification (Public → Top Secret)
   - Multi-version TLS support (1.0/1.2/1.3 based on policy)
   - 4 JSON-RPC security methods defined
   - 5 detailed use cases (modern API, legacy bank, IoT, policy violation, dev)
   - Per-connection security guarantees
   - Audit & compliance framework

---

## 🎉 PREVIOUS: TLS 1.3 HTTPS COMPLETE + CAPABILITY ABSTRACTION (January 24, 2026)

### ⭐ CURRENT WORK - NAT TRAVERSAL & SECURITY

6. **[STUN_SERVER_CAPABILITY_SPECIFICATION.md](STUN_SERVER_CAPABILITY_SPECIFICATION.md)** ⭐ READY TO IMPLEMENT (Feb 5, 2026)
   - Pure Rust STUN server for NAT traversal
   - Eliminates coturn C dependency
   - 3-5 days for Phase 1 MVP
   - Investigation: `ecoPrimals/sessions/2026-02-february/STUN_SERVER_INVESTIGATION_FEB_05_2026.md`

7. **[SECURE_COMMUNICATIONS_PROTOCOL.md](SECURE_COMMUNICATIONS_PROTOCOL.md)** ✅ COMPLETE (Jan 27, 2026)
   - BearDog policy delegation architecture
   - Data classification → Transport security mapping
   - Multi-version TLS support (1.0/1.2/1.3)
   - JSON-RPC security API (4 methods)
   - 5 use cases documented

8. **[SONGBIRD_TLS_13_COMPLETE.md](SONGBIRD_TLS_13_COMPLETE.md)** ✅ COMPLETE
   - Full RFC 8446 TLS 1.3 implementation
   - 100% Pure Rust, zero C dependencies
   - 93% success rate (81/87 sites)
   - Tested: AI/ML (100%), Cloud (90%), GitHub (100%)

9. **[SONGBIRD_FUTURE_WORK.md](SONGBIRD_FUTURE_WORK.md)** 📋 ROADMAP
   - Security hardening roadmap (certificate validation)
   - Performance improvements (session resumption, connection pooling)
   - Protocol extensions (HTTP/2, TLS 1.2 implementation)
   - Production cleanup tasks

10. **[SONGBIRD_EVOLUTION_EXECUTION.md](SONGBIRD_EVOLUTION_EXECUTION.md)** ✅ IMPLEMENTED
   - Capability-based crypto abstraction (`CryptoCapability` trait)
   - Runtime provider discovery (no hardcoded sockets)
   - Large file refactoring (HTTP client: 1,193 → 592 lines)
   - biomeOS semantic translation ready

---

## 🏗️ CORE ARCHITECTURE

### Foundational Principles

11. **[PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md](PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md)** ⭐ CRITICAL
   - **"Each Primal Knows Only Itself"** principle
   - No hardcoded knowledge of other primals
   - Discovery happens dynamically at runtime

12. **[CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md](CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md)** ⭐ CRITICAL
   - Pure capability-based routing
   - Zero hardcoded primal names
   - Universal adapter pattern

13. **[STANDALONE_NETWORK_EFFECTS_ARCHITECTURE_SPEC.md](STANDALONE_NETWORK_EFFECTS_ARCHITECTURE_SPEC.md)**
   - Build for self + sovereignty
   - Network effects emerge when primals connect

### Access Control & Privacy

14. **[SONGBIRD_ACCESS_CONTROL.md](SONGBIRD_ACCESS_CONTROL.md)**
   - 5 Trust Levels (Anonymous → Hardware-Verified)
   - Graduated information disclosure

15. **[CONSENT_MANAGEMENT.md](CONSENT_MANAGEMENT.md)**
   - Explicit consent for sensitive operations
   - User-controlled data sharing

16. **[INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md](INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md)**
   - Human dignity as architectural principle
   - User agency and autonomy

---

## 🔌 PROTOCOLS & INTEGRATION

### BearDog Integration

17. **[SONGBIRD_BEARDOG_INTEGRATION.md](SONGBIRD_BEARDOG_INTEGRATION.md)**
   - Cryptographic delegation model
   - JSON-RPC over Unix socket
   - Capability-based discovery

18. **[BEARDOG_ENTROPY_HIERARCHY_INTEGRATION.md](BEARDOG_ENTROPY_HIERARCHY_INTEGRATION.md)**
    - Security primal integration
    - Trust verification

### Protocol Framework

19. **[UNIVERSAL_PROTOCOL_FRAMEWORK_SPECIFICATION.md](UNIVERSAL_PROTOCOL_FRAMEWORK_SPECIFICATION.md)**
    - Multi-protocol support
    - Protocol negotiation

20. **[HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md](HYBRID_PROTOCOL_ARCHITECTURE_SPECIFICATION.md)**
    - HTTP/HTTPS + JSON-RPC
    - Protocol selection

21. **[TARPC_JSON_RPC_PROTOCOL_SPEC.md](TARPC_JSON_RPC_PROTOCOL_SPEC.md)**
    - tarpc implementation
    - High-performance RPC

### Specific Integrations

22. **[SONGBIRD_SQUIRREL_INTEGRATION_SPEC.md](SONGBIRD_SQUIRREL_INTEGRATION_SPEC.md)**
    - AI routing integration
    - MCP protocol support

---

## 🌐 FEDERATION & DISCOVERY

### Federation

23. **[FEDERATION_IMPLEMENTATION_SPECIFICATION.md](FEDERATION_IMPLEMENTATION_SPECIFICATION.md)**
    - Multi-tower federation
    - Cross-tower communication

24. **[FRACTAL_FEDERATION_SPECIFICATION.md](FRACTAL_FEDERATION_SPECIFICATION.md)**
    - Fractal federation architecture
    - Hierarchical scaling

25. **[SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md](SOVEREIGN_FEDERATION_IMPLEMENTATION_PLAN.md)**
    - Sovereignty-preserving federation
    - Peer-to-peer coordination

### Discovery & Routing

26. **[INTELLIGENT_CAPABILITY_ROUTING_SPEC.md](INTELLIGENT_CAPABILITY_ROUTING_SPEC.md)**
    - Smart routing based on capabilities
    - QoS-aware routing

27. **[CAPABILITY_REGISTRATION_API.md](CAPABILITY_REGISTRATION_API.md)**
    - How primals register capabilities
    - Dynamic capability updates

28. **[PRIMAL_REGISTRATION_PROTOCOL.md](PRIMAL_REGISTRATION_PROTOCOL.md)**
    - Universal Port Authority principle
    - 5-phase registration lifecycle

---

## 🏛️ UNIVERSAL ARCHITECTURE

### Universal Adapters

29. **[UNIVERSAL_CAPABILITY_ADAPTER_IMPLEMENTATION_SPEC.md](UNIVERSAL_CAPABILITY_ADAPTER_IMPLEMENTATION_SPEC.md)**
    - Zero-cost universal adapter
    - Capability-based routing

30. **[UNIVERSAL_PRIMAL_ADAPTER_SPECIFICATION.md](UNIVERSAL_PRIMAL_ADAPTER_SPECIFICATION.md)**
    - Primal-agnostic adapter
    - Dynamic primal discovery

31. **[UNIVERSAL_PRIMAL_SDK_INTEGRATION_SPECIFICATION.md](UNIVERSAL_PRIMAL_SDK_INTEGRATION_SPECIFICATION.md)**
    - Universal primal registry
    - Environment-adaptive discovery

---

## 🧪 TESTING & QUALITY

32. **[COMPREHENSIVE_TESTING_INFRASTRUCTURE_SPECIFICATION.md](COMPREHENSIVE_TESTING_INFRASTRUCTURE_SPECIFICATION.md)**
    - Unit, integration, E2E tests
    - Test organization

33. **[UNIFIED_TESTING_FRAMEWORK_SPECIFICATION_2025.md](UNIFIED_TESTING_FRAMEWORK_SPECIFICATION_2025.md)**
    - Modern testing framework
    - Best practices

---

## ⚡ PERFORMANCE & OPTIMIZATION

34. **[ZERO_COST_ARCHITECTURE_SPECIFICATION.md](ZERO_COST_ARCHITECTURE_SPECIFICATION.md)**
    - Zero-cost abstractions
    - No runtime overhead

35. **[ZERO_COST_PERFORMANCE_SPECIFICATION.md](ZERO_COST_PERFORMANCE_SPECIFICATION.md)**
    - Performance benchmarks
    - Optimization techniques

---

## 🏢 DEPLOYMENT & OPERATIONS

36. **[ADAPTIVE_DEPLOYMENT_SPECIFICATION.md](ADAPTIVE_DEPLOYMENT_SPECIFICATION.md)**
    - Adaptive deployment patterns
    - Auto-configuration

37. **[STANDALONE_TO_FAMILY_DEPLOYMENT_GUIDE.md](STANDALONE_TO_FAMILY_DEPLOYMENT_GUIDE.md)**
    - Single tower → Multi-tower
    - Migration guide

---

## 🔧 IMPLEMENTATION GUIDES

### Error Handling

38. **[UNIFIED_ERROR_HANDLING_SPECIFICATION.md](UNIFIED_ERROR_HANDLING_SPECIFICATION.md)**
    - Unified error types
    - Error recovery

39. **[ERROR_RECOVERY.md](ERROR_RECOVERY.md)**
    - Recovery strategies
    - Graceful degradation

### Task & Resource Management

40. **[TASK_LIFECYCLE.md](TASK_LIFECYCLE.md)**
    - Task lifecycle management
    - State transitions

41. **[RESOURCE_MANAGEMENT.md](RESOURCE_MANAGEMENT.md)**
    - Resource allocation
    - Resource cleanup

42. **[OBSERVABILITY.md](OBSERVABILITY.md)**
    - Logging and tracing
    - Monitoring

### API Specifications

43. **[AI_FIRST_CITIZEN_API_SPECIFICATION.md](AI_FIRST_CITIZEN_API_SPECIFICATION.md)**
    - AI-first API design
    - LLM-friendly endpoints

44. **[REMOTE_EXECUTION_API_SPEC.md](REMOTE_EXECUTION_API_SPEC.md)**
    - Remote execution API
    - Task submission

---

## 🔄 MIGRATION & EVOLUTION

45. **[ASYNC_TRAIT_MIGRATION_SPECIFICATION.md](ASYNC_TRAIT_MIGRATION_SPECIFICATION.md)**
    - async/await patterns
    - Trait migration

46. **[MODERN_CRATE_CONSOLIDATION_SPECIFICATION.md](MODERN_CRATE_CONSOLIDATION_SPECIFICATION.md)**
    - Crate consolidation
    - Dependency cleanup

---

## 📚 SPECIAL TOPICS

47. **[SONGBIRD_NATIVE_RPC_SPECIFICATION.md](SONGBIRD_NATIVE_RPC_SPECIFICATION.md)**
    - Native RPC implementation
    - Performance optimization

48. **[SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md](SONGBIRD_IPV6_DUAL_STACK_SPECIFICATION.md)**
    - IPv6 support
    - Dual-stack networking

49. **[BIRDSONG_PROTOCOL.md](BIRDSONG_PROTOCOL.md)**
    - Songbird messaging protocol
    - Inter-primal communication

50. **[LINEAGE_GATED_RELAY_PROTOCOL.md](LINEAGE_GATED_RELAY_PROTOCOL.md)**
    - Genetic lineage verification
    - Trust hierarchies

---

## 🗂️ ARCHIVE

### Pre-HTTPS Success (Historical)

- `archive/pre-https-success/PURE_SONGBIRD_TLS.md` - Original TLS spec (superseded)
- `archive/pre-https-success/CURRENT_IMPLEMENTATION_STATUS.md` - Old status

### Deprecated Protocols

- `archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated`

---

## 🎯 NAVIGATION

### By Topic

| Topic | Start Here |
|-------|------------|
| **TLS/HTTPS** | SONGBIRD_TLS_13_COMPLETE.md, SONGBIRD_FUTURE_WORK.md |
| **Architecture** | PRIMAL_SELF_KNOWLEDGE_EVOLUTION_SPEC.md |
| **Discovery** | CAPABILITY_BASED_DISCOVERY_SPECIFICATION.md |
| **Federation** | FEDERATION_IMPLEMENTATION_SPECIFICATION.md |
| **Testing** | COMPREHENSIVE_TESTING_INFRASTRUCTURE_SPECIFICATION.md |
| **BearDog** | SONGBIRD_BEARDOG_INTEGRATION.md |

### By Priority

- ⭐ **CRITICAL** - Core architectural principles
- 📋 **IMPORTANT** - Implementation specifications  
- 📚 **REFERENCE** - Detailed specifications

---

## 📊 Summary

| Category | Count |
|----------|-------|
| **Active Specs** | 51 |
| **Archived** | 3 |
| **Current Focus** | IGD Router Configuration (cross-network blocker) |
| **Status** | QUIC + NFC Complete, IGD Implementing |

---

**Built with:** 100% Pure Rust | Zero C Dependencies | 9-Tier Multi-Path Protocol | S+ Deep Debt
