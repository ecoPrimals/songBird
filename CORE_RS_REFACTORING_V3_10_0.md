# 🔧 Core.rs Smart Refactoring Plan - v3.10.0

**Date**: January 5, 2026  
**Current**: `core.rs` is **1711 lines** (71% over limit!)  
**Target**: Break into focused modules, each <400 lines  
**Status**: 🟡 **IN PROGRESS**

---

## 📊 Current State Analysis

### File: `crates/songbird-orchestrator/src/app/core.rs` (1711 lines)

**Method Distribution**:

| Category | Methods | Lines | % of File |
|----------|---------|-------|-----------|
| Initialization | `new()` | 287 | 16.8% |
| Lifecycle | `start()` | 337 | 19.7% |
| Discovery Bridge | `start_discovery_federation_bridge()` | 350 | 20.5% |
| Server Management | 4 methods | ~200 | 11.7% |
| Health & Monitoring | 6 methods | ~150 | 8.8% |
| Maintenance Tasks | 2 methods | 34 | 2.0% |
| Hardware Detection | 2 methods | 83 | 4.9% |
| Security/Identity | `query_security_identity()` | 40 | 2.3% |
| Struct & Getters | 5 methods | ~50 | 2.9% |
| Shutdown | `stop()` | 19 | 1.1% |

**Key Findings**:
1. **3 HUGE methods** account for 974 lines (57% of file!)
2. Clear separation of concerns already exists
3. Each extracted module would be highly cohesive

---

## 🎯 Smart Refactoring Strategy

### Principles

✅ **Group by Responsibility** (not arbitrary size)  
✅ **Maintain API Compatibility** (no breaking changes)  
✅ **Modern Idiomatic Rust** (trait-based design where appropriate)  
✅ **Zero Hardcoding** (capability-based, discoverable)  
✅ **Keep Related Code Together** (cohesion over coupling)

### Target Structure

```
crates/songbird-orchestrator/src/app/
├── core.rs (~200 lines) ✨ SLIM!
│   ├── SongbirdOrchestrator struct definition
│   ├── Getters (federation_state, service_registry, etc.)
│   ├── start() - high-level orchestration only
│   └── stop() - graceful shutdown
│
├── initialization.rs (~300 lines) 🆕
│   └── impl SongbirdOrchestrator::new()
│       ├── Config loading
│       ├── State initialization
│       ├── Component creation
│       └── Self-registration
│
├── discovery_bridge.rs (~360 lines) 🆕
│   └── impl SongbirdOrchestrator::start_discovery_federation_bridge()
│       ├── Peer discovery loop
│       ├── Trust evaluation
│       ├── Connection establishment
│       └── Federation registration
│
├── server_lifecycle.rs (~250 lines) 🆕
│   ├── start_http_server()
│   ├── start_ipc_server()
│   ├── start_tarpc_server()
│   └── verify_external_connectivity()
│
├── identity_provider.rs (~80 lines) 🆕
│   ├── query_security_identity()
│   └── BearDog integration helpers
│
├── maintenance_tasks.rs (~100 lines) 🆕
│   ├── start_session_ttl_cleanup()
│   ├── start_service_registry_cleanup()
│   └── Future cleanup tasks
│
├── hardware_detection.rs (~120 lines) 🆕
│   ├── detect_gpu()
│   ├── detect_storage_capacity()
│   ├── detect_cpu_info()
│   └── detect_memory_info()
│
├── health.rs (existing ✅)
│   ├── Health check implementations
│   ├── Status types
│   └── Monitoring
│
├── network.rs (existing ✅)
├── startup.rs (existing ✅)
├── http_server.rs (existing ✅)
├── connection_manager.rs (existing ✅)
├── discovery_status_manager.rs (existing ✅)
└── federation.rs (existing ✅)
```

---

## 🚀 Implementation Plan

### Phase 1: Extract Discovery Bridge (Highest Impact)

**File**: `discovery_bridge.rs`  
**Lines**: ~360  
**Impact**: Reduces core.rs by 20.5%

```rust
//! Discovery → Federation Bridge
//!
//! Manages the automatic bridging of UDP multicast discoveries
//! to federation membership via progressive trust evaluation.

use super::core::SongbirdOrchestrator;
use anyhow::Result;

impl SongbirdOrchestrator {
    /// Start discovery → federation bridge
    ///
    /// Polls the discovery listener every 10s and evaluates trust
    /// for each discovered peer, establishing connections as appropriate.
    pub(super) async fn start_discovery_federation_bridge(&self) -> Result<()> {
        // Move lines 1054-1404 here
        // ...
    }
}
```

