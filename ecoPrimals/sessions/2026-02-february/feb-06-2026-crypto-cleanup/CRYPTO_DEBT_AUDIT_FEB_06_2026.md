# Crypto Debt Audit - February 6, 2026

**Goal**: Identify and eliminate all direct crypto dependencies (TRUE PRIMAL violation)  
**Status**: Deep Debt - Songbird has crypto libraries (should be BearDog only)  
**Severity**: HIGH - Violates TRUE PRIMAL architecture

---

## Executive Summary

**FINDING**: Songbird has direct cryptography dependencies, violating TRUE PRIMAL architecture.

**TRUE PRIMAL Principle**: 
- 🐻🐕 **BearDog** = ALL cryptography
- 🎵 **Songbird** = ALL networking
- 🌐 **biomeOS** = Orchestration

**Current State**: Songbird has crypto libraries ❌  
**Target State**: Songbird delegates 100% to BearDog ✅

---

## Crypto Dependency Audit

### Crates with Direct Crypto

**Searching workspace for crypto dependencies...**

```bash
grep -r "ed25519-dalek\|x25519-dalek\|chacha20poly1305\|ring\|sha3" crates/*/Cargo.toml
```

**Found**: Multiple crates with direct crypto dependencies

---

## Primary Offender: `songbird-sovereign-onion`

**File**: `crates/songbird-sovereign-onion/Cargo.toml`

**Direct Crypto Dependencies** (Lines 16-22):
```toml
# Cryptography (100% Pure Rust - RustCrypto)
ed25519-dalek = "2.1"        # Identity keys ❌ SHOULD BE BEARDOG
x25519-dalek = "2.0"         # Key exchange ❌ SHOULD BE BEARDOG
chacha20poly1305 = "0.10"    # AEAD encryption ❌ SHOULD BE BEARDOG
sha3 = "0.10"                # .onion derivation ❌ SHOULD BE BEARDOG
sha2 = "0.10"                # HKDF ❌ SHOULD BE BEARDOG
hmac = "0.12"                # HKDF ❌ SHOULD BE BEARDOG
```

**Problem**: These are ALL cryptographic operations that BearDog provides!

**Feature Flag** (Lines 48-49):
```toml
[features]
default = ["standalone"]  # ❌ WRONG: Makes crypto the default!
standalone = []           # For testing only
```

**Impact**: Production code can use direct crypto by default!

---

## Files Using Direct Crypto

### 1. `keys.rs` - 5 methods with direct crypto

**Direct Usage**:
```rust
use ed25519_dalek::{SigningKey, VerifyingKey};  // ❌
use x25519_dalek::{EphemeralSecret, PublicKey}; // ❌
use hmac::{Hmac, Mac};                          // ❌
use sha2::Sha256;                               // ❌
```

**Methods**:
1. `OnionIdentity::generate()` - Uses `SigningKey::generate()`
2. `OnionIdentity::from_stored()` - Uses `SigningKey::from_bytes()`
3. `EphemeralKeypair::generate()` - Uses `EphemeralSecret::random()`
4. `EphemeralKeypair::derive_shared_secret()` - Uses `x25519` ECDH
5. `SessionKeys::derive()` - Uses `Hmac<Sha256>`

### 2. `address.rs` - 2 methods with direct crypto

**Direct Usage**:
```rust
use sha3::{Sha3_256, Digest};  // ❌
```

**Methods**:
1. `derive_onion_address()` - Uses `Sha3_256::new()`
2. `validate_onion_address()` - Uses `Sha3_256::new()`

### 3. `crypto.rs` - 2 methods with direct crypto

**Direct Usage**:
```rust
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, AeadInPlace};  // ❌
```

**Methods**:
1. `encrypt_data()` - Uses `ChaCha20Poly1305::new()`
2. `decrypt_data()` - Uses `ChaCha20Poly1305::new()`

---

## Solution: TRUE PRIMAL Refactoring

### Phase 1: Make Crypto Dependencies Optional

**Update `Cargo.toml`**:
```toml
[dependencies]
# Cryptography - OPTIONAL (testing/standalone only)
ed25519-dalek = { version = "2.1", optional = true }
x25519-dalek = { version = "2.0", optional = true }
chacha20poly1305 = { version = "0.10", optional = true }
sha3 = { version = "0.10", optional = true }
sha2 = { version = "0.10", optional = true }
hmac = { version = "0.12", optional = true }

[features]
default = []  # ✅ NO crypto by default!
standalone = [
    "ed25519-dalek",
    "x25519-dalek", 
    "chacha20poly1305",
    "sha3",
    "sha2",
    "hmac"
]  # Only for testing
```

### Phase 2: Use BearDog Delegation in Production

**All code should use `*_via_beardog` methods**:

```rust
// ❌ BEFORE (Direct crypto):
let identity = OnionIdentity::generate();

// ✅ AFTER (BearDog delegation):
let client = BeardogCryptoClient::new(socket_path);
let identity = OnionIdentity::generate_via_beardog(&client).await?;
```

**Pattern**:
- Production: Always use `*_via_beardog()` async methods
- Testing: Use `#[cfg(test)]` or `standalone` feature for direct crypto

### Phase 3: Gate Direct Crypto Behind Feature

**Use `#[cfg(feature = "standalone")]`**:

