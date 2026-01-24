# Songbird Future Work Specification

**Version:** 5.20.0  
**Date:** January 24, 2026  
**Status:** 🎯 PLANNING - Post-HTTPS Success  
**Focus:** Hardening, Performance, Protocol Extensions  

---

## 🎯 Executive Summary

With TLS 1.3 HTTPS **100% working**, Songbird's next evolution focuses on:

1. **Security Hardening** - Certificate validation, constant-time ops
2. **Performance** - Session resumption, connection pooling
3. **Protocol Extensions** - HTTP/2, TLS 1.2 fallback
4. **Production Cleanup** - Remove diagnostic logging

---

## 🔴 Priority 1: Security Hardening

### Certificate Validation (CRITICAL)

**Current State:** Certificate parsing exists, but validation is NOT implemented.

**Required Implementation:**

```rust
// Current (INSECURE)
info!("📜 (Certificate validation not yet implemented - INSECURE!)");

// Target
pub async fn validate_certificate_chain(
    &self,
    certs: &[Certificate],
    hostname: &str,
) -> Result<()> {
    // 1. Parse X.509 certificates
    // 2. Verify signatures (each cert signed by next)
    // 3. Check validity periods
    // 4. Validate chain to trusted root
    // 5. Check hostname matches
    // 6. (Optional) OCSP/CRL revocation check
}
```

**Options:**
- **A: BearDog Integration** - Add `crypto.verify_certificate_chain` to BearDog
- **B: Pure Rust** - Use `x509-parser` + `webpki` crates (adds dependencies)

**Recommendation:** Option A - Keeps crypto in BearDog, maintains TRUE PRIMAL architecture.

### Constant-Time Operations

**Audit Required:** Ensure all comparisons in BearDog use constant-time functions.

```rust
// Bad (timing attack vulnerable)
if computed_tag == expected_tag { ... }

// Good (constant-time)
use subtle::ConstantTimeEq;
if computed_tag.ct_eq(&expected_tag).into() { ... }
```

### Zeroization

**Required:** Ensure all secret keys are zeroized after use.

```rust
use zeroize::{Zeroize, Zeroizing};

let mut key = Zeroizing::new(derive_key(...));
// use key...
// Automatically zeroized on drop
```

---

## 🟡 Priority 2: Performance

### Session Resumption (0-RTT)

**Current State:** NewSessionTicket is consumed but not stored.

**Required Implementation:**

```rust
pub struct SessionCache {
    tickets: HashMap<String, SessionTicket>,  // hostname -> ticket
}

impl SessionCache {
    pub fn store(&mut self, hostname: &str, ticket: SessionTicket);
    pub fn retrieve(&self, hostname: &str) -> Option<&SessionTicket>;
}

// In handshake:
if let Some(ticket) = cache.retrieve(hostname) {
    // Add pre_shared_key extension to ClientHello
    // Use PSK-based key derivation
}
```

**Benefits:**
- Skip full key exchange on reconnect
- 1-RTT or 0-RTT handshakes
- Significant latency reduction

### Connection Pooling

**Current State:** Each request creates a new TCP + TLS connection.

**Required Implementation:**

```rust
pub struct ConnectionPool {
    connections: HashMap<String, Vec<PooledConnection>>,
    max_per_host: usize,
    idle_timeout: Duration,
}

impl ConnectionPool {
    pub async fn get(&mut self, host: &str) -> Result<PooledConnection>;
    pub fn release(&mut self, conn: PooledConnection);
}
```

**Benefits:**
- Reuse existing TLS sessions
- Eliminate handshake overhead for repeat requests
- Better resource utilization

---

## 🟢 Priority 3: Protocol Extensions

### HTTP/2 Support

**Current State:** ALPN advertises "h2" but HTTP/2 not implemented.

**Required Implementation:**

```rust
pub mod http2 {
    pub struct Http2Connection { ... }
    pub struct Http2Stream { ... }
    
    impl Http2Connection {
        pub async fn send_request(&mut self, request: Request) -> Result<Response>;
        pub fn multiplex(&mut self) -> Http2Stream;
    }
}
```

**Features:**
- Binary framing
- Header compression (HPACK)
- Stream multiplexing
- Server push (optional)

