# 🧅 Sovereign Onion - TRUE PRIMAL Architecture

**Date**: February 6, 2026  
**Status**: Architecture Defined (fossil record — see notes below)  
**Pattern**: Same as TLS 1.3

> **Wave 138b note (Apr 12, 2026):** This document reflects the original design intent.
> Implementation has since evolved: `sled` was fully removed (Wave 135, SB-03);
> `beardog.sock` paths are now `security.sock` via capability-based discovery;
> crypto deps delegate to the security provider via JSON-RPC IPC.
> See `REMAINING_WORK.md` and `CHANGELOG.md` for current state.

---

## 🎯 Core Principle

**Songbird has ZERO crypto primitives**. All cryptography belongs to BearDog.

```
┌─────────────────────────────────────────────────┐
│              biomeOS                            │
│         (Lifecycle Orchestrator)                │
│                                                 │
│  • Starts/stops primals                        │
│  • Wires CRYPTO_PROVIDER_SOCKET env var       │
│  • Monitors health                             │
│  • Coordinates dependencies                    │
└────────────┬─────────────────┬──────────────────┘
             │                 │
    ┌────────▼───────┐  ┌─────▼──────────┐
    │   BearDog      │  │   Songbird     │
    │  (Security)    │◄─┤   (Network)    │
    │                │  │                │
    │ • Ed25519      │  │ • TCP/UDP      │
    │ • X25519       │  │ • Protocol     │
    │ • ChaCha20     │  │ • State        │
    │ • SHA3         │  │ • Framing      │
    │ • HMAC         │  │ • Logic        │
    └────────────────┘  └────────────────┘
         (owns)              (delegates)
      all crypto           to BearDog
```

---

## 🔄 Crypto Delegation Flow

### Onion Identity Generation

```
Songbird                    BearDog
   │                           │
   │  ed25519_generate()       │
   ├──────────────────────────►│
   │                           │ Generate Ed25519 keypair
   │                           │ Store secret key
   │  {public_key, key_id}     │
   │◄──────────────────────────┤
   │                           │
   │  sha3_256(checksum_input) │
   ├──────────────────────────►│
   │                           │ Hash with SHA3-256
   │  {hash}                   │
   │◄──────────────────────────┤
   │                           │
   │ (Songbird assembles .onion)
   │ base32(pubkey || checksum || version).onion
```

### Session Handshake

```
Songbird                    BearDog
   │                           │
   │  x25519_generate()        │
   ├──────────────────────────►│
   │                           │ Generate X25519 ephemeral
   │  {public_key, key_id}     │
   │◄──────────────────────────┤
   │                           │
   │ (Send public key to peer) │
   │ (Receive peer public key) │
   │                           │
   │  x25519_derive(peer_pk)   │
   ├──────────────────────────►│
   │                           │ ECDH to get shared secret
   │  {shared_secret}          │
   │◄──────────────────────────┤
   │                           │
   │  hmac_sha256(HKDF-Extract)│
   ├──────────────────────────►│
   │  {prk}                    │
   │◄──────────────────────────┤
   │                           │
   │  hmac_sha256(HKDF-Expand) │
   ├──────────────────────────►│
   │  {session_keys}           │
   │◄──────────────────────────┤
```

### Data Encryption

```
Songbird                    BearDog
   │                           │
   │  chacha20_encrypt(data)   │
   ├──────────────────────────►│
   │                           │ Encrypt with ChaCha20-Poly1305
   │  {ciphertext}             │
   │◄──────────────────────────┤
   │                           │
   │ (Send ciphertext)         │
   │ (Receive ciphertext)      │
   │                           │
   │  chacha20_decrypt(data)   │
   ├──────────────────────────►│
   │                           │ Decrypt & verify MAC
   │  {plaintext}              │
   │◄──────────────────────────┤
```

---

## 📋 Responsibility Matrix

### BearDog (Security Primal)

**Owns**:
- ✅ All cryptographic operations
- ✅ Key generation (Ed25519, X25519)
- ✅ Key storage (secret keys never leave BearDog)
- ✅ Signing/verification
- ✅ Encryption/decryption
- ✅ Hashing (Blake3, SHA3-256)
- ✅ MAC (HMAC-SHA256)
- ✅ Audit logging (all crypto operations)

**Dependencies**: ZERO (Pure Rust crypto: RustCrypto)

**API**: 9 JSON-RPC methods over Unix socket

### Songbird (Network Primal)

**Owns**:
- ✅ Network protocols (TCP/UDP)
- ✅ Onion service protocol state machine
- ✅ Connection management
- ✅ Message framing
- ✅ Non-crypto logic (base32 encoding, address assembly)
- ✅ Persistence (Sled for non-secret data)

**Dependencies**: BearDog (for all crypto)

**Crypto**: ZERO direct crypto - 100% delegated to BearDog

### biomeOS (Orchestrator)

**Owns**:
- ✅ Primal lifecycle (start/stop/restart)
- ✅ Dependency resolution (start BearDog before Songbird)
- ✅ Environment wiring (`CRYPTO_PROVIDER_SOCKET`)
- ✅ Health monitoring
- ✅ Failure recovery

