# 🐦 Songbird TRUE PRIMAL Refactoring Handoff

**Date**: February 6, 2026  
**For**: Songbird Team  
**From**: biomeOS Team  
**Priority**: HIGH - Required before deployment validation

---

## Executive Summary

The `songbird-sovereign-onion` crate currently violates the **TRUE PRIMAL** pattern by performing cryptographic operations directly instead of delegating to BearDog. This handoff provides exact code changes needed to complete the refactoring.

### What Needs to Change

| File | Direct Crypto | Should Delegate To |
|------|---------------|-------------------|
| `keys.rs` | Ed25519, X25519, HMAC-SHA256 | `BeardogCryptoClient` |
| `address.rs` | SHA3-256 | `BeardogCryptoClient` |
| `crypto.rs` | ChaCha20-Poly1305 | `BeardogCryptoClient` |

### Why This Matters

1. **Security Audit Surface**: Single crypto implementation in BearDog
2. **TRUE PRIMAL**: Primals only have self-knowledge, discover capabilities at runtime
3. **Consistency**: Same pattern as TLS 1.3 (proven in production)
4. **Testing**: Easier to mock crypto in tests

---

## Files Already Completed ✅

### `src/beardog_crypto.rs` (NEW - Ready to Use)

The `BeardogCryptoClient` has been created with full API:

```rust
use songbird_sovereign_onion::BeardogCryptoClient;

// Create client from environment
let client = BeardogCryptoClient::from_env()?;

// Ed25519
let keypair = client.ed25519_generate_keypair()?;
let signature = client.ed25519_sign(&secret, &message)?;
let valid = client.ed25519_verify(&pubkey, &message, &signature)?;

// X25519
let ephemeral = client.x25519_generate_ephemeral()?;
let shared = client.x25519_derive_secret(&our_secret, &their_public)?;

// ChaCha20-Poly1305
let ciphertext = client.chacha20_poly1305_encrypt(&key, &nonce, &plaintext)?;
let plaintext = client.chacha20_poly1305_decrypt(&key, &nonce, &ciphertext)?;

// SHA3-256
let hash = client.sha3_256(&data)?;

// HMAC-SHA256
let mac = client.hmac_sha256(&key, &data)?;
```

### `src/error.rs` (Updated)

Added error variants:
- `RpcError(String)` - JSON-RPC errors from BearDog
- `ConnectionError(String)` - Socket connection errors
- `ConfigError(String)` - Missing socket/env vars
- `CryptoError(String)` - Generic crypto errors

### `Cargo.toml` (Updated)

Added `base64 = "0.21"` dependency.

---

## Files Requiring Refactoring

### 1. `src/keys.rs` - Identity & Key Exchange

**Current Debt**: Direct Ed25519/X25519/HMAC usage

#### A. OnionIdentity - Add Async Factory

Current:
```rust
impl OnionIdentity {
    pub fn generate() -> Self {
        // Direct ed25519_dalek usage
        let mut secret_bytes = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        // ...
    }
}
```

Refactor to:
```rust
impl OnionIdentity {
    /// Generate new random onion identity via BearDog
    ///
    /// TRUE PRIMAL: Delegates key generation to BearDog
    pub async fn generate_via_beardog(client: &BeardogCryptoClient) -> Result<Self> {
        let keypair = client.ed25519_generate_keypair()?;
        
        // Derive .onion address via BearDog
        let onion_address = derive_onion_address_via_beardog(client, &keypair.public_key).await?;
        
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
            onion_address,
            created_at,
        })
    }
    
    /// Legacy sync generate (for testing/offline only)
    #[cfg(any(test, feature = "offline"))]
    pub fn generate() -> Self {
        // Keep existing implementation for tests
    }
}
```

**Struct Change Required**:
```rust
pub struct OnionIdentity {
    // Change from ed25519_dalek types to raw bytes
    secret_key: [u8; 32],
    public_key: [u8; 32],
    onion_address: String,
    created_at: u64,
}
```

