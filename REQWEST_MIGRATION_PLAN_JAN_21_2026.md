# Reqwest Migration Plan - Pure Rust HTTP Evolution

**Date**: January 21, 2026  
**Status**: ⚠️ **PARTIAL** - Critical path migrated, non-critical paths pending  
**Priority**: 🟡 **MEDIUM** (Phase 1.5 features, not blocking Squirrel AI)

---

## 🎯 Current Status

### ✅ Critical Path: MIGRATED (Pure Rust)

**File**: `crates/songbird-orchestrator/src/ipc/unix_socket.rs`  
**Method**: `handle_http_request()`  
**Status**: ✅ **COMPLETE** - Now uses `songbird-http-client` (Pure Rust)

**Impact**:
- ✅ Squirrel AI HTTP delegation: **PURE RUST**
- ✅ External API requests (Anthropic): **PURE RUST**
- ✅ BearDog crypto delegation: **PURE RUST**
- ✅ **ZERO C DEPENDENCIES** in Squirrel AI path

---

## ⏳ Non-Critical Paths: PENDING (27 files)

### Why Not Migrated Yet?

**Reason**: These files are **Phase 1.5** features (lineage, genetic operations) and **NOT on the critical path** for Squirrel AI integration.

**Decision**: Keep `reqwest` temporarily for:
1. Non-blocking development (Squirrel AI unblocked)
2. Systematic migration (not rushed)
3. Feature stability (lineage operations stable with reqwest)

---

## 📋 Files Using Reqwest (27 total)

### 1. Security & Trust (8 files)
- `src/security_capability_client.rs` - Security provider client (Phase 1.5)
- `src/app/discovery_bridge.rs` - Discovery HTTP bridge
- `src/trust/lineage_auth.rs` - Lineage authentication
- `src/trust/escalation.rs` - Trust escalation  
- `src/trust/peer_trust.rs` - Peer trust evaluation
- `src/app/discovery_startup.rs` - Discovery startup
- `src/app/core.rs` - Core orchestrator (uses security client)
- `src/lib.rs` - Module exports

### 2. Network & Federation (6 files)
- `src/network/connectivity_test.rs` - Network connectivity tests
- `src/monitoring/btsp_health.rs` - BTSP health monitoring
- `src/core/routing/enhanced_router.rs` - Enhanced routing
- `src/core/routing/router.rs` - Basic routing
- `src/app/discovery_bridge.rs` (duplicate)
- `src/http_gateway/*.rs` - HTTP gateway modules

### 3. HTTP Gateway (3 files)
- `src/http_gateway/mod.rs` - HTTP gateway main
- `src/http_gateway/unix_listener.rs` - Unix listener
- `src/http_gateway/universal_proxy.rs` - Universal proxy

### 4. Substrate & Clients (4 files)
- `src/core/substrate/connection_pool.rs` - Connection pooling
- `src/core/substrate/clients.rs` - Substrate clients
- `src/core/substrate/os_substrate.rs` - OS substrate
- `src/universal_adapter.rs` - Universal adapter

### 5. BiomeOS Integration (3 files)
- `src/core/biomeos/universal_adapter.rs` - BiomeOS adapter
- `src/core/biomeos/universal_adapter_complete.rs` - Complete adapter
- `src/core/biomeos/client.rs` - BiomeOS client

### 6. Other (3 files)
- `src/core/api/ai_workload_classification/mod.rs` - AI workload classification
- `src/core/primal_integration.rs` - Primal integration
- `src/server/compute_api.rs` - Compute API
- `tests/http_server_sovereign_e2e_test.rs` - E2E test
- `tests/https_server_comprehensive_test.rs` - HTTPS test

---

## 🗺️ Migration Strategy

### Phase 1: Foundation ✅ **COMPLETE**
- [x] Build `songbird-http-client` (Pure Rust)
- [x] Migrate `handle_http_request()` (critical path)
- [x] Test Squirrel AI integration

### Phase 2: Security & Trust (Week 1-2)
- [ ] Migrate `security_capability_client.rs`
- [ ] Migrate trust evaluation modules
- [ ] Migrate lineage authentication
- [ ] Test security operations end-to-end

### Phase 3: Network & Monitoring (Week 2-3)
- [ ] Migrate connectivity tests
- [ ] Migrate BTSP health monitoring
- [ ] Migrate routing modules
- [ ] Test network operations

