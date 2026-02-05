# 🧹 Archive Cleanup Plan - February 5, 2026

**Date**: February 5, 2026  
**Version**: v3.24.0  
**Objective**: Clean archive code and outdated TODOs

---

## 📋 Items Found for Cleanup

### 1. Root-Level Completion Docs (Move to ecoPrimals/) ✅

These are completion reports that belong in the fossil record:

| File | Size | Purpose | Action |
|------|------|---------|--------|
| `ROOT_DOCS_CLEANUP_COMPLETE_FEB_05_2026.md` | 11KB | Root docs cleanup report | Move to `ecoPrimals/sessions/2026-02-february/` |
| `SESSION_COMPLETE_FEB_05_2026_RELAY_SERVER.md` | 20KB | Relay session completion | Move to `ecoPrimals/sessions/2026-02-february/` |
| `RELAY_SERVER_COMPLETE_FEB_04_2026.md` | 14KB | Relay implementation report | Move to `ecoPrimals/sessions/2026-02-february/` |
| `RELAY_IMPLEMENTATION_FINAL_STATUS.md` | 19KB | Relay final status | Move to `ecoPrimals/sessions/2026-02-february/` |

**Total**: 4 files, ~64KB to archive

---

### 2. Outdated "TEMPORARILY DISABLED" Comments 🔍

These comments are from Nov 2025 and should be reviewed:

#### A. Discovery Module Comments (Outdated)

**File**: `crates/songbird-discovery/src/discovery/mod.rs`

```rust
// TEMP DISABLED: Has syntax errors (mismatched delimiters) - needs fixing
// pub mod enhanced_discovery;

// TEMP DISABLED: Depend on enhanced_discovery or have syntax issues
// pub mod monitoring;
// pub mod network;
// pub mod resources;
// pub mod songbird_discovery;
```

**Status**: These modules were disabled due to syntax errors from Nov 2025.  
**Assessment**: Either fix and re-enable OR remove comments and keep disabled permanently  
**Recommendation**: **Remove commented modules** - they've been replaced by universal architecture

#### B. Discovery Tests (Can be cleaned)

**File**: `crates/songbird-discovery/src/discovery_tests.rs`

```rust
// use songbird_universal::  // TEMPORARILY DISABLED - {ServiceInfo as UniversalServiceInfo, UniversalHealthStatus};
```

**File**: `crates/songbird-discovery/src/discovery/event_streaming.rs`

```rust
// use songbird_universal::  // TEMPORARILY DISABLED - ServiceInfo;
```

**Assessment**: These were likely temporary during refactoring  
**Recommendation**: **Remove comments** - if imports not needed, clean them up

#### C. Config Module (Nov 2025)

**File**: `crates/songbird-config/src/canonical/mod.rs`

```rust
// TEMPORARILY DISABLED (Nov 10, 2025): Needs updating for consolidated APIs (~77 errors)
// TODO: Update testing.rs to match current canonical struct definitions
```

**Assessment**: Still valid - testing.rs still needs updates  
**Recommendation**: **Keep** - this is a legitimate TODO for Phase 2

#### D. Universal Communication (Minor)

**File**: `crates/songbird-universal/src/communication.rs`

```rust
// use songbird_config::AuthMethod;  // TEMPORARILY DISABLED
```

**Assessment**: Unused import  
**Recommendation**: **Remove entire commented line** if not needed

---

### 3. Legitimate TODOs (Keep) ✅

These TODOs are valid future work and should be kept:

#### Phase 2 / BearDog API Integration (15+ TODOs)
- `TODO: Use universal IPC service discovery` (crypto.rs)
- `TODO: Once BearDog adds certificate.generate_self_signed` (cert/generator.rs)
- `TODO: Add random generation method to BearDog` (handshake/mod.rs)
- `TODO(P0): Add BearDog signing integration` (messages.rs)
- Many more awaiting BearDog API additions

#### Platform-Specific (iOS, WASM) (10+ TODOs)
- `TODO: Requires platform-specific bindings` (ios.rs)
- `TODO: Implement XPC transport using Pure Rust bindings` (ios.rs)
- `TODO: Implement global WASM primal registry` (wasm.rs)

#### Future Features (20+ TODOs)
- `TODO: Implement full NAT type detection` (stun/client.rs)
- `TODO: Replace with watch channel or await-able broadcaster API` (relay.rs)
- `TODO: Add cluster support` (anonymous/broadcaster.rs)
- `TODO: Implement actual SoloKey/FIDO2 verification` (solokey.rs)

**Assessment**: All legitimate future work  
**Recommendation**: **Keep all** - well-documented future enhancements

---

### 4. False Positives (Ignore) ✅

These are NOT issues:

- `TEMP_VAR`, `TEMP_DIR` in tests - legitimate test variables
- `DEFAULT_RETRY_ATTEMPTS` - legitimate constant name
- `COMPLETE` in completion report filenames - intentional naming

---

## 🎯 Cleanup Actions

### Priority 1: Archive Completion Docs (High Priority) ✅

**Action**: Move 4 completion docs to `ecoPrimals/sessions/2026-02-february/`

**Commands**:
```bash
mv ROOT_DOCS_CLEANUP_COMPLETE_FEB_05_2026.md \
   ecoPrimals/sessions/2026-02-february/

mv SESSION_COMPLETE_FEB_05_2026_RELAY_SERVER.md \
   ecoPrimals/sessions/2026-02-february/

mv RELAY_SERVER_COMPLETE_FEB_04_2026.md \
   ecoPrimals/sessions/2026-02-february/

mv RELAY_IMPLEMENTATION_FINAL_STATUS.md \
   ecoPrimals/sessions/2026-02-february/
```

