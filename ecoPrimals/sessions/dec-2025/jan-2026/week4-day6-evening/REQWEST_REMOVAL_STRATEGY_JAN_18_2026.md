# 🚨 reqwest Removal Strategy - ecoBin Blocker

**Date**: January 18, 2026  
**Priority**: CRITICAL (blocks ecoBin)  
**Scope**: 15 Cargo.toml files, 30+ code usages  
**Impact**: Architectural evolution required

---

## 🔍 COMPREHENSIVE AUDIT RESULTS

### Cargo.toml Files with reqwest (15 total):
1. songbird-test-utils
2. songbird-types
3. songbird-orchestrator
4. songbird-compute-bridge
5. songbird-genesis
6. songbird-registry
7. songbird-execution-agent
8. songbird-universal
9. songbird-discovery
10. songbird-remote-deploy
11. songbird-primal-coordination
12. songbird-cli
13. songbird-config
14. songbird-primal-sdk
15. songbird-network-federation

### Code Usage Locations (30+ occurrences):
- **Test Utils**: Mock isolation analysis
- **Universal Primals**: Capability discovery (HTTP GET to /api/info)
- **Orchestrator** (18+ usages):
  - Network connectivity tests
  - BTSP health monitoring
  - Compute API routing
  - Security capability client
  - Access control (2FA auth)
  - Connection management (limited, federated, full-trust)
  - Trust/lineage auth
  - Primal integration
  - Core routing (router, enhanced_router)
  - Biome modules (orchestrator, lifecycle)
  - Execution client
  - BiomeOS client

---

## 🎯 ANALYSIS: WHY IS REQWEST HERE?

### Pattern Analysis:

1. **Legacy HTTP-based Discovery** (DEPRECATED)
   - Capability discovery via HTTP GET
   - Primal discovery via HTTP endpoints
   - **Should be**: BTSP over Unix sockets

2. **External Service Communication** (ARCHITECTURAL ISSUE)
   - 2FA authentication to BearDog via HTTP
   - Health monitoring via HTTP
   - **Should be**: Unix socket JSON-RPC

3. **Test Utilities** (ACCEPTABLE IF FEATURE-GATED)
   - Mock isolation analysis
   - **Could be**: Feature-gated for testing only

4. **Federation/Routing** (LEGACY PATTERN)
   - HTTP routing to other primals
   - **Should be**: BTSP over Unix sockets

---

## 🚀 STRATEGIC DECISION REQUIRED

### Option A: Complete Removal (PURIST - TRUE ecoBin) ✅

**Philosophy**: Songbird is TLS PRIMAL, not HTTP client

**Approach**:
1. Remove reqwest from ALL Cargo.toml files
2. Replace ALL HTTP client code with:
   - Unix socket communication (BTSP/JSON-RPC)
   - Capability-based discovery (no HTTP)
   - Runtime primal discovery (BirdSong)

**Benefits**:
- ✅ TRUE ecoBin (zero C deps)
- ✅ Architectural purity
- ✅ Aligns with Concentrated Gap strategy
- ✅ Forces completion of Unix socket evolution

**Challenges**:
- 🔧 30+ code locations to update
- 🔧 Existing HTTP-based discovery to migrate
- 🔧 2FA/auth flows to update (already JSON-RPC capable!)
- 🔧 Test utilities need feature-gating

**Timeline**: 2-3 days (systematic migration)

---

### Option B: Feature-Gate reqwest (PRAGMATIC - Staged ecoBin)

**Philosophy**: Optional HTTP client for specific use cases

**Approach**:
1. Move reqwest to optional feature: `http-client`
2. Feature-gate ALL reqwest code
3. Default build: NO reqwest (ecoBin compliant)
4. Optional build: `--features http-client` (for specific cases)

**Benefits**:
- ✅ ecoBin by default
- ✅ Flexibility for edge cases
- ✅ Faster to implement

**Challenges**:
- ⚠️ Still questionable architecture (why HTTP client in TLS primal?)
- ⚠️ Maintenance burden (two code paths)
- ⚠️ Doesn't force completion of Unix socket evolution

**Timeline**: 1 day (feature-gating)

---

### Option C: Selective Removal (HYBRID - Targeted ecoBin)

**Philosophy**: Remove where possible, keep where critical

**Approach**:
1. **Remove** from:
   - Discovery (use BTSP/BirdSong)
   - Routing (use BTSP)
   - Health monitoring (use Unix sockets)
   - Primal integration (use BTSP)

2. **Keep** (feature-gated) in:
   - Test utilities (testing only)
   - CLI (user-facing tool, acceptable?)

3. **Migrate** to Unix sockets:
   - 2FA auth (BearDog via JSON-RPC)
   - Capability discovery
   - Federation communication

**Benefits**:
- ✅ Achieves ecoBin for main binary
- ✅ Pragmatic for testing/CLI
- ✅ Forces most architectural evolution

**Challenges**:
- 🔧 Complex migration plan
- 🔧 Multiple code paths

**Timeline**: 1-2 days

---

## 💡 RECOMMENDED: Option A (Complete Removal)

### Rationale:

