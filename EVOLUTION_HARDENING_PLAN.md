# 🛡️ Songbird & BearDog Evolution & Hardening Plan

**Created**: Jan 24, 2026  
**Updated**: Jan 24, 2026  
**Version**: v5.23.0  
**Status**: Phase 3 Complete - Production Clean  
**Focus**: Production Hardening & Capability Evolution

---

## Executive Summary

With TLS 1.3 HTTPS now working at 100% and capability abstraction complete, we focus on:
1. **Hardening** - Production-grade error handling, security, performance
2. **Evolution** - Modern Rust idioms, capability-based infrastructure ✅ (v5.21.0-v5.22.0)
3. **Production Cleanup** - Logging rationalization ✅ (v5.23.0)
4. **Agnostic Architecture** - Semantic translation for Neural API deployment

### Completed Milestones

| Version | Milestone | Status |
|---------|-----------|--------|
| v5.20.0 | HTTPS 100% Working | ✅ |
| v5.21.0 | CryptoCapability Abstraction | ✅ |
| v5.22.0 | Full TLS Migration | ✅ |
| v5.23.0 | Production Logging Cleanup | ✅ |

---

## 🐕 BearDog Hardening

### Current State
BearDog provides all TLS 1.3 cryptographic operations via JSON-RPC:
- X25519 key exchange
- AES-128/256-GCM encryption/decryption
- ChaCha20-Poly1305 encryption/decryption
- SHA-256 hashing
- HKDF key derivation
- TLS-specific key derivation (handshake, application, finished)

### Required Hardening

#### 1. Remove Diagnostic Logging
**Priority**: High  
**Files**: `crates/beardog-tunnel/src/unix_socket_ipc/crypto_handlers_aes_gcm.rs`

```rust
// REMOVE: Production code should not print to stderr
eprintln!("🔐 AES-128-GCM ENCRYPT DIAGNOSTIC...");
```

**Action**: Remove all `eprintln!` diagnostic logs from production code paths.

#### 2. Constant-Time Operations
**Priority**: High  
**Files**: All crypto handlers

**Review**: Ensure all cryptographic comparisons use constant-time functions:
```rust
// Bad: Variable-time comparison
if computed_tag == expected_tag { ... }

// Good: Constant-time comparison
use subtle::ConstantTimeEq;
if computed_tag.ct_eq(&expected_tag).into() { ... }
```

#### 3. Zeroize Sensitive Data
**Priority**: High  
**Files**: All crypto handlers

**Review**: Ensure all secret keys are zeroized after use:
```rust
use zeroize::Zeroize;
let mut key = derive_key(...);
// use key...
key.zeroize(); // Or use Zeroizing<T> wrapper
```

#### 4. Rate Limiting & DoS Protection
**Priority**: Medium  
**Files**: Unix socket IPC handlers

**Add**: Request rate limiting per client to prevent resource exhaustion.

#### 5. Input Validation Hardening
**Priority**: Medium  
**Files**: All RPC handlers

**Review**: Ensure all inputs are validated:
- Key lengths (16 bytes for AES-128, 32 for AES-256, 32 for X25519)
- Nonce lengths (12 bytes for AEAD ciphers)
- Max plaintext/ciphertext lengths

---

## 🐦 Songbird Hardening

### Current State
Songbird provides TLS 1.3 HTTPS client via:
- `handshake_legacy.rs` - Full RFC 8446 handshake
- `record.rs` - TLS record layer
- `session.rs` - Session key management
- `alert.rs` - TLS alert parsing
- `client.rs` - HTTP request/response

### Required Hardening

#### 1. Remove Diagnostic Logging
**Priority**: High  
**Files**: `handshake_legacy.rs`, `record.rs`

```rust
// KEEP: Info-level for important events
info!("✅ TLS 1.3 handshake complete in {:?}", total_time);

// REMOVE: Verbose hex dumps
info!("First 16 bytes (hex): {}", hex::encode(&decrypted[..16]));
```

**Action**: Create log levels:
- `info!` - Key events (handshake start/end, connection established)
- `debug!` - Protocol details (extensions, cipher suite)
- `trace!` - Byte-level dumps (only for debugging)

#### 2. Connection Timeouts
**Priority**: High  
**Files**: `handshake_legacy.rs`, `record.rs`

**Current**: 5000ms handshake timeout, 200ms post-handshake  
**Action**: Make configurable via `TlsConfig`:
```rust
pub struct TlsConfig {
    pub handshake_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}
```

#### 3. Certificate Validation
**Priority**: Critical  
**Files**: `handshake_legacy.rs` (currently skipped)

**Current State**: Certificate validation is NOT implemented!
```rust
// Current: Skip certificate validation
info!("   📜 (Certificate validation not yet implemented - INSECURE!)");
```

