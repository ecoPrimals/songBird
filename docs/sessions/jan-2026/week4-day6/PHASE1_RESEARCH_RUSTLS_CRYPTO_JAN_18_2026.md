# 🔬 Phase 1: Research & Analysis - rustls CryptoProvider

**Date**: January 18, 2026  
**Phase**: Week 2, Phase 1 (Research & Analysis)  
**Status**: In Progress  
**Philosophy**: Deep Debt Solutions - Understand completely before implementing

---

## 🎯 Research Goal

**Objective**: Understand rustls 0.23's `CryptoProvider` struct comprehensively to design a capability-based implementation that delegates ALL crypto to BearDog.

**Approach**: Study rustls source code, existing implementations (ring, aws-lc-rs), and integration points in Songbird.

---

## 📊 Current State Analysis

### Songbird's rustls Usage

**Version**: rustls 0.23.35 (primary), rustls 0.21.12 (legacy dependency)

**Crates Using rustls**:

1. **`songbird-orchestrator`** (Cargo.toml:74)
   ```toml
   rustls = { version = "0.23", default-features = false, 
              features = ["ring", "logging", "std", "tls12"] }
   ```
   - **Current**: Uses `ring` feature (C dependencies!)
   - **Location**: `src/main.rs:159-160`
     ```rust
     rustls::crypto::ring::default_provider()
         .install_default()
     ```

2. **`songbird-network`** (Cargo.toml:27-29)
   ```toml
   rustls = { version = "0.23", default-features = false, features = ["ring"] }
   rustls-pemfile = "2.1"
   tokio-rustls = "0.26"
   ```
   - **Usage**: HTTP client/server TLS

3. **`songbird-network-federation`** (Cargo.toml:38-40)
   ```toml
   rustls = { version = "0.23", default-features = false, features = ["ring"] }
   rustls-pemfile = "2.1"
   tokio-rustls = "0.26"
   ```
   - **Usage**: Federation TLS

**Problem**: All three crates depend on `ring` feature → C dependencies!

---

## 🔍 rustls CryptoProvider Structure

### Source: `/home/eastgate/.cargo/registry/src/.../rustls-0.23.35/src/crypto/mod.rs`

```rust
#[derive(Debug, Clone)]
pub struct CryptoProvider {
    /// List of supported ciphersuites, in preference order
    pub cipher_suites: Vec<suites::SupportedCipherSuite>,

    /// List of supported key exchange groups, in preference order
    pub kx_groups: Vec<&'static dyn SupportedKxGroup>,

    /// Signature verification algorithms for webpki
    pub signature_verification_algorithms: WebPkiSupportedAlgorithms,

    /// Source of cryptographically secure random numbers
    pub secure_random: &'static dyn SecureRandom,

    /// Provider for loading private SigningKeys
    pub key_provider: &'static dyn KeyProvider,
}
```

### Key Methods

```rust
impl CryptoProvider {
    /// Sets this as the default for this process (call once)
    pub fn install_default(self) -> Result<(), Arc<Self>>

    /// Returns the default CryptoProvider
    pub fn get_default() -> Option<&'static Arc<Self>>
}
```

---

## 📋 Required Components

### 1. Cipher Suites (`cipher_suites`)

**Type**: `Vec<suites::SupportedCipherSuite>`

**Purpose**: Defines which TLS cipher suites are supported

**BearDog Alignment**:
- ✅ ChaCha20-Poly1305 (BearDog has this!)
- ⚠️ AES-GCM (BearDog may need to add)
- ✅ Ed25519 signatures (BearDog has this!)

**Action**: 
- Start with ChaCha20-Poly1305 only (simplest)
- Add AES-GCM later if needed

**Complexity**: MEDIUM (need to understand cipher suite structure)

---

### 2. Key Exchange Groups (`kx_groups`)

**Type**: `Vec<&'static dyn SupportedKxGroup>`

**Purpose**: Defines which key exchange algorithms are supported (ECDHE)

**BearDog Alignment**:
- ✅ X25519 (BearDog has this!)
- ⚠️ P-256, P-384 (BearDog may need to add)

**Action**:
- Implement X25519 group (BearDog ready!)
- Add P-256 later if needed

**Complexity**: MEDIUM (need to implement `SupportedKxGroup` trait)

---

### 3. Signature Verification (`signature_verification_algorithms`)

