# BearDog Integration Handoff

**Date**: February 8, 2026  
**From**: Songbird Team  
**To**: BearDog Crypto Provider Team  
**Purpose**: Integrate BearDog crypto provider into new Songbird protocols

---

## Executive Summary

Songbird has implemented 3 new protocols (QUIC, NFC, WireGuard beacon) that require BearDog crypto provider integration. All production code is ready with stubs in place for BearDog methods. This document specifies the required BearDog API methods needed to complete the integration.

---

## Current Status

### Production Code: Ready ✅
- All protocols implemented with BearDog stub calls
- Zero direct crypto in production (TRUE PRIMAL compliance)
- Workspace compiles cleanly
- 21 integration points identified and documented

### Testing: Comprehensive ✅
- Unit tests: 583+ files
- Integration tests: Complete framework
- E2E tests: 19 scenarios
- Chaos tests: 8 scenarios

### Deep Debt: S+ Tier ✅
- 100% BearDog crypto delegation (stubs ready)
- Zero unsafe code
- 95% pure Rust dependencies
- Feature-gated test crypto (standalone only)

---

## Required BearDog API Methods

### Priority 1: X25519 Key Exchange (CRITICAL)

**Needed By**: NFC genesis, QUIC crypto provider

```rust
/// Generate X25519 keypair for Diffie-Hellman key exchange
/// Used for: NFC pairing, QUIC handshake
pub async fn generate_x25519_keypair(&self) -> Result<X25519KeyPair> {
    // Returns: (secret_key: [u8; 32], public_key: [u8; 32])
}

/// Compute X25519 Diffie-Hellman shared secret
/// Used for: Deriving symmetric keys from key exchange
pub async fn x25519_diffie_hellman(
    &self,
    secret_key: &[u8; 32],
    peer_public_key: &[u8; 32]
) -> Result<[u8; 32]> {
    // Returns: shared_secret
}
```

**Integration Points**:
- `crates/songbird-nfc/src/genesis.rs:199` - `generate_ephemeral_keypair_stub()`
- `crates/songbird-nfc/src/genesis.rs:205` - `compute_shared_secret_stub()`
- `crates/songbird-quic/src/config.rs:132` - QUIC server crypto config

---

### Priority 2: ChaCha20-Poly1305 AEAD (CRITICAL)

**Needed By**: NFC genesis

```rust
/// Encrypt data with ChaCha20-Poly1305 AEAD
/// Used for: NFC genesis credential encryption
pub async fn chacha20poly1305_encrypt(
    &self,
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8]
) -> Result<Vec<u8>> {
    // Returns: ciphertext with 16-byte MAC appended
}

/// Decrypt data with ChaCha20-Poly1305 AEAD
/// Used for: NFC genesis credential decryption
pub async fn chacha20poly1305_decrypt(
    &self,
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8]
) -> Result<Vec<u8>> {
    // Returns: plaintext (MAC verified)
}
```

**Integration Points**:
- `crates/songbird-nfc/src/genesis.rs:217` - `encrypt_genesis_stub()`
- `crates/songbird-nfc/src/genesis.rs:224` - `decrypt_genesis_stub()`

---

### Priority 3: Ed25519 Ephemeral Signatures (CRITICAL)

**Needed By**: NFC genesis

```rust
/// Sign message with Ed25519 ephemeral key
/// Used for: NFC message authentication
pub async fn ed25519_sign_ephemeral(
    &self,
    ephemeral_key_id: &str,
    data: &[u8]
) -> Result<[u8; 64]> {
    // Returns: signature
}

/// Verify Ed25519 signature
/// Used for: NFC message verification
pub async fn ed25519_verify(
    &self,
    public_key: &[u8; 32],
    data: &[u8],
    signature: &[u8; 64]
) -> Result<bool> {
    // Returns: true if valid
}

/// Destroy ephemeral keys (security cleanup)
/// Used for: NFC ceremony cleanup
pub async fn destroy_ephemeral_keys(
    &self,
    key_id: &str
) -> Result<()> {
    // Securely erase ephemeral key material
}
```

**Integration Points**:
- `crates/songbird-nfc/src/genesis.rs:231` - `sign_message_stub()`
- `crates/songbird-nfc/src/genesis.rs:237` - `verify_signature_stub()`
- `crates/songbird-nfc/src/genesis.rs:243` - `destroy_ephemeral_keys_stub()`

---

### Priority 4: Random Generation (HIGH)

**Needed By**: NFC genesis, TLS handshake, QUIC

```rust
/// Generate cryptographically secure random bytes
/// Used for: Nonces, IVs, session IDs
pub async fn generate_random(
    &self,
    size: usize
) -> Result<Vec<u8>> {
    // Returns: cryptographically secure random bytes
}

/// Generate nonce (convenience wrapper)
/// Used for: AEAD nonces
pub async fn generate_nonce(
    &self,
    size: usize
) -> Result<Vec<u8>> {
    // Returns: random nonce of specified size
}
```