#### B. EphemeralKeypair - Delegate to BearDog

Current:
```rust
impl EphemeralKeypair {
    pub fn generate() -> Self {
        let secret = x25519_dalek::EphemeralSecret::random_from_rng(rand::thread_rng());
        let public = x25519_dalek::PublicKey::from(&secret).to_bytes();
        Self { secret, public }
    }
    
    pub fn derive_shared_secret(self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer_key = x25519_dalek::PublicKey::from(*peer_public);
        let shared = self.secret.diffie_hellman(&peer_key);
        shared.to_bytes()
    }
}
```

Refactor to:
```rust
impl EphemeralKeypair {
    /// Generate via BearDog (TRUE PRIMAL)
    pub fn generate_via_beardog(client: &BeardogCryptoClient) -> Result<Self> {
        let keypair = client.x25519_generate_ephemeral()?;
        Ok(Self {
            secret_key: keypair.secret_key,
            public_key: keypair.public_key,
        })
    }
    
    /// Derive shared secret via BearDog (TRUE PRIMAL)
    pub fn derive_shared_secret_via_beardog(
        self,
        client: &BeardogCryptoClient,
        peer_public: &[u8; 32],
    ) -> Result<[u8; 32]> {
        client.x25519_derive_secret(&self.secret_key, peer_public)
    }
}
```

**Struct Change Required**:
```rust
pub struct EphemeralKeypair {
    secret_key: [u8; 32],
    public_key: [u8; 32],
}
```

#### C. SessionKeys - Delegate HMAC to BearDog

Current:
```rust
impl SessionKeys {
    pub fn derive(
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Self {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        // Direct HMAC usage
        let mut mac = HmacSha256::new_from_slice(&[0u8; 32]).unwrap();
        mac.update(shared_secret);
        let prk = mac.finalize().into_bytes();
        // ...
    }
}
```

Refactor to:
```rust
impl SessionKeys {
    /// Derive session keys via BearDog (TRUE PRIMAL)
    pub fn derive_via_beardog(
        client: &BeardogCryptoClient,
        shared_secret: &[u8; 32],
        client_nonce: &[u8; 24],
        server_nonce: &[u8; 24],
        is_client: bool,
    ) -> Result<Self> {
        // 1. HKDF-Extract: PRK = HMAC-SHA256(salt=zeros, IKM=shared_secret)
        let prk = client.hmac_sha256(&[0u8; 32], shared_secret)?;
        
        // 2. HKDF-Expand for client key
        let mut client_info = Vec::new();
        client_info.extend_from_slice(b"sovereign-onion client");
        client_info.extend_from_slice(client_nonce);
        client_info.extend_from_slice(server_nonce);
        client_info.push(0x01);
        let client_key = client.hmac_sha256(&prk, &client_info)?;
        
        // 3. HKDF-Expand for server key  
        let mut server_info = Vec::new();
        server_info.extend_from_slice(b"sovereign-onion server");
        server_info.extend_from_slice(client_nonce);
        server_info.extend_from_slice(server_nonce);
        server_info.push(0x01);
        let server_key = client.hmac_sha256(&prk, &server_info)?;
        
        if is_client {
            Ok(Self { send_key: client_key, recv_key: server_key })
        } else {
            Ok(Self { send_key: server_key, recv_key: client_key })
        }
    }
}
```

---

### 2. `src/address.rs` - Onion Address Derivation

**Current Debt**: Direct SHA3-256 usage

#### derive_onion_address - Delegate to BearDog

Current:
```rust
pub fn derive_onion_address(pubkey: &VerifyingKey) -> String {
    use sha3::{Digest, Sha3_256};
    
    // Direct SHA3-256 usage
    let mut hasher = Sha3_256::new();
    hasher.update(b".onion checksum");
    hasher.update(pubkey.as_bytes());
    hasher.update(&[0x03]);
    let hash = hasher.finalize();
    // ...
}
```

