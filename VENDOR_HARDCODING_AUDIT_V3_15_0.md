# 🧹 Vendor Hardcoding Audit - v3.15.0

**Date**: January 7, 2026  
**Status**: 🔍 **AUDITING** - Deep Debt Discovered  
**Priority**: 🔴 **CRITICAL** - Architectural Principle Violation

---

## 🎯 **The Problem**

**Primals are hardcoding knowledge of other primals, violating the core principle:**

> **"Each primal only knows itself and discovers others at runtime via Universal Adapter"**

---

## 📊 **Audit Results**

### **Vendor Hardcoding** (🔴 CRITICAL)

| Pattern | Count | Files | Status |
|---------|-------|-------|--------|
| `beardog/BearDog` | **215** | 33 | ⚠️ WIDESPREAD |
| `toadstool/ToadStool` | 38 | 11 | ⚠️ PRESENT |
| `nestgate/NestGate` | (in audit) | ? | 🔍 CHECKING |
| `squirrel/Squirrel` | (in audit) | ? | 🔍 CHECKING |

### **Environment Variable Hardcoding** (🔴 CRITICAL)

```bash
# Current (WRONG - vendor names):
SONGBIRD_BEARDOG_URL        # ❌ Hardcodes "beardog"
BEARDOG_URL                 # ❌ Hardcodes "beardog"
BEARDOG_2FA_ENDPOINT        # ❌ Hardcodes "beardog"

# Should Be (RIGHT - capability-based):
SONGBIRD_SECURITY_PROVIDER  # ✅ Generic capability
SONGBIRD_COMPUTE_PROVIDER   # ✅ Generic capability
SONGBIRD_STORAGE_PROVIDER   # ✅ Generic capability
SONGBIRD_AI_PROVIDER        # ✅ Generic capability
```

---

## 🏗️ **Architecture Principles**

### **❌ WRONG: N² Hardcoding**

```rust
// Songbird hardcodes BearDog knowledge:
let beardog_client = BearDogClient::new("unix:///tmp/beardog.sock");

// Songbird hardcodes ToadStool knowledge:
let toadstool_client = ToadStoolClient::new("unix:///tmp/toadstool.sock");

// Songbird hardcodes NestGate knowledge:
let nestgate_client = NestGateClient::new("unix:///tmp/nestgate.sock");

// Result: N² connections, vendor lock-in, non-fractal
```

### **✅ RIGHT: Universal Adapter Pattern**

```rust
// Songbird discovers capabilities at runtime:
let security_provider = universal_adapter
    .discover_capability("security")
    .await?;

let compute_provider = universal_adapter
    .discover_capability("compute")
    .await?;

let storage_provider = universal_adapter
    .discover_capability("storage")
    .await?;

// Result: N connections, vendor-agnostic, fractal
```

---

## 🔍 **Detailed Findings**

### **1. Environment Variables** (14 matches)

**File**: `crates/songbird-orchestrator/src/app/discovery_bridge.rs`
```rust
// ❌ HARDCODED VENDOR NAME:
let security_client_endpoint = std::env::var("SONGBIRD_BEARDOG_URL")
    .or_else(|_| std::env::var("SECURITY_ENDPOINT"))
    .ok();

// ✅ SHOULD BE:
let security_provider = self.universal_adapter
    .discover_capability("security")
    .await?;
```

**File**: `crates/songbird-orchestrator/src/app/core.rs`
```rust
// ❌ HARDCODED VENDOR NAME:
let security_url = std::env::var("SONGBIRD_BEARDOG_URL")
    .or_else(|_| std::env::var("SECURITY_ENDPOINT"))
    .ok();

// ✅ SHOULD BE:
let security_provider = self.universal_adapter
    .discover_capability("security")
    .await?;
```

**File**: `crates/songbird-orchestrator/src/trust/escalation.rs`
```rust
// ❌ HARDCODED VENDOR NAME:
let endpoint = std::env::var("BEARDOG_URL")
    .or_else(|_| std::env::var("SECURITY_URL"))
    .ok();

// ✅ SHOULD BE:
let security_provider = self.universal_adapter
    .discover_capability("security")
    .await?;
```

