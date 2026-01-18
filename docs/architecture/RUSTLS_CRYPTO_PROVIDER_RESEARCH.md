# rustls CryptoProvider Implementation Research

**Date**: January 18, 2026  
**Status**: 🔬 Research & Preparation for Phase 2  
**Goal**: Understand rustls 0.23 CryptoProvider API for BearDog integration

---

## 🎯 Overview

This document captures research on rustls's CryptoProvider trait to prepare for implementing BearDog crypto delegation.

**Objective**: Replace `ring` crypto provider with `BeardogCryptoProvider` that delegates all crypto operations to BearDog via JSON-RPC.

---

## 📊 Current rustls Usage in Songbird

### Crates Using rustls
1. **songbird-network** (`crates/songbird-network/Cargo.toml`)
   ```toml
   rustls = { version = "0.23", default-features = false, features = ["ring"] }
   ```

2. **songbird-network-federation** (`crates/songbird-network-federation/Cargo.toml`)
   ```toml
   rustls = { version = "0.23", default-features = false, features = ["ring", "logging", "std", "tls12"] }
   ```

3. **songbird-orchestrator** (`crates/songbird-orchestrator/Cargo.toml`)
   ```toml
   rustls = { version = "0.23", default-features = false, features = ["ring", "logging", "std", "tls12"] }
   ```

### Current CryptoProvider Installation

**Location**: `crates/songbird-network-federation/src/tls.rs`

```rust
static CRYPTO_PROVIDER_INIT: Once = Once::new();

fn ensure_crypto_provider() {
    CRYPTO_PROVIDER_INIT.call_once(|| {
        match rustls::crypto::ring::default_provider().install_default() {
            Ok(()) => {
                debug!("✅ Rustls crypto provider (ring) installed");
            }
            Err(_) => {
                debug!("ℹ️  Rustls crypto provider already installed");
            }
        }
    });
}
```

**Key Points**:
- Uses `Once` to ensure single initialization
- Installs `ring` as default provider
- Called before any TLS operations

---

## 🔍 rustls 0.23 CryptoProvider API

### CryptoProvider Trait (Estimated)

Based on rustls 0.23 architecture, the `CryptoProvider` likely includes:

```rust
pub trait CryptoProvider: Send + Sync {
    /// Return the list of supported cipher suites
    fn cipher_suites(&self) -> Vec<SupportedCipherSuite>;
    
    /// Return the list of supported key exchange groups
    fn kx_groups(&self) -> Vec<&'static dyn SupportedKxGroup>;
    
    /// Return signature verification algorithms
    fn signature_verification_algorithms(&self) -> Vec<&'static dyn SignatureVerificationAlgorithm>;
    
    /// Return secure random number generator
    fn secure_random(&self) -> &dyn SecureRandom;
    
    /// Return key provider for certificate private keys
    fn key_provider(&self) -> &dyn KeyProvider;
}
```

### Key Components to Implement

1. **Cipher Suites**
   - TLS 1.3: `TLS_CHACHA20_POLY1305_SHA256`
   - Need to define which suites BearDog supports

2. **Key Exchange Groups**
   - X25519 (ECDHE)
   - Need to implement key generation and derivation

3. **Signature Algorithms**
   - Ed25519
   - Need to implement signing and verification

4. **Secure Random**
   - May delegate to BearDog or use local `rand` crate
   - Decision: Local is probably better (avoid IPC overhead)

5. **Key Provider**
   - Load and use private keys
   - Delegate signing operations to BearDog

---

## 🏗️ Proposed Implementation Structure

### New Crate: `songbird-tls-beardog`

**Purpose**: Implement rustls CryptoProvider using BearDog

**Structure**:
```
crates/songbird-tls-beardog/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Main exports
│   ├── provider.rs               # BeardogCryptoProvider
│   ├── cipher_suites.rs          # TLS cipher suite implementations
│   ├── key_exchange.rs           # X25519 key exchange via BearDog
│   ├── signatures.rs             # Ed25519 signatures via BearDog
│   ├── aead.rs                   # ChaCha20-Poly1305 via BearDog
│   ├── hashing.rs                # Blake3/SHA256 via BearDog
│   ├── random.rs                 # Secure random (local or BearDog)
│   └── tests/
│       ├── unit_tests.rs
│       └── integration_tests.rs
```

