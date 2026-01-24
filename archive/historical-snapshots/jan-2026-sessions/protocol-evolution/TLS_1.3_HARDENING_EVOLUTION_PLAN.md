# TLS 1.3 Hardening & Evolution Plan

**Date**: January 23, 2026  
**Version**: v5.12.0 → v6.0.0  
**Focus**: Harden and evolve existing TLS 1.3 implementation  
**Scope**: No TLS 1.2 yet - perfect what we have first

---

## 🎯 Philosophy: Perfect Before Expand

**Current Status**: v5.12.0
- ✅ Complete TLS 1.3 implementation
- ✅ Real-world validated (example.com, github.com)
- ✅ Production ready (A++ grade, 114/114 tests)

**Evolution Goal**: v6.0.0
- 🎯 Rock-solid reliability
- 🎯 Advanced TLS 1.3 features
- 🎯 Performance optimization
- 🎯 Enhanced security
- 🎯 Better observability

**Principle**: Master TLS 1.3 completely before adding TLS 1.2 backwards compatibility

---

## 📋 HARDENING ROADMAP

### Phase 1: Reliability & Robustness (1 week)

#### 1.1 Connection Resilience

**Goal**: Handle all edge cases gracefully

**Tasks**:
- [ ] **Timeout Management**
  ```rust
  pub struct TlsConfig {
      pub connection_timeout: Duration,    // TCP connect (default: 5s)
      pub handshake_timeout: Duration,     // TLS handshake (default: 10s)
      pub read_timeout: Duration,          // Read operations (default: 30s)
      pub write_timeout: Duration,         // Write operations (default: 30s)
  }
  ```

- [ ] **Retry Logic**
  ```rust
  pub struct RetryConfig {
      pub max_retries: usize,              // Default: 3
      pub base_delay: Duration,            // Default: 100ms
      pub max_delay: Duration,             // Default: 5s
      pub backoff_multiplier: f64,         // Default: 2.0 (exponential)
  }
  
  // Smart retry: Don't retry on permanent errors (bad certificate, etc.)
  fn is_retryable(error: &Error) -> bool {
      match error {
          Error::Io(_) => true,            // Network issues
          Error::Timeout(_) => true,       // Temporary
          Error::TlsHandshake(_) => false, // Likely permanent
          Error::Certificate(_) => false,  // Permanent
      }
  }
  ```

- [ ] **Circuit Breaker Pattern**
  ```rust
  pub struct CircuitBreaker {
      failure_threshold: usize,            // Open after N failures
      success_threshold: usize,            // Close after N successes
      timeout: Duration,                   // Try again after timeout
      state: Arc<RwLock<CircuitState>>,
  }
  
  enum CircuitState {
      Closed,       // Normal operation
      Open,         // Failing - reject requests immediately
      HalfOpen,     // Testing - allow limited requests
  }
  ```

**Tests**:
- [ ] Timeout scenarios (slow server, no response)
- [ ] Retry scenarios (transient failures)
- [ ] Circuit breaker scenarios (sustained failures)

**Files**:
- `crates/songbird-http-client/src/resilience/timeout.rs`
- `crates/songbird-http-client/src/resilience/retry.rs`
- `crates/songbird-http-client/src/resilience/circuit_breaker.rs`

#### 1.2 Error Handling Enhancement

**Goal**: Rich, actionable error information

