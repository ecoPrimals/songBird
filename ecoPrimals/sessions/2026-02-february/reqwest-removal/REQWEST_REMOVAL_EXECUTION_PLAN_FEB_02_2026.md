# 🔧 reqwest Removal Execution Plan

**Date**: February 2, 2026  
**Priority**: HIGH - ecoBin v2.0 Compliance  
**Estimated**: 4-6 hours  
**Status**: Ready to Execute

═══════════════════════════════════════════════════════════════════

## 🎯 **OBJECTIVE**

Remove all `reqwest` dependencies from Songbird to achieve 100% Pure Rust compliance for ecoBin v2.0.

**Current**: 50 reqwest usages across 9 crates  
**Target**: 0 reqwest usages, 100% Pure Rust

═══════════════════════════════════════════════════════════════════

## 📊 **VERIFIED CURRENT STATE**

### **Reqwest Usage by Crate** (Confirmed):
```
songbird-orchestrator:   17 usages  (HIGH priority)
songbird-universal:      16 usages  (HIGH priority)
songbird-http-client:     5 usages  (MEDIUM priority)
songbird-registry:        3 usages  (MEDIUM priority)
songbird-cli:             2 usages  (LOW priority)
songbird-discovery:       2 usages  (LOW priority)
songbird-observability:   2 usages  (LOW priority)
songbird-test-utils:      2 usages  (LOW priority)
songbird-config:          1 usage   (LOW priority)
────────────────────────────────────
TOTAL:                   50 usages
```

### **Workspace Dependency** (Line 166):
```toml
reqwest = { version = "0.11", features = ["json"], default-features = false }
```

### **Replacement Available**:
✅ `IpcHttpClient` - For Unix socket HTTP (primal IPC)
✅ `SongbirdHttpClient` - For external HTTPS via Tower Atomic (BearDog crypto)

═══════════════════════════════════════════════════════════════════

## 📋 **3-PHASE EXECUTION PLAN**

### **Phase 1: High Priority Crates** (2-3 hours)
**Target**: 33 usages (66% of total)

1. **songbird-orchestrator** (17 usages):
   - Files affected: 9 (lifecycle, orchestrator, primal_integration, etc.)
   - Replace with: `IpcHttpClient` for primal communication
   - Replace with: `SongbirdHttpClient` for external APIs

2. **songbird-universal** (16 usages):
   - Files affected: 8 (discovery adapters, jsonrpc_client, etc.)
   - Replace with: `IpcHttpClient` primarily
   - Focus: Discovery and service adapters

**Checkpoint**: Build + test after Phase 1

---

### **Phase 2: Medium Priority Crates** (1-2 hours)
**Target**: 8 usages (16% of total)

3. **songbird-http-client** (5 usages):
   - Internal references (likely examples/tests)
   - May be compatibility layer - evaluate carefully

4. **songbird-registry** (3 usages):
   - Service persistence HTTP calls
   - Replace with: `IpcHttpClient`

**Checkpoint**: Build + test after Phase 2

---

### **Phase 3: Low Priority Crates** (30-60 minutes)
**Target**: 9 usages (18% of total)

5. **songbird-cli** (2 usages)
6. **songbird-discovery** (2 usages)
7. **songbird-observability** (2 usages)
8. **songbird-test-utils** (2 usages)
9. **songbird-config** (1 usage)

**Checkpoint**: Final build + full test suite

---

### **Phase 4: Cleanup** (15 minutes)
1. Remove reqwest from workspace Cargo.toml
2. Remove reqwest from all crate Cargo.toml files
3. Final verification: `grep -r "reqwest" crates/` → 0 results

═══════════════════════════════════════════════════════════════════

## 🔄 **MIGRATION PATTERNS**

### **Pattern 1: Simple GET Request**
```rust
// BEFORE (reqwest)
use reqwest;
let client = reqwest::Client::new();
let response = client.get(&url).send().await?;
let data: MyType = response.json().await?;

// AFTER (IpcHttpClient)
use songbird_http_client::IpcHttpClient;
let client = IpcHttpClient::new().await?;
let response = client.get(&url).await?;
let data: MyType = response.json().await?;
```

### **Pattern 2: POST with JSON**
```rust
// BEFORE (reqwest)
let response = client
    .post(&url)
    .json(&body)
    .send()
    .await?;

// AFTER (IpcHttpClient)
let response = client
    .post(&url)
    .await
    .json(&body)?
    .send()
    .await?;
```

### **Pattern 3: Status Check**
```rust
// BEFORE (reqwest)
if response.status().is_success() { }

// AFTER (IpcHttpClient)
if response.is_success() { }
```

### **Pattern 4: Client with Timeout**
```rust
// BEFORE (reqwest)
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

// AFTER (IpcHttpClient)
let client = IpcHttpClient::builder()
    .timeout(Duration::from_secs(30))
    .build()
    .await?;
```

═══════════════════════════════════════════════════════════════════

## ⚠️ **CRITICAL GOTCHAS**

### **1. Async Initialization**
```rust
❌ WRONG: let client = IpcHttpClient::new();
✅ RIGHT: let client = IpcHttpClient::new().await?;
```

### **2. POST Method Async**
```rust
❌ WRONG: client.post(&url).json(&body)?.send().await?
✅ RIGHT: client.post(&url).await.json(&body)?.send().await?
```

### **3. JSON Returns Result**
```rust
❌ WRONG: .json(&body).send()
✅ RIGHT: .json(&body)?.send()
```

