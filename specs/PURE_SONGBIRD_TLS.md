# Pure Songbird TLS - Technical Specification

**Version:** 1.0.0  
**Date:** January 18, 2026  
**Status:** 🎯 ACTIVE - Implementation Phase  
**Author:** Songbird Team  

---

## 🎯 Executive Summary

**Pure Songbird TLS** is a 100% Pure Rust TLS 1.3 implementation designed for the biomeOS ecosystem. Unlike traditional approaches that wrap existing libraries (rustls, openssl), Songbird implements the TLS 1.3 protocol directly, delegating ALL cryptographic operations to BearDog via runtime-discovered Unix sockets.

### Why Pure Songbird TLS?

**Problem with rustls Integration:**
- ❌ rustls has built-in ring/aws-lc-rs dependencies (C code)
- ❌ API mismatch (rustls generates nonces, TLS 1.3 needs specific nonces)
- ❌ Tight coupling to rustls's architecture
- ❌ Not TRUE Pure Rust - just replacing one provider

**Pure Songbird TLS Solution:**
- ✅ 100% Pure Rust - Zero C dependencies (TRUE ecoBin)
- ✅ Perfect API fit with BearDog delegation model
- ✅ Complete control over protocol implementation
- ✅ Tower Architecture - Songbird + BearDog as HTTPS relay
- ✅ Protocol-agnostic foundation (HTTP/1.1, HTTP/2, HTTP/3, WebSocket)
- ✅ Deep debt solution - Own the entire stack

---

## 🏗️ Architecture Overview

### Component Separation

```
┌─────────────────────────────────────────────────────────────┐
│                     External Client                         │
│                    (Browser, API, etc)                       │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTPS (TLS 1.3)
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                  SONGBIRD (Protocol Layer)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ TLS 1.3 Protocol (100% Pure Rust)                   │   │
│  │ • Handshake State Machine                           │   │
│  │ • Record Layer (framing, encrypt/decrypt)           │   │
│  │ • Key Schedule (HKDF key derivation)                │   │
│  │ • Certificate Validation                            │   │
│  │ • Alert Protocol                                    │   │
│  └─────────────────┬───────────────────────────────────┘   │
│                    │ JSON-RPC over Unix Socket              │
│                    ↓                                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Crypto Delegation (Capability Discovery)            │   │
│  │ • Discovers crypto provider at runtime              │   │
│  │ • No hardcoded "BearDog" references                 │   │
│  │ • Pure capability-based communication               │   │
│  └─────────────────┬───────────────────────────────────┘   │
└────────────────────┼────────────────────────────────────────┘
                     │ Unix Socket
                     ↓
┌─────────────────────────────────────────────────────────────┐
│                  BEARDOG (Crypto Layer)                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Pure Rust Crypto Operations (RustCrypto)            │   │
│  │ • Ed25519 (signing/verification)                    │   │
│  │ • X25519 (key exchange)                             │   │
│  │ • ChaCha20-Poly1305 (AEAD)                          │   │
│  │ • Blake3 (hashing)                                  │   │
│  │ • HMAC-SHA256 (key derivation)                      │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Tower Deployment Model

In biomeOS, a "Tower" is a Songbird + BearDog pair deployed as a communication relay:

```
Internet → Tower (Songbird + BearDog) → Internal Services
```

**Benefits:**
- Centralized TLS termination
- Single security audit point (BearDog)
- Protocol flexibility (HTTP/1.1, HTTP/2, HTTP/3, WebSocket)
- 100% Pure Rust sovereignty
- Musl-static deployment (universal portability)

---

## 📋 TLS 1.3 Components

### 1. Handshake State Machine

**Purpose:** Negotiate connection parameters and establish shared secrets.

**States:**
```rust
enum HandshakeState {
    Start,                    // Initial state
    ClientHello,              // Received ClientHello
    ServerHello,              // Sent ServerHello
    EncryptedExtensions,      // Sent EncryptedExtensions
    Certificate,              // Sent Certificate
    CertificateVerify,        // Sent CertificateVerify
    Finished,                 // Sent Finished (server)
    ClientFinished,           // Received Finished (client)
    Connected,                // Handshake complete
    Error(HandshakeError),    // Error state
}
```

**Key Operations:**
- Parse ClientHello (TLS version, cipher suites, extensions)
- Generate ServerHello (selected cipher suite, key share)
- Derive handshake secrets (HKDF)
- Send server certificate chain
- Sign handshake transcript (Ed25519 via BearDog)
- Verify client Finished message
- Transition to application data

**Crypto Delegation:**
- X25519 key exchange → BearDog
- Ed25519 certificate signing → BearDog
- HKDF key derivation → BearDog (HMAC-SHA256)
- Transcript hashing → BearDog (Blake3)

**Estimated Complexity:** 800-1000 lines

---

### 2. Record Layer

**Purpose:** Frame, encrypt, and decrypt TLS records.

**Record Format (RFC 8446 Section 5.2):**
```
struct TLSPlaintext {
    ContentType type;         // 1 byte (alert, handshake, application_data)
    ProtocolVersion version;  // 2 bytes (legacy 0x0303 for TLS 1.3)
    uint16 length;            // 2 bytes (max 2^14 = 16384)
    opaque fragment[length];  // Payload
}
```

**TLS 1.3 Encrypted Record:**
```
struct TLSCiphertext {
    ContentType opaque_type = application_data; // Always 0x17
    ProtocolVersion version = 0x0303;           // Legacy
    uint16 length;                              // ciphertext + tag length
    opaque encrypted_record[length];            // AEADEncrypted content
}

