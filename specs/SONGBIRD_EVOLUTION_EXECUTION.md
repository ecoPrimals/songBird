# Songbird Evolution Execution Plan

**Date:** January 24, 2026  
**Version:** v5.20.0 → v5.23.0 (Phase 3 Complete)  
**Status:** ✅ PHASE 3 COMPLETE - Production Logging Cleanup  

---

## 📊 Audit Results

### External Dependencies ✅

| Dependency | Type | Status | Action |
|------------|------|--------|--------|
| tokio | Pure Rust | ✅ Keep | None |
| serde/serde_json | Pure Rust | ✅ Keep | None |
| hyper | Pure Rust | ✅ Keep | None |
| sha2 | Pure Rust | ✅ Keep | None |
| reqwest | Mixed | ⚠️ Review | Only used internally, no TLS |
| sys-info | C bindings | 🔴 Eliminate | Replace with pure Rust |

**Verdict:** 99% Pure Rust. Only `sys-info` has C bindings - target for elimination.

### Large Files Requiring Smart Refactoring 🔄

| File | Lines | Priority | Action |
|------|-------|----------|--------|
| `handshake_legacy.rs` | 2765 | 🔴 HIGH | Split into modules |
| `beardog_client.rs` | 1661 | 🔴 HIGH | Abstract to capability |
| `server_complete.rs` | 953 | 🟡 MEDIUM | Modularize |
| `core.rs` | 915 | 🟡 MEDIUM | Extract components |

### Unsafe Code ✅

| Location | Usage | Status |
|----------|-------|--------|
| `quantum_allocator.rs` | `GlobalAlloc` trait | ✅ Required - Cannot eliminate |

**Verdict:** Only 1 legitimate unsafe block (required by `GlobalAlloc` trait). 99.99% Safe Rust.

### Production Mocks ✅

| Location | Type | Status |
|----------|------|--------|
| `beardog/mock.rs` | `#[cfg(test)]` | ✅ Test-only |
| `noop.rs` | No-Op provider | ✅ Production-safe degradation |
| `test-utils/mocks/` | Test helpers | ✅ Test-only |

**Verdict:** All mocks properly isolated to testing. No production mocks found.

### Hardcoded Primal References 🔴

| Pattern | Files | Action |
|---------|-------|--------|
| `BearDog` | 251 files | Evolve to capability discovery |
| `/tmp/beardog.sock` | ~20 files | Evolve to runtime discovery |

---

## 🎯 Evolution Phases

### Phase 1: Capability Abstraction (v5.21.0)

**Goal:** Abstract `BearDogClient` to generic `CryptoCapabilityClient`

```rust
// BEFORE (hardcoded)
pub struct BearDogClient {
    socket_path: String,
}

// AFTER (capability-based)
pub struct CryptoCapabilityClient {
    provider: Box<dyn CryptoCapability>,
}

pub trait CryptoCapability: Send + Sync {
    async fn generate_keypair(&self, algorithm: &str) -> Result<KeyPair>;
    async fn derive_shared_secret(&self, our_secret: &[u8], their_public: &[u8]) -> Result<Vec<u8>>;
    async fn encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    // ... TLS-specific operations
}
```

**Files to modify:**
- `beardog_client.rs` → `crypto_capability.rs`
- All files importing `BearDogClient`

### Phase 2: Runtime Discovery (v5.22.0)

**Goal:** Discover crypto provider at runtime instead of hardcoded socket

```rust
// BEFORE (hardcoded)
let beardog = BearDogClient::new("/tmp/beardog.sock");

// AFTER (runtime discovery)
let crypto = CryptoCapabilityClient::discover().await?;
// Discovers via:
// 1. Environment: CRYPTO_CAPABILITY_SOCKET
// 2. Neural API capability query
// 3. Well-known default
```

### Phase 3: Smart File Refactoring (v5.23.0)

**Goal:** Refactor `handshake_legacy.rs` (2765 lines) into cohesive modules

```
tls/
├── handshake/
│   ├── mod.rs           # Public API
│   ├── state_machine.rs # Handshake state
│   ├── client_hello.rs  # ClientHello building
│   ├── server_hello.rs  # ServerHello parsing
│   ├── encrypted.rs     # Encrypted handshake messages
│   ├── finished.rs      # Finished message handling
│   ├── keys.rs          # Key derivation
│   └── transcript.rs    # Transcript management
```

### Phase 4: Semantic Translation (v6.0.0)

**Goal:** Support biomeOS semantic translation

```rust
// Songbird uses semantic intent
let result = crypto.call_semantic("encrypt_application_data", params).await?;

// biomeOS translates to provider-specific
// "encrypt_application_data" → "crypto.aes128_gcm_encrypt" (BearDog)
// "encrypt_application_data" → "aes.encrypt" (other provider)
```

---

## 🔧 Execution Steps

### Step 1: Create `CryptoCapability` Trait

Create new file with capability trait:

```rust
// crates/songbird-http-client/src/crypto/capability.rs

use async_trait::async_trait;
use crate::error::Result;

/// Cryptographic capability abstraction
/// 
/// Abstracts the underlying crypto provider (BearDog, etc.)
/// enabling runtime discovery and semantic translation.
#[async_trait]
pub trait CryptoCapability: Send + Sync {
    /// Provider name for debugging
    fn name(&self) -> &str;
    
    /// Check if provider is available
    async fn is_available(&self) -> bool;
    
    // === Key Exchange ===
    async fn generate_x25519_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)>;
    async fn derive_x25519_shared_secret(&self, our_secret: &[u8], their_public: &[u8]) -> Result<Vec<u8>>;
    
    // === AEAD Encryption ===
    async fn aes128_gcm_encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn aes128_gcm_decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn aes256_gcm_encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn aes256_gcm_decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn chacha20_poly1305_encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    async fn chacha20_poly1305_decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
    
    // === Hashing ===
    async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>>;
    
    // === Key Derivation ===
    async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>>;
    async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>>;
    
    // === TLS 1.3 Specific ===
    async fn tls_derive_handshake_secrets(&self, shared_secret: &[u8], transcript_hash: &[u8]) -> Result<TlsHandshakeSecrets>;
    async fn tls_derive_application_secrets(&self, handshake_secret: &[u8], transcript_hash: &[u8]) -> Result<TlsApplicationSecrets>;
    async fn tls_compute_finished_verify_data(&self, base_key: &[u8], transcript_hash: &[u8]) -> Result<Vec<u8>>;
}
```

### Step 2: Implement BearDog Provider

```rust
// crates/songbird-http-client/src/crypto/beardog_provider.rs

/// BearDog implementation of CryptoCapability
pub struct BearDogProvider {
    socket_path: String,
    request_id: AtomicU64,
}

impl BearDogProvider {
    pub fn new(socket_path: impl Into<String>) -> Self { ... }
    
    pub async fn discover() -> Result<Self> {
        // 1. Try BEARDOG_SOCKET env var
        // 2. Try CRYPTO_CAPABILITY_SOCKET env var
        // 3. Try well-known /tmp/beardog.sock
        // 4. Try Neural API capability discovery
    }
}

#[async_trait]
impl CryptoCapability for BearDogProvider {
    fn name(&self) -> &str { "BearDog" }
    // ... implement all methods via JSON-RPC
}
```

### Step 3: Update Client to Use Capability

```rust
// crates/songbird-http-client/src/client.rs

pub struct SongbirdHttpClient {
    crypto: Arc<dyn CryptoCapability>,  // Changed from BearDogClient
    // ...
}

impl SongbirdHttpClient {
    /// Create with discovered crypto provider
    pub async fn new() -> Result<Self> {
        let crypto = BearDogProvider::discover().await?;
        Self::with_crypto(Arc::new(crypto))
    }
    
    /// Create with explicit crypto provider
    pub fn with_crypto(crypto: Arc<dyn CryptoCapability>) -> Result<Self> {
        // ...
    }
}
```

---

## 📋 File Changes Summary

### New Files

| File | Purpose |
|------|---------|
| `src/crypto/mod.rs` | Crypto module |
| `src/crypto/capability.rs` | `CryptoCapability` trait |
| `src/crypto/beardog_provider.rs` | BearDog implementation |
| `src/crypto/discovery.rs` | Runtime discovery |

### Renamed Files

| From | To |
|------|-----|
| `beardog_client.rs` | `crypto/beardog_provider.rs` |

### Modified Files

| File | Changes |
|------|---------|
| `client.rs` | Use `CryptoCapability` trait |
| `tls/handshake_legacy.rs` | Use `CryptoCapability` trait |
| `tls/record.rs` | Use `CryptoCapability` trait |
| `lib.rs` | Export new crypto module |

---

## 🎯 Success Criteria

### v5.21.0 (Capability Abstraction) ✅ COMPLETE
- [x] `CryptoCapability` trait defined
- [x] `BearDogProvider` implements trait
- [x] Runtime discovery via `discover_crypto_capability()`
- [x] Backward compatible (BearDogClient still works)
- [x] All 164 tests pass

### v5.22.0 (Full TLS Migration) ✅ COMPLETE
- [x] `handshake_legacy.rs` migrated to `CryptoCapability`
- [x] `record.rs` migrated to `CryptoCapability`
- [x] `client.rs` migrated to `CryptoCapability`
- [x] All 161+ tests pass
- [x] Build passes across workspace
- [x] Discovery via environment variables (CRYPTO_CAPABILITY_SOCKET, BEARDOG_SOCKET)

### v5.23.0 (Production Cleanup) ✅ COMPLETE
- [x] Analyzed handshake_legacy.rs structure (2770 lines, 499 log statements)
- [x] Converted verbose diagnostic `info!` logs to `trace!`
- [x] Hex dumps and byte-level logging now at `trace!` level
- [x] Production output clean and focused
- [x] All 161+ tests pass

### v5.24.0 (Smart Refactoring) 📋 FUTURE
- [ ] `handshake_legacy.rs` split into <500 line modules
- [ ] Each module has single responsibility
- [ ] All tests pass

### v6.0.0 (Semantic Translation)
- [ ] biomeOS semantic translation working
- [ ] Provider can be swapped without code changes
- [ ] Capability discovery fully dynamic

---

## 📚 References

- RFC 8446 - TLS 1.3
- biomeOS Capability Discovery Spec
- Songbird TRUE PRIMAL Architecture