### Phase 2: Extract Initialization (Second Highest Impact)

**File**: `initialization.rs`  
**Lines**: ~300  
**Impact**: Reduces core.rs by 16.8%

```rust
//! Orchestrator Initialization
//!
//! Handles the complex initialization sequence for SongbirdOrchestrator.

use super::core::SongbirdOrchestrator;
use anyhow::Result;
use songbird_types::config::CanonicalSongbirdConfig;

impl SongbirdOrchestrator {
    /// Create new orchestrator instance
    ///
    /// This performs:
    /// 1. Configuration validation
    /// 2. State initialization
    /// 3. Component creation (federation, trust, discovery)
    /// 4. Self-registration
    pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
        // Move lines 64-351 here
        // ...
    }
}
```

### Phase 3: Extract Server Lifecycle

**File**: `server_lifecycle.rs`  
**Lines**: ~250  
**Impact**: Reduces core.rs by 14.6%

```rust
//! Server Lifecycle Management
//!
//! Manages HTTP, IPC, and tRPC server lifecycles.

use super::core::SongbirdOrchestrator;
use anyhow::Result;

impl SongbirdOrchestrator {
    pub(super) async fn start_http_server(&self) -> Result<u16> {
        // Move lines 760-824 here
    }
    
    pub(super) async fn start_ipc_server(&mut self) -> Result<()> {
        // Move lines 825-913 here
    }
    
    pub(super) async fn start_tarpc_server(&self) -> Result<()> {
        // Move lines 914-951 here
    }
    
    pub(super) async fn verify_external_connectivity(&self) -> Result<()> {
        // Move lines 952-1053 here
    }
}
```

### Phase 4: Extract Identity Provider

**File**: `identity_provider.rs`  
**Lines**: ~80  
**Impact**: Reduces core.rs by 4.7%

```rust
//! Security Identity Provider Integration
//!
//! Queries BearDog (or other security providers) for identity attestations.

use super::core::SongbirdOrchestrator;
use anyhow::Result;

impl SongbirdOrchestrator {
    pub(super) async fn query_security_identity(&self) -> Result<()> {
        // Move lines 381-421 here
        // Expand with additional identity methods
    }
}
```

### Phase 5: Extract Maintenance & Hardware Detection

**File**: `maintenance_tasks.rs` (~100 lines)  
**File**: `hardware_detection.rs` (~120 lines)  
**Impact**: Reduces core.rs by 12.6%

---

## 📏 Before & After Metrics

### Before Refactoring

```
core.rs: 1711 lines
  ├── new():                                287 lines ❌ TOO BIG
  ├── start():                              337 lines ❌ TOO BIG
  ├── start_discovery_federation_bridge():  350 lines ❌ TOO BIG
  ├── Other methods:                        737 lines
  └── Status: 71% OVER LIMIT
```

### After Refactoring

```
core.rs:                      ~200 lines ✅ UNDER LIMIT
initialization.rs:            ~300 lines ✅ UNDER LIMIT
discovery_bridge.rs:          ~360 lines ✅ UNDER LIMIT
server_lifecycle.rs:          ~250 lines ✅ UNDER LIMIT
identity_provider.rs:          ~80 lines ✅ UNDER LIMIT
maintenance_tasks.rs:         ~100 lines ✅ UNDER LIMIT
hardware_detection.rs:        ~120 lines ✅ UNDER LIMIT

Total: 1,410 lines across 7 cohesive modules
Average: 201 lines per module
Max: 360 lines (discovery_bridge)
Status: ✅ ALL UNDER LIMIT
```

---

## 🎯 Success Criteria

| Criterion | Target | Status |
|-----------|--------|--------|
| core.rs size | < 300 lines | 🟡 Pending |
| Max module size | < 400 lines | 🟡 Pending |
| Module cohesion | High | 🟡 Pending |
| API compatibility | 100% | 🟡 Pending |
| Zero hardcoding | Yes | 🟡 Pending |
| Zero unsafe | Yes | ✅ Already met |
| Compilation | Clean | 🟡 Pending |
| Tests pass | 100% | 🟡 Pending |
| Documentation | Complete | 🟡 Pending |

---

## 🔬 Modern Rust Patterns to Apply