**Result**: Root directory cleaner, fossil record preserved ✅

---

### Priority 2: Clean Discovery Module Comments (Medium Priority) ⚠️

**Option A**: Remove dead commented-out modules (Recommended)

**File**: `crates/songbird-discovery/src/discovery/mod.rs`

**Change**:
```diff
-// Enhanced discovery with federation capabilities (NEW)
-// TEMP DISABLED: Has syntax errors (mismatched delimiters) - needs fixing
-// See: lines 260, 304, 339, 410, 414, 471, 480
-// pub mod enhanced_discovery;
-
-// Existing submodules (already well-organized)
 pub mod config;
-// TEMP DISABLED: Depend on enhanced_discovery or have syntax issues
-// pub mod monitoring;
-// pub mod network;
-// pub mod resources;
-// pub mod songbird_discovery;
 pub mod types;
```

**Rationale**: These modules haven't been used since Nov 2025. Universal architecture has replaced them.

**Option B**: Keep comments as historical reference

**Decision**: Ask user preference

---

### Priority 3: Remove Orphaned Import Comments (Low Priority) 🔧

**File**: `crates/songbird-universal/src/communication.rs`

**Change**:
```diff
 use songbird_types::{AuthMethod, PrimalIdentity, SongbirdResult};
-// use songbird_config::AuthMethod;  // TEMPORARILY DISABLED
```

**File**: `crates/songbird-discovery/src/discovery_tests.rs`

**Change**:
```diff
 use super::*;
-// use songbird_universal::  // TEMPORARILY DISABLED - {ServiceInfo as UniversalServiceInfo, UniversalHealthStatus};
```

**Rationale**: These are remnants from refactoring. Either remove or uncomment if needed.

---

## 📊 Impact Assessment

### Root Directory Before Cleanup

```
songbird/
├── README.md                                    (KEEP - essential)
├── EXECUTIVE_SUMMARY.md                         (KEEP - essential)
├── CHANGELOG.md                                 (KEEP - essential)
├── DEPLOYMENT_READY_STATUS.md                   (KEEP - essential)
├── ROOT_DOCS_INDEX.md                           (KEEP - essential)
├── UPSTREAM_EVOLUTION_TRACKER.md                (KEEP - active tracking)
├── NEXT_EVOLUTION_OPPORTUNITIES_FEB_05_2026.md  (KEEP - active roadmap)
├── FINAL_HANDOFF_FEB_05_2026.md                 (KEEP - integration doc)
├── ROOT_DOCS_CLEANUP_COMPLETE_FEB_05_2026.md    (MOVE - completion report)
├── SESSION_COMPLETE_FEB_05_2026_RELAY_SERVER.md (MOVE - session report)
├── RELAY_SERVER_COMPLETE_FEB_04_2026.md         (MOVE - implementation report)
├── RELAY_IMPLEMENTATION_FINAL_STATUS.md         (MOVE - status report)
└── ...
```

### Root Directory After Cleanup

```
songbird/
├── README.md                                    ✅ (essential)
├── EXECUTIVE_SUMMARY.md                         ✅ (essential)
├── CHANGELOG.md                                 ✅ (essential)
├── DEPLOYMENT_READY_STATUS.md                   ✅ (essential)
├── ROOT_DOCS_INDEX.md                           ✅ (essential)
├── UPSTREAM_EVOLUTION_TRACKER.md                ✅ (active tracking)
├── NEXT_EVOLUTION_OPPORTUNITIES_FEB_05_2026.md  ✅ (active roadmap)
├── FINAL_HANDOFF_FEB_05_2026.md                 ✅ (integration doc)
└── ...

(4 completion reports moved to ecoPrimals/sessions/2026-02-february/)
```

**Result**: Cleaner, more focused root directory ✅

---

## 🎯 Recommendation

### Proceed with:

1. ✅ **Move 4 completion docs** to ecoPrimals/sessions/2026-02-february/
2. ⚠️ **Review discovery module comments** - likely safe to remove
3. 🔧 **Remove orphaned import comments** - minor cleanup
4. ✅ **Keep all legitimate TODOs** - document future work

### Skip:

- ❌ Don't touch legitimate TODOs (Phase 2, BearDog API, platform-specific)
- ❌ Don't touch test variables (TEMP_VAR, etc.)
- ❌ Don't touch canonical/mod.rs TODO (still valid)

---

## 📝 Questions for User

1. **Discovery module cleanup**: Remove old commented-out modules (enhanced_discovery, monitoring, network, etc.)?
   - Recommendation: **Remove** (replaced by universal architecture)

2. **Import comments**: Remove orphaned "TEMPORARILY DISABLED" import comments?
   - Recommendation: **Remove** (cleanup from old refactoring)

---

## ✅ Safe to Execute

**Low Risk Actions**:
1. Move completion docs to ecoPrimals/ (reversible)
2. Remove orphaned import comments (cosmetic)

**Review Before Action**:
1. Discovery module commented code (may have historical value)

---

**Status**: ✅ **Ready for cleanup**  
**Risk**: Low (mostly archiving and comment cleanup)  
**Impact**: Cleaner root directory, easier navigation