### Phase 4: Gateway & Adapters (Week 3-4)
- [ ] Migrate HTTP gateway modules
- [ ] Migrate substrate clients
- [ ] Migrate BiomeOS adapters
- [ ] Test gateway operations

### Phase 5: Cleanup & Validation (Week 4)
- [ ] Remove all `reqwest` dependencies
- [ ] Verify zero C dependencies (cargo tree)
- [ ] Run full test suite
- [ ] Performance benchmarks
- [ ] Cross-compile ecoBin (x86_64-musl)

---

## 📊 Migration Template

For each file:

```rust
// BEFORE (reqwest):
use reqwest::Client;

let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;

let response = client.get(&url).send().await?;
let body = response.json::<T>().await?;

// AFTER (songbird-http-client):
use songbird_http_client::SongbirdHttpClient;

let beardog_socket = std::env::var("SONGBIRD_SECURITY_PROVIDER")
    .unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());

let client = SongbirdHttpClient::new(beardog_socket);

let response = client.request(
    "GET",
    &url,
    HashMap::new(),
    None,
).await?;

// response.body already deserialized as serde_json::Value
let typed_body: T = serde_json::from_value(response.body)?;
```

---

## ✅ Success Criteria (Per Phase)

### Phase 1 (Foundation): ✅ **COMPLETE**
- ✅ Pure Rust HTTP client built
- ✅ Critical path migrated
- ✅ Squirrel AI unblocked

### Phase 2-4 (Migration): ⏳ **PENDING**
- ⏳ All 27 files migrated
- ⏳ All tests passing
- ⏳ No `reqwest` imports

### Phase 5 (Validation): ⏳ **PENDING**
- ⏳ `cargo tree | grep -E '(reqwest|ring|rustls)'` → empty
- ⏳ Cross-compile successful
- ⏳ Performance unchanged or better

---

## 🎯 Priority Guidance

### High Priority (Week 1-2):
1. Security & trust modules (used by core features)
2. Network connectivity tests (used in CI)

### Medium Priority (Week 2-3):
3. HTTP gateway modules (used for external APIs)
4. BiomeOS adapters (used for ecosystem integration)

### Low Priority (Week 3-4):
5. Substrate clients (Phase 1.5 experimental)
6. Test files (can use reqwest in tests temporarily)

---

## 📝 Notes

### Why Keep Reqwest Temporarily?

**Pragmatic Decision**:
1. ✅ **Critical path unblocked**: Squirrel AI works with Pure Rust
2. ✅ **Feature stability**: Phase 1.5 features stable with reqwest
3. ✅ **Systematic migration**: Not rushed, properly tested
4. ✅ **No production impact**: Non-critical paths can wait

**Trade-off Accepted**:
- ⚠️ Temporary C dependencies (via reqwest)
- ✅ Squirrel AI path is Pure Rust (critical)
- ✅ Migration roadmap clear (4-week plan)

### When Is "Pure Rust" Complete?

**Definition**: When `cargo tree` shows **zero C dependencies**.

**Current Status**: 
- ✅ **Critical path**: Pure Rust (Squirrel AI)
- ⚠️ **Full codebase**: Pending migration (27 files)

**Timeline**: 4 weeks for complete Pure Rust codebase.

---

## 🚀 Next Actions

### Immediate (This Week):
1. ✅ Document migration plan (this document)
2. ⏳ Start Phase 2 (security modules)

### Short-Term (Next 2 Weeks):
3. ⏳ Migrate high-priority modules
4. ⏳ Test security operations

### Long-Term (Month):
5. ⏳ Complete all migrations
6. ⏳ Validate zero C dependencies
7. ⏳ Production deployment

---

## 📚 References

- **Foundation**: `crates/songbird-http-client/README.md`
- **Integration**: `SQUIRREL_HTTP_INTEGRATION_JAN_21_2026.md`
- **Plan**: `TOWER_ATOMIC_HTTP_EVOLUTION_JAN_21_2026.md`

---

**Document**: REQWEST_MIGRATION_PLAN_JAN_21_2026.md  
**Date**: January 21, 2026  
**Status**: Critical Path Complete, Full Migration Pending  
**Timeline**: 4 weeks for 100% Pure Rust

🐦🦀 **Pure Rust Evolution - Systematic & Pragmatic!** ✨