### **4. Import Changes**
```rust
❌ WRONG: use reqwest;
✅ RIGHT: use songbird_http_client::IpcHttpClient;
```

═══════════════════════════════════════════════════════════════════

## 🧪 **TESTING STRATEGY**

### **Per-Phase Testing**:
```bash
# After each phase:
cargo build -p <crate>
cargo test -p <crate>
cargo clippy -p <crate>
```

### **Integration Testing**:
```bash
# After Phase 3:
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

### **Final Verification**:
```bash
# Check no reqwest remains:
grep -r "reqwest::\|use reqwest" crates/*/src --include="*.rs" | wc -l
# Expected: 0

# Check Cargo.toml:
grep "reqwest" Cargo.toml crates/*/Cargo.toml
# Expected: (no output)
```

═══════════════════════════════════════════════════════════════════

## 📊 **SUCCESS CRITERIA**

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| reqwest usages | 50 | 0 | ⏳ Pending |
| C dependencies | Yes (openssl-sys) | No | ⏳ Pending |
| ecoBin v2.0 | Partial | 100% | ⏳ Pending |
| TLS Provider | Native | BearDog | ⏳ Pending |
| Build | ✅ Pass | ✅ Pass | ⏳ Pending |
| Tests | ✅ Pass | ✅ Pass | ⏳ Pending |

═══════════════════════════════════════════════════════════════════

## 🏆 **DEEP DEBT ALIGNMENT**

This migration directly supports deep debt principles:

✅ **Pure Rust**: Removes all C dependencies (openssl-sys)
✅ **Zero Unsafe**: IpcHttpClient and SongbirdHttpClient are 100% safe
✅ **Runtime Discovery**: Clients discover primals at runtime (no hardcoding)
✅ **Agnostic Design**: Works with any HTTP endpoint
✅ **Smart Refactoring**: Replaces external dep with internal solution
✅ **Modern Rust**: Uses Tower Atomic pattern (cutting-edge)

═══════════════════════════════════════════════════════════════════

## 📁 **FILES IDENTIFIED FOR MODIFICATION**

### **Phase 1 (High Priority)**:

**songbird-orchestrator** (9 files):
- `src/core/biome/modules/lifecycle.rs`
- `src/core/biome/modules/orchestrator.rs`
- `src/core/primal_integration.rs`
- `src/core/substrate/connection_pool.rs`
- `src/core/substrate/os_substrate.rs`
- `src/core/substrate/clients.rs`
- `src/core/api/ai_workload_classification/mod.rs`
- `src/core/biomeos/universal_adapter_complete.rs`
- `src/core/biomeos/client.rs`

**songbird-universal** (8 files):
- `src/jsonrpc_client.rs`
- `src/adapters/tests_protocol_detection.rs`
- `src/infant_discovery_engine.rs`
- `src/service_discovery.rs`
- `src/self_discovery.rs`
- `src/enhanced_infant_discovery.rs`
- `src/infant_discovery.rs`
- `src/ecosystem_discovery.rs`

### **Phase 2 (Medium Priority)**:

**songbird-http-client** (files TBD - investigate)
**songbird-registry** (3 files):
- `src/zero_cost_service_registry.rs`
- `src/persistence/production_registry.rs`
- (1 more TBD)

### **Phase 3 (Low Priority)**:
Files TBD after Phase 1 & 2 analysis

═══════════════════════════════════════════════════════════════════

## 🚀 **EXECUTION WORKFLOW**

### **Step 1: Investigation** ✅ COMPLETE
- Verified reqwest usage (50 occurrences)
- Confirmed replacement clients available
- Created execution plan

### **Step 2: Phase 1 Migration** ⏳ NEXT
- Start with songbird-orchestrator
- Then songbird-universal
- Test after each crate

### **Step 3: Phase 2 Migration** ⏳ Pending
- songbird-http-client
- songbird-registry
- Test build

### **Step 4: Phase 3 Migration** ⏳ Pending
- Low priority crates (5 crates)
- Final testing

### **Step 5: Cleanup** ⏳ Pending
- Remove reqwest from Cargo.toml files
- Final verification
- Commit and document

═══════════════════════════════════════════════════════════════════

## 📝 **PROGRESS TRACKING**

### **Phase 1**: ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ 0/33 (0%)
### **Phase 2**: ⬜⬜⬜⬜⬜⬜⬜⬜ 0/8 (0%)
### **Phase 3**: ⬜⬜⬜⬜⬜⬜⬜⬜⬜ 0/9 (0%)
### **Overall**: ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ 0/50 (0%)

═══════════════════════════════════════════════════════════════════

## ⏱️ **TIME ESTIMATES**

| Phase | Tasks | Time | Running Total |
|-------|-------|------|---------------|
| **Phase 1** | 33 usages | 2-3h | 2-3h |
| **Phase 2** | 8 usages | 1-2h | 3-5h |
| **Phase 3** | 9 usages | 30-60m | 3.5-6h |
| **Cleanup** | Final | 15m | 4-6h |

**Total Estimated**: 4-6 hours

═══════════════════════════════════════════════════════════════════

## ✅ **READY TO EXECUTE**

**Prerequisites**: ✅ All met
- Investigation complete
- Replacement clients confirmed available
- Migration patterns documented
- Files identified
- Testing strategy defined

**Next Action**: Begin Phase 1 - songbird-orchestrator migration

═══════════════════════════════════════════════════════════════════

**Status**: READY FOR EXECUTION  
**Priority**: HIGH  
**Impact**: ecoBin v2.0 Compliance Blocker

🚀 **Ready to proceed with Phase 1!**
