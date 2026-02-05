# 🔒 Songbird TLS Evolution Analysis - January 26, 2026

## 📊 Executive Summary

| Capability | Current State | Complexity | Effort |
|------------|---------------|------------|--------|
| TLS 1.3 Client | ✅ 100% Complete | - | Done |
| TLS 1.2 Fallback | 🔜 Design needed | High | 40-60 hours |
| TLS Server Mode | 🚧 Foundation (20%) | Medium | 20-30 hours |
| TLS Relay/Proxy | 📋 Design only | High | 60-80 hours |
| WebSocket | 📋 Upgrade path exists | Low | 8-12 hours |
| Database TLS | 📋 Protocol varies | Medium | 20-40 hours per DB |

---

## 🔐 TLS 1.2 Fallback Analysis

### Why TLS 1.2?

**Remaining 6% of sites** (from analysis):
- `news.ycombinator.com` - TLS 1.2 only (protocol_version alert)
- `elastic.co` - TLS 1.2 only
- `postgresql.org` - TLS 1.2 only (handshake_failure)

### Security Considerations

**TLS 1.2 vs 1.3 Security Differences**:

| Aspect | TLS 1.3 | TLS 1.2 |
|--------|---------|---------|
| Key Exchange | ECDHE only (forward secrecy) | RSA, DHE, ECDHE |
| Cipher Suites | AEAD only (3 suites) | 300+ including weak |
| Round Trips | 1-RTT (0-RTT optional) | 2-RTT minimum |
| Renegotiation | Removed | Vulnerable if misused |
| Record Padding | Mandatory | Optional |
| Downgrade Protection | Built-in | Server-dependent |

**Songbird's Secure 1.2 Approach** (if implemented):
```rust
// ONLY allow secure TLS 1.2 cipher suites
const TLS_1_2_SECURE_CIPHERS: &[u16] = &[
    0xC02F, // TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
    0xC030, // TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
    0xCCA8, // TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
    0xC02B, // TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
    0xC02C, // TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
    0xCCA9, // TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256
];
// NO RSA key exchange (no forward secrecy)
// NO CBC mode ciphers (BEAST, POODLE vulnerable)
// NO RC4, DES, 3DES
```

### Architecture Changes Required

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        TLS Version Negotiation                          │
├─────────────────────────────────────────────────────────────────────────┤
│  ClientHello                                                            │
│  ├─ legacy_version: 0x0303 (TLS 1.2)                                   │
│  ├─ supported_versions extension: [0x0304, 0x0303]  ← NEW: Add 1.2     │
│  └─ cipher_suites: TLS 1.3 + TLS 1.2 secure only                       │
│                                                                         │
│  Server Response                                                        │
│  ├─ If supported_versions contains 0x0304 → TLS 1.3 handshake         │
│  └─ Else version 0x0303 → TLS 1.2 handshake (different flow!)         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Implementation Components

1. **Version Negotiation** (`tls/negotiation.rs`)
   - Parse ServerHello to detect TLS 1.2 vs 1.3
   - Currently exists but needs completion

2. **TLS 1.2 Handshake Flow** (NEW: `tls/handshake_1_2/`)
   ```
   Client → Server: ClientHello (with 1.2 ciphers)
   Server → Client: ServerHello, Certificate, ServerKeyExchange, ServerHelloDone
   Client → Server: ClientKeyExchange, ChangeCipherSpec, Finished
   Server → Client: ChangeCipherSpec, Finished
   ```

3. **Key Exchange** (BearDog additions needed)
   - `tls.ecdhe_key_exchange` - Ephemeral ECDHE
   - `tls.compute_master_secret` - PRF for TLS 1.2
   - `tls.compute_key_block` - Key expansion

4. **Record Layer Differences**
   - TLS 1.2: MAC-then-encrypt or AEAD
   - Sequence numbers in explicit IV (for AEAD)

### Estimated Effort Breakdown

