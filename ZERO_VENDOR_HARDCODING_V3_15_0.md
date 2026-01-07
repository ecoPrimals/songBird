# 🧹 Zero Vendor Hardcoding Evolution - v3.15.0

**Date**: January 7, 2026  
**Status**: 🎯 **READY TO EXECUTE**  
**Priority**: 🔴 **CRITICAL** - Architectural Principle

---

## 🎯 **The Principle**

> **"Each primal only knows itself and discovers others at runtime via Universal Adapter"**

**NO vendor names. NO primal names. ONLY capabilities.**

---

## 📊 **Current State**

### **The Problem** ⚠️

**Songbird hardcodes knowledge of other primals:**

```rust
// ❌ WRONG - Dev knowledge, not self-knowledge:
let beardog_url = env::var("SONGBIRD_BEARDOG_URL")?;  // Hardcodes "beardog"
let beardog_client = BearDogClient::new(&beardog_url)?;  // Vendor-specific client
let result = beardog_client.evaluate_trust(&request).await?;  // Direct call
```

### **What Should Be** ✅

```rust
// ✅ RIGHT - Self-knowledge + capability discovery:
let security_provider = self.universal_adapter
    .discover_capability("security")  // Generic capability
    .await?;

let result = self.universal_adapter
    .call(&security_provider, "evaluate_trust", params)  // Universal call
    .await?;
```

---

## 🔧 **Implementation Plan**

### **Phase 1: Environment Variables** (2-3 hours)

**Goal**: Add capability-based env vars, deprecate vendor names

#### **Step 1.1: Add New Environment Variables**

**File**: `crates/songbird-orchestrator/src/app/security_setup.rs`

```rust
/// Discover security provider endpoint
///
/// Priority:
/// 1. `SONGBIRD_SECURITY_PROVIDER` (NEW - generic capability)
/// 2. `SECURITY_ENDPOINT` (existing - generic)
/// 3. `SONGBIRD_BEARDOG_URL` (DEPRECATED - vendor-specific)
/// 4. Discovery via Universal Adapter (fallback)
pub async fn discover_security_provider(
    universal_adapter: &mut UniversalAdapter,
) -> Result<String> {
    // 1. NEW: Generic capability env var (HIGHEST PRIORITY)
    if let Ok(endpoint) = std::env::var("SONGBIRD_SECURITY_PROVIDER") {
        info!("🔐 Security provider: {} (via SONGBIRD_SECURITY_PROVIDER)", endpoint);
        return Ok(endpoint);
    }

    // 2. EXISTING: Generic security endpoint
    if let Ok(endpoint) = std::env::var("SECURITY_ENDPOINT") {
        info!("🔐 Security provider: {} (via SECURITY_ENDPOINT)", endpoint);
        return Ok(endpoint);
    }

    // 3. DEPRECATED: Vendor-specific env var (backward compat)
    if let Ok(endpoint) = std::env::var("SONGBIRD_BEARDOG_URL") {
        warn!("⚠️  DEPRECATED: SONGBIRD_BEARDOG_URL is deprecated.");
        warn!("   Use SONGBIRD_SECURITY_PROVIDER instead.");
        warn!("   This will be removed in v3.16.0.");
        return Ok(endpoint);
    }

    // 4. FALLBACK: Discover via Universal Adapter
    info!("🔍 No security provider configured, discovering via Universal Adapter...");
    let providers = universal_adapter.discover_capability("security").await?;
    
    if providers.is_empty() {
        return Err(anyhow!("No security provider found"));
    }

    let endpoint = providers[0].endpoint.clone();
    info!("✅ Discovered security provider: {}", endpoint);
    Ok(endpoint)
}
```

#### **Step 1.2: Update All Call Sites**

**Files to Update**:
- `crates/songbird-orchestrator/src/app/discovery_bridge.rs`
- `crates/songbird-orchestrator/src/app/core.rs`
- `crates/songbird-orchestrator/src/app/discovery_startup.rs`
- `crates/songbird-orchestrator/src/trust/escalation.rs`
- `crates/songbird-orchestrator/src/access_control/auth.rs`