Add new BearDog-delegated version:
```rust
/// Derive .onion address via BearDog (TRUE PRIMAL)
///
/// Uses BearDog's crypto.sha3_256 for the checksum calculation.
pub async fn derive_onion_address_via_beardog(
    client: &BeardogCryptoClient,
    pubkey_bytes: &[u8; 32],
) -> Result<String> {
    let mut data = Vec::with_capacity(35);
    
    // 1. Add public key (32 bytes)
    data.extend_from_slice(pubkey_bytes);
    
    // 2. Compute checksum via BearDog: SHA3-256(".onion checksum" || pubkey || 0x03)[0..2]
    let mut checksum_input = Vec::new();
    checksum_input.extend_from_slice(b".onion checksum");
    checksum_input.extend_from_slice(pubkey_bytes);
    checksum_input.push(0x03); // Version 3
    
    let hash = client.sha3_256(&checksum_input)?;
    let checksum = &hash[..2];
    
    // 3. Add checksum (2 bytes)
    data.extend_from_slice(checksum);
    
    // 4. Add version (1 byte)
    data.push(0x03);
    
    // 5. Base32 encode (RFC 4648, lowercase, no padding)
    let encoded = base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &data);
    
    Ok(format!("{}.onion", encoded))
}

/// Validate .onion address via BearDog (TRUE PRIMAL)
pub async fn validate_onion_address_via_beardog(
    client: &BeardogCryptoClient,
    onion: &str,
) -> Result<[u8; 32]> {
    // ... existing parsing logic ...
    
    // Verify checksum via BearDog
    let mut checksum_input = Vec::new();
    checksum_input.extend_from_slice(&pubkey_bytes);
    checksum_input.push(version);
    
    let hash = client.sha3_256(&checksum_input)?;
    let expected_checksum = &hash[..2];
    
    if checksum != expected_checksum {
        return Err(OnionError::ChecksumMismatch);
    }
    
    Ok(pubkey_array)
}
```

---

### 3. `src/crypto.rs` - Data Encryption

**Current Debt**: Direct ChaCha20-Poly1305 usage

#### encrypt_data / decrypt_data - Delegate to BearDog

Current:
```rust
pub fn encrypt_data(key: &[u8; 32], sequence: u64, plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.into());
    // Direct encryption
    cipher.encrypt(&nonce, plaintext)
}
```

Refactor to:
```rust
/// Encrypt data via BearDog (TRUE PRIMAL)
pub fn encrypt_data_via_beardog(
    client: &BeardogCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    plaintext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());
    
    client.chacha20_poly1305_encrypt(key, &nonce, plaintext)
}

/// Decrypt data via BearDog (TRUE PRIMAL)
pub fn decrypt_data_via_beardog(
    client: &BeardogCryptoClient,
    key: &[u8; 32],
    sequence: u64,
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    // Nonce: 12 bytes (8-byte sequence || 4 bytes zero)
    let mut nonce = [0u8; 12];
    nonce[..8].copy_from_slice(&sequence.to_le_bytes());
    
    client.chacha20_poly1305_decrypt(key, &nonce, ciphertext)
}
```

---

## Implementation Checklist

### Phase 1: Struct Changes (~30 min)

- [ ] `keys.rs`: Change `OnionIdentity` to use `[u8; 32]` instead of `ed25519_dalek` types
- [ ] `keys.rs`: Change `EphemeralKeypair` to use `[u8; 32]` instead of `x25519_dalek` types
- [ ] Run `cargo check` to identify all broken usages

### Phase 2: Add BearDog Methods (~1 hour)

