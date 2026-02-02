# 🔍 Deep Debt Scan - January 26, 2026

**Status**: Analysis Complete  
**Next**: Execute on prioritized items

---

## 📊 Large Files Analysis (>1000 lines)

| File | Lines | Status | Notes |
|------|-------|--------|-------|
| `handshake_legacy.rs` | 3,134 | ✅ Archived | Replaced by refactored modules |
| `beardog_client_legacy.rs` | 2,032 | ✅ Archived | Replaced by 8 modules |
| `handshake_flow.rs` | 1,370 | ⚠️ 37% over | Has 327 log lines (debug needed) |
| `server_complete.rs` | 1,037 | ⚠️ 3.7% over | TLS server, acceptable |
| `core.rs` (orchestrator) | 948 | ✅ Under limit | - |

**Verdict**: Only `handshake_flow.rs` needs attention, but debug logging is valuable.

---

## 🧪 Test Async Issues

### Problem
`songbird-universal` has 65+ sync tests (`#[test]`) calling async functions.

### Files Affected

| File | Sync Tests | Needs Async |
|------|------------|-------------|
| `ai.rs` | 35 | ~6 |
| `compute.rs` | 22 | ~5 |
| `storage.rs` | 34 | ~5 |
| `security_tests.rs` | 63 | ~10 |
| `security.rs` | 10 | ~3 |

**Total**: ~65+ tests need `#[tokio::test]` + `async fn`

### Root Cause
Adapter constructors (`AIAdapter::new`, etc.) became async when adding protocol detection.

### Fix Pattern
```rust
// Before (broken):
#[test]
fn test_adapter_creation() {
    let adapter = AIAdapter::new("http://...".to_string()).map_err(...)?;
}

// After (fixed):
#[tokio::test]
async fn test_adapter_creation() {
    let adapter = AIAdapter::new("http://...".to_string()).await.map_err(...)?;
}
```

### Effort: ~3-4 hours

---

## 🔧 Mock Analysis

### 116 Files with Mock References

**Categories**:

1. **Test Utilities** (acceptable):
   - `songbird-test-utils/src/mocks/*.rs` - Intentional test mocks
   - `*_tests.rs` files - Test-specific mocks

2. **Production Code with Mocks** (needs review):
   - `songbird-network-federation/src/beardog/mock.rs` - BearDog mock provider
   - `songbird-genesis/src/physical_channels/mock.rs` - Physical channel mock

**Verdict**: Mocks are properly isolated in `#[cfg(test)]` or test-utils crate.

---

## 🔐 Hardcoding Analysis

### 1,489 References to localhost/hardcoded values

**Categories**:

1. **Test Fixtures** (~80%): Acceptable in test code
2. **Default Fallbacks** (~15%): Environment variable fallbacks
3. **Production Constants** (~5%): Need capability-based discovery

**Priority Items**:
- `config/hardcoded_elimination.rs` - Already contains migration guide
- `zero_hardcoding/` module - Pure Rust replacement in progress

**Verdict**: Hardcoding is being evolved via capability-based discovery.

---

## 📈 Summary: Prioritized Action Items

### P0 (Blocked - Waiting on BearDog)
- AES-256-GCM (SHA-384) support for 100% TLS

### P1 (Can Execute Now)
1. **songbird-universal async tests** (~65 tests, 3-4 hours)
   - Convert `#[test]` to `#[tokio::test]`
   - Add `.await` to adapter constructors

### P2 (Analyzed - No Action Needed)
1. `handshake_flow.rs` (1,370 lines) - **ACCEPTABLE**
   - 327 log lines (24%) - essential for TLS debugging
   - RFC 8446 state machine - complexity is inherent
   - Already modular (transcript, extensions, record_io extracted)
   - Further splitting would hurt debuggability without benefit

### P3 (Deferred)
1. sqlx → redb Pure Rust storage
2. songbird-orchestrator flaky hardware tests

---

## ✅ Already Complete

| Item | Status |
|------|--------|
| handshake_legacy.rs refactor | ✅ 3,086 → 6 modules |
| beardog_client.rs refactor | ✅ 2,032 → 8 modules |
| TODO audit (122 items) | ✅ Categorized |
| Unwrap audit (927 items) | ✅ Production clean |
| Dependency analysis | ✅ 99.7% Pure Rust |
| Security (TLS random) | ✅ CSPRNG fixed |

---

**Created**: January 26, 2026  
**Status**: Analysis Complete