**Before**:
```rust
let security_client_endpoint = std::env::var("SONGBIRD_BEARDOG_URL")
    .or_else(|_| std::env::var("SECURITY_ENDPOINT"))
    .ok();
```

**After**:
```rust
let security_endpoint = discover_security_provider(&mut self.universal_adapter).await.ok();
```

---

### **Phase 2: Evolve Discovery Bridge** (3-4 hours)

**Goal**: Use Universal Adapter for all primal interactions

#### **Step 2.1: Refactor Trust Evaluation**

**File**: `crates/songbird-orchestrator/src/app/discovery_bridge.rs`

**Before** (WRONG):
```rust
// Hardcoded vendor name and direct client creation:
let security_client_endpoint = std::env::var("SONGBIRD_BEARDOG_URL")
    .or_else(|_| std::env::var("SECURITY_ENDPOINT"))
    .ok();

if let Some(ref sec_endpoint) = security_client_endpoint {
    let security_client = SecurityCapabilityClient::from_endpoint(sec_endpoint)?;
    let decision = evaluate_peer_trust(&discovered_peer, &security_client).await?;
    // ...
}
```

**After** (RIGHT):
```rust
// Generic capability discovery and universal call:
if let Ok(security_provider) = self.universal_adapter
    .discover_capability("security")
    .await
{
    let decision = self.evaluate_peer_trust_via_adapter(
        &discovered_peer,
        &security_provider,
    ).await?;
    // ...
}

async fn evaluate_peer_trust_via_adapter(
    &self,
    peer: &DiscoveredPeer,
    security_provider: &DiscoveredProvider,
) -> Result<TrustDecision> {
    // Build request
    let request = serde_json::json!({
        "peer_id": peer.node_id,
        "peer_family": extract_family_from_tags(&peer.tags),
        "peer_tags": peer.tags,
        "connection_info": {
            "endpoint": peer.endpoint,
            "protocol": "tarpc",
        },
    });

    // Call via Universal Adapter (handles protocol automatically!)
    let response = self.universal_adapter
        .call(security_provider, "evaluate_trust", request)
        .await?;

    // Parse response
    let decision: TrustEvaluationResponse = serde_json::from_value(response)?;
    Ok(decision.into())
}
```

#### **Step 2.2: Evolve BTSP Tunnel Requests**

**Before** (WRONG - from original plan):
```rust
// ❌ Creating vendor-specific client:
let beardog = BearDogClient::new(&beardog_endpoint)?;
let tunnel = beardog.establish_tunnel(peer_id, peer_endpoint).await?;
```

**After** (RIGHT):
```rust
// ✅ Using Universal Adapter:
async fn establish_btsp_tunnel(
    &self,
    peer_id: &str,
    peer_endpoint: &str,
) -> Result<TunnelInfo> {
    // Discover security provider (could be BearDog, could be anything!)
    let providers = self.universal_adapter
        .discover_capability("security.btsp")
        .await?;

    if providers.is_empty() {
        return Err(anyhow!("No BTSP tunnel provider found"));
    }

    let security_provider = &providers[0];

    // Request tunnel via Universal Adapter
    let request = serde_json::json!({
        "peer_id": peer_id,
        "peer_endpoint": peer_endpoint,
    });

    let response = self.universal_adapter
        .call(security_provider, "establish_tunnel", request)
        .await?;

    let tunnel_info: TunnelInfo = serde_json::from_value(response)?;
    Ok(tunnel_info)
}
```

---

### **Phase 3: Clean Documentation** (2-3 hours)

**Goal**: Remove vendor names from comments, logs, docs

#### **Step 3.1: Replace Vendor Names**

**Pattern**: Find and replace (case-insensitive):

| Find | Replace |
|------|---------|
| `BearDog` | `security provider` |
| `beardog` | `security provider` |
| `ToadStool` | `compute provider` |
| `toadstool` | `compute provider` |
| `NestGate` | `storage provider` |
| `nestgate` | `storage provider` |
| `Squirrel` | `AI provider` |
| `squirrel` | `AI provider` |

**Examples**:

```rust
// ❌ BEFORE:
/// Query BearDog for trust evaluation
let beardog_client = ...;
info!("Connecting to BearDog at {}", endpoint);

// ✅ AFTER:
/// Query security provider for trust evaluation
let security_provider = ...;
info!("Connecting to security provider at {}", endpoint);
```