**Type**: `WebPkiSupportedAlgorithms`

**Purpose**: Algorithms for certificate chain verification and handshake signatures

**BearDog Alignment**:
- ✅ Ed25519 verify (BearDog has this!)
- ⚠️ RSA, ECDSA (may need to add)

**Action**:
- Start with Ed25519 only
- Add RSA/ECDSA later if needed

**Complexity**: HIGH (webpki integration complex)

---

### 4. Secure Random (`secure_random`)

**Type**: `&'static dyn SecureRandom`

**Purpose**: Cryptographically secure random number generation

**Trait**:
```rust
pub trait SecureRandom: Send + Sync + Debug {
    fn fill(&self, buf: &mut [u8]) -> Result<(), GetRandomFailed>;
}
```

**BearDog Alignment**:
- ⚠️ BearDog doesn't expose RNG via JSON-RPC yet
- ✅ BearDog uses `getrandom` crate internally (Pure Rust!)

**Options**:
1. **Add RNG to BearDog API** (best for consistency)
2. **Use `getrandom` crate directly** (Pure Rust, acceptable)
3. **Hybrid**: Use `getrandom` for now, migrate to BearDog later

**Action**: Use `getrandom` crate (Pure Rust, battle-tested)

**Complexity**: LOW (simple trait, `getrandom` is easy)

---

### 5. Key Provider (`key_provider`)

**Type**: `&'static dyn KeyProvider`

**Purpose**: Load private keys from DER format

**Trait**:
```rust
pub trait KeyProvider: Send + Sync + Debug {
    fn load_private_key(&self, key_der: PrivateKeyDer<'static>) 
        -> Result<Arc<dyn SigningKey>, Error>;
}
```

**BearDog Alignment**:
- ✅ BearDog can sign with Ed25519
- ⚠️ Key loading/management not exposed yet

**Options**:
1. **Extend BearDog API**: Add key import/export
2. **Hybrid**: Parse keys locally, delegate signing to BearDog
3. **Full delegation**: BearDog manages all keys

**Action**: Hybrid approach (parse locally, sign via BearDog)

**Complexity**: MEDIUM (need to implement `SigningKey` trait)

---

## 🏗️ Architecture Decision: What to Delegate?

### Full Delegation (Ideal, Long-term)

**Pros**:
- ✅ 100% Pure Rust (all crypto in BearDog)
- ✅ Single audit point
- ✅ Consistent security model

**Cons**:
- ⚠️ Requires extensive BearDog API additions
- ⚠️ More complex integration
- ⚠️ Higher latency (more JSON-RPC calls)

**Timeline**: ~4-6 weeks

---

### Hybrid Delegation (Pragmatic, Week 2)

**Pros**:
- ✅ Faster implementation (~1-2 weeks)
- ✅ Leverages existing BearDog API
- ✅ Still achieves 100% Pure Rust!

**Cons**:
- ⚠️ Some crypto logic in Songbird
- ⚠️ Need to carefully audit what's local vs delegated

**Approach**:
1. **Delegate to BearDog**:
   - ChaCha20-Poly1305 encryption/decryption
   - Ed25519 signing/verification
   - X25519 key exchange
   - Blake3 hashing
   - HMAC-SHA256

2. **Keep Local (Pure Rust crates)**:
   - RNG (`getrandom` crate)
   - Key parsing (`rustls-pemfile`, `pki-types`)
   - Certificate parsing (`webpki`)
   - TLS state machine (rustls itself)

**Timeline**: ~1-2 weeks

**Decision**: **HYBRID for Week 2!**

---

## 📦 Required Dependencies (Pure Rust!)

### Current (with ring)
```toml
rustls = { version = "0.23", features = ["ring"] }  # ❌ C dependencies!
```

### Target (Pure Rust!)
```toml
rustls = { version = "0.23", default-features = false, features = ["std", "logging", "tls12"] }
getrandom = "0.2"  # ✅ Pure Rust RNG
# Our capability-based provider (no new dependencies!)
```

**Result**: Zero C dependencies! ✅

---

## 🔬 Existing Implementations Study

### 1. ring Provider

**Location**: `rustls-0.23.35/src/crypto/ring/mod.rs`