1. **Architectural Purity**:
   - Songbird is TLS **SERVER**, not HTTP **CLIENT**
   - Concentrated Gap strategy: Only Songbird handles external HTTP/TLS
   - Inter-primal: Unix sockets ONLY

2. **Deep Debt Solution**:
   - Forces completion of Unix socket evolution
   - Eliminates legacy HTTP-based patterns
   - Aligns with capability-based discovery

3. **TRUE ecoBin**:
   - Zero C dependencies (no transitive rustls/ring)
   - Universal cross-compilation
   - Ecosystem leadership (first TLS primal to achieve ecoBin)

4. **Implementation Path**:
   - BearDog JSON-RPC: Already implemented ✅
   - BTSP: Already implemented ✅
   - BirdSong discovery: Already implemented ✅
   - Just need to MIGRATE existing HTTP code to use them!

---

## 🔧 IMPLEMENTATION PLAN (Option A)

### Phase 1: Test Utilities (30 min)
**File**: `crates/songbird-test-utils/src/mock_isolation_analysis.rs`

**Current**: Uses reqwest::Client
**Target**: Feature-gate or remove (test-only utility)

**Action**:
```toml
[dependencies]
reqwest = { version = "0.11", optional = true }

[features]
test-http-client = ["reqwest"]
```

---

### Phase 2: Discovery Migration (2 hours)
**File**: `crates/songbird-universal-primals/src/discovery/capability_based.rs`

**Current**:
```rust
reqwest::get(&format!("{endpoint}/api/info"))
```

**Target**: Use BirdSong UDP discovery or BTSP Unix socket
```rust
// Use BirdSong discovery
let discovered = birdsong::discover_primal_by_capability("storage").await?;

// Or BTSP if endpoint known
let response = btsp_client.call("/api/info", ()).await?;
```

---

### Phase 3: Auth/Security Migration (1 hour)
**Files**: 
- `src/access_control/auth.rs` (2FA)
- `src/security_capability_client.rs`
- `src/trust/lineage_auth.rs`

**Current**: HTTP requests to BearDog
**Target**: JSON-RPC over Unix socket (ALREADY IMPLEMENTED!)

**Action**: Use existing `crate::crypto::beardog_crypto_client`

---

### Phase 4: Monitoring Migration (1 hour)
**File**: `src/monitoring/btsp_health.rs`

**Current**: HTTP health checks
**Target**: BTSP health checks over Unix sockets

---

### Phase 5: Routing/Federation Migration (2 hours)
**Files**:
- `src/core/routing/router.rs`
- `src/core/routing/enhanced_router.rs`
- `src/connections/*.rs`

**Current**: HTTP routing to primals
**Target**: BTSP routing over Unix sockets

---

### Phase 6: Orchestration/Integration Migration (2 hours)
**Files**:
- `src/core/primal_integration.rs`
- `src/core/biome/modules/*.rs`
- `src/core/execution/client.rs`
- `src/core/biomeos/client.rs`

**Current**: HTTP communication
**Target**: BTSP/JSON-RPC over Unix sockets

---

### Phase 7: Remove from Cargo.toml (30 min)
Remove reqwest from all 15 Cargo.toml files

---

### Phase 8: Verification (30 min)
```bash
# 1. Build succeeds
cargo build --workspace

# 2. No C dependencies
cargo tree | grep -E "(rustls|ring|aws-lc)"
# Expected: NO MATCHES

# 3. Tests pass
cargo test --workspace

# 4. ecoBin validation
cargo build --target x86_64-unknown-linux-musl
```

---

## 📊 TIMELINE ESTIMATE

**Total**: 2-3 days (systematic, thorough)

**Breakdown**:
- Day 1 AM: Test utils, discovery, auth (4 hours)
- Day 1 PM: Monitoring, routing (3 hours)
- Day 2 AM: Orchestration, integration (4 hours)
- Day 2 PM: Cargo.toml cleanup, verification (3 hours)
- Day 3: Testing, documentation, ecoBin validation

---

## ✅ SUCCESS CRITERIA

1. ✅ Zero reqwest in Cargo.toml files
2. ✅ Zero reqwest:: in code (except feature-gated tests)
3. ✅ All functionality migrated to Unix sockets
4. ✅ cargo tree shows NO rustls/ring
5. ✅ Build succeeds for musl target
6. ✅ All tests pass
7. ✅ ecoBin validated

---

## 🎯 ARCHITECTURAL BENEFITS

**After Removal**:
- ✅ TRUE ecoBin (zero C deps)
- ✅ Architectural purity (Unix sockets only)
- ✅ Forced completion of BTSP/BirdSong evolution
- ✅ Concentrated Gap strategy enforced
- ✅ Ecosystem leadership (first TLS primal ecoBin)
- ✅ 100% Pure Rust sovereignty

---

**Strategy**: Complete reqwest Removal (Option A)  
**Timeline**: 2-3 days  
**Priority**: CRITICAL (blocks ecoBin)  
**Benefit**: TRUE ecoBin + Architectural Evolution

🦀✨ **Deep Debt Solution - Systematic Migration to Unix Sockets!** ✨🦀
