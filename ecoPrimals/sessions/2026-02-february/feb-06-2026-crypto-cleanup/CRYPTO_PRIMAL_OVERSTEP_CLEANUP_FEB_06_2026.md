# Crypto Primal Overstep - Cleanup Plan

**Date**: February 6, 2026  
**Severity**: HIGH - TRUE PRIMAL Violation  
**Finding**: Songbird has crypto libraries (should be BearDog only!)  
**Impact**: Deep Debt Score reduction due to architecture violation

---

## Executive Summary

**VIOLATION**: Songbird has direct cryptography dependencies across 10 crates.

**TRUE PRIMAL Principle**:
- 🐻🐕 **BearDog** = ALL cryptography (single audit surface)
- 🎵 **Songbird** = ALL networking (ZERO crypto)
- 🌐 **biomeOS** = Orchestration

**Current Reality**: Songbird has 18+ crypto dependencies ❌  
**Target State**: Songbird has ZERO crypto dependencies ✅

---

## Crypto Dependency Audit Results

### PRIMARY OFFENDERS

#### 1. `songbird-sovereign-onion` ⚠️ CRITICAL

**Crypto Dependencies** (6):
```toml
ed25519-dalek = "2.1"        # ❌ SHOULD BE BEARDOG
x25519-dalek = "2.0"         # ❌ SHOULD BE BEARDOG  
chacha20poly1305 = "0.10"    # ❌ SHOULD BE BEARDOG
sha3 = "0.10"                # ❌ SHOULD BE BEARDOG
sha2 = "0.10"                # ❌ SHOULD BE BEARDOG (via HMAC)
hmac = "0.12"                # ❌ SHOULD BE BEARDOG
```

**Status**: Has `standalone` feature, but it's **default**! ❌  
**Impact**: Production builds include crypto by default  
**Solution**: Make crypto optional, remove from default

#### 2. `songbird-orchestrator` ⚠️ CRITICAL

**Crypto Dependencies** (6):
```toml
aes-gcm = "0.10"             # ❌ SHOULD BE BEARDOG
ed25519-dalek = "2.1"        # ❌ SHOULD BE BEARDOG
x25519-dalek = "2.0"         # ❌ SHOULD BE BEARDOG
hmac = "0.12"                # ❌ SHOULD BE BEARDOG
argon2 = "0.5"               # ❌ SHOULD BE BEARDOG
chacha20poly1305 = "0.10"    # ❌ SHOULD BE BEARDOG
```

**Comments Say**:
> "RustCrypto: Audited pure Rust crypto"  
> "For internal operations (BTSP, BirdSong, auth)"

**Problem**: This is STILL crypto in Songbird! Should be BearDog.  
**Solution**: Remove all, delegate to BearDog via IPC

#### 3. `songbird-network-federation` ⚠️ HIGH

**Crypto Dependencies** (1):
```toml
aes-gcm = "0.10.3"           # ❌ SHOULD BE BEARDOG
```

**Usage**: Likely for encrypted federation messages  
**Solution**: Delegate to BearDog

#### 4. `songbird-tls` ⚠️ MEDIUM

**Crypto Dependencies** (1):
```toml
ed25519-dalek = "2.1"  # 100% Pure Rust Ed25519 signatures
```

**Comment**: "Pure Rust certificate generation (standalone mode)"  
**Problem**: TLS crypto should be via BearDog!  
**Solution**: Delegate certificate signing to BearDog

#### 5. `songbird-cli` ⚠️ LOW

**Crypto Dependencies** (via rustls):
```toml
rustls = { version = "0.23", features = ["ring"] }  # ❌ ring = C code
```

**Comment**: "✅ No cmake dependency! (ring = vetted BoringSSL subset)"  
**Problem**: Still C code, not Pure Rust!  
**Solution**: Use songbird-http-client (BearDog delegation) or accept for CLI

---

## SECONDARY OFFENDERS

### Crates with crypto mentions (not direct deps)

6. `songbird-bluetooth` - Category mentions "cryptography"  
7. `songbird-genesis` - Keywords: "cryptography"  
8. `songbird-lineage-relay` - Category: "cryptography"  
9. `songbird-stun` - No crypto (false positive)  
10. `songbird-observability` - No crypto (false positive)

**Status**: FALSE POSITIVES (metadata only, no actual crypto deps)  
**Action**: Clean up categories/keywords to avoid confusion

---

## Impact Analysis

### Deep Debt Score

**Current TRUE PRIMAL Score**: ~30% (due to crypto in Songbird)

| Crate | Crypto Deps | Severity |
|-------|-------------|----------|
| sovereign-onion | 6 deps | ⚠️ CRITICAL |
| orchestrator | 6 deps | ⚠️ CRITICAL |
| network-federation | 1 dep | ⚠️ HIGH |
| tls | 1 dep | ⚠️ MEDIUM |
| cli | 1 dep (ring) | ⚠️ LOW |
| **Total** | **15 crypto deps** | ❌ **VIOLATION** |

