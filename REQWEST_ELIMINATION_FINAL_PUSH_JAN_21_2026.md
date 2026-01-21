# reqwest Elimination - Final Push to 100% Pure Rust
## Session: January 21, 2026

## 🎯 Mission: Complete reqwest Elimination

**Current Status**: 18 files, 36 instances remaining  
**Critical Paths**: ✅ Already Pure Rust (Session 6)  
**Goal**: Eliminate ALL reqwest usage, achieve 100% Pure Rust networking

---

## 📊 Remaining reqwest Usage (18 files, 36 instances)

### High Priority (Production-Adjacent) - 6 files, 12 instances
1. **`monitoring/btsp_health.rs`** (3) - Health monitoring for BTSP connections
2. **`network/connectivity_test.rs`** (1) - Network connectivity checks
3. **`access_control/auth.rs`** (3) - Authentication endpoints
4. **`trust/lineage_auth.rs`** (3) - Lineage authentication
5. **`core/primal_integration.rs`** (3) - Primal integration logic
6. **`core/routing/router.rs`** (1) - Legacy routing

### Medium Priority (BiomeOS/Core) - 7 files, 14 instances
7. **`core/biomeos/client.rs`** (2) - BiomeOS client (may be deprecated)
8. **`core/biomeos/universal_adapter.rs`** (2) - Universal adapter
9. **`core/biomeos/universal_adapter_complete.rs`** (2) - Complete adapter
10. **`core/execution/client.rs`** (2) - Execution client
11. **`core/substrate/connection_pool.rs`** (3) - Substrate connection pool
12. **`core/substrate/clients.rs`** (1) - Substrate clients
13. **`core/substrate/os_substrate.rs`** (2) - OS substrate

### Low Priority (Experimental/Legacy) - 5 files, 10 instances
14. **`core/routing/enhanced_router.rs`** (1) - Enhanced routing (experimental)
15. **`core/biome/modules/orchestrator.rs`** (1) - Biome orchestrator
16. **`core/biome/modules/lifecycle.rs`** (1) - Biome lifecycle
17. **`core/api/ai_workload_classification/mod.rs`** (2) - AI workload (experimental)
18. **`universal_adapter.rs`** (2) - Universal adapter (may be duplicate)

**Total**: 18 files, 36 instances

---

## 🎯 Elimination Strategy

### Phase 1: Quick Wins (Struct Fields) - 30 min
Many files just have `reqwest::Client` as a struct field but don't actively use it.

**Pattern**:
```rust
// Before
struct MyClient {
    http_client: reqwest::Client,
}

// After  
struct MyClient {
    http_client: songbird_http_client::SongbirdHttpClient,
}
```

**Target Files**: Most of them! Just struct field updates.

### Phase 2: Active Usage Migration - 1-2 hours
Files that actually call reqwest methods need migration.

**Pattern**:
```rust
// Before
let response = self.http_client.get(url).send().await?;

// After
let response = self.http_client.get(url).await?;
```

**Key Difference**: SongbirdHttpClient methods return the future directly (no `.send()`).

### Phase 3: Cargo.toml Cleanup - 5 min
```toml
# Remove completely
# reqwest = { version = "0.11", features = ["json"], default-features = false }
```

---

## 🚀 Execution Plan

### Step 1: Batch Replace Struct Fields (10-15 files)
Use search-replace for simple struct field declarations.

### Step 2: Fix Active Usage (3-5 files)
Manually migrate files with actual reqwest method calls.

### Step 3: Build & Fix Errors
Iterative: build, fix errors, repeat.

### Step 4: Remove from Cargo.toml
Final step after all usage eliminated.

### Step 5: Verify & Document
- Run `cargo build --lib`
- Verify no `reqwest::` in src
- Update docs

---

## 📋 Execution Checklist

### Preparation
- [x] Survey all remaining files
- [x] Categorize by priority
- [x] Create elimination plan

### Execution
- [ ] Phase 1: Struct field replacements (batch)
- [ ] Phase 2: Active usage migration (careful)
- [ ] Phase 3: Build verification
- [ ] Phase 4: Cargo.toml cleanup
- [ ] Phase 5: Final verification

### Documentation
- [ ] Update README.md (100% reqwest-free)
- [ ] Update STATUS.md
- [ ] Create completion document
- [ ] Git commit and push

---

## ⏱️ Time Estimate

- Phase 1 (Struct fields): 30-45 min
- Phase 2 (Active usage): 60-90 min  
- Phase 3 (Build/fix): 30-45 min
- Phase 4 (Cleanup): 5-10 min
- Phase 5 (Docs): 15-20 min

**Total**: 2.5-3.5 hours to 100% reqwest-free

---

## 🎯 Success Criteria

1. ✅ Zero `reqwest::` in `crates/songbird-orchestrator/src`
2. ✅ reqwest removed from `Cargo.toml`
3. ✅ `cargo build --lib` succeeds
4. ✅ All HTTP/HTTPS via `SongbirdHttpClient`
5. ✅ 100% Pure Rust networking stack

---

*Created: January 21, 2026*  
*Status: Ready to Execute*  
*Goal: 100% reqwest Elimination*

