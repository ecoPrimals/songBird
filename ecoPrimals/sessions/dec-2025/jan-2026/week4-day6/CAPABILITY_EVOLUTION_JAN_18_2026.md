# 🎯 Capability-Based Crypto Provider Evolution

**Date**: January 18, 2026  
**Status**: 🔄 Evolving to TRUE PRIMAL Principles  
**Philosophy**: Deep Debt Solutions + Capability-Based Discovery

---

## 🎯 Problem Statement

**Current State**: Hardcoded to BearDog
```rust
// Discovery looks for "BEARDOG_CRYPTO_SOCKET"
// Functions know about "BearDog" explicitly
// Acceptable for first solution, but not TRUE PRIMAL
```

**Goal**: Capability-Based Discovery
```rust
// Discovery looks for ANY primal with "crypto" capability
// Functions only know: "I need crypto operations"
// Primal only knows itself, discovers others at runtime
```

---

## 🏗️ TRUE PRIMAL Principles

### Self-Knowledge Only
- **Songbird** only knows itself
- Does NOT know about BearDog, ToadStool, etc.
- Discovers other primals at runtime

### Capability-Based Discovery
- Discovers primals by **what they can do** (capabilities)
- Not by **who they are** (names)
- Example: "I need crypto" → discovers ANY primal offering crypto

### Runtime Discovery
- No hardcoded primal names in production code
- Environment variables guide discovery (orchestration)
- Fallback to automatic discovery if not orchestrated

---

## 🎯 Evolution Strategy

### Phase 1: Abstraction Layer ✅
Create capability-based abstractions:
```rust
pub trait CryptoProvider {
    async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;
    async fn sign_ed25519(&self, message: &[u8], key_id: &str, purpose: &str) -> Result<Vec<u8>>;
    // ... etc
}
```

### Phase 2: Capability Discovery 🔄
Update discovery to be capability-based:
```rust
// OLD (hardcoded BearDog):
pub async fn get_beardog_crypto_socket() -> Result<String>

// NEW (capability-based):
pub async fn discover_crypto_provider() -> Result<CryptoProvider>
```

### Phase 3: Environment Guidance
Use environment variables for orchestration:
```bash
# Orchestrator sets this to guide discovery:
CRYPTO_PROVIDER_SOCKET=/tmp/beardog.sock

# Or capability-based:
CRYPTO_CAPABILITY_SOCKET=/tmp/any-crypto-primal.sock
```

### Phase 4: Automatic Discovery
If not orchestrated, discover automatically:
1. Check common socket paths
2. Query via mDNS/BirdSong for "crypto" capability
3. Select first primal offering crypto
4. Cache for session

---

## 🎯 Implementation Plan

### Step 1: Create Crypto Provider Trait
```rust
// crates/songbird-orchestrator/src/crypto/provider.rs

pub trait CryptoProvider: Send + Sync {
    async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>>;
    async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;
    async fn sign_ed25519(
        &self,
        message: &[u8],
        key_id: &str,
        purpose: &str,
    ) -> Result<Vec<u8>>;
    async fn verify_ed25519(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool>;
    async fn x25519_generate_ephemeral(
        &self,
        purpose: &str,
    ) -> Result<(Vec<u8>, Vec<u8>)>;
    async fn x25519_derive_secret(
        &self,
        our_secret_key: &[u8],
        their_public_key: &[u8],
    ) -> Result<Vec<u8>>;
    async fn chacha20_poly1305_encrypt(
        &self,
        plaintext: &[u8],
        key: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)>;
    async fn chacha20_poly1305_decrypt(
        &self,
        ciphertext: &[u8],
        key: &[u8],
        nonce: &[u8],
        tag: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>>;
}
```

### Step 2: Implement for Unix Socket Provider
```rust
pub struct UnixSocketCryptoProvider {
    socket_path: String,
}

impl CryptoProvider for UnixSocketCryptoProvider {
    // Delegate to our existing beardog_crypto_client functions
    // But don't mention "BearDog" - it's just a Unix socket provider
}
```