### Key Design Decisions

#### 1. Synchronous vs Async Crypto Operations

**Challenge**: rustls crypto operations are synchronous, but our BearDog client is async.

**Options**:
- **A**: Use `tokio::runtime::Handle::block_on()` to bridge async → sync
- **B**: Spawn a background tokio runtime for crypto ops
- **C**: Use synchronous JSON-RPC over Unix sockets

**Recommendation**: Option A (block_on)
- Simplest implementation
- TLS handshake is already blocking by nature
- Performance acceptable (Unix socket IPC is fast)

#### 2. Caching Strategy

**Challenge**: Repeated crypto operations (e.g., same signature verification)

**Solution**: Implement LRU cache for:
- Signature verifications (keyed by public key + message hash)
- HMAC operations (for TLS key derivation)
- NOT for encryption/decryption (security risk)

#### 3. Error Handling

**Challenge**: Map BearDog errors to rustls errors

**Solution**:
```rust
impl From<BeardogCryptoError> for rustls::Error {
    fn from(e: BeardogCryptoError) -> Self {
        match e {
            BeardogCryptoError::SignatureInvalid => {
                rustls::Error::InvalidCertificate(CertificateError::BadSignature)
            }
            BeardogCryptoError::ConnectionFailed => {
                rustls::Error::General("Crypto provider unavailable".into())
            }
            // ... other mappings
        }
    }
}
```

#### 4. Fallback Strategy

**Challenge**: What if BearDog is unavailable?

**Options**:
- **A**: Fail hard (no TLS without BearDog)
- **B**: Fall back to ring temporarily
- **C**: Use feature flags for dual support

**Recommendation**: Option C (feature flags)
```toml
[features]
default = ["beardog-crypto"]
beardog-crypto = ["songbird-tls-beardog"]
ring-crypto = ["rustls/ring"]  # Temporary fallback
```

---

## 📋 Implementation Checklist

### Phase 2A: Core CryptoProvider (~1 week)
- [ ] Create `songbird-tls-beardog` crate
- [ ] Implement `BeardogCryptoProvider` trait
- [ ] Implement cipher suite (TLS_CHACHA20_POLY1305_SHA256)
- [ ] Implement X25519 key exchange
- [ ] Implement Ed25519 signatures
- [ ] Add sync/async bridge (block_on)

### Phase 2B: Integration (~3-4 days)
- [ ] Update `songbird-network-federation/src/tls.rs`
- [ ] Replace `ring::default_provider()` with `beardog::default_provider()`
- [ ] Add feature flags for dual crypto support
- [ ] Update Cargo.toml dependencies

### Phase 2C: Testing (~3-4 days)
- [ ] Unit tests for each crypto operation
- [ ] Integration tests (full TLS handshake)
- [ ] Performance benchmarks
- [ ] Chaos tests (BearDog unavailable, slow, etc.)

### Phase 2D: Optimization (~2-3 days)
- [ ] Implement caching for signature verification
- [ ] Profile IPC overhead
- [ ] Optimize hot paths
- [ ] Document performance characteristics

---

## 🔬 Prototype: BeardogCryptoProvider Skeleton