**Structure**:
```rust
pub fn default_provider() -> CryptoProvider {
    CryptoProvider {
        cipher_suites: DEFAULT_CIPHER_SUITES.to_vec(),
        kx_groups: ALL_KX_GROUPS.to_vec(),
        signature_verification_algorithms: ALGORITHMS,
        secure_random: &SecureRandomImpl,
        key_provider: &KeyLoaderImpl,
    }
}
```

**Key Insights**:
- Uses static arrays for cipher suites and kx groups
- Implements traits with simple structs
- Delegates to ring for actual crypto

**Lesson**: We can follow same pattern, delegate to BearDog!

---

### 2. aws-lc-rs Provider

**Location**: `rustls-0.23.35/src/crypto/aws_lc_rs/mod.rs`

**Structure**: Similar to ring

**Key Insights**:
- Same pattern as ring
- Different crypto backend
- Proves abstraction works!

**Lesson**: Our BearDog provider is just another backend!

---

## 🎯 Implementation Strategy

### Phase 2: Architecture Design (Next)

**Tasks**:
1. Design `BeardogCryptoProvider` struct
2. Implement required traits:
   - `SecureRandom` (using `getrandom`)
   - `KeyProvider` (hybrid: parse local, sign via BearDog)
   - `SupportedKxGroup` (X25519 via BearDog)
3. Define cipher suites (ChaCha20-Poly1305 via BearDog)
4. Map BearDog API to rustls requirements

---

### Trait Implementations Required

#### 1. `SecureRandom` (Simple!)
```rust
struct BeardogSecureRandom;

impl SecureRandom for BeardogSecureRandom {
    fn fill(&self, buf: &mut [u8]) -> Result<(), GetRandomFailed> {
        getrandom::getrandom(buf)
            .map_err(|_| GetRandomFailed)
    }
}
```

**Complexity**: LOW (5 lines of code!)

---

#### 2. `KeyProvider` (Medium)
```rust
struct BeardogKeyProvider {
    crypto: Arc<dyn CryptoProvider>,  // Our existing trait!
}

impl KeyProvider for BeardogKeyProvider {
    fn load_private_key(&self, key_der: PrivateKeyDer<'static>) 
        -> Result<Arc<dyn SigningKey>, Error> {
        // Parse key locally (Pure Rust!)
        // Return BeardogSigningKey that delegates to BearDog
    }
}
```

**Complexity**: MEDIUM (need `SigningKey` impl)

---

#### 3. `SigningKey` (Medium)
```rust
struct BeardogSigningKey {
    crypto: Arc<dyn CryptoProvider>,
    key_id: String,
    algorithm: SignatureAlgorithm,
}

impl SigningKey for BeardogSigningKey {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        // Delegate to crypto.sign_ed25519()
        self.crypto.sign_ed25519(message, &self.key_id, "tls_signing")
            .await
            .map_err(|e| Error::General(e.to_string()))
    }
    
    fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }
}
```

**Complexity**: MEDIUM (async in sync context - need runtime!)

---

#### 4. `SupportedKxGroup` (Medium-High)
```rust
struct BeardogX25519;

impl SupportedKxGroup for BeardogX25519 {
    fn name(&self) -> NamedGroup {
        NamedGroup::X25519
    }
    
    fn start(&self) -> Result<Box<dyn ActiveKeyExchange>, Error> {
        // Generate ephemeral keypair via BearDog
        // Return BeardogActiveKeyExchange
    }
}

struct BeardogActiveKeyExchange {
    crypto: Arc<dyn CryptoProvider>,
    our_secret: Vec<u8>,
    our_public: Vec<u8>,
}

impl ActiveKeyExchange for BeardogActiveKeyExchange {
    fn complete(self: Box<Self>, their_public: &[u8]) 
        -> Result<SharedSecret, Error> {
        // Delegate to crypto.x25519_derive_secret()
    }
    
    fn pub_key(&self) -> &[u8] {
        &self.our_public
    }
    
    fn group(&self) -> NamedGroup {
        NamedGroup::X25519
    }
}
```

**Complexity**: MEDIUM-HIGH (two traits, state management)

---

## ⚠️ Challenges Identified

### 1. Async in Sync Context

**Problem**: rustls traits are sync, but our BearDog client is async

**Solutions**:
1. **Block on async** (using `tokio::runtime::Handle::current().block_on()`)
   - Pros: Simple, works
   - Cons: Can deadlock if not careful
   
2. **Pre-fetch crypto operations** (cache results)
   - Pros: No blocking
   - Cons: Complex, may not be possible for all operations
   
