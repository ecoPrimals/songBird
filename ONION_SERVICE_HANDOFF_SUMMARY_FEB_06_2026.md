# 🧅 Onion Service - Handoff Summary

**Date**: February 6, 2026  
**Status**: ✅ Architecture Corrected - Ready for BearDog Team  
**Pattern**: TRUE PRIMAL (Same as TLS 1.3)

---

## 🎯 Key Realization

**Songbird has NO cryptography**. That belongs to `../beardog/`.  
Together they are: **BearDog (crypto) + Songbird (network)** coordinated by **biomeOS**.

---

## ✅ What We Completed Today

### 1. Investigated Sovereign Onion Service

**Decision**: Build our own minimal onion service instead of using Arti  
**Reason**: Arti has C dependencies (`libsqlite3`), we need 100% Pure Rust

### 2. Created Foundation (Phase 1)

**New Crate**: `songbird-sovereign-onion/`
- ✅ Onion address derivation (Tor v3 format)
- ✅ Protocol messages (KEY_EXCHANGE, DATA, CLOSE)
- ✅ Storage layer (Sled for non-secret data)
- ✅ 24 unit tests passing

**Problem**: Phase 1 had crypto dependencies directly (violates TRUE PRIMAL)

### 3. Corrected Architecture

**Realized**: Songbird delegates all crypto to BearDog (same as TLS 1.3)

**Created Documents**:
1. `BEARDOG_ONION_CRYPTO_HANDOFF_FEB_06_2026.md` - **→ For BearDog team**
2. `SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md` - Architecture specification

---

## 📋 What BearDog Needs to Add

### ONE New JSON-RPC Method

**Method**: `beardog.crypto.sha3_256`

**Purpose**: Hash data with SHA3-256 (for .onion address checksum calculation)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.sha3_256",
  "params": {
    "data": "base64_encoded_data",
    "purpose": "onion_address_checksum"
  },
  "id": 9
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "base64_encoded_hash_32_bytes"
  },
  "id": 9
}
```

**Implementation**:
- Add dependency: `sha3 = "0.10"` (Pure Rust, RustCrypto)
- Handler: ~20 lines of code
- Effort: ~1 hour (very simple)

### Everything Else Already Exists

BearDog already has (from TLS 1.3):
- ✅ `beardog.crypto.ed25519_generate` - For onion identity keys
- ✅ `beardog.crypto.ed25519_sign` - For signatures
- ✅ `beardog.crypto.x25519_generate_ephemeral` - For session keys
- ✅ `beardog.crypto.x25519_derive_secret` - For ECDH
- ✅ `beardog.crypto.chacha20_poly1305_encrypt` - For data encryption
- ✅ `beardog.crypto.chacha20_poly1305_decrypt` - For data decryption
- ✅ `beardog.crypto.hmac_sha256` - For HKDF key derivation

**Total New Work for BearDog**: Add SHA3-256 (~1 hour)

---

## 📋 What Songbird Needs to Change

### Refactor to Use BearDog Delegation

**File**: `crates/songbird-sovereign-onion/`

**Changes**:
1. Remove direct crypto dependencies from `Cargo.toml`:
   - ❌ Remove `ed25519-dalek`
   - ❌ Remove `x25519-dalek`
   - ❌ Remove `chacha20poly1305`
   - ❌ Remove `sha3`
   - ❌ Remove `hmac`
   - ❌ Remove `sha2`
   - ✅ Keep `sled`, `base32`, `serde`, `tokio` (non-crypto)

2. Refactor modules to use `BeardogCryptoClient`:
   - `address.rs`: Call `beardog.crypto.sha3_256()` for checksum
   - `keys.rs`: Call `beardog.crypto.ed25519_*()` and `x25519_*()` for keys
   - `crypto.rs`: Call `beardog.crypto.chacha20_*()` for encryption

3. Update tests to use mock BearDog

**Effort**: ~4 hours (straightforward refactor, same pattern as TLS 1.3)

---

## 📋 What biomeOS Needs to Do

### Coordinate BearDog + Songbird Lifecycle

**Deployment Graph**: `deployment/graphs/sovereign_onion_genome.toml`

**Pattern** (same as TLS 1.3):
```toml
# 1. Deploy BearDog first
[[nodes]]
name = "beardog"
primal = "beardog"
health_check = { endpoint = "/health", interval_secs = 10 }

# 2. Deploy Songbird (depends on BearDog)
[[nodes]]
name = "songbird"
primal = "songbird"
depends_on = ["beardog"]
env = [
    "CRYPTO_PROVIDER_SOCKET=${beardog_socket}",
]
```

**Effort**: ~30 minutes (copy from TLS 1.3 pattern)

---

## 🔄 Crypto Delegation Flow

### How It Works (Example: Generate .onion Address)

```
1. Songbird discovers BearDog via env var (set by biomeOS):
   CRYPTO_PROVIDER_SOCKET=/run/user/1000/biomeos/beardog.sock

2. Songbird → BearDog (JSON-RPC over Unix socket):
   beardog.crypto.ed25519_generate("onion_service")
   → {public_key, secret_key_id}
   
3. Songbird → BearDog:
   beardog.crypto.sha3_256(checksum_input, "onion_address")
   → {hash}
   