**Target TRUE PRIMAL Score**: 100% (ZERO crypto in Songbird)

**Deep Debt Impact**: +5-10% when fixed!

---

## Cleanup Strategy

### Phase 1: Make Crypto Optional (1 hour)

**For each crate with crypto**:

```toml
[dependencies]
# Make ALL crypto dependencies optional:
ed25519-dalek = { version = "2.1", optional = true }
x25519-dalek = { version = "2.0", optional = true }
chacha20poly1305 = { version = "0.10", optional = true }
sha3 = { version = "0.10", optional = true }
sha2 = { version = "0.10", optional = true }
hmac = { version = "0.12", optional = true }
aes-gcm = { version = "0.10", optional = true }
argon2 = { version = "0.5", optional = true }

[features]
default = []  # ✅ NO crypto by default!
standalone = [
    "ed25519-dalek",
    "x25519-dalek",
    "chacha20poly1305",
    "sha3",
    "sha2",
    "hmac",
    "aes-gcm",
    "argon2"
]  # Only for testing/offline development
```

### Phase 2: Gate Direct Crypto Behind Feature (2 hours)

**Pattern for all crypto code**:

```rust
// Production (always available):
pub async fn encrypt_via_beardog(
    client: &BeardogCryptoClient,
    data: &[u8]
) -> Result<Vec<u8>> {
    client.chacha20_poly1305_encrypt(...).await
}

// Testing only:
#[cfg(any(test, feature = "standalone"))]
pub fn encrypt(data: &[u8]) -> Vec<u8> {
    // Direct crypto
}
```

### Phase 3: Update All Call Sites (1 hour)

**Search and replace pattern**:

```bash
# Find all direct crypto calls
grep -r "OnionIdentity::generate()" crates/
grep -r "encrypt_data(" crates/
grep -r "derive_onion_address(" crates/

# Replace with *_via_beardog variants
```

### Phase 4: Remove from Metadata (15 minutes)

**Clean up Cargo.toml categories/keywords**:

```toml
# ❌ BEFORE:
categories = ["network-programming", "cryptography"]
keywords = ["onion", "tor", "p2p", "pure-rust", "cryptography"]

# ✅ AFTER:
categories = ["network-programming", "distributed-systems"]
keywords = ["onion", "tor", "p2p", "pure-rust", "networking"]
```

### Phase 5: Test & Validate (1 hour)

```bash
# Production build (no crypto deps):
cargo build --workspace --release

# Test build (with standalone):
cargo test --workspace --features standalone

# Verify no crypto in production:
cargo tree -p songbird-sovereign-onion | grep -E "ed25519|x25519|chacha|sha3"
# Should return NOTHING!
```

---

## Detailed Cleanup Plan

### 1. `songbird-sovereign-onion` (2 hours)

**Current State**:
- 6 mandatory crypto dependencies
- `standalone` feature is **default** ❌
- 9 methods use direct crypto

**Actions**:
1. Make all 6 crypto deps optional
2. Remove `standalone` from default features
3. Gate all direct crypto with `#[cfg(feature = "standalone")]`
4. Ensure `*_via_beardog` methods are always available
5. Update exports in `lib.rs`
6. Update tests to use `--features standalone`

**Files to Modify**:
- `Cargo.toml` - Make crypto optional
- `keys.rs` - Gate 5 methods
- `address.rs` - Gate 2 methods
- `crypto.rs` - Gate 2 methods
- `storage.rs` - Update to use async methods
- `lib.rs` - Update exports
- `tests/` - Add `standalone` feature

**Verification**:
```bash
# Should build without crypto:
cargo build -p songbird-sovereign-onion

# Should test with crypto:
cargo test -p songbird-sovereign-onion --features standalone
```

---

### 2. `songbird-orchestrator` (1.5 hours)

**Current State**:
- 6 crypto dependencies for "internal operations"
- Comment says "For BTSP, BirdSong, auth - NOT for TLS"
- But this is STILL crypto in Songbird!

**Actions**:
1. Identify what actually uses these deps:
   ```bash
   grep -r "aes_gcm\|AesGcm" crates/songbird-orchestrator/src/
   grep -r "ed25519_dalek" crates/songbird-orchestrator/src/
   grep -r "x25519_dalek" crates/songbird-orchestrator/src/
   grep -r "Hmac\|hmac::" crates/songbird-orchestrator/src/
   grep -r "Argon2\|argon2::" crates/songbird-orchestrator/src/
   grep -r "ChaCha20Poly1305" crates/songbird-orchestrator/src/
   ```

