# Songbird Cleanup Plan - February 2026

**Date**: February 5, 2026  
**Version**: v3.21.0  
**Purpose**: Archive outdated code, remove false positives, clean TODOs

---

## Executive Summary

Review identified **5 categories** of cleanable code:
1. **Outdated TODOs** (35 items) - Completed or superseded functionality
2. **Temporarily Disabled Code** (8 sections) - Can be removed or re-enabled
3. **Commented-Out Imports/Modules** (12 items) - Dead code to remove
4. **Legacy Compatibility** (maintained intentionally) - Document and keep
5. **False Positive `#[allow(dead_code)]`** (150+ items) - Many legitimate

---

## ✅ High Priority Cleanup Items

### 1. Outdated TODOs (Completed Features)

These TODOs reference features that are **already implemented**:

#### **Sled/TaskLifecycle Serialization** (COMPLETED v3.21.0)
```rust
// crates/songbird-orchestrator/src/task_lifecycle/types.rs:16
// NOTE: Using default (externally tagged) serde representation for bincode compatibility.
// ✅ FIXED: Now using serde_json instead of bincode
```
**Action**: Update comment to reflect JSON serialization (Feb 5, 2026).

#### **BirdSong family_id Integration** (COMPLETED v3.21.0)
```rust
// crates/songbird-discovery/src/birdsong_integration.rs:860
// TODO: Call BearDog's beacon.try_decrypt_with_id when available
// ✅ FIXED: family_id now passed to all crypto operations
```
**Action**: Remove TODO, feature complete.

#### **JSON-RPC Standard Methods** (COMPLETED v3.21.0)
```rust
// crates/songbird-orchestrator/src/server/jsonrpc_api.rs
// Added: health, identity, network.beacon_exchange
```
**Action**: No TODOs here, already clean.

#### **TLS Protocol Detection** (COMPLETED v3.21.0)
```rust
// crates/songbird-orchestrator/src/app/http_server.rs
// HTTP/HTTPS on same port via peek byte detection
```
**Action**: Clean, no TODOs.

---

### 2. Temporarily Disabled Code (Safe to Remove?)

#### **A. Gaming Manager** (Disabled since Jan 6, 2026)
```rust
// crates/songbird-orchestrator/src/app/core.rs:33-34
// gaming_manager: Arc<GamingManager>, // Temporarily disabled
// federation_manager: Arc<CanonicalFederation>, // Temporarily disabled
```
**Questions**:
- Is gaming integration planned for Phase 2?
- Can we remove or move to Phase 2 branch?

**Recommendation**: Remove commented fields, add to Phase 2 backlog.

#### **B. Security Integration** (Disabled since Jan 6, 2026)
```rust
// crates/songbird-orchestrator/src/app/core.rs:40
// security_integration: Arc<UniversalSecurityIntegration>, // Temporarily disabled
```
**Question**: Is UniversalSecurityIntegration superseded by TrustEscalationManager?

**Recommendation**: If superseded, remove. If deferred, document in Phase 2 plan.

#### **C. Federation-Aware Discovery** (Disabled - syntax errors)
```rust
// crates/songbird-discovery/src/lib.rs:166-173
// NOTE: Extensive syntax errors throughout file (32+ errors)
// pub mod federation_aware_discovery;
// pub mod migration;
```
**Recommendation**: 
- If Phase 2 feature, move to separate branch
- If abandoned, delete module files entirely
- Keep comments as fossil record

---

### 3. Commented-Out Imports/Modules (Dead Code)

#### **Remove These**:

```rust
// crates/songbird-config/src/lib.rs:119
// pub mod environment_config_clean; // REMOVED

// crates/songbird-test-utils/src/lib.rs:20
// REMOVED: fixtures_legacy (Nov 8, 2025) - No active usage

// crates/songbird-discovery/src/lib.rs:187-199
// TEMP DISABLED: federation_aware_discovery (syntax errors)
// TEMP DISABLED: migration (depends on federation_aware_discovery)

// crates/songbird-orchestrator/src/rpc/mod.rs:21-25
// Former pure_jsonrpc_handler.rs (ARCHIVED JAN 21, 2026)
```

**Action**: Remove commented code, these are already documented in git history.

---

## 🟡 Medium Priority Items

### 4. Legacy Compatibility Code (KEEP - Document)