```rust
// crates/songbird-tls-beardog/src/provider.rs

use rustls::crypto::{CryptoProvider, SecureRandom};
use std::sync::Arc;

pub struct BeardogCryptoProvider {
    /// BearDog crypto client (async)
    beardog_client: Arc<BeardogCryptoClient>,
    
    /// Tokio runtime handle for sync/async bridge
    runtime: tokio::runtime::Handle,
    
    /// Optional caching layer
    cache: Option<Arc<CryptoCache>>,
}

impl BeardogCryptoProvider {
    /// Create a new BearDog crypto provider
    pub fn new(beardog_socket: &str) -> Result<Self, Error> {
        let runtime = tokio::runtime::Handle::try_current()
            .or_else(|_| {
                // Create a new runtime if not in tokio context
                tokio::runtime::Runtime::new()
                    .map(|rt| rt.handle().clone())
            })?;
        
        let beardog_client = Arc::new(BeardogCryptoClient::new(beardog_socket)?);
        
        Ok(Self {
            beardog_client,
            runtime,
            cache: Some(Arc::new(CryptoCache::new(1000))), // 1000 entries
        })
    }
    
    /// Create provider with default BearDog socket discovery
    pub fn with_discovery() -> Result<Self, Error> {
        let socket = discover_beardog_crypto_socket()?;
        Self::new(&socket)
    }
}

impl CryptoProvider for BeardogCryptoProvider {
    fn cipher_suites(&self) -> Vec<SupportedCipherSuite> {
        vec![
            // TLS 1.3 ChaCha20-Poly1305
            cipher_suite::TLS13_CHACHA20_POLY1305_SHA256,
        ]
    }
    
    fn kx_groups(&self) -> Vec<&'static dyn SupportedKxGroup> {
        vec![
            // X25519 key exchange
            &X25519_VIA_BEARDOG,
        ]
    }
    
    fn signature_verification_algorithms(&self) 
        -> Vec<&'static dyn SignatureVerificationAlgorithm> 
    {
        vec![
            // Ed25519 signatures
            &ED25519_VIA_BEARDOG,
        ]
    }
    
    fn secure_random(&self) -> &dyn SecureRandom {
        // Use local rand for performance
        // (avoid IPC overhead for every random byte)
        &LOCAL_SECURE_RANDOM
    }
    
    fn key_provider(&self) -> &dyn KeyProvider {
        // Delegate signing to BearDog
        &self.beardog_key_provider
    }
}

/// Install BearDog as default crypto provider
pub fn install_default() -> Result<(), rustls::Error> {
    let provider = BeardogCryptoProvider::with_discovery()
        .map_err(|e| rustls::Error::General(format!("BearDog unavailable: {}", e)))?;
    
    Arc::new(provider).install_default()
}
```

---

## 🎯 Success Criteria

### Functional
- [ ] TLS 1.3 handshake completes successfully
- [ ] Certificate verification works (Ed25519)
- [ ] Data encryption/decryption works (ChaCha20-Poly1305)
- [ ] All rustls tests pass with BearDog provider

### Performance
- [ ] Handshake latency < 10ms (< 5ms additional vs ring)
- [ ] Throughput > 100 MB/s (acceptable for most use cases)
- [ ] Memory overhead < 1MB per connection

### Reliability
- [ ] Graceful degradation if BearDog slow
- [ ] Clear error messages
- [ ] No panics or crashes
- [ ] Chaos tests pass (BearDog unavailable, etc.)

---

## 📚 References

### rustls Documentation
- **CryptoProvider**: https://docs.rs/rustls/0.23/rustls/crypto/trait.CryptoProvider.html
- **Custom Crypto Providers**: https://docs.rs/rustls/0.23/rustls/crypto/index.html

### BearDog Integration
- **Crypto Client**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`
- **Discovery**: `crates/songbird-orchestrator/src/crypto/discovery.rs`
- **API Spec**: `docs/architecture/BEARDOG_CRYPTO_API_SPEC.md`

### Implementation Examples
- **ring provider**: rustls source code (reference implementation)
- **aws-lc provider**: rustls-aws-lc crate
- **rustcrypto provider**: rustls-rustcrypto (experimental)

---

## 🚀 Next Steps

1. **Research rustls 0.23 API** (deep dive)
   - Read rustls source code
   - Understand CryptoProvider trait fully
   - Study ring implementation as reference

2. **Create `songbird-tls-beardog` crate**
   - Set up crate structure
   - Add dependencies
   - Implement skeleton

3. **Implement Core Operations**
   - Start with Ed25519 signatures
   - Add X25519 key exchange
   - Add ChaCha20-Poly1305 AEAD

4. **Integration & Testing**
   - Replace ring in test environment
   - Run full test suite
   - Fix issues, iterate

---

**Status**: 🔬 **RESEARCH PHASE**  
**Next**: Deep dive into rustls source code  
**Timeline**: Ready to implement once BearDog API available

🦀🔬✨ **Research | Prepare | Execute!** ✨🔬🦀