4. Songbird (local, no crypto):
   - Extract checksum from hash
   - Assemble: pubkey || checksum || version
   - Base32 encode
   - Append ".onion"
   → "vww6ybal...npyyd.onion"
```

**Key Point**: BearDog owns all crypto, Songbird just coordinates.

---

## 📊 Architecture Comparison

### Before (Incorrect)

```
songbird-sovereign-onion/
├── Cargo.toml (10 crypto dependencies) ❌
├── src/
│   ├── address.rs (uses sha3 directly) ❌
│   ├── keys.rs (uses ed25519-dalek, x25519-dalek) ❌
│   ├── crypto.rs (uses chacha20poly1305) ❌
```

**Issues**:
- ❌ Violates TRUE PRIMAL (crypto in network primal)
- ❌ Duplicates crypto (BearDog already has these)
- ❌ Split audit surface (crypto in two places)

### After (Correct)

```
songbird-sovereign-onion/
├── Cargo.toml (ZERO crypto dependencies) ✅
├── src/
│   ├── address.rs (calls beardog.crypto.sha3_256) ✅
│   ├── keys.rs (calls beardog.crypto.ed25519/x25519) ✅
│   ├── crypto.rs (calls beardog.crypto.chacha20) ✅
```

**Benefits**:
- ✅ TRUE PRIMAL compliant
- ✅ Single crypto codebase (BearDog)
- ✅ Single audit surface
- ✅ Same pattern as TLS 1.3

---

## 📈 Timeline

| Step | Owner | Effort | Status |
|------|-------|--------|--------|
| **Add SHA3-256 to BearDog** | BearDog Team | ~1 hour | ⚠️ Pending |
| **Refactor Songbird** | Songbird Team | ~4 hours | ⚠️ Pending |
| **Integration Testing** | Both Teams | ~2 hours | ⚠️ Pending |
| **biomeOS Coordination** | biomeOS Team | ~30 minutes | ⚠️ Pending |

**Total**: ~8 hours  
**Timeline**: 1-2 days  
**Result**: TRUE PRIMAL onion service

---

## 📚 Documents Created

### For BearDog Team

1. **`BEARDOG_ONION_CRYPTO_HANDOFF_FEB_06_2026.md`** ⭐
   - Complete handoff document
   - API specification for SHA3-256
   - Integration patterns
   - Testing strategy

### For Architecture Understanding

2. **`SOVEREIGN_ONION_TRUE_PRIMAL_ARCHITECTURE.md`**
   - Responsibility matrix (BearDog/Songbird/biomeOS)
   - Crypto delegation flow diagrams
   - Security model
   - Performance analysis

### For Context

3. **`SONGBIRD_ONION_EVOLUTION_PLAN_FEB_06_2026.md`**
   - Strategic analysis (build vs use Arti)
   - 5-phase implementation plan
   - Technical design

4. **`SONGBIRD_ONION_PHASE1_COMPLETE_FEB_06_2026.md`**
   - Phase 1 completion report
   - What was built (24 tests passing)
   - What needs to change (remove direct crypto)

5. **`specs/SOVEREIGN_ONION_PROTOCOL.md`**
   - Protocol specification
   - Crypto primitives needed
   - Wire format

---

## 🎯 Next Actions

### Immediate (BearDog Team)

1. Read: `BEARDOG_ONION_CRYPTO_HANDOFF_FEB_06_2026.md`
2. Implement: `beardog.crypto.sha3_256` method
3. Test: 3 unit tests for SHA3-256
4. Deploy: Updated BearDog with new method

**Deliverable**: BearDog with SHA3-256 support

### After BearDog is Ready (Songbird Team)

1. Refactor: `songbird-sovereign-onion` to use BearDog client
2. Remove: All direct crypto dependencies
3. Test: Integration tests with BearDog
4. Verify: All crypto goes through BearDog

**Deliverable**: TRUE PRIMAL onion service

### Final (biomeOS Team)

1. Create: Deployment graph
2. Wire: `CRYPTO_PROVIDER_SOCKET` env var
3. Test: Lifecycle coordination
4. Deploy: BearDog + Songbird together

**Deliverable**: Production-ready onion service

---

## ✅ Summary

### What We Learned

**Songbird has NO crypto**. All cryptography belongs to BearDog.

**Pattern**: Same as TLS 1.3
- BearDog: Security primal (owns all crypto)
- Songbird: Network primal (owns protocols)
- biomeOS: Orchestrator (coordinates lifecycle)

### What We Built

✅ Phase 1 foundation (onion address derivation, protocol, storage)  
✅ 24 unit tests passing  
✅ Complete architecture specification  
✅ Handoff document for BearDog

### What's Next

⚠️ BearDog: Add SHA3-256 (~1 hour)  
⚠️ Songbird: Refactor to use BearDog (~4 hours)  
⚠️ Integration: Test crypto delegation (~2 hours)  
⚠️ biomeOS: Deploy coordination (~30 minutes)

**Timeline**: 1-2 days  
**Result**: TRUE PRIMAL onion service (100% Pure Rust, zero crypto in Songbird)

---

**Handoff Complete**: February 6, 2026  
**Next**: BearDog team implements SHA3-256  
**Pattern**: Proven with TLS 1.3 (production-ready)

🐻🐕 **BearDog (crypto)** + 🎵 **Songbird (network)** = 🧅 **Sovereign Onion**