### 1. Builder Pattern for Complex Initialization

**Before**:
```rust
pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
    // 287 lines of initialization...
}
```

**After**:
```rust
pub async fn new(config: CanonicalSongbirdConfig) -> Result<Self> {
    OrchestratorBuilder::new(config)
        .with_federation()
        .with_discovery()
        .with_observability()
        .build()
        .await
}
```

### 2. Trait-Based Server Management

**Before**:
```rust
async fn start_http_server(&self) -> Result<u16> { ... }
async fn start_ipc_server(&mut self) -> Result<()> { ... }
async fn start_tarpc_server(&self) -> Result<()> { ... }
```

**After**:
```rust
trait ServerLifecycle {
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
}

impl ServerLifecycle for HttpServer { ... }
impl ServerLifecycle for IpcServer { ... }
impl ServerLifecycle for TarpcServer { ... }
```

### 3. Capability-Based Identity Resolution

**Before**:
```rust
let security_client_endpoint = std::env::var("SONGBIRD_BEARDOG_URL").ok();
```

**After**:
```rust
let identity_provider = IdentityProviderRegistry::discover()
    .with_capability("security.identity_attestation")
    .resolve()
    .await?;
```

---

## 🚦 Implementation Order

### Priority 1: Discovery Bridge (P0 - Critical)

**Why**: Largest single method (350 lines), most complex, highest maintenance burden

**Steps**:
1. Create `discovery_bridge.rs`
2. Move `start_discovery_federation_bridge()` method
3. Add comprehensive documentation
4. Add unit tests for trust evaluation logic
5. Verify compilation
6. Update imports in core.rs

**ETA**: 2-3 hours

### Priority 2: Initialization (P0 - Critical)

**Why**: Second largest method (287 lines), critical path

**Steps**:
1. Create `initialization.rs`
2. Move `new()` method
3. Consider builder pattern
4. Add documentation
5. Verify all initialization paths
6. Update imports

**ETA**: 2-3 hours

### Priority 3: Server Lifecycle (P1 - High)

**Why**: 4 related methods, natural cohesion

**Steps**:
1. Create `server_lifecycle.rs`
2. Move 4 server methods
3. Consider trait-based design
4. Add documentation
5. Verify all server paths

**ETA**: 1-2 hours

### Priority 4: Identity & Maintenance (P2 - Medium)

**Why**: Smaller, self-contained

**Steps**:
1. Create `identity_provider.rs` and `maintenance_tasks.rs`
2. Move respective methods
3. Add capability-based discovery to identity
4. Add documentation

**ETA**: 1 hour

### Priority 5: Hardware Detection (P3 - Low)

**Why**: Utility functions, lowest coupling

**Steps**:
1. Create `hardware_detection.rs`
2. Move detection methods
3. Expand with CPU/memory detection
4. Add cross-platform support

**ETA**: 1 hour

---

## 📚 Documentation Requirements

Each new module MUST have:

1. **Module-level doc comment**:
   ```rust
   //! Brief description
   //!
   //! Detailed explanation of:
   //! - Responsibility
   //! - Key types
   //! - Usage examples
   //! - Related modules
   ```

2. **Method documentation**:
   ```rust
   /// Brief description
   ///
   /// # Arguments
   ///
   /// * `arg` - Description
   ///
   /// # Returns
   ///
   /// * `Ok(T)` - Success case
   /// * `Err(E)` - Error cases
   ///
   /// # Example
   ///
   /// ```rust,ignore
   /// let result = method().await?;
   /// ```
   ```

3. **Architecture notes** for complex logic

4. **TODO comments** for future improvements

---

## 🎯 Next Steps

1. ✅ Create this refactoring plan
2. ⏳ Implement Priority 1 (Discovery Bridge)
3. ⏳ Implement Priority 2 (Initialization)
4. ⏳ Implement Priority 3 (Server Lifecycle)
5. ⏳ Implement Priority 4 (Identity & Maintenance)
6. ⏳ Implement Priority 5 (Hardware Detection)
7. ⏳ Update README with new architecture
8. ⏳ Add integration tests
9. ⏳ Deploy and verify

---

**Status**: 🟡 **PLAN COMPLETE - READY FOR EXECUTION**  
**Impact**: Core.rs reduced from 1711 → ~200 lines (88% reduction!)  
**Maintainability**: 🏆 Excellent (focused, cohesive modules)

🚀 **Ready to execute - proceeding with Priority 1!**

