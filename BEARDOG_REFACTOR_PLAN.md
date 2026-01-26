# BearDog Client Smart Refactoring Plan
## January 26, 2026

**Current State**: 2,020 lines in single file (VIOLATION of 1,000 line limit!)

**Target**: ~250 lines per module (8 modules)

---

## Module Breakdown

```
beardog_client/
├── mod.rs              (~50 lines)   - Module organization and re-exports
├── types.rs            (~100 lines)  - JSON-RPC types, TlsSecrets struct
├── core.rs             (~200 lines)  - BearDogClient struct, BearDogMode enum, constructors
├── rpc.rs              (~150 lines)  - Base RPC call() method, error handling
├── key_exchange.rs     (~150 lines)  - generate_keypair(), ecdh_derive()
├── tls_secrets.rs      (~400 lines)  - TLS key derivation (handshake, application, finished)
├── aead.rs             (~350 lines)  - AEAD encrypt/decrypt (AES-GCM, ChaCha20-Poly1305)
└── hash.rs             (~100 lines)  - SHA-256, SHA-384, HKDF operations
```

**Total**: ~1,500 lines across 8 modules (from 2,020-line monolith)

---

## Extraction Order

### Session 1: Foundation (~20 min)
1. Create `beardog_client/` directory
2. Create `mod.rs` with module declarations
3. Extract `types.rs` - JSON-RPC structures, TlsSecrets

### Session 2: Core Client (~20 min)
4. Extract `core.rs` - BearDogClient struct, BearDogMode enum, constructors
5. Extract `rpc.rs` - Base call() method, semantic routing

### Session 3: Crypto Operations (~30 min)
6. Extract `key_exchange.rs` - X25519 key generation, ECDH
7. Extract `tls_secrets.rs` - All TLS key derivation methods
8. Extract `aead.rs` - AEAD encrypt/decrypt operations
9. Extract `hash.rs` - Hash and HKDF operations

### Session 4: Finalize (~10 min)
10. Update parent mod.rs to use new module
11. Run all tests
12. Verify build

---

## Key Principles

1. **Preserve API**: All public exports remain the same
2. **Zero behavioral changes**: Exact same functionality
3. **Test preservation**: All 25+ tests must pass
4. **pub(super) for internals**: Keep encapsulation
5. **Clear separation**: Each module has single responsibility

---

## Current Public API (must preserve)

```rust
// Types
pub enum BearDogMode { Direct, NeuralApi }
pub struct BearDogClient
pub struct TlsSecrets

// Constructors
impl BearDogClient {
    pub fn new_direct(socket: impl Into<String>) -> Self
    pub fn new_neural_api(socket: impl Into<String>) -> Self
    pub fn new(socket: impl Into<String>) -> Self
    pub fn from_env() -> Self
}

// Key Exchange
pub async fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>)>
pub async fn ecdh_derive(&self, private: &[u8], public: &[u8]) -> Result<Vec<u8>>

// TLS Secrets
pub async fn tls_derive_handshake_secrets(...) -> Result<TlsSecrets>
pub async fn tls_derive_application_secrets(...) -> Result<TlsSecrets>
pub async fn tls_compute_finished_verify_data(...) -> Result<Vec<u8>>
pub async fn tls_derive_secrets(...) -> Result<TlsSecrets>

// AEAD
pub async fn encrypt(&self, cipher: &str, key: &[u8], nonce: &[u8], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>
pub async fn decrypt(&self, cipher: &str, key: &[u8], nonce: &[u8], aad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>
pub async fn encrypt_aes_128_gcm(...) -> Result<Vec<u8>>
pub async fn encrypt_aes_256_gcm(...) -> Result<Vec<u8>>
pub async fn decrypt_aes_128_gcm(...) -> Result<Vec<u8>>
pub async fn decrypt_aes_256_gcm(...) -> Result<Vec<u8>>

// Hash/KDF
pub async fn sha256(&self, data: &[u8]) -> Result<Vec<u8>>
pub async fn sha384(&self, data: &[u8]) -> Result<Vec<u8>>
pub async fn hkdf_extract(&self, salt: &[u8], ikm: &[u8]) -> Result<Vec<u8>>
pub async fn hkdf_expand(&self, prk: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>>
```

---

## Benefits

1. **Maintainability**: Each module is ~200-400 lines max
2. **Testability**: Tests can target specific modules
3. **Readability**: Clear separation of concerns
4. **Evolution**: Easy to swap implementations
5. **Compliance**: Meets 1,000 line limit with room to spare

---

*Plan created: January 26, 2026*

