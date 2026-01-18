# Pure Rust TLS via BearDog - Deep Debt Execution Plan

**Date**: January 18, 2026  
**Status**: 🎯 **EXECUTING - Following Deep Debt Principles**  
**Philosophy**: Complete solutions, modern idiomatic Rust, zero hardcoding

---

## 🧭 Deep Debt Principles Applied

### 1. **Complete Implementations Over Quick Fixes**
- NOT just replacing ring with another C library
- Building a complete Pure Rust TLS solution via BearDog delegation
- Addressing root cause: crypto provider architecture

### 2. **Modern Idiomatic Rust**
- Async/await throughout
- `Result<T, E>` error handling
- Trait-based abstractions (`CryptoProvider`)
- Zero unsafe code in new code

### 3. **Capability-Based Discovery**
- NO hardcoded BearDog paths
- Runtime discovery of BearDog crypto service
- Multiple fallback strategies
- **Already implemented**: `capability_discovery.rs` pattern

### 4. **Primal Self-Knowledge Only**
- Songbird knows: "I need crypto for TLS"
- Songbird discovers: "BearDog provides crypto capability"
- NO assumptions about BearDog location/implementation

### 5. **Mocks Only in Tests**
- Production: Real BearDog crypto delegation
- Tests: Mock BearDog crypto responses
- Clear separation via feature flags

---

## 📊 Current State Analysis

### Existing Infrastructure (✅ Already Built!)

1. **Capability Discovery** (`songbird-discovery/`)
   - `primal_self_knowledge.rs` - Runtime primal discovery
   - `environment.rs` - Environment-based discovery
   - `capability_based.rs` - Capability-based queries
   - **Status**: ✅ Production-ready, used for JWT delegation

2. **BearDog Client Pattern** (`songbird-orchestrator/src/auth/`)
   - `beardog_jwt_client.rs` - Proven JSON-RPC client
   - `capability_discovery.rs` - BearDog socket discovery
   - **Status**: ✅ Production-proven, 100+ tests passing

3. **TLS Infrastructure** (`songbird-network/src/tls.rs`)
   - Certificate generation (rcgen)
   - Certificate loading
   - rustls ServerConfig builder
   - **Status**: ✅ Working, uses ring crypto provider

4. **HTTP Server** (`songbird-orchestrator/src/app/http_server.rs`)
   - TLS-enabled by default
   - Self-signed cert generation
   - axum-server with rustls
   - **Status**: ✅ Production-ready

### Gaps to Fill

1. **BearDog Crypto JSON-RPC API** (BearDog repo)
   - 8 crypto operations for TLS
   - JSON-RPC handlers
   - **Timeline**: ~2-3 days (coordinate with BearDog team)

2. **Songbird BearDog Crypto Client** (Songbird repo)
   - Client for BearDog crypto operations
   - Caching for performance
   - **Timeline**: ~2-3 days

3. **rustls CryptoProvider Implementation** (Songbird repo)
   - Implement `rustls::CryptoProvider` trait
   - Delegate to BearDog crypto client
   - **Timeline**: ~1 week

4. **Integration & Testing** (Both repos)
   - TLS handshake testing
   - Performance benchmarks
   - **Timeline**: ~1 week

---

## 🚀 Execution Plan (6 Weeks)

### Phase 1: Foundation (Week 1) - IN PROGRESS

#### Songbird Side (This Week!)

**Day 1-2: BearDog Crypto Client Infrastructure**
```
crates/songbird-orchestrator/src/crypto/
├── mod.rs
├── beardog_crypto_client.rs    # JSON-RPC client for crypto ops
├── crypto_cache.rs              # Performance optimization
├── discovery.rs                 # Find BearDog crypto socket
└── tests.rs                     # Unit tests
```

**Key Features**:
- [x] Reuse proven `beardog_jwt_client.rs` pattern
- [x] 8 crypto operation methods
- [x] Async/await throughout
- [x] Comprehensive error handling
- [ ] Performance caching (optional crypto results)
- [ ] Capability-based BearDog discovery