**Dependencies**: Both BearDog and Songbird

---

## 🔌 Integration Points

### 1. Discovery

**Environment Variable** (set by biomeOS):
```bash
CRYPTO_PROVIDER_SOCKET=/run/user/1000/biomeos/beardog.sock
```

**Songbird Discovery Code**:
```rust
// crates/songbird-orchestrator/src/crypto/discovery.rs
pub async fn discover_beardog() -> Result<BeardogCryptoClient> {
    // 1. Check env var (set by biomeOS)
    if let Ok(socket) = env::var("CRYPTO_PROVIDER_SOCKET") {
        return BeardogCryptoClient::connect(&socket).await;
    }
    
    // 2. Check XDG paths
    if let Some(xdg) = env::var_os("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(xdg).join("biomeos/beardog.sock");
        if path.exists() {
            return BeardogCryptoClient::connect(&path).await;
        }
    }
    
    // 3. Fallback paths
    // ...
}
```

### 2. Client Interface

**File**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`

```rust
pub struct BeardogCryptoClient {
    socket: UnixStream,
    request_id: AtomicU64,
}

impl BeardogCryptoClient {
    // Ed25519 operations
    pub async fn ed25519_generate(&self, purpose: &str) -> Result<(Vec<u8>, String)>;
    pub async fn ed25519_sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>>;
    pub async fn ed25519_verify(&self, pubkey: &[u8], data: &[u8], sig: &[u8]) -> Result<bool>;
    
    // X25519 operations
    pub async fn x25519_generate(&self, purpose: &str) -> Result<(Vec<u8>, String)>;
    pub async fn x25519_derive(&self, key_id: &str, peer_pk: &[u8]) -> Result<Vec<u8>>;
    