**Tasks**:
- [ ] **Structured Errors**
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum TlsError {
      #[error("Handshake failed: {reason}")]
      HandshakeFailed {
          reason: String,
          server: String,
          attempted_version: TlsVersion,
          alert_code: Option<u8>,
      },
      
      #[error("Certificate validation failed: {reason}")]
      CertificateError {
          reason: String,
          server: String,
          cert_chain_length: usize,
      },
      
      #[error("Connection timeout: {phase}")]
      Timeout {
          phase: ConnectionPhase,  // Connect, Handshake, Read, Write
          duration: Duration,
      },
  }
  ```

- [ ] **Error Context Chain**
  ```rust
  // Track error context through call stack
  error!("Failed to establish connection")
      .context("TLS handshake failed")
      .context("Invalid server certificate")
      .context("Hostname mismatch: expected 'api.example.com', got 'other.com'")
  ```

- [ ] **Actionable Error Messages**
  ```rust
  // Current: "handshake failed"
  // Better: "TLS handshake with api.example.com failed: server sent Alert 40 (handshake_failure). This usually means the server doesn't support any of our cipher suites. Try enabling additional cipher suites in TlsConfig."
  ```

**Tests**:
- [ ] Error message clarity (human-readable)
- [ ] Error context preservation
- [ ] Actionable suggestions included

**Files**:
- `crates/songbird-http-client/src/error.rs` (enhance existing)

#### 1.3 Resource Management

**Goal**: No resource leaks, proper cleanup

**Tasks**:
- [ ] **Connection Pooling**
  ```rust
  pub struct ConnectionPool {
      max_idle_connections: usize,         // Per host
      max_connections: usize,              // Total
      idle_timeout: Duration,              // Reuse connections
      connections: HashMap<String, VecDeque<PooledConnection>>,
  }
  
  // Reuse TLS sessions (within TLS 1.3 session ticket lifetime)
  struct PooledConnection {
      stream: TcpStream,
      session_keys: SessionKeys,
      created_at: Instant,
      last_used: Instant,
  }
  ```

- [ ] **Graceful Shutdown**
  ```rust
  impl Drop for TlsConnection {
      fn drop(&mut self) {
          // Send close_notify before dropping
          // Don't wait for response (best effort)
          let _ = self.send_close_notify();
      }
  }
  ```

- [ ] **Memory Limits**
  ```rust
  pub struct TlsConfig {
      pub max_record_size: usize,          // Default: 16KB (RFC limit)
      pub max_handshake_size: usize,       // Default: 64KB
      pub max_certificate_chain: usize,    // Default: 10 certs
  }
  ```

**Tests**:
- [ ] Connection pooling (reuse, expiry)
- [ ] Graceful shutdown (close_notify sent)
- [ ] Memory limit enforcement

**Files**:
- `crates/songbird-http-client/src/pool/mod.rs` (new)
- `crates/songbird-http-client/src/pool/connection.rs`
- `crates/songbird-http-client/src/pool/manager.rs`

---

### Phase 2: Advanced TLS 1.3 Features (1 week)

#### 2.1 Session Resumption (TLS 1.3 Tickets)

**Goal**: 0-RTT handshakes for repeat connections

**Benefits**:
- First connection: Full handshake (~100ms)
- Repeat connections: 0-RTT (~20ms) - **5x faster!**
- Reduced server load
- Better user experience

**Implementation**:
```rust
pub struct SessionCache {
    /// Store session tickets by server hostname
    tickets: Arc<RwLock<HashMap<String, SessionTicket>>>,
    max_age: Duration,  // Default: 24 hours
}

struct SessionTicket {
    ticket: Vec<u8>,           // From NewSessionTicket message
    created_at: Instant,
    cipher_suite: u16,
    resumption_master_secret: Vec<u8>,
}