**Action**: Implement full certificate chain validation:
1. Parse X.509 certificates
2. Verify certificate signatures
3. Check validity periods
4. Validate certificate chain to trusted roots
5. Check revocation (CRL/OCSP)

**Option A**: Implement in BearDog (add `crypto.verify_certificate_chain`)  
**Option B**: Use `webpki` crate in Songbird (adds dependency)

#### 4. Session Resumption (0-RTT)
**Priority**: Medium  
**Files**: `session.rs`, `handshake_legacy.rs`

**Current**: NewSessionTicket is consumed but not stored  
**Action**: Implement PSK-based resumption for faster connections

#### 5. Chunked Transfer Encoding
**Priority**: Medium  
**Files**: `client.rs`

**Current**: Basic body parsing, chunked may fail  
**Action**: Implement proper HTTP/1.1 chunked transfer encoding parser

#### 6. HTTP/2 Support
**Priority**: Low  
**Files**: New module

**Action**: Add HTTP/2 framing over TLS (ALPN "h2" already advertised)

---

## 🔄 Evolution to Agnostic Capability Infrastructure

### Current: Direct RPC

```
Songbird ────[JSON-RPC]────> BearDog
         "crypto.aes128_gcm_encrypt"
```

### Target: Neural API with Semantic Translation

```
Songbird ────[Neural API]────> biomeOS Graph ────[Semantic Translation]────> BearDog
         "encrypt_data"                        "crypto.aes128_gcm_encrypt"
```

### Evolution Path

#### Phase 1: Capability Declaration (BearDog)

BearDog declares its capabilities:
```json
{
  "capabilities": [
    {
      "name": "symmetric_encryption",
      "algorithms": ["AES-128-GCM", "AES-256-GCM", "ChaCha20-Poly1305"],
      "operations": ["encrypt", "decrypt"]
    },
    {
      "name": "key_exchange",
      "algorithms": ["X25519"],
      "operations": ["generate_keypair", "derive_shared_secret"]
    },
    {
      "name": "tls_key_derivation",
      "versions": ["TLS 1.3"],
      "operations": ["handshake_secrets", "application_secrets", "finished_verify"]
    }
  ]
}
```

#### Phase 2: Semantic Request (Songbird)

Songbird requests capabilities semantically:
```json
{
  "intent": "encrypt_application_data",
  "context": {
    "protocol": "TLS 1.3",
    "cipher_suite": "TLS_AES_128_GCM_SHA256"
  },
  "data": { ... }
}
```

#### Phase 3: Graph Translation (biomeOS)

biomeOS Neural API:
1. Receives semantic request from Songbird
2. Discovers available primals with matching capabilities
3. Translates semantic intent to primal-specific RPC
4. Routes request to appropriate primal (BearDog)
5. Returns result to Songbird

#### Phase 4: Runtime Discovery

Instead of hardcoded:
```rust
let beardog = BearDogClient::new("/tmp/beardog.sock");
```

Use capability discovery:
```rust
let crypto = biome.discover_capability("symmetric_encryption")
    .with_algorithm("AES-128-GCM")
    .await?;
crypto.encrypt(key, nonce, plaintext, aad).await?
```

---

## 📋 Implementation Priority

### Immediate (v5.21.0)
1. ✅ Archive investigation docs (DONE)
2. 🔄 Clean diagnostic logging
3. 🔄 Add TlsConfig for timeouts
4. 🔄 Run full test suite

### Short-term (v5.22.0)
1. Certificate validation (BearDog + Songbird)
2. Proper error types (not string errors)
3. Connection pooling

### Medium-term (v5.23.0)
1. Session resumption (PSK)
2. HTTP/1.1 chunked encoding fix
3. Capability declaration API

### Long-term (v6.0.0)
1. Neural API integration
2. Semantic translation layer
3. HTTP/2 support
4. Full agnostic capability infrastructure

---

## 🧪 Test Strategy

### Unit Tests
```bash
# Songbird
cargo test -p songbird-http-client --lib

# BearDog
cargo test -p beardog-tunnel --lib
```

### Integration Tests
```bash
# Real HTTPS servers
cargo run --release --example test_https -- https://cloudflare.com
cargo run --release --example test_https -- https://google.com
cargo run --release --example test_https -- https://github.com
```

### Self-Test (Client-Server)
```bash
# Run Songbird TLS server
cargo run --example tls_server &

# Test against it
cargo run --example test_https -- https://localhost:8443
```

---

## Summary

| Component | Hardening Priority | Evolution Priority |
|-----------|-------------------|-------------------|
| BearDog | Remove logging, constant-time ops, zeroize | Capability declaration |
| Songbird | Certificate validation, timeouts | Semantic requests, capability discovery |
| biomeOS | N/A | Graph translation, primal discovery |

**Goal**: Pure Rust, zero C dependencies, RFC-compliant, production-ready, agnostic capability infrastructure ready for Neural API semantic translation.