### TLS 1.2 Fallback

**Current State:** Only TLS 1.3 supported.

**Required Implementation:**

```rust
pub enum TlsVersion {
    Tls12,
    Tls13,
}

impl TlsHandshake {
    pub async fn negotiate(&mut self) -> Result<TlsVersion> {
        match self.try_tls13().await {
            Ok(session) => Ok(TlsVersion::Tls13),
            Err(e) if e.is_version_error() => self.try_tls12().await,
            Err(e) => Err(e),
        }
    }
}
```

**Note:** TLS 1.2 requires different key derivation (PRF instead of HKDF).

---

## 🧹 Priority 4: Production Cleanup

### Remove Diagnostic Logging

**Current State:** Verbose hex dumps and diagnostic info in production code.

**Required Changes:**

```rust
// REMOVE (too verbose for production)
info!("First 16 bytes (hex): {}", hex::encode(&decrypted[..16]));
info!("UTF-8 preview (first 200 bytes): ...");

// KEEP (important events)
info!("✅ TLS 1.3 handshake complete in {:?}", total_time);
```

**Log Levels:**
- `error!` - Failures requiring attention
- `warn!` - Recoverable issues
- `info!` - Key events (connect, handshake complete)
- `debug!` - Protocol details
- `trace!` - Byte-level dumps (for debugging only)

### Configurable Timeouts

**Current State:** Hardcoded timeouts.

**Required Implementation:**

```rust
pub struct TlsConfig {
    pub handshake_timeout: Duration,    // Default: 5s
    pub read_timeout: Duration,         // Default: 30s
    pub write_timeout: Duration,        // Default: 30s
    pub post_handshake_timeout: Duration, // Default: 200ms
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            handshake_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            post_handshake_timeout: Duration::from_millis(200),
        }
    }
}
```

---

## 📋 Implementation Roadmap

### v5.21.0 - Production Cleanup
- [ ] Remove diagnostic hex dump logging
- [ ] Add configurable timeouts
- [ ] Run full test suite
- **ETA:** 1 day

### v5.22.0 - Security Hardening  
- [ ] Certificate validation (via BearDog)
- [ ] Constant-time operation audit
- [ ] Zeroization verification
- **ETA:** 3-5 days

### v5.23.0 - Performance
- [ ] Session resumption (0-RTT)
- [ ] Connection pooling
- **ETA:** 3-5 days

### v6.0.0 - Protocol Extensions
- [ ] HTTP/2 framing
- [ ] TLS 1.2 fallback
- **ETA:** 1-2 weeks

---

## 🧪 Testing Strategy

### Unit Tests
```bash
cargo test -p songbird-http-client --lib
```

### Integration Tests
```bash
cargo run --example test_https -- https://cloudflare.com
cargo run --example test_https -- https://google.com
cargo run --example test_https -- https://github.com
```

### Security Tests
- [ ] Invalid certificate rejection
- [ ] Expired certificate rejection
- [ ] Hostname mismatch rejection
- [ ] Revoked certificate rejection

### Performance Tests
- [ ] Session resumption latency
- [ ] Connection pool efficiency
- [ ] Concurrent request handling

---

## 📁 File Impact

| File | Changes |
|------|---------|
| `handshake_legacy.rs` | Remove diagnostic logs, add cert validation |
| `record.rs` | Remove hex dumps |
| `session.rs` | Add session cache |
| `client.rs` | Add connection pooling |
| `config.rs` | Add configurable timeouts |

---

## 🎯 Success Criteria

### v5.21.0
- [ ] No hex dumps in production logs
- [ ] All timeouts configurable
- [ ] All existing tests pass

### v5.22.0
- [ ] Rejects invalid certificates
- [ ] Constant-time operations verified
- [ ] No plaintext keys in memory after use

### v5.23.0
- [ ] 50%+ latency reduction on repeat connections
- [ ] Connection reuse working

### v6.0.0
- [ ] HTTP/2 working with multiplexing
- [ ] TLS 1.2 fallback for legacy servers

---

## 📚 References

- RFC 8446 - TLS 1.3
- RFC 7540 - HTTP/2
- RFC 5246 - TLS 1.2
- RFC 6066 - TLS Extensions