2. For each usage:
   - If for BirdSong: Delegate to BearDog
   - If for BTSP: Delegate to BearDog
   - If for auth: Already uses BearDog for JWT! Remove direct crypto.

3. Make all crypto deps optional
4. Gate any remaining crypto with `#[cfg(test)]`
5. Remove from production builds

**Expected Finding**: Most of these are probably UNUSED (leftover from migration)

---

### 3. `songbird-network-federation` (30 minutes)

**Current State**:
- 1 crypto dependency: `aes-gcm`
- Comments say rustls was removed, but AES-GCM remains

**Actions**:
1. Find what uses `aes-gcm`:
   ```bash
   grep -r "aes_gcm\|AesGcm" crates/songbird-network-federation/src/
   ```

2. Options:
   - If unused: Remove dependency ✅
   - If used: Delegate to BearDog

**Expected**: Likely unused (leftover from rustls removal)

---

### 4. `songbird-tls` (30 minutes)

**Current State**:
- 1 crypto dependency: `ed25519-dalek`
- Comment: "Pure Rust certificate generation (standalone mode)"

**Actions**:
1. Make `ed25519-dalek` optional
2. Gate direct usage with `#[cfg(test)]`
3. Add BearDog delegation for certificate signing
4. Update certificate generator

**Expected**: Needs BearDog integration for production cert generation

---

### 5. `songbird-cli` (15 minutes)

**Current State**:
- Uses `rustls` with `ring` feature (C code)

**Options**:
1. **Accept as technical debt** (CLI not critical path)
2. **Switch to songbird-http-client** (uses BearDog)
3. **Remove HTTPS support** from CLI (use HTTP only)

**Recommendation**: Accept for now (LOW priority, CLI not production)

---

### 6. Clean Metadata (15 minutes)

**Crates with crypto in metadata** (but no actual crypto):
- `songbird-bluetooth` - Remove "cryptography" category
- `songbird-genesis` - Remove "cryptography" keyword
- `songbird-lineage-relay` - Remove "cryptography" category

**Action**: Clean up to avoid confusion

---

## Expected Results

### Before Cleanup

```bash
$ cargo tree -p songbird-sovereign-onion | grep -E "ed25519|x25519|chacha|sha3"
ed25519-dalek v2.1.0
x25519-dalek v2.0.0
chacha20poly1305 v0.10.0
sha3 v0.10.0
hmac v0.12.0
sha2 v0.10.0
```

### After Cleanup

```bash
$ cargo tree -p songbird-sovereign-onion | grep -E "ed25519|x25519|chacha|sha3"
(no output - ZERO crypto in production!)

$ cargo tree -p songbird-sovereign-onion --features standalone | grep -E "ed25519|x25519|chacha|sha3"
ed25519-dalek v2.1.0  ← Only with standalone feature!
x25519-dalek v2.0.0
chacha20poly1305 v0.10.0
sha3 v0.10.0
...
```

---

## Deep Debt Score Impact

### TRUE PRIMAL Compliance

| Crate | Before | After | Status |
|-------|--------|-------|--------|
| songbird-sovereign-onion | 6 crypto deps | 0 deps | ✅ Fixed |
| songbird-orchestrator | 6 crypto deps | 0 deps | ✅ Fixed |
| songbird-network-federation | 1 crypto dep | 0 deps | ✅ Fixed |
| songbird-tls | 1 crypto dep | 0 deps | ✅ Fixed |
| songbird-cli | 1 crypto dep | 1 dep | ⚠️ Accepted |
| **Songbird Workspace** | **15 crypto deps** | **0-1 deps** | **95-100%** ✅ |

### Score Improvement

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **TRUE PRIMAL** | 30% | **95-100%** | **+65-70%** ✅ |
| **Overall Deep Debt** | 97.1% | **98.5%+** | **+1.4%** ✅ |
| **Pure Rust (No C)** | 98% | **99%+** | **+1%** ✅ |

**New Grade**: A++ → S tier (near-perfect!) 🏆

---

## Execution Order (Priority)

### IMMEDIATE (Critical Path)

1. **songbird-sovereign-onion** (2 hours)
   - PRIMARY offender (6 deps)
   - Blocks validation
   - Already has BearDog client ready!

2. **songbird-orchestrator** (1.5 hours)
   - CRITICAL (6 deps)
   - Core orchestrator should be crypto-free
   - Likely mostly unused (leftover from migration)

### HIGH PRIORITY

3. **songbird-network-federation** (30 minutes)
   - One dep (aes-gcm)
   - Likely unused

4. **songbird-tls** (30 minutes)
   - One dep (ed25519-dalek)
   - Needs BearDog integration for certs

### LOW PRIORITY

5. **songbird-cli** (15 minutes or DEFER)
   - CLI not production critical
   - Can accept ring for now

