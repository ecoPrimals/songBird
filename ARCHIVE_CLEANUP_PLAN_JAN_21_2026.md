# Archive Code Cleanup Plan
## Session: January 21, 2026

## Executive Summary
Comprehensive review of archive code candidates, outdated TODOs, and false positives.

---

## 🎯 Archive Code Candidates

### 1. `crates/songbird-orchestrator/src/rpc/pure_jsonrpc_handler.rs` (328 lines)
**Status**: ORPHANED - Ready for Archive
**Evidence**:
- Exported from `rpc/mod.rs` but NEVER ACTUALLY CALLED in production
- Contains 11 TODO stubs (all handler implementations are placeholders)
- Production uses TWO different implementations:
  - `server/jsonrpc_api.rs` (448 lines) - HTTP JSON-RPC gateway
  - `ipc/pure_rust_server/` - Unix socket JSON-RPC
- Comment in `rpc/mod.rs` line 11: "Ready for full migration when IPC handlers are updated"
  - This never happened - production evolved to use the implementations above

**TODOs Found**:
```
Line 143: // TODO: Track actual uptime
Line 161: // TODO: Implement service discovery
Line 178: // TODO: Implement actual registration
Line 194: // TODO: Implement actual unregistration
Line 205: // TODO: Implement actual service listing
Line 217: // TODO: Implement actual connection status
Line 229: // TODO: Implement actual connection listing
Line 241: // TODO: Implement actual config retrieval
Line 252: // TODO: Implement actual config validation
Line 261: // TODO: Implement actual metrics
```

**Decision**: DELETE (orphaned code with placeholder TODOs)

---

### 2. `crates/songbird-orchestrator/src/core/biome/modules/types.rs` (850 lines)
**Status**: CORRUPT SYNTAX - Ready for Archive
**Evidence**:
- Severe syntax corruption (mismatched parens, broken `use` statements)
- Lines 19, 27, 28, 39: Invalid parentheses instead of braces
- Only used in 2 test modules (lifecycle.rs, orchestrator.rs)
- Biome functionality appears to be deprecated/experimental

**Sample Corruption**:
```rust
Line 19:  pub services: HashMap<String, ServiceSpec>)  // Should be }
Line 27:  pub primals: Option<HashMap<String, PrimalCoordination>> )  // Should be }
Line 28:} )  // Extra )
```

**Used By**:
- `core/biome/modules/lifecycle.rs` (test only)
- `core/biome/modules/orchestrator.rs` (test only)

**Decision**: NEEDS REVIEW - Check if biome module is active or deprecated

---

### 3. `crates/songbird-orchestrator/src/trust/escalation.rs` (661 lines)
**Status**: OUTDATED IMPORT - Needs Update, Not Archive
**Evidence**:
- Line 8: `use crate::security_capability_client::SecurityCapabilityClient;`
- This was refactored to `crate::security_client::client::SecurityCapabilityClient`
- File is production code, just needs import fix

**Decision**: FIX (update import to `security_client::client`)

---

### 4. `crates/songbird-orchestrator/src/access_control/auth.rs` (537 lines)
**Status**: PRODUCTION CODE - Keep
**Evidence**:
- Production JWT authentication implementation
- No deprecated markers found
- Active use in production

**Decision**: KEEP (production code)

---

### 5. `specs/archive/deprecated-protocols/GRPC_GATEWAY_ADAPTER_SPECIFICATION.md.deprecated`
**Status**: ALREADY ARCHIVED
**Decision**: KEEP (fossil record documentation)

---

## 🧹 Empty Directories Found (Candidates for Deletion)

```
./scripts/testing
./scripts/deployment
./scripts/development
./examples/ai_discovery
./showcase/benchmarks
./showcase/scripts
./showcase/06-toadstool-ml-orchestration/scripts
./showcase/06-toadstool-ml-orchestration/results
./showcase/01-isolated/scripts
./showcase/01-isolated/configs
./showcase/visualizations
./showcase/14-physical-genesis/configs
./showcase/03-inter-primal/scripts
./showcase/03-inter-primal/configs
./showcase/03-inter-primal/logs
./showcase/02-federation/configs
./showcase/02-federation/data/tower-a
./showcase/02-federation/data/tower-c
./showcase/02-federation/data/tower-b
./crates/songbird-universal-ipc/src/nestgate
```

**Decision**: KEEP (may be needed by showcase/example scripts, verify first)

---

## 📝 Outdated TODOs in Active Code

### IPC Handlers
- `crates/songbird-orchestrator/src/ipc/unix/handlers.rs:279` - "TODO: Add actual RPC call to peer's endpoint"
- `crates/songbird-orchestrator/src/ipc/handlers/p2p_discovery.rs` - 6 TODOs for unimplemented methods

### HTTP Gateway
- `crates/songbird-orchestrator/src/http_gateway/unix_listener.rs:371` - "TODO: Implement caching logic"
- `crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs:106` - "TODO: Implement proper caching with TTL"
- `crates/songbird-orchestrator/src/http_gateway/universal_proxy.rs:204,238` - "TODO: Implement template-based transformation"

**Decision**: DOCUMENT (these are legitimate TODOs for future work, not false positives)

---

## 🔧 Refactor TODOs (from Current Session)

### Outdated Refactor Tasks:
- `refactor-5`: beardog_crypto_client.rs - DEFERRED (not urgent)
- `refactor-6`: coordination.rs - DEFERRED (not urgent)
- `refactor-8`: biome/modules/types.rs - BLOCKED (corrupt file, needs biome module decision)
- `refactor-9`: ai_orchestration_engine.rs - DEFERRED (exists, 850+ lines, not urgent)
- `refactor-10`: core/mod.rs cleanup - DEFERRED (not urgent)

**Decision**: CANCEL these TODOs as they're deferred/blocked

---

## 🎬 Action Plan

### Phase 1: Quick Wins
1. DELETE `pure_jsonrpc_handler.rs` and `pure_jsonrpc_types.rs`
2. UPDATE imports in `rpc/mod.rs`
3. FIX `trust/escalation.rs` import to use `security_client`
4. CANCEL deferred refactor TODOs

### Phase 2: Biome Module Decision
1. Check if biome module is active or deprecated
2. If deprecated: Archive entire `core/biome/` directory
3. If active: Fix `types.rs` syntax corruption

### Phase 3: Documentation Update
1. Update root docs to reflect cleanup
2. Create fossil record in `specs/archive/`

### Phase 4: Git Push
1. Commit with message: "chore: archive cleanup - remove orphaned code"
2. Push via SSH

---

## 📊 Impact Summary

**Code to Delete**:
- `pure_jsonrpc_handler.rs`: 328 lines
- `pure_jsonrpc_types.rs`: ~150 lines
- Potential biome module: ~3,000 lines if deprecated

**Total Cleanup**: 478+ lines (minimum), up to 3,500 lines if biome is deprecated

**Risk Level**: LOW
- Files identified as orphaned/unused
- Comprehensive testing exists for production paths
- All deletions are reversible via git history

---

## ✅ Verification Checklist

- [ ] Verify `pure_jsonrpc_handler.rs` has no callers
- [ ] Confirm biome module status (active vs deprecated)
- [ ] Fix `trust/escalation.rs` import
- [ ] Update `rpc/mod.rs` after deletion
- [ ] Run `cargo build --lib -p songbird-orchestrator`
- [ ] Run `cargo test -p songbird-orchestrator --lib`
- [ ] Update root docs
- [ ] Git commit and push via SSH

---

*Generated: January 21, 2026*
*Part of: Deep Debt Cleanup Initiative*