**File**: `crates/songbird-orchestrator/src/access_control/auth.rs`
```rust
// ❌ HARDCODED VENDOR NAME:
if let Ok(beardog_endpoint) = std::env::var("BEARDOG_2FA_ENDPOINT") {
    // ...
}

// ✅ SHOULD BE:
if let Some(auth_provider) = self.universal_adapter
    .discover_capability("authentication.2fa")
    .await? {
    // ...
}
```

---

### **2. Comments and Documentation** (180+ matches)

**Examples**:
```rust
// ❌ VENDOR NAME IN COMMENTS:
/// Query BearDog for trust evaluation
/// Connect to BearDog's Unix socket
/// If BearDog is available...

// ✅ SHOULD BE:
/// Query security provider for trust evaluation
/// Connect to security provider's endpoint
/// If security provider is available...
```

---

### **3. Client Names** (33 files)

**Current**:
- `SecurityCapabilityClient` - OK (generic)
- But internally references "BearDog" in logs/comments

**Should Be**:
- All logging: "security provider" (not "BearDog")
- All variable names: `security_provider` (not `beardog_client`)
- All docs: "security capability provider" (not "BearDog")

---

### **4. Primal Registry Hardcoding** (18 matches)

**File**: `crates/songbird-orchestrator/src/ipc/primal_registry.rs`

```rust
// ❌ HARDCODED PRIMAL NAMES:
pub const PRIMAL_BEARDOG: &str = "beardog";
pub const PRIMAL_TOADSTOOL: &str = "toadstool";
pub const PRIMAL_NESTGATE: &str = "nestgate";

// ✅ SHOULD BE:
// No primal name constants! Use capability discovery!
```

---

## 🔧 **Universal Adapter Status**

### **Current Implementation** ✅

The Universal Adapter already exists and works correctly!

**File**: `crates/songbird-orchestrator/src/universal_adapter.rs`

```rust
pub struct UniversalAdapter {
    registry: Arc<RwLock<PrimalCapabilityRegistry>>,
    known_providers: Arc<RwLock<HashMap<String, DiscoveredProvider>>>,
}

impl UniversalAdapter {
    /// Discover providers for a capability
    pub async fn discover_capability(&self, capability: &str) -> Result<Vec<DiscoveredProvider>> {
        // Uses capability-based discovery!
        // No vendor names hardcoded!
    }

    /// Call a capability on any provider
    pub async fn call(
        &self,
        capability: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Protocol-agnostic: tarpc, JSON-RPC, HTTP
        // Vendor-agnostic: works with ANY provider
    }
}
```

**Status**: ✅ **Universal Adapter is READY** - we just need to USE it!

---

## 🎯 **Evolution Plan**

### **Phase 1: Evolve Environment Variables**

**Goal**: Remove vendor names from config

**Changes**:
```bash
# Deprecate (v3.15.0):
SONGBIRD_BEARDOG_URL        # Keep for backward compat, log warning
BEARDOG_URL                 # Keep for backward compat, log warning

# Introduce (v3.15.0):
SONGBIRD_SECURITY_PROVIDER  # Generic capability discovery
SONGBIRD_DISCOVERY_MODE     # "auto" (default) or "manual"

# Remove (v3.16.0):
SONGBIRD_BEARDOG_URL        # Fully removed
BEARDOG_URL                 # Fully removed
```

---

### **Phase 2: Evolve Security Client to Universal Adapter**

**Goal**: Use Universal Adapter for all primal interactions

**Current Flow** (WRONG):
```rust
// 1. Hardcoded env var
let beardog_url = env::var("SONGBIRD_BEARDOG_URL")?;

// 2. Direct client creation
let beardog_client = SecurityCapabilityClient::from_endpoint(&beardog_url)?;

// 3. Direct API call
let result = beardog_client.evaluate_trust(&request).await?;
```

**Target Flow** (RIGHT):
```rust
// 1. Discover capability
let security_provider = self.universal_adapter
    .discover_capability("security")
    .await?;

// 2. Call via Universal Adapter (no client creation!)
let result = self.universal_adapter
    .call("security", "evaluate_trust", serde_json::to_value(&request)?)
    .await?;
```