6. **Metadata cleanup** (15 minutes)
   - Low impact, but good hygiene

---

## Testing Strategy

### Per-Crate Testing

**For each refactored crate**:

```bash
# 1. Production build (no crypto):
cargo build -p $CRATE

# 2. Test build (with standalone):
cargo test -p $CRATE --features standalone --lib

# 3. Verify no crypto in production:
cargo tree -p $CRATE | grep -E "ed25519|x25519|chacha|sha3|aes|hmac"
# Should return NOTHING!
```

### Workspace Testing

**After all refactoring**:

```bash
# 1. Full workspace build:
cargo build --workspace --release

# 2. Full workspace tests:
cargo test --workspace --lib

# 3. Verify no crypto anywhere:
for crate in crates/songbird-*/Cargo.toml; do
    cargo tree -p $(basename $(dirname $crate)) 2>/dev/null | grep -E "ed25519|x25519|chacha|sha3|ring|aes" || true
done | sort -u
```

**Expected Result**: ZERO crypto dependencies in production builds!

---

## Code Pattern

### Direct Crypto → BearDog Delegation

#### Before (Primal Overstep) ❌

```rust
use ed25519_dalek::{SigningKey, VerifyingKey};

pub fn generate() -> Self {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    // ...
}
```

#### After (TRUE PRIMAL) ✅

```rust
// Production (always available):
pub async fn generate_via_beardog(
    client: &BeardogCryptoClient
) -> Result<Self> {
    let keypair = client.ed25519_generate_keypair().await?;
    // ...
}

// Testing only:
#[cfg(any(test, feature = "standalone"))]
pub fn generate() -> Self {
    use ed25519_dalek::{SigningKey, VerifyingKey};
    let signing_key = SigningKey::generate(&mut OsRng);
    // ...
}
```

---

## Success Criteria

### Must Have ✅

1. ✅ `songbird-sovereign-onion` has ZERO mandatory crypto deps
2. ✅ `songbird-orchestrator` has ZERO crypto deps
3. ✅ `songbird-network-federation` has ZERO crypto deps
4. ✅ Production builds have ZERO crypto dependencies
5. ✅ Tests pass with `--features standalone`
6. ✅ `cargo tree` shows ZERO crypto in production

### Nice to Have

1. `songbird-tls` has ZERO crypto deps (BearDog delegation)
2. `songbird-cli` uses songbird-http-client (BearDog)
3. Metadata cleaned (no "cryptography" in non-crypto crates)
4. Documentation updated

---

## Timeline

**Total Effort**: 5-7 hours

| Task | Duration | Priority |
|------|----------|----------|
| sovereign-onion cleanup | 2 hours | CRITICAL |
| orchestrator cleanup | 1.5 hours | CRITICAL |
| network-federation cleanup | 30 min | HIGH |
| tls cleanup | 30 min | MEDIUM |
| cli cleanup | 15 min | LOW (or DEFER) |
| metadata cleanup | 15 min | LOW |
| testing & validation | 1 hour | CRITICAL |
| documentation | 30 min | MEDIUM |

**Fast Track** (Critical path only): 3.5 hours  
**Complete Track** (all cleanup): 5-7 hours

---

## Risk Assessment

### Risks

1. **Breaking Tests**: Tests depend on direct crypto
   - **Mitigation**: Add `standalone` feature to dev-dependencies
   - **Testing**: `cargo test --features standalone`

2. **Breaking Production Code**: Code may call direct methods
   - **Mitigation**: Grep for usage, update to `*_via_beardog`
   - **Verification**: Build without `standalone` feature

3. **Performance**: BearDog IPC adds latency
   - **Mitigation**: Acceptable for architecture purity
   - **Benchmark**: Measure before/after

### Rollback Plan

**If critical issues**:
1. Revert Cargo.toml changes
2. Keep `standalone` in default
3. Document as "Phase 2 work" (defer)

**Safety**: All changes in git, easy to revert

---

## Next Actions

### IMMEDIATE: Start with songbird-sovereign-onion

**Steps**:
1. Read current Cargo.toml
2. Make crypto deps optional
3. Remove `standalone` from default
4. Verify build still works with/without standalone
5. Run tests with `--features standalone`

**Time**: 2 hours  
**Impact**: Largest crypto offender fixed!

---

## References

- `SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md` - Architecture spec
- `ecoPrimals/.../BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md` - Handoff
- BearDog API: All methods ready (`sha3_256`, `ed25519_*`, `x25519_*`, etc.)

---

**Status**: AUDIT COMPLETE  
**Finding**: 15 crypto deps across 5 crates ❌  
**Solution**: Make optional, gate with features, delegate to BearDog ✅  
**Impact**: +5-10% Deep Debt Score  
**Priority**: HIGH (TRUE PRIMAL violation)

**Ready to Execute**: Yes ✅