#### **Step 3.2: Update Variable Names**

```rust
// ❌ BEFORE:
let beardog_endpoint = ...;
let beardog_client = ...;
let beardog_response = ...;

// ✅ AFTER:
let security_endpoint = ...;
let security_provider = ...;
let security_response = ...;
```

#### **Step 3.3: Update Log Messages**

```rust
// ❌ BEFORE:
info!("✅ BearDog trust evaluation: {}", decision);
warn!("⚠️  BearDog unavailable, falling back");
error!("❌ BearDog connection failed");

// ✅ AFTER:
info!("✅ Security provider trust evaluation: {}", decision);
warn!("⚠️  Security provider unavailable, falling back");
error!("❌ Security provider connection failed");
```

---

### **Phase 4: Remove Primal Name Constants** (1-2 hours)

**Goal**: Delete primal name constants from registry

#### **Step 4.1: Audit `primal_registry.rs`**

**File**: `crates/songbird-orchestrator/src/ipc/primal_registry.rs`

**Before** (WRONG):
```rust
pub const PRIMAL_BEARDOG: &str = "beardog";
pub const PRIMAL_TOADSTOOL: &str = "toadstool";
pub const PRIMAL_NESTGATE: &str = "nestgate";
```

**After** (RIGHT):
```rust
// ✅ NO primal name constants!
// Primals register themselves with capabilities, not names
```

#### **Step 4.2: Update Registration Logic**

**Before** (WRONG):
```rust
// Hardcoded primal names:
registry.register(PRIMAL_BEARDOG, capabilities)?;
```

**After** (RIGHT):
```rust
// Self-registration via capabilities:
registry.register_capabilities(vec!["security", "encryption", "btsp"])?;
// Primal name is never used!
```

---

## 🧪 **Testing Strategy**

### **Unit Tests**

```rust
#[tokio::test]
async fn test_discover_security_provider_new_env_var() {
    std::env::set_var("SONGBIRD_SECURITY_PROVIDER", "unix:///tmp/security.sock");
    
    let mut adapter = UniversalAdapter::new().await.unwrap();
    let endpoint = discover_security_provider(&mut adapter).await.unwrap();
    
    assert_eq!(endpoint, "unix:///tmp/security.sock");
    std::env::remove_var("SONGBIRD_SECURITY_PROVIDER");
}

#[tokio::test]
async fn test_deprecated_env_var_warns() {
    std::env::set_var("SONGBIRD_BEARDOG_URL", "unix:///tmp/beardog.sock");
    
    let mut adapter = UniversalAdapter::new().await.unwrap();
    let endpoint = discover_security_provider(&mut adapter).await.unwrap();
    
    // Should work but log deprecation warning
    assert_eq!(endpoint, "unix:///tmp/beardog.sock");
    std::env::remove_var("SONGBIRD_BEARDOG_URL");
}

#[tokio::test]
async fn test_trust_evaluation_via_adapter() {
    // Set up Universal Adapter with mock security provider
    std::env::set_var("CAPABILITY_PROVIDERS", "security=http://localhost:9000");
    
    let mut adapter = UniversalAdapter::new().await.unwrap();
    let providers = adapter.discover_capability("security").await.unwrap();
    
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].capabilities, vec!["security"]);
    
    std::env::remove_var("CAPABILITY_PROVIDERS");
}
```

### **Integration Tests**

```rust
#[tokio::test]
async fn test_btsp_tunnel_via_universal_adapter() {
    // Test BTSP tunnel establishment without hardcoding vendor name
    // ...
}

#[tokio::test]
async fn test_multiple_security_providers() {
    // Test with multiple security providers available
    // Ensure Songbird can use any of them
    // ...
}
```

---

## 📋 **Migration Checklist**

### **Phase 1: Environment Variables** (2-3 hours)
- [ ] Add `discover_security_provider()` function
- [ ] Add `SONGBIRD_SECURITY_PROVIDER` support
- [ ] Add deprecation warnings for `SONGBIRD_BEARDOG_URL`
- [ ] Update `discovery_bridge.rs` call sites
- [ ] Update `core.rs` call sites
- [ ] Update `discovery_startup.rs` call sites
- [ ] Update `trust/escalation.rs` call sites
- [ ] Update `access_control/auth.rs` call sites
- [ ] Add unit tests