- [ ] `keys.rs`: Add `OnionIdentity::generate_via_beardog()`
- [ ] `keys.rs`: Add `OnionIdentity::from_stored_via_beardog()` (if needed)
- [ ] `keys.rs`: Add `EphemeralKeypair::generate_via_beardog()`
- [ ] `keys.rs`: Add `EphemeralKeypair::derive_shared_secret_via_beardog()`
- [ ] `keys.rs`: Add `SessionKeys::derive_via_beardog()`
- [ ] `address.rs`: Add `derive_onion_address_via_beardog()`
- [ ] `address.rs`: Add `validate_onion_address_via_beardog()`
- [ ] `crypto.rs`: Add `encrypt_data_via_beardog()`
- [ ] `crypto.rs`: Add `decrypt_data_via_beardog()`

### Phase 3: Update Service/Connector (~1 hour)

- [ ] `service.rs`: Update `OnionService::new()` to use BearDog methods
- [ ] `connector.rs`: Prepare for Phase 4 with BearDog integration
- [ ] Add `BeardogCryptoClient` field to service struct

### Phase 4: Update Tests (~1.5 hours)

- [ ] Create mock BearDog server for testing
- [ ] Update all tests to use mocked BearDog
- [ ] Add integration test that verifies BearDog delegation
- [ ] Keep legacy tests under `#[cfg(test)]` with `#[cfg(feature = "offline")]`

### Phase 5: Cleanup (~30 min)

- [ ] Consider removing direct crypto dependencies from `Cargo.toml`
- [ ] Or gate them behind `offline` feature for testing
- [ ] Update documentation

---

## Testing Strategy

### Mock BearDog for Unit Tests

```rust
#[cfg(test)]
mod tests {
    use std::net::UnixListener;
    use std::thread;
    
    fn spawn_mock_beardog() -> String {
        let socket_path = format!("/tmp/test-beardog-{}.sock", std::process::id());
        let listener = UnixListener::bind(&socket_path).unwrap();
        
        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                // Handle JSON-RPC requests with test responses
                // ...
            }
        });
        
        socket_path
    }
    
    #[tokio::test]
    async fn test_identity_via_beardog() {
        let socket = spawn_mock_beardog();
        std::env::set_var("BEARDOG_SOCKET", &socket);
        
        let client = BeardogCryptoClient::from_env().unwrap();
        let identity = OnionIdentity::generate_via_beardog(&client).await.unwrap();
        
        assert!(identity.onion_address().ends_with(".onion"));
    }
}
```

### Integration Test with Live BearDog

```rust
#[tokio::test]
#[ignore] // Run manually with BearDog running
async fn test_live_beardog_integration() {
    // Requires: BEARDOG_SOCKET env var set
    let client = BeardogCryptoClient::from_env()
        .expect("BearDog socket not found - is BearDog running?");
    
    // Test SHA3-256 (critical for .onion addresses)
    let hash = client.sha3_256(b"test data").unwrap();
    assert_eq!(hash.len(), 32);
    
    // Test Ed25519 keypair generation
    let keypair = client.ed25519_generate_keypair().unwrap();
    assert_eq!(keypair.public_key.len(), 32);
    assert_eq!(keypair.secret_key.len(), 32);
    
    // Test full identity flow
    let identity = OnionIdentity::generate_via_beardog(&client).await.unwrap();
    assert!(identity.onion_address().ends_with(".onion"));
    assert_eq!(identity.onion_address().len(), 62);
}
```

---

## Environment Variables

For development/testing, ensure these are set:

```bash
# Direct BearDog socket (highest priority)
export BEARDOG_SOCKET="/run/user/1000/biomeos/beardog.sock"

# Or biomeOS-wired provider
export CRYPTO_PROVIDER_SOCKET="/run/user/1000/biomeos/beardog.sock"

# Family ID (for socket path resolution)
export FAMILY_ID="nat0"
```

---

## BearDog API Reference

All methods accept base64-encoded parameters and return base64-encoded results.

### SHA3-256 (CRITICAL for .onion)

```json
{
  "jsonrpc": "2.0",
  "method": "crypto.sha3_256",
  "params": { "data": "base64..." },
  "id": 1
}
→ { "hash": "base64...", "hash_base64": "...", "algorithm": "sha3_256" }
```

### Ed25519 Keypair

