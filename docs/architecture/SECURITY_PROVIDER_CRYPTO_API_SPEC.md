# BearDog Crypto JSON-RPC API Specification

**Date**: January 18, 2026  
**Version**: 1.0.0  
**Status**: 🎯 Specification for TLS Crypto Delegation

---

## Overview

This document specifies the JSON-RPC API that BearDog exposes for crypto operations, specifically designed to support Songbird's Pure Rust TLS implementation.

**Purpose**: Enable Songbird to perform TLS crypto operations via BearDog delegation  
**Transport**: JSON-RPC 2.0 over Unix sockets  
**Security**: Unix socket permissions (peer authentication)

---

## API Methods

### 1. Ed25519 Signing

**Method**: `beardog.crypto.sign_ed25519`

**Purpose**: Sign a message with Ed25519 (for TLS CertificateVerify)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.sign_ed25519",
  "params": {
    "message": "base64_encoded_message",
    "key_id": "tls_signing_key",
    "purpose": "tls_handshake"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "signature": "base64_encoded_signature"
  },
  "id": 1
}
```

---

### 2. Ed25519 Verification

**Method**: `beardog.crypto.verify_ed25519`

**Purpose**: Verify an Ed25519 signature (for TLS Certificate validation)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.verify_ed25519",
  "params": {
    "message": "base64_encoded_message",
    "signature": "base64_encoded_signature",
    "public_key": "base64_encoded_public_key"
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true
  },
  "id": 2
}
```

---

### 3. X25519 Ephemeral Key Generation

**Method**: `beardog.crypto.x25519_generate_ephemeral`

**Purpose**: Generate ephemeral X25519 key pair (for TLS ECDHE)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.x25519_generate_ephemeral",
  "params": {
    "purpose": "tls_key_exchange"
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "public_key": "base64_encoded_public_key",
    "secret_key_id": "ephemeral_key_12345"
  },
  "id": 3
}
```

**Note**: Secret key is stored in BearDog, only ID is returned

---

### 4. X25519 Shared Secret Derivation

**Method**: `beardog.crypto.x25519_derive_secret`

**Purpose**: Derive shared secret from X25519 key exchange

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.x25519_derive_secret",
  "params": {
    "our_secret_key_id": "ephemeral_key_12345",
    "their_public_key": "base64_encoded_public_key"
  },
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "shared_secret": "base64_encoded_shared_secret"
  },
  "id": 4
}
```

---

### 5. ChaCha20-Poly1305 Encryption

**Method**: `beardog.crypto.chacha20_poly1305_encrypt`

**Purpose**: Encrypt data with ChaCha20-Poly1305 AEAD (for TLS records)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.chacha20_poly1305_encrypt",
  "params": {
    "plaintext": "base64_encoded_plaintext",
    "key": "base64_encoded_key",
    "nonce": "base64_encoded_nonce",
    "aad": "base64_encoded_aad"
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "ciphertext": "base64_encoded_ciphertext"
  },
  "id": 5
}
```

**Note**: Ciphertext includes authentication tag

---

### 6. ChaCha20-Poly1305 Decryption

**Method**: `beardog.crypto.chacha20_poly1305_decrypt`

**Purpose**: Decrypt data with ChaCha20-Poly1305 AEAD (for TLS records)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.chacha20_poly1305_decrypt",
  "params": {
    "ciphertext": "base64_encoded_ciphertext",
    "key": "base64_encoded_key",
    "nonce": "base64_encoded_nonce",
    "aad": "base64_encoded_aad"
  },
  "id": 6
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "plaintext": "base64_encoded_plaintext"
  },
  "id": 6
}
```

**Error Response** (authentication failure):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Authentication tag verification failed"
  },
  "id": 6
}
```

---

### 7. Blake3 Hashing

**Method**: `beardog.crypto.blake3_hash`

**Purpose**: Hash data with Blake3 (for TLS transcript hash)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.blake3_hash",
  "params": {
    "data": "base64_encoded_data"
  },
  "id": 7
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "base64_encoded_hash"
  },
  "id": 7
}
```

---

### 8. HMAC-SHA256

**Method**: `beardog.crypto.hmac_sha256`

**Purpose**: Compute HMAC-SHA256 (for TLS key derivation)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.hmac_sha256",
  "params": {
    "key": "base64_encoded_key",
    "data": "base64_encoded_data"
  },
  "id": 8
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hmac": "base64_encoded_hmac"
  },
  "id": 8
}
```

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32600 | Invalid Request | Malformed JSON-RPC |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid parameters |
| -32603 | Internal error | BearDog internal error |
| -32001 | Crypto error | Generic crypto error |
| -32002 | Key not found | Key ID not found |
| -32003 | Authentication failed | AEAD auth tag failed |

---

## Performance Targets

| Operation | Target Latency | Notes |
|-----------|----------------|-------|
| Ed25519 Sign | < 100 µs | Pure Rust, very fast |
| Ed25519 Verify | < 150 µs | Pure Rust, fast |
| X25519 Generate | < 50 µs | Pure Rust, very fast |
| X25519 Derive | < 100 µs | Pure Rust, fast |
| ChaCha20 Encrypt | < 50 µs / KB | Pure Rust, very fast |
| ChaCha20 Decrypt | < 50 µs / KB | Pure Rust, very fast |
| Blake3 Hash | < 20 µs / KB | Pure Rust, extremely fast |
| HMAC-SHA256 | < 50 µs / KB | Pure Rust, fast |

**Total TLS Handshake**: ~500 µs to 1 ms (5-10 crypto ops)

---

## Security Considerations

### Key Management
- Long-term TLS keys stored in BearDog
- Ephemeral keys generated per-connection
- Automatic key cleanup after use

### Authentication
- Unix socket peer authentication
- Purpose field for audit logging
- Rate limiting per client

### Audit Logging
- All crypto operations logged
- Key ID, operation, timestamp
- Purpose/context included

---

## Implementation Notes

### BearDog Side
- Implement handlers in `beardog-crypto-service/src/json_rpc_handlers.rs`
- Use existing BearDog crypto primitives (already Pure Rust!)
- Add to existing JSON-RPC server

### Songbird Side
- Create `BeardogCryptoClient` in `songbird-orchestrator/src/crypto/beardog_client.rs`
- Implement `rustls::CryptoProvider` trait
- Delegate all crypto operations to BearDog

---

## Testing Strategy

### Unit Tests
- Test each crypto operation independently
- Verify correct base64 encoding/decoding
- Test error handling

### Integration Tests
- Full TLS handshake via BearDog crypto
- Performance benchmarks
- Concurrent connection tests

### Security Tests
- Invalid signatures
- Authentication tag tampering
- Key exhaustion

---

## Migration Path

### Phase 1 (Week 1-2)
- Implement BearDog JSON-RPC handlers
- Test crypto operations in isolation
- Document API

### Phase 2 (Week 3-4)
- Implement Songbird `BeardogCryptoClient`
- Integrate with rustls fork
- Test TLS handshake

### Phase 3 (Week 5)
- Performance optimization
- Security testing
- Production readiness

---

**Status**: 🎯 Ready for implementation!  
**Timeline**: ~2-3 days for BearDog API  
**Result**: Pure Rust crypto provider for TLS!

🐻🐕✨ **BearDog Crypto API - Foundation for Pure Rust TLS!** ✨🐕🐻