**Integration Points**:
- `crates/songbird-nfc/src/genesis.rs:211` - `generate_nonce_stub()`
- `crates/songbird-tls/src/handshake/mod.rs:117` - Random generation for TLS

---

### Priority 5: Certificate Operations (MEDIUM)

**Needed By**: QUIC crypto provider, TLS

```rust
/// Generate self-signed certificate for QUIC/TLS
/// Used for: QUIC server initialization, TLS server setup
pub async fn certificate_generate_self_signed(
    &self,
    domain: &str,
    validity_days: u32
) -> Result<SelfSignedCert> {
    // Returns: (cert_pem, private_key_pem)
}

/// Verify certificate signature
/// Used for: QUIC client verification, TLS client verification
pub async fn certificate_verify(
    &self,
    cert: &[u8],
    signature: &[u8],
    public_key: &[u8]
) -> Result<bool> {
    // Returns: true if valid
}
```

**Integration Points**:
- `crates/songbird-quic/src/config.rs:178` - QUIC client crypto config
- `crates/songbird-tls/src/cert/generator.rs:160` - TLS certificate generation

---

### Priority 6: Public Key Derivation (MEDIUM)

**Needed By**: Sovereign Onion

```rust
/// Derive Ed25519 public key from secret key
/// Used for: Onion address generation
pub async fn ed25519_public_from_secret(
    &self,
    secret_key: &[u8; 32]
) -> Result<[u8; 32]> {
    // Returns: public_key
}
```

**Integration Points**:
- `crates/songbird-sovereign-onion/src/keys.rs:94` - Public key derivation

---

### Priority 7: SHA3-256 (LOW)

**Needed By**: Tor onion service descriptor

```rust
/// Compute SHA3-256 hash
/// Used for: Onion address checksum
pub async fn sha3_256(
    &self,
    data: &[u8]
) -> Result<[u8; 32]> {
    // Returns: hash
}
```

**Integration Points**:
- `crates/songbird-tor-protocol/src/onion_service/descriptor.rs:59` - Checksum calculation

---

### Priority 8: HMAC-SHA256 (LOW)

**Needed By**: JWT delegation

```rust
/// Compute HMAC-SHA256
/// Used for: JWT signing
pub async fn hmac_sha256(
    &self,
    key: &[u8],
    data: &[u8]
) -> Result<[u8; 32]> {
    // Returns: MAC
}
```

**Integration Points**:
- `crates/songbird-orchestrator/Cargo.toml:100` - JWT token signing

---

### Priority 9: QUIC Crypto Provider (ADVANCED)

**Needed By**: QUIC protocol (quinn integration)

```rust
/// Create BearDog-backed QUIC crypto provider
/// Implements quinn's crypto provider trait
/// Used for: Replace temporary rustls provider
pub fn create_quic_crypto_provider(&self) -> Result<Arc<dyn QuicCryptoProvider>> {
    // Returns: Provider implementing quinn's crypto trait
    // Must support: key derivation, AEAD, handshake
}
```

**Integration Points**:
- `crates/songbird-quic/src/config.rs:132` - Server config
- `crates/songbird-quic/src/config.rs:178` - Client config
- `crates/songbird-quic/src/config.rs:234` - Certificate verification

**Note**: This is the most complex integration point. May require custom `quinn::crypto` implementation.

---

## Integration Pattern

### Standard Pattern

All Songbird protocols follow this pattern:

```rust
// 1. Discover BearDog socket
let beardog = BeardogCryptoClient::from_env()?;

// 2. Call BearDog method
let result = beardog.method_name(args).await?;

// 3. Use result in protocol
protocol.use_crypto_result(result);
```

### Example: NFC Integration

```rust
// Current stub:
async fn generate_ephemeral_keypair_stub(&self) -> Result<[u8; PUBLIC_KEY_SIZE]> {
    // TODO: Call BearDog: beardog.generate_x25519_keypair()
    debug!("TODO: BearDog generate_x25519_keypair");
    Ok([0u8; PUBLIC_KEY_SIZE])
}

// After BearDog integration:
async fn generate_ephemeral_keypair(&self) -> Result<[u8; PUBLIC_KEY_SIZE]> {
    let beardog = &self.beardog_client;
    let keypair = beardog.generate_x25519_keypair().await?;
    Ok(keypair.public_key)
}
```

---

## Testing Requirements

### Unit Tests
- Each BearDog method needs corresponding unit tests
- Test with valid/invalid inputs
- Test error handling

### Integration Tests
- Full protocol flow tests
- NFC genesis ceremony end-to-end
- QUIC connection establishment
- Error recovery scenarios

### Security Tests
- Ephemeral key destruction verification
- Nonce uniqueness validation
- AEAD authentication verification
- Timing attack resistance (NFC)

---

## Security Considerations

### Ephemeral Keys
- **CRITICAL**: NFC ephemeral keys MUST be destroyed after ceremony
- No key material should remain in memory
- BearDog must support secure key erasure