```json
{
  "method": "crypto.ed25519_generate_keypair",
  "params": {}
}
→ { "public_key": "base64...", "secret_key": "base64..." }
```

### X25519 (Session Keys)

```json
{
  "method": "crypto.x25519_generate_ephemeral",
  "params": {}
}
→ { "public_key": "base64...", "secret_key": "base64..." }

{
  "method": "crypto.x25519_derive_secret",
  "params": { "secret_key": "base64...", "public_key": "base64..." }
}
→ { "shared_secret": "base64..." }
```

### ChaCha20-Poly1305

```json
{
  "method": "crypto.chacha20_poly1305_encrypt",
  "params": { "key": "base64...", "nonce": "base64...", "plaintext": "base64..." }
}
→ { "ciphertext": "base64..." }
```

### HMAC-SHA256 (HKDF)

```json
{
  "method": "crypto.hmac_sha256",
  "params": { "key": "base64...", "data": "base64..." }
}
→ { "mac": "base64..." }
```

---

## Estimated Effort

| Phase | Task | Time |
|-------|------|------|
| 1 | Struct changes + cargo check | 30 min |
| 2 | Add BearDog methods | 1 hour |
| 3 | Update service/connector | 1 hour |
| 4 | Update tests | 1.5 hours |
| 5 | Cleanup | 30 min |
| **Total** | | **~4.5 hours** |

---

## Validation Criteria

After refactoring, verify:

1. ✅ `cargo test -p songbird-sovereign-onion --lib` passes
2. ✅ `cargo clippy -p songbird-sovereign-onion` clean
3. ✅ Integration test with live BearDog passes
4. ✅ `.onion` address generation produces valid addresses
5. ✅ No direct crypto imports in production code (only `BeardogCryptoClient`)

---

## Other Songbird Crypto Usage (Already Compliant)

### `songbird-tls/src/cert/generator.rs` ✅

Already uses hybrid pattern with BearDog delegation:
- `CertGenerationMode::Standalone` - Fallback for offline
- `CertGenerationMode::BearDog` - TRUE PRIMAL mode
- `CertGenerationMode::Auto` - Try BearDog, fallback to standalone

**No changes needed** - this is the pattern to follow!

### `songbird-orchestrator/src/access_control/pure_rust_jwt.rs`

JWT handling - may need review for BearDog HMAC delegation.

---

## Design Pattern to Follow

The `songbird-tls` cert generator shows the ideal hybrid pattern:

```rust
pub enum CryptoMode {
    /// Standalone: Use built-in crypto (100% Pure Rust, for testing/offline)
    Standalone,
    /// BearDog: Delegate to BearDog (TRUE PRIMAL, production)
    BearDog,
    /// Auto: Try BearDog, fallback to standalone (default)
    #[default]
    Auto,
}

impl OnionService {
    pub async fn new(port: u16, mode: CryptoMode) -> Result<Self> {
        let crypto_client = match &mode {
            CryptoMode::BearDog | CryptoMode::Auto => {
                BeardogCryptoClient::from_env().ok()
            }
            CryptoMode::Standalone => None,
        };
        
        // Use crypto_client if available, else standalone
    }
}
```

This allows:
- **Production**: Always use BearDog (`CryptoMode::BearDog`)
- **Testing**: Use standalone (`CryptoMode::Standalone`) with mock
- **Development**: Auto-discovery (`CryptoMode::Auto`)

---

## References

- `src/beardog_crypto.rs` - Ready-to-use BearDog client
- `songbird-tls/src/cert/generator.rs` - Reference hybrid implementation
- biomeOS `graphs/tower_atomic_bootstrap.toml` - Updated deployment graph
- biomeOS `capability_translation.rs` - Semantic translations verified
- BearDog `crypto_handler.rs` - All methods available

---

**Status**: Ready for Songbird team to begin refactoring

🐦 Songbird | 🐻🐕 BearDog | ✅ TRUE PRIMAL Pattern