| Component | Hours | Notes |
|-----------|-------|-------|
| Version negotiation | 4 | Detect 1.2 vs 1.3 response |
| ClientHello extensions | 8 | Add 1.2 cipher suites |
| ServerHello parsing | 8 | Different format than 1.3 |
| Key exchange (ECDHE) | 12 | BearDog capability needed |
| Certificate parsing | 8 | Reuse from 1.3 |
| Finished verification | 8 | Different PRF |
| Record layer | 12 | Different AEAD construction |
| **Total** | **60** | Conservative estimate |

---

## 🖥️ TLS Server Mode Analysis

### Current State

**Foundation exists** in `tls/server.rs` and `tls/server_complete.rs`:
- ✅ Basic structure for accepting connections
- ✅ Transcript tracking (same as client)
- ✅ ClientHello reading
- ❌ ServerHello generation (stub)
- ❌ Key derivation (needs completion)
- ❌ Certificate handling
- ❌ Encrypted handshake messages

### Architecture (Reuse Client Logic!)

```rust
// The key insight: Server reuses 90% of client code!
pub struct TlsServer {
    // SAME crypto provider as client
    crypto: Arc<dyn CryptoCapability>,
    
    // SAME transcript tracking
    transcript: TranscriptManager,
    
    // SAME key derivation
    // SAME record layer
    
    // Server-specific: Certificate + private key
    cert_chain: CertificateChain,
    private_key: PrivateKey,
}

impl TlsServer {
    pub async fn accept(&mut self, stream: &mut TcpStream) -> Result<SessionKeys> {
        // 1. Read ClientHello (already implemented)
        let client_hello = self.read_client_hello(stream).await?;
        self.transcript.add(&client_hello);
        
        // 2. Generate our key pair
        let (our_private, our_public) = self.crypto.generate_x25519_keypair().await?;
        
        // 3. Extract client's key share
        let client_key_share = parse_key_share(&client_hello)?;
        
        // 4. Compute shared secret (SAME as client!)
        let shared_secret = self.crypto.x25519_dh(&our_private, &client_key_share).await?;
        
        // 5. Send ServerHello
        let server_hello = self.build_server_hello(&our_public)?;
        self.transcript.add(&server_hello);
        stream.write_all(&server_hello).await?;
        
        // 6. Derive handshake keys (SAME as client!)
        let handshake_keys = self.crypto.tls_derive_handshake_secrets(...).await?;
        
        // 7. Send encrypted handshake messages
        // ... EncryptedExtensions, Certificate, CertificateVerify, Finished
        
        // 8. Derive application keys (SAME as client!)
        let app_keys = self.crypto.tls_derive_application_secrets(...).await?;
        
        Ok(app_keys)
    }
}
```

### BearDog Capabilities Needed

| Capability | Status | Notes |
|------------|--------|-------|
| `crypto.x25519_generate_keypair` | ✅ Exists | Key generation |
| `crypto.x25519_dh` | ✅ Exists | DH computation |
| `tls.derive_handshake_secrets` | ✅ Exists | Same as client |
| `tls.derive_application_secrets` | ✅ Exists | Same as client |
| `crypto.sign_ecdsa` | ❓ May need | For CertificateVerify |
| `crypto.load_certificate` | 🔜 New | Load cert from config |
| `crypto.load_private_key` | 🔜 New | Load key from config |

### Estimated Effort

| Component | Hours | Notes |
|-----------|-------|-------|
| ClientHello parsing | 8 | Extract extensions |
| ServerHello generation | 6 | Already started |
| Key exchange | 4 | Mostly reuse client |
| EncryptedExtensions | 4 | Simple message |
| Certificate message | 6 | Format our cert chain |
| CertificateVerify | 8 | Sign transcript hash |
| Finished message | 4 | Same as client |
| Application data | 4 | Reuse record layer |
| Certificate management | 8 | Load/store certs |
| **Total** | **52** | But ~20h is reuse |

### Net New Work: ~25-30 hours

---

## 🔄 TLS Relay/Proxy Mode Analysis

### Use Cases