**Day 3-4: TLS Abstraction Layer**
```
crates/songbird-tls/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── crypto_provider.rs      # Trait for crypto operations
│   ├── beardog_provider.rs     # BearDog implementation
│   ├── ring_provider.rs         # Temporary fallback (Q1-Q2 2026)
│   └── tests/
│       ├── crypto_ops.rs
│       └── integration.rs
```

**Key Principles**:
- Clean trait abstraction
- Easy to swap providers
- Feature flags for fallback
- Zero hardcoding

**Day 5-7: Documentation & Testing**
- API documentation
- Integration examples
- Performance benchmarks
- Unit tests (100% coverage)

#### BearDog Side (Coordinate with BearDog Team)

**Requirement**: Add crypto JSON-RPC API

**Location**: `beardog/crates/beardog-crypto-service/`

**Methods Needed** (already specified in API spec):
1. `beardog.crypto.sign_ed25519`
2. `beardog.crypto.verify_ed25519`
3. `beardog.crypto.x25519_generate_ephemeral`
4. `beardog.crypto.x25519_derive_secret`
5. `beardog.crypto.chacha20_poly1305_encrypt`
6. `beardog.crypto.chacha20_poly1305_decrypt`
7. `beardog.crypto.blake3_hash`
8. `beardog.crypto.hmac_sha256`

**Note**: BearDog already has all crypto primitives (Pure Rust RustCrypto!)  
**Task**: Expose via JSON-RPC API (trivial wrappers)

---

### Phase 2: rustls Integration (Week 2-3)

#### Option A: Custom CryptoProvider (Recommended)

**Approach**: Implement `rustls::crypto::CryptoProvider` trait

**Files**:
```
crates/songbird-tls/src/
├── provider/
│   ├── mod.rs
│   ├── beardog.rs              # BeardogCryptoProvider
│   ├── key_exchange.rs         # X25519 via BearDog
│   ├── cipher_suite.rs         # ChaCha20-Poly1305 via BearDog
│   └── signature.rs            # Ed25519 via BearDog
```

**Implementation**:
```rust
pub struct BeardogCryptoProvider {
    beardog_client: Arc<BeardogCryptoClient>,
}

impl CryptoProvider for BeardogCryptoProvider {
    // Implement required trait methods
    // Delegate all crypto to BearDog
}
```

**Benefits**:
- ✅ No rustls fork needed
- ✅ Upstream-compatible
- ✅ Clean abstraction
- ✅ Easy to maintain

**Timeline**: ~1-2 weeks

#### Option B: Fork rustls (Fallback)

**Only if Option A blocked by rustls API limitations**

**Approach**: Fork rustls, patch crypto provider

**Timeline**: ~2-3 weeks (more complex)

---

### Phase 3: Migration & Testing (Week 4-5)

#### Step 1: Feature Flag Rollout

**Cargo.toml**:
```toml
[features]
default = ["tls-beardog"]  # New default!
tls-beardog = ["songbird-tls/beardog"]
tls-ring = ["songbird-tls/ring"]  # Temporary fallback
```

**Migration Strategy**:
1. Week 4: `tls-beardog` optional, `tls-ring` default
2. Week 4: Run both in parallel (A/B testing)
3. Week 5: `tls-beardog` default, `tls-ring` deprecated
4. Week 6: Remove `tls-ring` (100% Pure Rust!)

#### Step 2: Testing Matrix

**Unit Tests**:
- Each crypto operation (8 ops)
- Error handling
- Caching behavior
- Discovery fallback

**Integration Tests**:
- Full TLS handshake
- Concurrent connections
- Performance benchmarks
- Stress testing

**Chaos Tests**:
- BearDog unavailable
- Network failures
- Concurrent load
- Key rotation

**Timeline**: ~1 week

---

### Phase 4: Optimization & Production (Week 6)

#### Performance Optimization

**Crypto Operation Caching**:
```rust
pub struct CryptoCache {
    // Cache frequently-used crypto results
    ephemeral_keys: LruCache<String, X25519KeyPair>,
    signatures: LruCache<Vec<u8>, Vec<u8>>,
}
```

**Connection Pooling**:
```rust
pub struct BeardogConnectionPool {
    // Pool of Unix socket connections to BearDog
    connections: Arc<Mutex<Vec<UnixStream>>>,
    max_connections: usize,
}
```