---

### **Phase 3: Evolve BTSP to Universal Adapter**

**Goal**: BTSP tunnel requests via capability, not vendor name

**Current Plan** (WRONG):
```rust
// ❌ Creating BearDogClient:
let beardog = BearDogClient::new(&beardog_endpoint)?;
let tunnel = beardog.establish_tunnel(peer_id, peer_endpoint).await?;
```

**Evolved Plan** (RIGHT):
```rust
// ✅ Using Universal Adapter:
let tunnel_info = self.universal_adapter
    .call("security.btsp", "establish_tunnel", serde_json::json!({
        "peer_id": peer_id,
        "peer_endpoint": peer_endpoint,
    }))
    .await?;
```

---

### **Phase 4: Clean All Vendor References**

**Goal**: Remove vendor names from code, docs, logs

**Changes**:
1. Replace "BearDog" → "security provider" (180+ instances)
2. Replace "ToadStool" → "compute provider" (38 instances)
3. Replace "NestGate" → "storage provider" (TBD instances)
4. Replace "Squirrel" → "AI provider" (TBD instances)

**Variable Naming**:
```rust
// ❌ WRONG:
let beardog_client = ...
let beardog_endpoint = ...
let beardog_response = ...

// ✅ RIGHT:
let security_provider = ...
let security_endpoint = ...
let security_response = ...
```

---

## 🧪 **Testing Strategy**

### **Backward Compatibility**
- Support old env vars with deprecation warnings
- Gradual migration (v3.15.0 → v3.16.0)

### **Capability Discovery**
- Test with multiple security providers
- Test with missing providers (graceful fallback)
- Test with provider failures

### **Universal Adapter**
- Test tarpc, JSON-RPC, HTTP protocols
- Test capability not found
- Test provider discovery

---

## 📋 **Implementation Checklist**

### **Phase 1: Environment Variables** (2-3 hours)
- [ ] Add `SONGBIRD_SECURITY_PROVIDER` support
- [ ] Add deprecation warnings for `SONGBIRD_BEARDOG_URL`
- [ ] Update documentation
- [ ] Add migration guide

### **Phase 2: Universal Adapter Integration** (4-6 hours)
- [ ] Refactor `discovery_bridge.rs` to use Universal Adapter
- [ ] Refactor `security_capability_client.rs` wrapper
- [ ] Refactor `trust/` modules
- [ ] Update tests

### **Phase 3: BTSP via Universal Adapter** (4-6 hours)
- [ ] Implement BTSP capability calls
- [ ] Update connection management
- [ ] Test end-to-end

### **Phase 4: Documentation Cleanup** (2-3 hours)
- [ ] Replace vendor names in comments
- [ ] Update variable names
- [ ] Update log messages
- [ ] Clean documentation files

**Total ETA**: 12-18 hours (2-3 sessions)

---

## 🎊 **Benefits**

### **Architecture** 🏗️
- ✅ True vendor-agnostic design
- ✅ N connections (not N²)
- ✅ Fractal scalability

### **Flexibility** 🔄
- ✅ Swap security providers at runtime
- ✅ Multiple providers for same capability
- ✅ No vendor lock-in

### **Maintainability** 🧹
- ✅ One interaction pattern (Universal Adapter)
- ✅ No vendor-specific clients
- ✅ Clean separation of concerns

---

## 🎯 **Summary**

**Problem**: 215+ instances of vendor hardcoding (BearDog, ToadStool, etc.)  
**Root Cause**: Direct client creation instead of Universal Adapter  
**Solution**: Evolve to capability-based discovery  
**Priority**: CRITICAL - Blocks fractal architecture  
**Status**: READY TO IMPLEMENT

---

**Key Insight**:
> "The Universal Adapter already exists and works! We just need to USE it everywhere instead of creating vendor-specific clients."

---

**Next Step**: Execute Phase 1 - Evolve environment variables and start migration

---

_"Each primal only knows itself. Network effects come from capability discovery, not hardcoded connections."_