encrypted_record = AEAD-Encrypt(
    key: traffic_secret,
    nonce: IV XOR sequence_number,
    plaintext: content + content_type + padding,
    aad: TLSCiphertext header (5 bytes)
)
```

**Key Operations:**
- Frame plaintext into records (max 16KB)
- Construct TLS 1.3 nonce (IV XOR sequence number)
- Construct AAD (5-byte record header)
- Encrypt records (ChaCha20-Poly1305 via BearDog)
- Decrypt records (ChaCha20-Poly1305 via BearDog)
- Remove padding and extract content type

**Crypto Delegation:**
- ChaCha20-Poly1305 encryption → BearDog
- ChaCha20-Poly1305 decryption → BearDog
- Nonce construction: Local (IV XOR seq)
- AAD construction: Local (record header)

**Estimated Complexity:** 600-800 lines

---

### 3. Key Schedule (HKDF)

**Purpose:** Derive all keys from handshake secrets using HKDF.

**TLS 1.3 Key Derivation (RFC 8446 Section 7.1):**
```
                0
                |
                v
  PSK ->  HKDF-Extract = Early Secret
                |
                +-----> Derive-Secret(., "ext binder" | "res binder")
                |
                v
          Derive-Secret(., "c e traffic", ClientHello)
          Derive-Secret(., "e exp master", ClientHello)
                |
                v
  (EC)DHE -> HKDF-Extract = Handshake Secret
                |
                +-----> Derive-Secret(., "c hs traffic", ClientHello...ServerHello)
                +-----> Derive-Secret(., "s hs traffic", ClientHello...ServerHello)
                v
          Derive-Secret(., "derived", "")
                |
                v
  0 -> HKDF-Extract = Master Secret
                |
                +-----> Derive-Secret(., "c ap traffic", ClientHello...server Finished)
                +-----> Derive-Secret(., "s ap traffic", ClientHello...server Finished)
                +-----> Derive-Secret(., "exp master", ClientHello...server Finished)
                +-----> Derive-Secret(., "res master", ClientHello...client Finished)