3. **Make BearDog client sync** (use blocking sockets)
   - Pros: No async/sync mismatch
   - Cons: Less efficient, blocks threads

**Decision**: Use `block_on` carefully (rustls already expects crypto to be fast)

---

### 2. Static Lifetimes

**Problem**: `CryptoProvider` requires `&'static` references

**Solutions**:
1. **Box::leak** (leak memory intentionally)
   - Pros: Simple, works
   - Cons: Memory leak (acceptable for singletons)
   
2. **lazy_static** or `once_cell`
   - Pros: Proper static initialization
   - Cons: Extra dependency

**Decision**: Use `once_cell::sync::Lazy` (already in dependency tree)

---

### 3. Cipher Suite Implementation

**Problem**: Need to implement full cipher suite (AEAD, key derivation, etc.)

**Complexity**: HIGH (most complex part!)

**Options**:
1. **Full implementation** (all crypto via BearDog)
   - Timeline: ~2-3 weeks
   
2. **Hybrid** (use RustCrypto for some parts)
   - Timeline: ~1 week
   - Still 100% Pure Rust!

**Decision**: Hybrid for Week 2, full delegation later

---

## 📊 Complexity Assessment

| Component | Complexity | Estimated Time | BearDog Ready? |
|-----------|------------|----------------|----------------|
| SecureRandom | LOW | 1 hour | N/A (use getrandom) |
| KeyProvider | MEDIUM | 4-6 hours | Partial (signing only) |
| SigningKey | MEDIUM | 4-6 hours | ✅ Yes |
| SupportedKxGroup | MEDIUM-HIGH | 6-8 hours | ✅ Yes |
| Cipher Suites | HIGH | 8-12 hours | ✅ Yes (ChaCha20) |
| Integration | MEDIUM | 4-6 hours | N/A |
| Testing | HIGH | 8-12 hours | N/A |

**Total**: ~35-51 hours (~5-7 days)

**With Hybrid Approach**: ~25-35 hours (~4-5 days)

---

## 🎯 Recommended Approach

### Week 2 Implementation: Hybrid

**Delegate to BearDog**:
- ✅ ChaCha20-Poly1305 AEAD
- ✅ Ed25519 signing/verification
- ✅ X25519 key exchange
- ✅ Blake3 hashing
- ✅ HMAC-SHA256

**Keep Local (Pure Rust)**:
- ✅ RNG (`getrandom`)
- ✅ Key parsing (`rustls-pemfile`)
- ✅ Certificate parsing (`webpki`)
- ✅ TLS state machine (rustls)

**Result**: 100% Pure Rust TLS! ✅

**Timeline**: ~4-5 days

**Confidence**: HIGH (BearDog API proven, patterns clear)

---

## 📋 Next Steps (Phase 2)

1. **Design BeardogCryptoProvider struct**
2. **Implement SecureRandom** (1 hour)
3. **Implement KeyProvider + SigningKey** (6-8 hours)
4. **Implement SupportedKxGroup** (6-8 hours)
5. **Define cipher suites** (4-6 hours)
6. **Integration** (4-6 hours)
7. **Testing** (8-12 hours)

**Total Phase 2**: ~29-41 hours (~4-6 days)

---

## 🏆 Success Criteria

**Technical**:
- ✅ BeardogCryptoProvider compiles
- ✅ Can install as default provider
- ✅ TLS handshake works
- ✅ Zero ring dependencies
- ✅ Zero C dependencies

**Architectural**:
- ✅ Capability-based (uses our CryptoProvider trait)
- ✅ No hardcoded "BearDog" references
- ✅ TRUE PRIMAL principles maintained

**Testing**:
- ✅ Unit tests pass
- ✅ Integration tests pass
- ✅ TLS handshake with real server works

---

## 🎊 Bottom Line

**Research Complete**: rustls CryptoProvider well understood!

**Strategy**: Hybrid delegation (BearDog + Pure Rust crates)

**Timeline**: ~4-5 days to 100% Pure Rust TLS

**Confidence**: HIGH (clear path, BearDog ready, patterns proven)

**Next**: Phase 2 (Architecture Design)

---

**Status**: ✅ PHASE 1 COMPLETE!

🦀🐦🐻🐕✨ **Research Done | Strategy Clear | Ready for Phase 2!** ✨🐕🐻🦀