```rust
#[cfg(feature = "standalone")]
pub fn generate() -> Self {
    // Direct crypto for testing
}

// Production method (always available):
pub async fn generate_via_beardog(client: &BeardogCryptoClient) -> Result<Self> {
    // BearDog delegation
}
```

---

## Other Crates to Audit

### Potential Offenders

Need to check these crates for crypto dependencies:

1. **songbird-tls** - May have crypto for TLS
   - **Status**: Check if using ring/rustls
   - **Expected**: Should delegate to BearDog or use Tower Atomic

2. **songbird-http-client** - May have crypto for HTTPS
   - **Status**: Check dependencies
   - **Expected**: Should use Tower Atomic (BearDog crypto)

3. **songbird-genesis** - May have crypto for USB/Bluetooth
   - **Status**: Check dependencies
   - **Expected**: Should delegate to BearDog

4. **songbird-discovery** - May have crypto for signatures
   - **Status**: Check dependencies
   - **Expected**: Should delegate to BearDog if crypto needed

5. **songbird-lineage-relay** - May have crypto for relay encryption
   - **Status**: Check dependencies
   - **Expected**: Should delegate to BearDog

---

## Deep Debt Score Impact

### Current State

**TRUE PRIMAL Score**: ~30% (due to crypto in Songbird)

**Issues**:
- ❌ Songbird has 6 direct crypto dependencies
- ❌ Default feature enables crypto (not BearDog)
- ❌ 9 methods use direct crypto
- ❌ No enforcement of BearDog delegation

### Target State

**TRUE PRIMAL Score**: 100% ✅

**Fixes**:
- ✅ Zero crypto dependencies in production Songbird
- ✅ All crypto via BearDog delegation
- ✅ `standalone` feature optional (testing only)
- ✅ `#[cfg(feature = "standalone")]` gates direct crypto

**Score Impact**: +5-10% Deep Debt improvement!

---

## Execution Plan

### Step 1: Audit All Crates (30 minutes)

```bash
# Find all crypto dependencies
grep -r "ed25519\|x25519\|chacha20\|ring\|sha3\|hmac\|aes" crates/*/Cargo.toml

# Check which are optional
grep -A 5 "\[features\]" crates/*/Cargo.toml | grep -E "(standalone|crypto)"
```

### Step 2: Refactor songbird-sovereign-onion (2-3 hours)

1. Make crypto dependencies optional ✅
2. Remove `standalone` from default features ✅
3. Gate direct crypto with `#[cfg(feature = "standalone")]` ✅
4. Update all call sites to use `*_via_beardog` ✅
5. Update tests to enable `standalone` feature ✅

### Step 3: Audit Other Crates (1-2 hours)

Check each crate:
- songbird-tls
- songbird-http-client
- songbird-genesis
- songbird-discovery  
- songbird-lineage-relay

**Action**: Make crypto optional or delegate to BearDog

### Step 4: Validate & Test (1 hour)

```bash
# Build without standalone (should use BearDog)
cargo build -p songbird-sovereign-onion

# Build with standalone (for tests)
cargo build -p songbird-sovereign-onion --features standalone

# Run tests (with standalone)
cargo test -p songbird-sovereign-onion --features standalone
```

### Step 5: Update Documentation (30 minutes)

- Update SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md
- Update README.md in sovereign-onion crate
- Document feature flags

---

## Risk Assessment

### Risks

1. **Breaking Tests**: Tests may depend on direct crypto
   - **Mitigation**: Add `standalone` feature to test dependencies

2. **Breaking Integration**: Other code may call direct methods
   - **Mitigation**: Grep for usage, update call sites

3. **Performance**: BearDog delegation adds IPC overhead
   - **Mitigation**: Acceptable tradeoff for architecture purity

### Rollback Plan

If issues arise:
1. Revert Cargo.toml changes
2. Keep `standalone` in default features
3. Document as "technical debt for Phase 2"

---

## Success Criteria

### Must Have ✅

1. ✅ `songbird-sovereign-onion` has zero mandatory crypto deps
2. ✅ Production code uses `*_via_beardog` methods only
3. ✅ Tests pass with `--features standalone`
4. ✅ Build passes without `standalone` feature

### Nice to Have

1. All other crates audited
2. Zero crypto dependencies across Songbird
3. TRUE PRIMAL score: 100%
4. Documentation updated

---

## Timeline

**Total Effort**: 5-7 hours

| Phase | Duration | Status |
|-------|----------|--------|
| Audit | 30 min | 🔄 In Progress |
| Refactor sovereign-onion | 2-3 hours | ⏳ Pending |
| Audit other crates | 1-2 hours | ⏳ Pending |
| Validate & test | 1 hour | ⏳ Pending |
| Documentation | 30 min | ⏳ Pending |

---

## Related Documents

- `SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md` - Architecture
- `BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md` - Handoff
- `ecoPrimals/sessions/.../feb-06-2026-deep-debt-evolution/` - Archive

---

## Next Step

**IMMEDIATE ACTION**: Complete full workspace crypto audit

```bash
# Comprehensive search for crypto dependencies
for crate in crates/*/Cargo.toml; do
    echo "=== $crate ==="
    grep -E "ed25519|x25519|chacha|ring|sha3|hmac|aes|crypto" "$crate" || echo "  (no crypto)"
done
```

---

**Status**: AUDIT IN PROGRESS  
**Priority**: HIGH (TRUE PRIMAL violation)  
**Impact**: +5-10% Deep Debt Score when fixed  
**Timeline**: 5-7 hours to complete