1. **Terminating Proxy** (L7)
   - Songbird terminates TLS from client
   - Re-encrypts to upstream
   - Can inspect/modify HTTP

2. **Transparent Proxy** (L4)
   - Songbird passes TLS through
   - No decryption
   - SNI-based routing only

3. **mTLS Gateway**
   - Client authenticates to Songbird with cert
   - Songbird authenticates to upstream
   - Primal-to-primal secure communication

### Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        TLS Relay Architecture                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Terminating Proxy (Full Inspection)                                     │
│  ┌─────────┐     TLS 1.3      ┌───────────┐     TLS 1.3      ┌────────┐ │
│  │ Client  │ ◄──────────────► │ Songbird  │ ◄──────────────► │ Server │ │
│  └─────────┘                  │  (decrypt │                  └────────┘ │
│                               │  /inspect │                              │
│                               │  /encrypt)│                              │
│                               └───────────┘                              │
│                                                                          │
│  Transparent Proxy (SNI Routing)                                         │
│  ┌─────────┐     TLS 1.3      ┌───────────┐     TLS 1.3      ┌────────┐ │
│  │ Client  │ ◄──────────────► │ Songbird  │ ◄──────────────► │ Server │ │
│  └─────────┘                  │  (SNI peek │                  └────────┘ │
│                               │  /forward) │                              │
│                               └───────────┘                              │
│                                                                          │
│  mTLS Gateway (Mutual Auth)                                              │
│  ┌─────────┐     mTLS         ┌───────────┐     mTLS         ┌────────┐ │
│  │ Primal  │ ◄──────────────► │ Songbird  │ ◄──────────────► │ Primal │ │
│  │ Client  │   (client cert)  │  Gateway  │   (both certs)   │ Server │ │
│  └─────────┘                  └───────────┘                  └────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Implementation Components

```rust
pub enum RelayMode {
    /// Full termination - decrypt, inspect, re-encrypt
    Terminating {
        /// Our certificate for client-facing
        client_cert: CertificateChain,
        /// Certificate verification for upstream
        upstream_verify: VerifyMode,
    },
    
    /// Transparent pass-through - SNI routing only
    Transparent {
        /// SNI-based routing rules
        routes: Vec<SniRoute>,
    },
    
    /// mTLS gateway - mutual auth both sides
    MutualTls {
        /// Our certificate
        our_cert: CertificateChain,
        /// Required client certificate attributes
        client_requirements: ClientCertPolicy,
        /// Upstream mTLS config
        upstream_mtls: UpstreamMtlsConfig,
    },
}

pub struct TlsRelay {
    mode: RelayMode,
    client_handler: TlsServer,   // Reuse server mode!
    upstream_client: SongbirdHttpClient, // Reuse client mode!
}
```

### Estimated Effort

| Component | Hours | Notes |
|-----------|-------|-------|
| Terminating proxy | 20 | Reuse server + client |
| Transparent proxy | 16 | SNI parsing + forwarding |
| mTLS client auth | 24 | Certificate request/verify |
| mTLS upstream | 16 | Client cert sending |
| Routing rules | 12 | SNI/path-based |
| Connection pooling | 16 | Upstream connection reuse |
| **Total** | **~80** | High estimate |

---

## 🌐 Other Subsystem Capabilities

### 1. WebSocket Secure (WSS)

**Current State**: HTTP HTTPS works → WSS is straightforward!

```rust
// WSS is just: TLS handshake + HTTP upgrade
impl SongbirdHttpClient {
    pub async fn websocket(&self, url: &str) -> Result<WebSocketConnection> {
        // 1. TLS handshake (ALREADY WORKS!)
        let (stream, keys) = self.tls_connect(url).await?;
        
        // 2. HTTP upgrade request
        let upgrade = self.send_websocket_upgrade(&stream).await?;
        
        // 3. Verify 101 Switching Protocols
        if upgrade.status != 101 { return Err(...); }
        
        // 4. Return WebSocket connection over TLS stream
        Ok(WebSocketConnection::new(stream, keys))
    }
}
```