### Step 3: Update Discovery
```rust
pub async fn discover_crypto_provider() -> Result<Arc<dyn CryptoProvider>> {
    // 1. Check CRYPTO_PROVIDER_SOCKET (orchestrator guidance)
    if let Ok(socket) = std::env::var("CRYPTO_PROVIDER_SOCKET") {
        return Ok(Arc::new(UnixSocketCryptoProvider::new(socket)));
    }

    // 2. Check common socket paths
    for path in &["/tmp/crypto.sock", "/tmp/beardog.sock", "/var/run/crypto.sock"] {
        if std::path::Path::new(path).exists() {
            return Ok(Arc::new(UnixSocketCryptoProvider::new(path.to_string())));
        }
    }

    // 3. Query via mDNS/BirdSong for "crypto" capability
    // (Future work)

    Err(anyhow!("No crypto provider found"))
}
```

### Step 4: Update Client Functions
```rust
// OLD:
pub async fn blake3_hash(socket_path: &str, data: &[u8]) -> Result<Vec<u8>>

// NEW:
pub async fn blake3_hash(provider: &dyn CryptoProvider, data: &[u8]) -> Result<Vec<u8>> {
    provider.blake3_hash(data).await
}

// Or even better - just use the provider directly:
let provider = discover_crypto_provider().await?;
let hash = provider.blake3_hash(data).await?;
```

---

## 🎯 Benefits

### 1. TRUE PRIMAL Principles ✅
- Songbird only knows itself
- Discovers crypto capability at runtime
- No hardcoded primal names

### 2. Flexibility ✅
- Can use BearDog (current)
- Can use ANY primal offering crypto
- Can add new crypto providers easily

### 3. Testability ✅
- Can mock CryptoProvider for tests
- No need to start real BearDog for unit tests
- Integration tests can use real provider

### 4. Future-Proof ✅
- When rustls needs crypto, uses same provider
- If we add more crypto operations, extend trait
- If we switch crypto backends, transparent

---

## 🎯 Migration Path

### Current Code (Acceptable First Solution)
```rust
// Hardcoded BearDog discovery
let socket = discover_beardog_crypto_socket().await?;
let hash = blake3_hash(&socket, data).await?;
```

### Evolved Code (TRUE PRIMAL)
```rust
// Capability-based discovery
let provider = discover_crypto_provider().await?;
let hash = provider.blake3_hash(data).await?;
```

### Benefits of Evolution
- ✅ No breaking changes to BearDog
- ✅ Still works with existing BearDog
- ✅ But now capability-based, not hardcoded
- ✅ Follows TRUE PRIMAL principles

---

## 🎯 Testing Strategy

### Unit Tests (Mock Provider)
```rust
struct MockCryptoProvider;

impl CryptoProvider for MockCryptoProvider {
    async fn blake3_hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32]) // Mock hash
    }
    // ... other mocks
}

#[tokio::test]
async fn test_with_mock_provider() {
    let provider = MockCryptoProvider;
    let hash = provider.blake3_hash(b"test").await.unwrap();
    assert_eq!(hash.len(), 32);
}
```

### Integration Tests (Real Provider)
```rust
#[tokio::test]
#[ignore]
async fn test_with_real_provider() {
    let provider = discover_crypto_provider().await.unwrap();
    let hash = provider.blake3_hash(b"test").await.unwrap();
    assert_eq!(hash.len(), 32); // Real hash from real provider
}
```

---

## 🎯 Success Criteria

- ✅ CryptoProvider trait defined
- ✅ UnixSocketCryptoProvider implemented
- ✅ Discovery is capability-based (not name-based)
- ✅ All 5 tests still passing
- ✅ No hardcoded "BearDog" in abstractions
- ✅ TRUE PRIMAL principles followed

---

**Status**: Ready to implement!  
**Timeline**: ~2-3 hours  
**Risk**: Low (non-breaking evolution)

🦀🐻🐕✨ Capability-Based | TRUE PRIMAL | Deep Debt! ✨🐕🐻🦀