```

**Keys Derived:**
- `client_handshake_traffic_secret` - Client handshake encryption
- `server_handshake_traffic_secret` - Server handshake encryption
- `client_application_traffic_secret_0` - Client application data
- `server_application_traffic_secret_0` - Server application data

**From Each Traffic Secret, Derive:**
- `key` - AEAD encryption key (32 bytes for ChaCha20)
- `iv` - AEAD IV (12 bytes)

**Crypto Delegation:**
- HKDF-Extract → BearDog (HMAC-SHA256)
- HKDF-Expand-Label → BearDog (HMAC-SHA256)
- Transcript hashing → BearDog (Blake3)

**Estimated Complexity:** 400-600 lines

---

### 4. Certificate Validation

**Purpose:** Validate server certificate chain and verify signatures.

**Operations:**
- Parse X.509 certificates (DER format)
- Verify certificate chain (root CA → intermediate → leaf)
- Check certificate validity (dates, revocation)
- Verify signature on handshake transcript

**Crypto Delegation:**
- Ed25519 signature verification → BearDog
- Certificate hash (fingerprint) → BearDog (Blake3)

**Simplification for v1.0:**
- Focus on Ed25519 certificates (no RSA/ECDSA)
- Use `webpki` crate for X.509 parsing (Pure Rust)
- Delegate signature verification to BearDog

**Estimated Complexity:** 300-500 lines

---

### 5. Alert Protocol

**Purpose:** Signal errors and warnings to peer.

**Alert Levels:**
- `warning (1)` - Recoverable errors
- `fatal (2)` - Connection must close

**Common Alerts:**
- `close_notify (0)` - Clean shutdown
- `unexpected_message (10)` - Protocol error
- `bad_record_mac (20)` - Authentication failure
- `handshake_failure (40)` - Negotiation failed
- `illegal_parameter (47)` - Invalid parameter
- `decode_error (50)` - Malformed message
- `decrypt_error (51)` - Decryption failed
- `protocol_version (70)` - Unsupported version
- `internal_error (80)` - Implementation error

**Estimated Complexity:** 100-200 lines

---

## 🎯 Implementation Phases

### Phase 1: Core Protocol Types (Week 1)

**Goal:** Define all TLS 1.3 message types and serialization.

**Tasks:**
1. Create `crates/songbird-tls/` module
2. Define message types:
   - `ClientHello`, `ServerHello`
   - `EncryptedExtensions`, `Certificate`, `CertificateVerify`
   - `Finished`
   - `Alert`
3. Implement TLS wire format serialization/deserialization
4. Unit tests for all message types

**Deliverables:**
- `crates/songbird-tls/src/messages/`
- `crates/songbird-tls/src/codec/`
- 50+ unit tests

**Estimated Time:** 4-5 days

---

### Phase 2: Record Layer (Week 2)

**Goal:** Implement record framing, encryption, and decryption.

**Tasks:**
1. Implement `RecordLayer` struct
2. Record framing (split into max 16KB fragments)
3. TLS 1.3 nonce construction (IV XOR sequence)
4. AAD construction (5-byte header)
5. Integrate BearDog crypto delegation
6. Handle padding and content type extraction

**Deliverables:**
- `crates/songbird-tls/src/record_layer.rs`
- Integration with BearDog crypto client
- 30+ unit tests
- 10+ integration tests

**Estimated Time:** 5-6 days

---

### Phase 3: Key Schedule (Week 3)

**Goal:** Implement HKDF-based key derivation.

**Tasks:**
1. Implement `KeySchedule` struct
2. HKDF-Extract delegation to BearDog
3. HKDF-Expand-Label delegation to BearDog
4. Traffic secret derivation
5. Key and IV extraction

**Deliverables:**
- `crates/songbird-tls/src/key_schedule.rs`
- 20+ unit tests (RFC test vectors)

**Estimated Time:** 3-4 days

---

### Phase 4: Handshake State Machine (Week 4-5)

**Goal:** Implement TLS 1.3 server handshake.

**Tasks:**
1. Implement `HandshakeStateMachine`
2. Parse ClientHello
3. Generate ServerHello (X25519 key share via BearDog)
4. Send EncryptedExtensions, Certificate, CertificateVerify
5. Sign handshake transcript (Ed25519 via BearDog)
6. Send/verify Finished messages
7. Transition to application data

**Deliverables:**
- `crates/songbird-tls/src/handshake/`
- End-to-end handshake tests
- 40+ unit tests
- 15+ integration tests

**Estimated Time:** 8-10 days

---

### Phase 5: Certificate Validation (Week 6)

**Goal:** Validate certificate chains.

**Tasks:**
1. Parse X.509 certificates (webpki)
2. Verify certificate chain
3. Check validity dates
4. Verify Ed25519 signatures via BearDog

**Deliverables:**
- `crates/songbird-tls/src/cert/`
- 15+ unit tests

**Estimated Time:** 3-4 days

---

### Phase 6: Integration & Testing (Week 7)

**Goal:** Integrate into Songbird, comprehensive testing.

**Tasks:**
1. Replace existing HTTP/TLS with Pure Songbird TLS
2. End-to-end HTTPS tests
3. Interoperability tests (curl, browsers)
4. Performance benchmarks
5. Chaos/fault testing

**Deliverables:**
- Full integration into `songbird-orchestrator`
- 50+ integration tests
- Performance report
- Security audit checklist

**Estimated Time:** 5-7 days

---

## 🔒 Security Considerations

### Threat Model

**In Scope:**
- Protocol-level attacks (replays, downgrade, MITM)
- Cryptographic weaknesses (weak ciphers, poor randomness)
- Implementation bugs (buffer overflows, panics)

**Out of Scope (Delegated to BearDog):**
- Side-channel attacks on crypto primitives
- Key material security
- Hardware security

### Security Properties

**Confidentiality:**
- All application data encrypted with ChaCha20-Poly1305
- Forward secrecy via ephemeral X25519 key exchange

**Integrity:**
- AEAD authentication tags prevent tampering
- Handshake transcript signatures (Ed25519)

**Authentication:**
- Server authenticated via certificate + signature
- Client authentication optional (mTLS)

### Cryptographic Agility

**Supported (v1.0):**
- TLS 1.3 only (no TLS 1.2 fallback)
- `TLS_CHACHA20_POLY1305_SHA256` cipher suite
- X25519 key exchange
- Ed25519 certificates

**Future Extensions:**
- Additional cipher suites (AES-GCM)
- Post-quantum key exchange (Kyber)
- Additional signature algorithms

---

## 📊 Performance Targets

### Handshake Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| Full Handshake | < 10ms | Includes BearDog round-trips |
| Resumed Session | < 2ms | Session tickets (future) |
| X25519 Key Exchange | < 1ms | BearDog delegation |
| Ed25519 Signing | < 0.5ms | BearDog delegation |

### Record Layer Performance

| Operation | Target | Notes |
|-----------|--------|-------|
| Encrypt 1KB | < 0.2ms | ChaCha20 very fast |
| Decrypt 1KB | < 0.2ms | ChaCha20 very fast |
| Throughput | > 1 GB/s | CPU-bound, parallel streams |

### Memory

| Resource | Target | Notes |
|----------|--------|-------|
| Per Connection | < 16 KB | State machine + buffers |
| Certificate Chain | < 4 KB | Ed25519 keys are small |

---

## 🧪 Testing Strategy

### Unit Tests (200+)

- Message serialization/deserialization
- Nonce construction
- Key derivation (RFC test vectors)
- State machine transitions

### Integration Tests (100+)

- Full handshake flows
- Record encryption/decryption
- Error handling (bad MAC, invalid cert)
- BearDog delegation (with mock)

### Interoperability Tests (50+)

- curl HTTPS requests
- Browser connections (Firefox, Chrome)
- openssl s_client
- TLS 1.3 test servers

### Performance Tests (20+)

- Handshake latency
- Throughput benchmarks
- Memory usage
- Connection concurrency

### Security Tests (30+)

- Downgrade attacks
- Replay attacks
- Invalid signatures
- Buffer boundary conditions

---

## 🚀 Migration Path

### From Current State (rustls Integration In-Progress)

**Step 1:** Pause rustls integration
- Document current progress (3/7 components done)
- Archive rustls-specific code to `archive/rustls-integration/`

**Step 2:** Create `songbird-tls` crate
- Pure Rust TLS 1.3 implementation
- No external TLS dependencies

**Step 3:** Parallel Development
- Keep existing `rustls` code working
- Build `songbird-tls` alongside
- Switch when feature-complete (Week 7)

**Step 4:** Cutover
- Feature flag: `--features pure-tls` (default)
- Deprecate rustls integration
- Remove rustls dependency (Q2 2026)

---

## 📈 Success Metrics

### Functional

- ✅ TLS 1.3 handshake with Ed25519 certificates
- ✅ Application data encryption/decryption
- ✅ Interoperability with standard clients (curl, browsers)
- ✅ All crypto delegated to BearDog (zero local crypto)

### Non-Functional

- ✅ 100% Pure Rust (zero C dependencies)
- ✅ < 10ms handshake latency
- ✅ > 1 GB/s throughput
- ✅ < 16 KB memory per connection
- ✅ 300+ tests passing
- ✅ Clean linter (zero warnings)

### Architectural

- ✅ Loose coupling to BearDog (capability discovery)
- ✅ Protocol-agnostic foundation
- ✅ Tower deployment ready
- ✅ ecoBin compliant (musl-static)

---

## 🎓 References

### Specifications

- [RFC 8446 - TLS 1.3](https://datatracker.ietf.org/doc/html/rfc8446)
- [RFC 5869 - HKDF](https://datatracker.ietf.org/doc/html/rfc5869)
- [RFC 8032 - Ed25519](https://datatracker.ietf.org/doc/html/rfc8032)
- [RFC 7748 - X25519](https://datatracker.ietf.org/doc/html/rfc7748)
- [RFC 8439 - ChaCha20-Poly1305](https://datatracker.ietf.org/doc/html/rfc8439)

### Implementation Guides

- [The Illustrated TLS 1.3 Connection](https://tls13.xargs.org/)
- [cloudflare/boring - TLS internals](https://github.com/cloudflare/boring)

### Related Projects (for reference only, NOT dependencies)

- `rustls` - TLS library (what we're NOT using)
- `webpki` - X.509 parsing (Pure Rust, we MAY use)
- `RustCrypto` - Crypto primitives (what BearDog uses)

---

## 🏆 Why This Approach Wins

### vs. rustls Integration

| Criterion | rustls Integration | Pure Songbird TLS | Winner |
|-----------|-------------------|-------------------|---------|
| Pure Rust | ❌ No (rustls → ring) | ✅ Yes (100%) | **Songbird** |
| C Dependencies | ❌ Yes (ring/aws-lc) | ✅ No (zero) | **Songbird** |
| API Control | ❌ Limited | ✅ Complete | **Songbird** |
| Protocol Agnostic | ❌ TLS-only | ✅ Extensible | **Songbird** |
| Tower Architecture | ⚠️ Partial | ✅ Perfect fit | **Songbird** |
| Deep Debt Solution | ❌ Workaround | ✅ Root fix | **Songbird** |
| Implementation Time | 1 week | 6-7 weeks | *rustls* |
| Long-term Ownership | ❌ External | ✅ Internal | **Songbird** |

**Verdict:** Pure Songbird TLS wins on EVERY criterion except short-term speed. Since we prioritize **deep debt solutions** over quick fixes, **Pure Songbird TLS is the clear winner**.

---

## 📝 Open Questions

1. **Client-side TLS?** - Start with server-only, add client in Q2 2026
2. **Session resumption?** - Not in v1.0, add in v1.1
3. **TLS 1.2 fallback?** - NO - TLS 1.3 only (security first)
4. **Additional cipher suites?** - ChaCha20 only in v1.0, expand later
5. **QUIC support?** - Future (HTTP/3), not in scope for v1.0

---

**Status:** ✅ APPROVED - Ready for Implementation  
**Next Step:** Create root tracking document and begin Phase 1  
**Timeline:** 6-7 weeks to production-ready Pure Songbird TLS  

🦀🐦✨ **Pure Rust Sovereignty - Own the Entire Stack!** ✨🐦🦀