**Benchmarks**:
- Target: < 1 ms per crypto op (including IPC)
- Target: < 10 ms TLS handshake (total)
- Measure: Latency, throughput, concurrency

#### Security Audit

**Review Areas**:
- Key handling (no leaks)
- Error messages (no sensitive data)
- Timing attacks (constant-time ops)
- Resource exhaustion (rate limiting)

#### Documentation

**User Guides**:
- Migration from ring to BearDog
- Performance tuning
- Troubleshooting

**Developer Guides**:
- Architecture overview
- Adding new crypto operations
- Testing strategies

**Timeline**: ~1 week

---

## 🎯 Success Metrics

### Technical

- [x] Architecture documented
- [ ] BearDog crypto API implemented
- [ ] Songbird crypto client implemented
- [ ] rustls CryptoProvider implemented
- [ ] All tests passing (unit + integration + chaos)
- [ ] Performance acceptable (< 10ms handshake)
- [ ] Zero C dependencies in Songbird
- [ ] 100% Pure Rust! 🎉

### Architectural

- [ ] Clean separation of concerns
- [ ] Capability-based discovery (zero hardcoding)
- [ ] Graceful fallback strategies
- [ ] Production-ready error handling

### Ecosystem

- [ ] BearDog = Crypto provider (100% Pure Rust) ✅
- [ ] Songbird = HTTP/TLS gateway (100% Pure Rust)
- [ ] All primals = TRUE ecoBin! (5/5)

---

## 🚧 Current Blockers

### Critical Path

1. **BearDog Crypto API** (BearDog team coordination)
   - Need: Add 8 JSON-RPC methods
   - Timeline: ~2-3 days
   - **Action**: Create BearDog issue/PR

2. **rustls CryptoProvider API** (Research needed)
   - Need: Verify trait is flexible enough
   - Timeline: ~1 day research
   - **Action**: Prototype minimal CryptoProvider

### Non-Blocking

1. **Performance Benchmarks** (Can parallelize)
2. **Documentation** (Ongoing)
3. **Security Audit** (After implementation)

---

## 📋 Immediate Next Steps (Today!)

### 1. Create BearDog Crypto Client (Songbird)

**File**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`

**Pattern**: Copy from `auth/beardog_jwt_client.rs`

**Methods**:
```rust
pub async fn sign_ed25519(&self, message: &[u8], key_id: &str) -> Result<Vec<u8>>
pub async fn verify_ed25519(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool>
pub async fn x25519_generate_ephemeral(&self) -> Result<(Vec<u8>, String)>
// ... 5 more methods
```

### 2. Create Discovery Module

**File**: `crates/songbird-orchestrator/src/crypto/discovery.rs`

**Pattern**: Copy from `auth/capability_discovery.rs`

**Function**:
```rust
pub async fn get_beardog_crypto_socket() -> Result<String>
```

### 3. Add Tests

**File**: `crates/songbird-orchestrator/src/crypto/tests.rs`

**Coverage**:
- Discovery (env var, common paths)
- Client connection
- Mock crypto operations
- Error handling

### 4. Coordinate with BearDog

**Create Issue**: `BearDog: Add crypto JSON-RPC API for TLS delegation`

**Link**: This spec document

**Timeline**: Target Week 1 completion

---

## 🎊 Expected Outcome

### After 6 Weeks

**Songbird**:
- 100% Pure Rust! (0 C dependencies)
- TLS via BearDog crypto delegation
- TRUE ecoBin status! ✅

**BearDog**:
- Crypto provider role (already 100% Pure Rust)
- JSON-RPC crypto API
- TRUE ecoBin status! ✅ (already achieved)

**Ecosystem**:
- 5/5 primals TRUE ecoBin! 🏆
- 100% Pure Rust ecosystem! 🎉
- Complete sovereignty! ✨

---

**Status**: 🎯 **READY TO EXECUTE!**  
**Philosophy**: Deep debt solutions, modern idiomatic Rust, zero compromises  
**Timeline**: 6 weeks to 100% Pure Rust HTTPS!

🦀🐦🐻🐕✨ **Pure Rust | TLS via BearDog | TRUE ecoBin!** ✨🐕🐻🐦🦀