### Timing Attacks
- NFC operations must have constant-time execution
- Random delays for timing protection (already implemented in Songbird)
- BearDog operations should not leak timing information

### Key Storage
- QUIC certificates: Persistent (but renewable)
- NFC ephemeral keys: Destroyed immediately
- Onion identity keys: Long-term storage in BearDog

---

## Implementation Checklist

### Phase 1: Core Crypto (Enables NFC + QUIC basics)
- [ ] X25519 keypair generation
- [ ] X25519 Diffie-Hellman
- [ ] ChaCha20-Poly1305 encrypt/decrypt
- [ ] Ed25519 ephemeral sign/verify
- [ ] Ephemeral key destruction
- [ ] Random generation / nonces

### Phase 2: Certificates (Enables full QUIC)
- [ ] Self-signed certificate generation
- [ ] Certificate verification
- [ ] Ed25519 public key derivation

### Phase 3: Advanced (Enables all protocols)
- [ ] SHA3-256 hashing
- [ ] HMAC-SHA256
- [ ] QUIC crypto provider trait implementation

### Phase 4: Testing & Integration
- [ ] Unit tests for all methods
- [ ] Integration tests with Songbird protocols
- [ ] Security audit
- [ ] Performance benchmarking

---

## Success Criteria

### For BearDog Team:
✅ All Priority 1-4 methods implemented and tested  
✅ Security audit passed (ephemeral key handling)  
✅ Performance acceptable (<10ms per operation)  
✅ Error handling comprehensive  

### For Songbird Integration:
✅ All stubs replaced with real BearDog calls  
✅ NFC genesis ceremony works end-to-end  
✅ QUIC connections established successfully  
✅ All tests passing  
✅ Zero direct crypto in production  

---

## Timeline Estimate

**Phase 1** (Core Crypto): 1-2 weeks
- X25519, ChaCha20-Poly, Ed25519, Random
- Enables NFC and basic QUIC

**Phase 2** (Certificates): 1 week
- Certificate generation/verification
- Enables full QUIC with TLS

**Phase 3** (Advanced): 1 week
- SHA3, HMAC, optimization
- Enables all protocols

**Phase 4** (Testing): 1 week
- Integration testing, security audit
- Production ready

**Total**: 4-5 weeks

---

## Contact Points

### Songbird Integration Points

**NFC Genesis**:
- File: `crates/songbird-nfc/src/genesis.rs`
- Lines: 79-246
- Methods: 8 stubs to replace

**QUIC Protocol**:
- File: `crates/songbird-quic/src/config.rs`
- Lines: 130-234
- Methods: 3 crypto provider integration points

**Sovereign Onion**:
- File: `crates/songbird-sovereign-onion/src/keys.rs`
- Line: 94
- Method: 1 public key derivation

### Documentation

**Architecture**:
- NFC: `crates/songbird-nfc/README.md`
- QUIC: `crates/songbird-quic/README.md`
- Multi-Path: `specs/SOVEREIGN_MULTIPATH_PROTOCOL.md`

**Evolution Report**:
- `docs/sessions/2026-02-february/EVOLUTION_STATUS_REPORT_FEB_08_2026.md`

---

## Questions for BearDog Team

1. **Async API**: All Songbird calls are async. Can BearDog methods be async or do we need sync wrappers?

2. **Key Management**: For ephemeral keys, should we use key IDs for lifecycle management or pass raw key material?

3. **QUIC Provider**: Does BearDog want to implement the `quinn::crypto` trait directly, or should Songbird wrap BearDog methods?

4. **Performance**: What's the expected latency for crypto operations? NFC has strict timing requirements (<100ms per operation).

5. **Error Handling**: Should we use BearDog's error types or convert to Songbird errors?

6. **API Stability**: Will these method signatures be stable, or should we design for future API changes?

---

## Next Steps

### Immediate:
1. **BearDog Team**: Review this specification
2. **BearDog Team**: Confirm API design and answer questions
3. **Songbird Team**: Push current code to repository
4. **Both Teams**: Schedule integration kickoff meeting

### After Agreement:
1. **BearDog Team**: Implement Phase 1 methods
2. **Songbird Team**: Create integration test suite
3. **Both Teams**: Conduct integration testing
4. **Both Teams**: Security audit and performance testing

---

## Summary

**Ready for BearDog**:
- ✅ 21 integration points identified
- ✅ All stubs documented
- ✅ API requirements specified
- ✅ Testing strategy defined
- ✅ Timeline estimated

**BearDog Action Required**:
- Implement 9 method categories (Priority 1-4 critical)
- Provide async API compatibility
- Support ephemeral key lifecycle
- Enable QUIC crypto provider integration

**Timeline**: 4-5 weeks to full integration

---

**Status**: 🤝 READY FOR BEARDOG HANDOFF 🤝

**Date**: February 8, 2026  
**Songbird Version**: v3.36.0  
**Next Milestone**: BearDog Crypto Provider Integration