    // ChaCha20-Poly1305 operations
    pub async fn chacha20_encrypt(&self, key: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>>;
    pub async fn chacha20_decrypt(&self, key: &[u8], nonce: &[u8], data: &[u8]) -> Result<Vec<u8>>;
    
    // Hashing operations
    pub async fn sha3_256(&self, data: &[u8], purpose: &str) -> Result<Vec<u8>>;
    pub async fn hmac_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>>;
}
```

### 3. Protocol Messages

**JSON-RPC 2.0 over Unix Socket**:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.sha3_256",
  "params": {
    "data": "base64_encoded_data",
    "purpose": "onion_address_checksum"
  },
  "id": 42
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "base64_encoded_hash_32_bytes"
  },
  "id": 42
}

// Error
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Crypto operation failed"
  },
  "id": 42
}
```

---

## 📦 Dependencies

### Songbird Onion Service

**Before** (incorrect, violates TRUE PRIMAL):
```toml
[dependencies]
ed25519-dalek = "2.1"       # ❌ Should be in BearDog
x25519-dalek = "2.0"        # ❌ Should be in BearDog
chacha20poly1305 = "0.10"   # ❌ Should be in BearDog
sha3 = "0.10"               # ❌ Should be in BearDog
hmac = "0.12"               # ❌ Should be in BearDog
sha2 = "0.10"               # ❌ Should be in BearDog
sled = "0.34"               # ✅ OK (non-crypto persistence)
base32 = "0.5"              # ✅ OK (non-crypto encoding)
```

**After** (correct, TRUE PRIMAL):
```toml
[dependencies]
# NO crypto dependencies!
sled = "0.34"               # ✅ Persistence (non-secret data)
base32 = "0.5"              # ✅ Encoding (non-crypto)
serde = "1.0"               # ✅ Serialization
tokio = "1.35"              # ✅ Async runtime

# Internal deps
songbird-types = { path = "../songbird-types" }
songbird-orchestrator = { path = "../songbird-orchestrator" }  # For BearDog client
```

**Key Point**: Songbird's `songbird-orchestrator` provides `BeardogCryptoClient`, which handles all crypto delegation.

### BearDog Crypto Service

```toml
[dependencies]
# Pure Rust crypto (RustCrypto)
ed25519-dalek = "2.1"       # ✅ Identity keys
x25519-dalek = "2.0"        # ✅ Key exchange
chacha20poly1305 = "0.10"   # ✅ AEAD encryption
sha3 = "0.10"               # ✅ SHA3-256 (NEW for onion)
sha2 = "0.10"               # ✅ SHA2-256
hmac = "0.12"               # ✅ HMAC
blake3 = "1.5"              # ✅ Blake3 hashing
```

**Key Point**: BearDog owns ALL crypto dependencies.

---

## 🧪 Testing

### Unit Tests (Songbird)

**No crypto tests** - only protocol logic:

```rust
// crates/songbird-sovereign-onion/tests/protocol_tests.rs

#[test]
fn test_wire_message_framing() {
    // Test message framing (no crypto)
}

#[test]
fn test_base32_encoding() {
    // Test .onion address encoding (no crypto)
}
```

### Integration Tests (Songbird + Mock BearDog)

```rust
// crates/songbird-sovereign-onion/tests/integration_tests.rs

#[tokio::test]
async fn test_onion_address_generation() {
    // Start mock BearDog
    let beardog = MockBeardog::start().await;
    
    // Generate identity via mock
    let identity = OnionIdentity::generate_via_beardog(&beardog).await.unwrap();
    
    // Verify delegation happened
    assert_eq!(beardog.call_count("ed25519_generate"), 1);
    assert_eq!(beardog.call_count("sha3_256"), 1);
}
```

### E2E Tests (biomeOS)

```rust
// tests/e2e/sovereign_onion_e2e.rs

#[tokio::test]
async fn test_full_onion_service_lifecycle() {
    // 1. biomeOS starts BearDog
    // 2. biomeOS starts Songbird
    // 3. Songbird creates onion service (delegates to BearDog)
    // 4. Test encrypted connection
    // 5. Verify all crypto went through BearDog
}
```

---

## 🔒 Security

### Key Management

**BearDog Owns All Keys**:
- Long-term onion identity keys (Ed25519)
- Ephemeral session keys (X25519)
- Derived session keys (HKDF)

**Songbird Never Sees Secret Keys**:
- Only receives public keys
- Only receives key IDs (references)
- Secret keys never leave BearDog

**Example**:
```rust
// Songbird generates identity
let (public_key, key_id) = beardog.ed25519_generate("onion_service").await?;
//      ^^^^^^^^^^  ^^^^^^
//      Can see     Reference only, secret stays in BearDog
```

### Audit Trail

**All Crypto Operations Logged in BearDog**:
```
[INFO] beardog.crypto.ed25519_generate: purpose=onion_service, key_id=abc123
[INFO] beardog.crypto.sha3_256: purpose=onion_address_checksum, bytes=64
[INFO] beardog.crypto.x25519_generate: purpose=onion_handshake, key_id=xyz789
[INFO] beardog.crypto.chacha20_encrypt: key_id=session_1, bytes=1024
```

**Single Audit Point**: All crypto in one place (BearDog logs)

---

## 📊 Performance

### Latency Breakdown

**Onion Address Generation**:
1. Songbird → security provider: `ed25519_generate()` - ~10 µs IPC + 50 µs crypto = 60 µs
2. Songbird → security provider: `sha3_256()` - ~10 µs IPC + 30 µs crypto = 40 µs
3. Songbird local: base32 encode - ~5 µs

**Total**: ~105 µs (one-time operation)

**Session Handshake**:
1. `x25519_generate()` - 60 µs
2. `x25519_derive()` - 110 µs
3. `hmac_sha256()` x3 (HKDF) - 150 µs
4. Network RTT - ~1-50 ms

**Total**: ~320 µs crypto + network latency

**Data Encryption** (per message):
1. `chacha20_encrypt()` - ~60 µs (1KB)

**Overhead**: ~10-20 µs per call (Unix socket IPC)

---

## 🌟 Benefits

### Architecture

✅ **TRUE PRIMAL Compliance**: Clean separation of concerns  
✅ **Reusability**: BearDog crypto shared by TLS, onion service, future features  
✅ **Single Source of Truth**: All crypto in BearDog  
✅ **Testability**: Mock BearDog for Songbird tests

### Security

✅ **Centralized Audit**: One place to review crypto  
✅ **Key Isolation**: Secret keys never leave BearDog  
✅ **Attack Surface**: Minimal (only BearDog has crypto code)  
✅ **Upgradability**: Swap crypto without touching Songbird

### Maintainability

✅ **Single Crypto Codebase**: Update once, all features benefit  
✅ **Version Control**: BearDog API versioning  
✅ **Testing**: Centralized crypto test suite  
✅ **Documentation**: Single crypto API spec

---

## 🔄 Migration from Phase 1

### Current State (Phase 1)

`songbird-sovereign-onion` has direct crypto dependencies (incorrect).

### Migration Steps

1. **BearDog**: Add `sha3_256` method (~1 hour)
2. **Songbird**: Refactor to use `BeardogCryptoClient` (~4 hours)
3. **Remove**: All direct crypto deps from `songbird-sovereign-onion`
4. **Test**: Integration tests with mock BearDog
5. **Deploy**: biomeOS coordinates BearDog + Songbird

**Timeline**: 1-2 days  
**Result**: TRUE PRIMAL architecture

---

## 📚 Reference

**Pattern Established By**: TLS 1.3 implementation (January 2026)  
**Proof of Concept**: Production-ready TLS via BearDog delegation  
**Documentation**: `docs/architecture/BEARDOG_CRYPTO_API_SPEC.md`

**Same Pattern, Different Use Case**:
- TLS 1.3: BearDog provides crypto for HTTPS
- Onion Service: BearDog provides crypto for .onion addresses

---

**Architecture**: TRUE PRIMAL  
**Status**: Defined & Ready  
**Next**: Implement handoff to BearDog

🐻🐕 + 🎵 = 🧅 **TRUE PRIMAL Onion Service**
