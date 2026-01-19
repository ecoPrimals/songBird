# 🚀 Deep Evolution Execution Plan

**Date**: January 19, 2026  
**Status**: 🔥 **EXECUTING**  
**Focus**: Deep debt solutions + Modern idiomatic Rust

---

## 🎯 EXECUTION PRIORITIES

Based on user guidance, prioritized for **maximum architectural impact**:

### **1. Capability-Based Discovery Evolution** 🔥 HIGH PRIORITY
**Current State**: Hardcoded primal names (beardog, toadstool, nestgate, squirrel)  
**Target State**: Capability-based runtime discovery  
**Principle**: "Primal code only has self-knowledge and discovers other primals in runtime"

**Impact**: Architectural - enables true universality and loose coupling

### **2. Production Unwrap Audit** 🔥 HIGH PRIORITY
**Current State**: 473 unwrap/expect instances across 78 files  
**Target State**: Proper error handling with context  
**Principle**: "Fast AND safe Rust"

**Impact**: Reliability - prevents panics in production

### **3. TODO Evolution** 🟡 MEDIUM PRIORITY
**Current State**: 39 TODO/FIXME across 17 files  
**Target State**: Complete implementations or tickets  
**Principle**: "Deep debt solutions, not superficial fixes"

**Impact**: Completeness - no mocks in production

### **4. Universal IPC Integration** 🟡 MEDIUM PRIORITY
**Current State**: Platform-specific IPC in Tower Atomic  
**Target State**: Using songbird-universal-ipc  
**Principle**: "External dependencies evolved to Rust"

**Impact**: Universality - works on ALL platforms

---

## 📊 CURRENT STATE ANALYSIS

### **Hardcoded Primal References**: ~3,493 instances (from audit)
**Breakdown**:
- BearDog references: ~1,200
- ToadStool references: ~800
- NestGate references: ~900
- Squirrel references: ~593

**Files with most hardcoding**: Need to analyze

### **Unwrap/Expect Usage**: 473 instances
**Top files** (need analysis):
- `graph/availability.rs`: 26 instances
- `ipc/primal_registry.rs`: 24 instances
- `trust/escalation.rs`: 18 instances
- `resource_management/scheduler.rs`: 18 instances
- `process_manager.rs`: 18 instances

### **TODOs**: 39 instances across 17 files
**Categories**:
- Implementation TODOs: ~30
- Architecture TODOs: ~5
- Test TODOs: ~4

---

## 🏗️ EVOLUTION STRATEGY

### **Phase 1: Capability-Based Discovery** (THIS PHASE)

#### **Step 1: Analyze Current Discovery Patterns**
```bash
# Find all primal discovery code
grep -r "beardog\|toadstool\|nestgate\|squirrel" --include="*.rs"
```

#### **Step 2: Create Capability Registry**
```rust
// New: crates/songbird-orchestrator/src/capability/registry.rs

pub struct CapabilityRegistry {
    // Maps capability → primal endpoints
    capabilities: HashMap<String, Vec<PrimalEndpoint>>,
}

impl CapabilityRegistry {
    pub async fn discover(&self, capability: &str) -> Result<PrimalEndpoint> {
        // 1. Check local cache
        // 2. Query NestGate (persistent)
        // 3. Broadcast discovery (p2p)
        // 4. Fallback to environment vars (transition)
    }
}
```

#### **Step 3: Evolve Hardcoded References**
**Before**:
```rust
// ❌ Hardcoded primal name
let beardog_url = env::var("BEARDOG_SOCKET_PATH")
    .unwrap_or("/tmp/beardog.sock".to_string());
```

**After**:
```rust
// ✅ Capability-based discovery
let crypto_provider = capability_registry
    .discover("crypto")
    .await?;
```

#### **Step 4: Integration with Universal IPC**
```rust
// Combine capability discovery + universal IPC
let endpoint = capability_registry.discover("crypto").await?;
let stream = ipc::connect(&endpoint.virtual_path).await?;
// ✅ Pure capability-based, platform-agnostic!
```

---

### **Phase 2: Production Unwrap Evolution**

#### **Strategy**: Systematic file-by-file evolution

**Priority Order**:
1. **Hot paths** (high-frequency code)
2. **Critical paths** (security, data integrity)
3. **Error-prone areas** (I/O, network, parsing)

**Evolution Pattern**:
```rust
// ❌ Before: Panic on error
let config = load_config().unwrap();

// ✅ After: Contextual error
let config = load_config()
    .context("Failed to load configuration")?;

// ✅ Or: Graceful fallback
let config = load_config()
    .unwrap_or_else(|e| {
        warn!("Config load failed: {}, using defaults", e);
        Config::default()
    });
```

---

### **Phase 3: TODO Resolution**

#### **Categories**:

**1. Complete Implementations** (Mocks → Real)
```rust
// ❌ TODO: Implement real auth
fn authenticate() -> bool {
    true // Mock
}

// ✅ Evolved
async fn authenticate(&self, token: &str) -> Result<AuthResult> {
    self.auth_provider.verify_token(token).await
}
```

**2. Architecture Decisions** (Create Tickets)
```rust
// TODO: Decide on consensus algorithm
// → Create ARCHITECTURE_DECISION_RECORD.md
// → Link to issue tracker
```

**3. Optimization Opportunities** (Benchmark First)
```rust
// TODO: Optimize hot path
// → Benchmark current performance
// → Profile to find bottleneck
// → Evolve with data
```

---

## 🎯 SUCCESS METRICS

### **Capability-Based Discovery**:
- [ ] Zero hardcoded primal names in production code
- [ ] Capability registry implemented
- [ ] Integration with Universal IPC
- [ ] Fallback strategy (environment vars for transition)

### **Production Unwraps**:
- [ ] <50 unwraps in production code (down from 473)
- [ ] Zero unwraps in hot paths
- [ ] Zero unwraps in critical paths
- [ ] Contextual errors everywhere

### **TODO Resolution**:
- [ ] Zero implementation TODOs (complete or ticket)
- [ ] Architecture TODOs → ADRs
- [ ] Test TODOs → comprehensive tests

---

## 🔥 EXECUTION LOG

### **Session 1: January 19, 2026** (Current)

**Completed**:
- ✅ Archive code cleanup (removed obsolete crates)
- ✅ Universal IPC Phase 1 (Unix support complete)
- ✅ ecoBin compliance verified (100% Pure Rust)

**In Progress**:
- 🔥 Analyzing hardcoded primal discovery patterns
- 🔥 Designing capability registry architecture

**Next Steps**:
1. Map all primal discovery call sites
2. Create capability registry module
3. Implement discovery strategies
4. Begin systematic evolution

---

## 📚 REFERENCE DOCUMENTS

- `COMPREHENSIVE_CODEBASE_AUDIT_JAN_19_2026.md` - Initial audit
- `DEEP_EVOLUTION_PLAN_JAN_19_2026.md` - Strategic plan
- `UNIVERSAL_IPC_PHASE1_COMPLETE_JAN_19_2026.md` - Foundation
- `CONNECTION_MANAGER_REFACTOR_COMPLETE_JAN_19_2026.md` - Smart refactor example

---

**Status**: 🔥 **EXECUTING**  
**Current Focus**: Capability-based discovery architecture  
**Goal**: True primal sovereignty - runtime discovery only!

🦀🧬✨ **Deep Evolution in Progress!** ✨🧬🦀