**Effort**: 8-12 hours (HTTP upgrade + frame handling)

### 2. Database TLS

#### PostgreSQL

PostgreSQL uses a TLS upgrade mechanism:
```
Client → Server: SSLRequest (8 bytes)
Server → Client: 'S' (ready for TLS) or 'N' (no TLS)
< TLS Handshake >
< PostgreSQL protocol over TLS >
```

**Effort**: 20 hours (protocol understanding + integration)

#### MySQL

MySQL TLS is different:
```
Server → Client: Initial Handshake Packet
Client → Server: SSL Request Packet
< TLS Handshake >
< MySQL protocol over TLS >
```

**Effort**: 25 hours (different handshake flow)

#### MongoDB

MongoDB uses standard TLS on connect:
```
< TLS Handshake on connect >
< MongoDB Wire Protocol over TLS >
```

**Effort**: 15 hours (simpler, standard TLS)

### 3. gRPC over TLS

**Note**: gRPC typically requires HTTP/2. We currently support HTTP/1.1.

Options:
1. **HTTP/2 support** - Significant effort (~100 hours)
2. **gRPC-Web** - HTTP/1.1 compatible (~30 hours)
3. **Defer** - Use JSON-RPC/tarpc (our current approach)

**Recommendation**: Defer gRPC, continue with JSON-RPC/tarpc

### 4. MQTT over TLS (Port 8883)

MQTT is simpler than databases:
```
< TLS Handshake >
< MQTT Protocol over TLS >
```

**Effort**: 10-15 hours (standard TLS + MQTT framing)

---

## 📋 Recommended Evolution Path

### Phase 1: Server Mode (Priority: HIGH)
**Why**: Enables primal-to-primal secure communication
**Effort**: 25-30 hours
**Dependencies**: None (uses existing BearDog)

### Phase 2: TLS 1.2 Fallback (Priority: MEDIUM)
**Why**: 100% web compatibility
**Effort**: 40-60 hours
**Dependencies**: BearDog needs 1.2 PRF capabilities

### Phase 3: WebSocket (Priority: MEDIUM)
**Why**: Real-time primal communication
**Effort**: 8-12 hours
**Dependencies**: HTTP client works (✅)

### Phase 4: Database TLS (Priority: LOW)
**Why**: Pure Rust database connections
**Effort**: 15-25 hours per database
**Dependencies**: Server mode helps (similar patterns)

### Phase 5: Relay/Proxy (Priority: LOW)
**Why**: Gateway functionality
**Effort**: 60-80 hours
**Dependencies**: Server mode complete

---

## 🔧 Immediate Next Steps

### Option A: Server Mode Focus
```bash
# 1. Complete TlsServer implementation
# 2. Test with simple HTTP server
# 3. Integrate with capability system
```

### Option B: TLS 1.2 Focus
```bash
# 1. Add TLS 1.2 cipher suites to ClientHello
# 2. Implement version detection in ServerHello
# 3. Create handshake_1_2 module
# 4. Request BearDog 1.2 PRF capabilities
```

### Option C: Hybrid Approach
```bash
# 1. Add TLS 1.2 cipher suites (minimal effort)
# 2. Detect 1.2 response and log warning
# 3. Continue server mode in parallel
```

---

## 📊 Summary Matrix

| Feature | Impact | Effort | Dependencies | Priority |
|---------|--------|--------|--------------|----------|
| TLS Server | High (primal comms) | 30h | None | P0 |
| TLS 1.2 | Medium (6% sites) | 60h | BearDog | P1 |
| WebSocket | Medium (real-time) | 12h | None | P2 |
| mTLS Relay | Medium (gateway) | 80h | Server mode | P3 |
| PostgreSQL TLS | Low (one DB) | 20h | Server patterns | P4 |
| MongoDB TLS | Low (one DB) | 15h | Standard TLS | P4 |

---

**Created**: January 26, 2026
**Author**: Songbird Evolution Analysis
**Status**: Analysis Complete - Ready for Implementation Decisions