### **Phase 2: Universal Adapter Integration** (3-4 hours)
- [ ] Refactor `evaluate_peer_trust` to use adapter
- [ ] Implement `evaluate_peer_trust_via_adapter`
- [ ] Refactor `establish_btsp_tunnel` to use adapter
- [ ] Update `ConnectionManager` to store provider info
- [ ] Add integration tests
- [ ] Verify end-to-end flow

### **Phase 3: Documentation Cleanup** (2-3 hours)
- [ ] Replace "BearDog" → "security provider" (180+ instances)
- [ ] Replace "ToadStool" → "compute provider" (38 instances)
- [ ] Update variable names (`beardog_` → `security_`)
- [ ] Update log messages
- [ ] Update code comments
- [ ] Update README and documentation files

### **Phase 4: Primal Registry** (1-2 hours)
- [ ] Remove `PRIMAL_BEARDOG`, `PRIMAL_TOADSTOOL`, etc.
- [ ] Update registration logic to use capabilities
- [ ] Add tests for capability-based registration
- [ ] Verify no primal name constants remain

**Total ETA**: 8-12 hours (2 sessions)

---

## 🎊 **Benefits**

### **Architecture** 🏗️
- ✅ Zero vendor hardcoding
- ✅ True vendor-agnostic design
- ✅ N connections (not N²)
- ✅ Fractal scalability

### **Flexibility** 🔄
- ✅ Swap providers at runtime
- ✅ Multiple providers for same capability
- ✅ No vendor lock-in

### **Ecosystem** 🌐
- ✅ Any primal can provide capabilities
- ✅ No central coordination needed
- ✅ True decentralization

---

## 🎯 **Success Criteria**

### **v3.15.0**:
- ✅ Zero mentions of vendor names in env vars
- ✅ All primal interactions via Universal Adapter
- ✅ Deprecation warnings for old env vars
- ✅ Backward compatibility maintained

### **v3.16.0** (Future):
- ✅ Remove all deprecated env vars
- ✅ Zero vendor name constants
- ✅ 100% capability-based discovery

---

## 🚀 **Execution Order**

1. **Phase 1** (First): Environment variables + deprecation
   - Minimal disruption
   - Backward compatible
   - Easy to test

2. **Phase 2** (Second): Universal Adapter integration
   - Core refactoring
   - Requires Phase 1 complete
   - Needs comprehensive testing

3. **Phase 3** (Third): Documentation cleanup
   - Cosmetic but important
   - Low risk
   - Can be done in parallel with Phase 2

4. **Phase 4** (Last): Registry cleanup
   - Final cleanup
   - Requires all other phases complete
   - Easy verification

---

## 📊 **Impact Analysis**

### **Breaking Changes**: ❌ **NONE** (v3.15.0)
- Old env vars still work (with deprecation warnings)
- Direct client creation still works (internal)
- Gradual migration path

### **Deprecations**: ⚠️ **3** (v3.15.0)
- `SONGBIRD_BEARDOG_URL` → Use `SONGBIRD_SECURITY_PROVIDER`
- `BEARDOG_URL` → Use `SECURITY_PROVIDER`
- `BEARDOG_2FA_ENDPOINT` → Use `AUTHENTICATION_PROVIDER`

### **Removals**: ❌ **NONE** (v3.15.0)
- All removals deferred to v3.16.0
- Full deprecation cycle

---

## 🎯 **Summary**

**Problem**: 215+ instances of vendor hardcoding  
**Root Cause**: Direct client creation instead of Universal Adapter  
**Solution**: Evolve to capability-based discovery  
**Priority**: CRITICAL - Core architectural principle  
**Status**: READY TO IMPLEMENT  
**ETA**: 8-12 hours (2 sessions)

---

**Key Insight**:
> "The Universal Adapter already exists and works perfectly! We just need to USE it everywhere instead of creating vendor-specific clients."

---

**Next**: Execute Phase 1 - Add capability-based env vars

---

_"Each primal only knows itself. Network effects come from capability discovery, not hardcoded connections."_