These are **intentional** for backward compatibility:

```rust
// Legacy socket paths: /tmp/{primal}.sock
// Legacy BirdSong format: plaintext family_id
// Legacy environment variables: BEARDOG_ENDPOINT, etc.
```

**Action**: 
- ✅ **KEEP** - Required for gradual migration
- Document deprecation timeline in `LEGACY_SUPPORT.md`
- Set deprecation date: **June 2026** (4 months from now)

---

### 5. Legitimate `#[allow(dead_code)]` (KEEP)

Most `#[allow(dead_code)]` annotations are legitimate:

```rust
// Deserialization structs (used by serde)
// Future Phase 2 implementations (Bluetooth GATT, XPC, WASM)
// Platform-specific code (iOS, WASM)
// Test helpers and fixtures
```

**Action**: Review case-by-case, most should stay.

**False Positives** (can remove):
- `crates/songbird-orchestrator/src/server/jsonrpc_api.rs:110` - Check if actually used
- `crates/songbird-orchestrator/src/app/core.rs:28` - Entire struct marked, likely unnecessary

---

## 🔵 Low Priority (Future Work)

### 6. TODOs for Phase 2+ Features

Keep these TODOs (future work):

```rust
// TODO: Full NAT type detection (STUN - Phase 2)
// TODO: Implement XPC transport (iOS - Phase 3)
// TODO: Full UDP hole punching (P2P - Phase 2)
// TODO: mDNS service advertisement (Discovery - Phase 2)
// TODO: Kubernetes/etcd/Consul discovery (Enterprise - Phase 3)
```

**Action**: Move to `PHASE_2_ROADMAP.md`, remove inline TODOs.

---

## 📋 Cleanup Execution Plan

### Phase 1: Safe Removals (Today)

1. ✅ Remove outdated comments about completed features
2. ✅ Remove commented-out imports already marked REMOVED
3. ✅ Clean up `// TEMP DISABLED` comments (decide: remove or document)

### Phase 2: Temporarily Disabled Code (This Week)

1. ⏳ Decide: Gaming Manager - Phase 2 or remove?
2. ⏳ Decide: UniversalSecurityIntegration - superseded or deferred?
3. ⏳ Decide: Federation-aware discovery - Phase 2 branch or delete?

### Phase 3: Documentation (This Week)

1. 📝 Create `LEGACY_SUPPORT.md` with deprecation timeline
2. 📝 Create `PHASE_2_ROADMAP.md` with deferred features
3. 📝 Update `CHANGELOG.md` with cleanup notes

---

## 🎯 Recommended Immediate Actions

### Clean These Now (Zero Risk)

1. **Update comment**: `task_lifecycle/types.rs:16` - mention JSON not bincode
2. **Remove TODO**: `birdsong_integration.rs:860` - feature complete
3. **Remove commented imports**:
   - `songbird-config/src/lib.rs:119`
   - `songbird-test-utils/src/lib.rs:20`
4. **Remove `// TEMP DISABLED` sections** in:
   - `songbird-discovery/src/lib.rs:187-199`
   - `songbird-orchestrator/src/rpc/mod.rs:21-25`

### Document These (Medium Risk)

1. **Gaming Manager**: Add to Phase 2 backlog or remove
2. **Federation-aware discovery**: Move to feature branch or delete
3. **Legacy compatibility**: Document deprecation timeline

---

## 📦 Files to Archive

These files can be moved to `/archive/` folder:

1. `DEEP_DEBT_EVOLUTION_PHASE_5_COMPLETE_FEB_04_2026.md` (superseded by v3.21.0)
2. Any `federation_aware_discovery.rs` files (if deleting module)
3. Any `pure_jsonrpc_handler.rs` remnants

**Note**: Keep in git history, ecoPrimals maintains fossil record.

---

## ✅ Success Criteria

After cleanup:
- [ ] Zero commented-out imports with `// REMOVED`
- [ ] Zero `// TEMP DISABLED` without decision documented
- [ ] All completed TODOs removed or updated
- [ ] Legacy support documented with deprecation timeline
- [ ] Phase 2 features moved to roadmap document

---

**Status**: 🟡 Review Required  
**Next Step**: Decide on temporarily disabled code (gaming, security, federation)  
**Estimated Time**: 2-3 hours for safe cleanup, 1 day for decisions