impl TlsHandshake {
    /// Try 0-RTT if we have a cached ticket
    async fn handshake_with_resumption(&mut self, ...) -> Result<SessionKeys> {
        if let Some(ticket) = self.session_cache.get(host) {
            // Send ClientHello with:
            // • pre_shared_key extension (ticket)
            // • early_data extension (0-RTT)
            // • Application data immediately (no wait!)
            
            match self.try_0rtt(host, ticket).await {
                Ok(keys) => return Ok(keys),  // 0-RTT success!
                Err(_) => {
                    // Server rejected 0-RTT, fall back to full handshake
                    warn!("0-RTT rejected, doing full handshake");
                    self.full_handshake(host).await
                }
            }
        } else {
            // No ticket, do full handshake
            self.full_handshake(host).await
        }
    }
}
```

**Tasks**:
- [ ] Implement NewSessionTicket parsing
- [ ] Session ticket storage (encrypted at rest)
- [ ] 0-RTT ClientHello construction
- [ ] Early data handling
- [ ] Replay protection (nonce tracking)
- [ ] Ticket expiry/cleanup

**Security Considerations**:
- ✅ Replay protection (must track nonces)
- ✅ Only for idempotent requests (GET, not POST)
- ✅ Ticket encryption (don't store plaintext)

**Tests**:
- [ ] Session ticket storage/retrieval
- [ ] 0-RTT handshake success
- [ ] 0-RTT rejection fallback
- [ ] Replay protection
- [ ] Ticket expiry

**Files**:
- `crates/songbird-http-client/src/tls/session_cache.rs` (new)
- `crates/songbird-http-client/src/tls/resumption.rs` (new)

#### 2.2 Post-Handshake Authentication

**Goal**: Re-authenticate after initial handshake

**Use Case**: Long-lived connections, changing permissions

**Implementation**:
```rust
impl TlsConnection {
    /// Request client certificate after handshake (if server supports)
    pub async fn request_post_handshake_auth(&mut self) -> Result<()> {
        // RFC 8446 Section 4.6.2: CertificateRequest after handshake
        // Server can request client cert after connection established
        
        // We send:
        // • Certificate message (if we have client cert)
        // • CertificateVerify message (sign transcript)
        
        todo!("Implement post-handshake auth")
    }
}
```

**Tasks**:
- [ ] CertificateRequest parsing (post-handshake)
- [ ] Client certificate loading
- [ ] Certificate message construction
- [ ] CertificateVerify signature

**Tests**:
- [ ] Post-handshake cert request
- [ ] Client cert presentation
- [ ] Certificate verification

**Files**:
- `crates/songbird-http-client/src/tls/post_handshake_auth.rs` (new)

#### 2.3 Key Update

**Goal**: Refresh encryption keys without new handshake

**Use Case**: Long-lived connections, forward secrecy

**Implementation**:
```rust
impl TlsConnection {
    /// Request key update (refresh encryption keys)
    pub async fn update_keys(&mut self) -> Result<()> {
        // RFC 8446 Section 4.6.3: KeyUpdate message
        // Derive new application traffic keys from current keys
        
        // Send KeyUpdate message
        self.send_key_update().await?;
        
        // Derive new keys
        let new_keys = self.derive_updated_keys().await?;
        
        // Switch to new keys
        self.keys = new_keys;
        
        Ok(())
    }
}
```

**Tasks**:
- [ ] KeyUpdate message construction
- [ ] Key update derivation (HKDF-Expand-Label)
- [ ] Synchronization (sender/receiver key update)

**Tests**:
- [ ] Key update request/response
- [ ] Key derivation correctness
- [ ] Traffic continues after update

**Files**:
- `crates/songbird-http-client/src/tls/key_update.rs` (new)

---

### Phase 3: Performance Optimization (1 week)

#### 3.1 Zero-Copy Operations

**Goal**: Minimize memory allocations and copies

**Tasks**:
- [ ] **Buffer Pooling**
  ```rust
  pub struct BufferPool {
      // Reuse Vec<u8> buffers instead of allocating
      buffers: Vec<Vec<u8>>,
      default_size: usize,  // 16KB (typical TLS record)
  }
  ```

- [ ] **Vectored I/O**
  ```rust
  // Send TLS record in one syscall (no intermediate buffer)
  stream.write_vectored(&[
      IoSlice::new(&record_header),
      IoSlice::new(&encrypted_data),
  ]).await?;
  ```

- [ ] **In-Place Encryption**
  ```rust
  // Encrypt directly into send buffer (no allocation)
  let mut buffer = self.buffer_pool.acquire();
  buffer.extend_from_slice(plaintext);
  self.cipher.encrypt_in_place(&mut buffer)?;
  stream.write_all(&buffer).await?;
  ```

**Benchmarks**:
- [ ] Memory allocations per request
- [ ] Throughput (requests/sec)
- [ ] Latency (p50, p95, p99)

**Files**:
- `crates/songbird-http-client/src/pool/buffer_pool.rs` (new)

#### 3.2 Async Optimization

**Goal**: Better concurrency, lower latency

**Tasks**:
- [ ] **Pipelining**
  ```rust
  // Send multiple requests without waiting for responses
  async fn pipeline_requests(&self, requests: Vec<Request>) -> Vec<Response> {
      let mut futures = Vec::new();
      for req in requests {
          futures.push(self.send_request(req));
      }
      futures::future::join_all(futures).await
  }
  ```

- [ ] **Parallel Handshakes**
  ```rust
  // Connect to multiple servers concurrently
  let handles = vec![
      tokio::spawn(connect("api1.example.com")),
      tokio::spawn(connect("api2.example.com")),
      tokio::spawn(connect("api3.example.com")),
  ];
  let results = futures::future::join_all(handles).await;
  ```

- [ ] **Adaptive Buffering**
  ```rust
  // Adjust buffer sizes based on observed traffic patterns
  struct AdaptiveBuffer {
      size: usize,
      growth_factor: f64,
      shrink_threshold: Duration,
  }
  ```

**Benchmarks**:
- [ ] Concurrent connections (throughput)
- [ ] Pipeline performance (latency)
- [ ] Buffer efficiency (memory usage)

**Files**:
- `crates/songbird-http-client/src/async_utils/mod.rs` (new)

#### 3.3 Crypto Optimization

**Goal**: Faster encryption/decryption

**Tasks**:
- [ ] **Batch Operations**
  ```rust
  // Encrypt multiple records in one BearDog RPC call
  let encrypted_records = beardog.encrypt_batch(vec![
      (plaintext1, nonce1, aad1),
      (plaintext2, nonce2, aad2),
      (plaintext3, nonce3, aad3),
  ]).await?;
  ```

- [ ] **Cipher Context Reuse**
  ```rust
  // Reuse cipher contexts instead of recreating
  struct CipherContext {
      key: Vec<u8>,
      cipher_suite: u16,
      // Cached cipher state (if applicable)
  }
  ```

- [ ] **Parallel Crypto**
  ```rust
  // Process multiple TLS records in parallel
  let futures = encrypted_records.iter()
      .map(|record| beardog.decrypt(record))
      .collect::<Vec<_>>();
  let decrypted = futures::future::join_all(futures).await;
  ```

**Benchmarks**:
- [ ] Encryption throughput (MB/s)
- [ ] Decryption throughput (MB/s)
- [ ] Crypto latency (microseconds)

**Files**:
- `crates/songbird-http-client/src/tls/crypto_batch.rs` (new)

---

### Phase 4: Security Enhancements (1 week)

#### 4.1 Certificate Validation

**Goal**: Verify server certificates (not just trust-on-first-use)

**Tasks**:
- [ ] **Certificate Chain Validation**
  ```rust
  pub struct CertificateValidator {
      root_ca_store: RootCAStore,
      verify_hostname: bool,
      verify_expiry: bool,
      verify_purpose: bool,  // serverAuth extended key usage
  }
  
  impl CertificateValidator {
      pub fn validate(&self, cert_chain: &[Certificate], hostname: &str) -> Result<()> {
          // 1. Build chain from leaf to root
          // 2. Verify each signature
          // 3. Check expiry dates
          // 4. Verify hostname matches
          // 5. Check extended key usage
          // 6. Check revocation (optional: OCSP, CRL)
      }
  }
  ```

- [ ] **Root CA Store**
  ```rust
  // Load system root CAs
  let root_store = RootCAStore::from_system()?;
  
  // Or custom roots
  let root_store = RootCAStore::from_pem_file("roots.pem")?;
  ```

- [ ] **Hostname Verification**
  ```rust
  // RFC 6125: Hostname verification
  fn verify_hostname(cert: &Certificate, hostname: &str) -> bool {
      // Check Common Name (CN)
      // Check Subject Alternative Names (SAN)
      // Support wildcards (*.example.com)
  }
  ```

- [ ] **Certificate Pinning** (Optional)
  ```rust
  pub struct TlsConfig {
      // Pin specific certificates for high-value targets
      pinned_certificates: HashMap<String, Vec<u8>>,  // hostname -> cert hash
  }
  ```

**Tasks**:
- [ ] Integrate with BearDog for signature verification
- [ ] Load system root CAs
- [ ] Parse X.509 certificates
- [ ] Validate certificate chains
- [ ] Hostname verification
- [ ] Expiry checking

**Tests**:
- [ ] Valid certificate chain
- [ ] Expired certificate
- [ ] Hostname mismatch
- [ ] Invalid signature
- [ ] Unknown CA

**Files**:
- `crates/songbird-http-client/src/tls/certificate/mod.rs` (new)
- `crates/songbird-http-client/src/tls/certificate/validator.rs`
- `crates/songbird-http-client/src/tls/certificate/root_ca.rs`

#### 4.2 Key Management

**Goal**: Secure key storage and rotation

**Tasks**:
- [ ] **Session Key Security**
  ```rust
  // Zero keys from memory when dropped
  impl Drop for SessionKeys {
      fn drop(&mut self) {
          zeroize::Zeroize::zeroize(&mut self.client_write_key);
          zeroize::Zeroize::zeroize(&mut self.server_write_key);
          // ... zeroize all key material
      }
  }
  ```

- [ ] **Automatic Key Rotation**
  ```rust
  struct KeyRotationPolicy {
      max_age: Duration,           // Rotate after time
      max_bytes: u64,              // Rotate after data volume
      max_messages: u64,           // Rotate after message count
  }
  ```

**Files**:
- `crates/songbird-http-client/src/tls/key_management.rs` (new)

#### 4.3 Security Policies

**Goal**: Configurable security policies

**Tasks**:
- [ ] **Minimum Security Level**
  ```rust
  pub enum SecurityLevel {
      Strict,      // TLS 1.3 only, modern ciphers, full validation
      Standard,    // TLS 1.3, all ciphers, basic validation
      Compatible,  // TLS 1.2+, all ciphers, TOFU
  }
  ```

- [ ] **Cipher Suite Restrictions**
  ```rust
  pub struct TlsConfig {
      allowed_cipher_suites: Vec<u16>,
      forbidden_cipher_suites: Vec<u16>,
  }
  ```

- [ ] **Server Requirements**
  ```rust
  pub struct ServerPolicy {
      require_perfect_forward_secrecy: bool,
      require_strong_ciphers: bool,  // AES-GCM, ChaCha20 only
      minimum_key_size: usize,       // e.g., 2048 for RSA
  }
  ```

**Files**:
- `crates/songbird-http-client/src/tls/policy.rs` (new)

---

### Phase 5: Observability (1 week)

#### 5.1 Comprehensive Metrics

**Goal**: Understand system behavior in production

**Tasks**:
- [ ] **Handshake Metrics**
  ```rust
  metrics::histogram!("tls.handshake.duration_ms", duration);
  metrics::increment_counter!("tls.handshake.success");
  metrics::increment_counter!("tls.handshake.failure", "reason" => reason);
  ```

- [ ] **Connection Metrics**
  ```rust
  metrics::gauge!("tls.connections.active", active_count);
  metrics::increment_counter!("tls.connections.total");
  metrics::histogram!("tls.connection.duration_seconds", duration);
  ```

- [ ] **Traffic Metrics**
  ```rust
  metrics::histogram!("tls.bytes.sent", bytes);
  metrics::histogram!("tls.bytes.received", bytes);
  metrics::histogram!("tls.records.sent", count);
  ```

- [ ] **Error Metrics**
  ```rust
  metrics::increment_counter!("tls.errors", 
      "type" => error_type,
      "server" => hostname
  );
  ```

**Integration**: Prometheus format

**Files**:
- `crates/songbird-http-client/src/metrics/mod.rs` (new)

#### 5.2 Distributed Tracing

**Goal**: Track requests across system boundaries

**Tasks**:
- [ ] **OpenTelemetry Integration**
  ```rust
  use opentelemetry::trace::{Span, Tracer};
  
  async fn handshake(&mut self, host: &str) -> Result<SessionKeys> {
      let span = tracer.start("tls_handshake");
      span.set_attribute("server", host);
      span.set_attribute("tls_version", "1.3");
      
      match self.do_handshake(host).await {
          Ok(keys) => {
              span.set_status(StatusCode::Ok);
              span.set_attribute("cipher_suite", format!("0x{:04x}", keys.cipher_suite));
              Ok(keys)
          }
          Err(e) => {
              span.set_status(StatusCode::Error);
              span.record_error(&e);
              Err(e)
          }
      }
  }
  ```

**Files**:
- `crates/songbird-http-client/src/tracing/mod.rs` (new)

#### 5.3 Debug Logging

**Goal**: Rich, structured logging for troubleshooting

**Tasks**:
- [ ] **Structured Logging**
  ```rust
  info!(
      server = %host,
      cipher_suite = format!("0x{:04x}", cipher_suite),
      duration_ms = duration.as_millis(),
      "TLS handshake complete"
  );
  ```

- [ ] **Log Levels**
  - ERROR: Failures, security issues
  - WARN: Degraded performance, retries
  - INFO: Successful operations
  - DEBUG: Detailed flow
  - TRACE: Full protocol details

**Files**:
- Enhance existing logging throughout codebase

---

## 📊 SUCCESS METRICS

### Performance Targets

**Handshake**:
- First connection: < 100ms (p95)
- With resumption: < 20ms (p95)
- Throughput: > 1000 handshakes/sec

**Data Transfer**:
- Encryption overhead: < 5%
- Decryption overhead: < 5%
- Throughput: > 1 GB/s

**Resource Usage**:
- Memory per connection: < 64 KB
- CPU per connection: < 1% of single core

### Reliability Targets

**Uptime**:
- Connection success rate: > 99.9%
- Recovery from failures: < 1 second
- Circuit breaker effectiveness: > 95%

**Error Handling**:
- Actionable error messages: 100%
- Error context preserved: 100%
- Retry success rate: > 80%

### Security Targets

**Certificate Validation**:
- Validation accuracy: 100%
- False positive rate: < 0.1%
- Performance overhead: < 10ms

**Key Management**:
- Key zeroization: 100%
- Session ticket security: 100%
- Replay attack prevention: 100%

---

## 🎯 PRIORITIZATION

### Must Have (Phase 1-2)
1. ✅ Reliability improvements (timeouts, retries, circuit breaker)
2. ✅ Error handling enhancement
3. ✅ Session resumption (0-RTT)

### Should Have (Phase 3-4)
4. ✅ Performance optimization
5. ✅ Certificate validation
6. ✅ Connection pooling

### Nice to Have (Phase 5)
7. ✅ Advanced metrics
8. ✅ Distributed tracing
9. ✅ Post-handshake auth
10. ✅ Key update

---

## 📅 TIMELINE

**Phase 1: Reliability** (1 week)
- Week 1: Timeouts, retries, circuit breaker, error handling

**Phase 2: Advanced TLS 1.3** (1 week)
- Week 2: Session resumption (0-RTT), post-handshake auth, key update

**Phase 3: Performance** (1 week)
- Week 3: Zero-copy, async optimization, crypto batching

**Phase 4: Security** (1 week)
- Week 4: Certificate validation, key management, policies

**Phase 5: Observability** (1 week)
- Week 5: Metrics, tracing, enhanced logging

**Total**: 5 weeks to comprehensive TLS 1.3 system

---

## ✅ CURRENT FOUNDATION

**What We Have (v5.12.0)**:
- ✅ Complete TLS 1.3 handshake
- ✅ Multiple cipher suites
- ✅ Adaptive learning
- ✅ Progressive fallback
- ✅ Multi-record handling
- ✅ Real-world validated

**Solid Base**: Everything above builds on this strong foundation!

---

## 🚀 NEXT STEPS

**Immediate Focus** (Start with Phase 1):
1. Implement timeout management
2. Add retry logic with exponential backoff
3. Build circuit breaker pattern
4. Enhance error messages

**Validation**:
- Add comprehensive tests for each phase
- Benchmark performance improvements
- Validate security enhancements
- Test against wide range of servers

**Documentation**:
- Update as we evolve
- Add examples for new features
- Document best practices
- Create migration guides

---

**Date**: January 23, 2026  
**Status**: Roadmap defined, ready to evolve  
**Philosophy**: Perfect TLS 1.3 before expanding to TLS 1.2  
**Timeline**: 5 weeks to comprehensive system

**LET'S BUILD THE BEST TLS 1.3 IMPLEMENTATION IN PURE RUST!** 🚀🔒✨

