# 🐦🐕 Tower Atomic HTTP Evolution - Songbird Implementation Plan

**Date**: January 21, 2026  
**Status**: 🎯 **READY TO IMPLEMENT**  
**Priority**: 🔴 **CRITICAL** - Enables Squirrel AI Integration  
**Timeline**: 1-2 weeks (coordinated with BearDog)

---

## 🎯 MISSION

**Replace reqwest with Pure Rust HTTP/HTTPS client using BearDog crypto delegation.**

**Result**: Zero C dependencies, TRUE Pure Rust networking stack.

---

## 📊 CURRENT STATE ANALYSIS

### **Dependencies Audit**:

**Current (PROBLEMATIC)**:
```toml
reqwest = { version = "0.11", features = ["json"], default-features = false }
```

**Problem**: Even without rustls-tls, reqwest has transitive C dependencies.

**Used In**:
- `crates/songbird-orchestrator/src/ipc/unix_socket.rs:900` (handle_http_request)
- Other orchestrator modules

**Must Replace**: YES

---

## 🏗️ ARCHITECTURE

### **New Stack**:

```
┌──────────────────────────────────────────────────┐
│              Squirrel (AI Router)                │
│    query_ai("Hello") via JSON-RPC                │
└───────────────────┬──────────────────────────────┘
                    │ Unix Socket
                    │ discover_capabilities + http.request
┌───────────────────▼──────────────────────────────┐
│            Songbird (HTTP Delegator)             │
│  ┌────────────────────────────────────────────┐  │
│  │   songbird-http-client (NEW CRATE)        │  │
│  │   - hyper (HTTP/1.1, HTTP/2)              │  │
│  │   - Custom TLS 1.3 implementation         │  │
│  │   - BearDog crypto delegation via RPC     │  │
│  └────────────────┬───────────────────────────┘  │
└───────────────────┼──────────────────────────────┘
                    │ Unix Socket RPC
                    │ tls.*, crypto.* methods
┌───────────────────▼──────────────────────────────┐
│         BearDog (Crypto Operations)              │
│  - tls.derive_secrets                            │
│  - tls.sign_handshake                            │
│  - tls.verify_certificate                        │
│  - crypto.ecdh_derive                            │
│  - crypto.encrypt/decrypt                        │
└──────────────────────────────────────────────────┘
                    │
                    │ Pure Rust Crypto
                    ▼
┌──────────────────────────────────────────────────┐
│         External API (Anthropic, etc.)           │
│              HTTPS (TLS 1.3)                     │
└──────────────────────────────────────────────────┘
```

---

## 📋 IMPLEMENTATION PLAN

### **Phase 1: Crate Setup** (Day 1)

**Create**: `crates/songbird-http-client/`

**Structure**:
```
crates/songbird-http-client/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── client.rs        # HTTP/HTTPS client
│   ├── tls/
│   │   ├── mod.rs       # TLS module
│   │   ├── handshake.rs # TLS 1.3 handshake
│   │   ├── codec.rs     # TLS record layer
│   │   └── session.rs   # TLS session management
│   ├── error.rs         # Error types
│   └── types.rs         # Request/Response types
├── examples/
│   ├── simple_get.rs    # Basic GET request
│   └── tls_handshake.rs # TLS test
└── tests/
    ├── unit/
    ├── e2e/
    └── integration/
```

**Dependencies** (Pure Rust ONLY):
```toml
[dependencies]
hyper = { version = "1.0", features = ["client", "http1", "http2"] }
hyper-util = { version = "0.1", features = ["client", "client-legacy", "tokio"] }
tokio = { workspace = true, features = ["net", "io-util", "rt"] }
tower = "0.4"
http = "1.0"
http-body-util = "0.1"
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tracing = { workspace = true }
thiserror = "1.0"
base64 = "0.21"
bytes = "1.0"

# NO reqwest, NO rustls, NO ring!
```

---

### **Phase 2: BearDog RPC Client** (Day 2)

**File**: `src/beardog_client.rs`

**Purpose**: JSON-RPC client for BearDog crypto operations

```rust
/// BearDog RPC client for crypto operations
pub struct BearDogClient {
    socket_path: PathBuf,
}

impl BearDogClient {
    /// Connect to BearDog via Unix socket
    pub async fn connect(socket_path: &str) -> Result<Self>;
    
    // TLS-specific methods
    pub async fn tls_derive_secrets(&self, params: TlsDeriveParams) -> Result<TlsSecrets>;
    pub async fn tls_sign_handshake(&self, message: &[u8]) -> Result<Vec<u8>>;
    pub async fn tls_verify_certificate(&self, chain: &[Vec<u8>]) -> Result<CertInfo>;
    
    // Crypto methods
    pub async fn ecdh_derive(&self, our_key: &[u8], their_key: &[u8]) -> Result<Vec<u8>>;
    pub async fn encrypt(&self, plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>>;
    pub async fn decrypt(&self, ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>>;
}
```

---

### **Phase 3: TLS 1.3 Implementation** (Days 3-5)

**File**: `src/tls/handshake.rs`

**TLS 1.3 Handshake Flow**:
```rust
/// Pure Rust TLS 1.3 client using BearDog crypto
pub struct TlsClient {
    beardog: BearDogClient,
    tcp_stream: TcpStream,
    state: TlsState,
}

impl TlsClient {
    /// Perform TLS 1.3 handshake
    pub async fn handshake(&mut self, server_name: &str) -> Result<()> {
        // 1. Generate ephemeral x25519 key pair (via BearDog)
        let (our_priv, our_pub) = self.beardog.generate_keypair().await?;
        
        // 2. Send ClientHello
        self.send_client_hello(server_name, &our_pub).await?;
        
        // 3. Receive ServerHello + Certificate + CertificateVerify + Finished
        let server_hello = self.receive_server_hello().await?;
        let server_cert = self.receive_certificate().await?;
        
        // 4. Verify certificate via BearDog
        self.beardog.tls_verify_certificate(&server_cert.chain).await?;
        
        // 5. Derive shared secret via ECDH (via BearDog)
        let shared_secret = self.beardog.ecdh_derive(
            &our_priv,
            &server_hello.public_key
        ).await?;
        
        // 6. Derive TLS secrets (via BearDog)
        let secrets = self.beardog.tls_derive_secrets(TlsDeriveParams {
            pre_master_secret: shared_secret,
            client_random: self.client_random,
            server_random: server_hello.random,
            cipher_suite: "TLS_CHACHA20_POLY1305_SHA256",
        }).await?;
        
        // 7. Send Finished message (encrypted with derived keys)
        self.send_finished(&secrets).await?;
        
        // 8. Receive Finished message
        self.receive_finished(&secrets).await?;
        
        // 9. Handshake complete - ready for application data
        self.state = TlsState::Connected(secrets);
        Ok(())
    }
    
    /// Encrypt application data
    pub async fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let secrets = self.state.secrets()?;
        self.beardog.encrypt(plaintext, &secrets.client_write_key).await
    }
    
    /// Decrypt application data
    pub async fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let secrets = self.state.secrets()?;
        self.beardog.decrypt(ciphertext, &secrets.server_write_key).await
    }
}
```

---

### **Phase 4: HTTP Client** (Days 6-7)

**File**: `src/client.rs`

```rust
/// Pure Rust HTTP/HTTPS client using BearDog crypto
pub struct SongbirdHttpClient {
    beardog_socket: PathBuf,
}

impl SongbirdHttpClient {
    /// Create new HTTP client
    pub fn new(beardog_socket: impl Into<PathBuf>) -> Self {
        Self {
            beardog_socket: beardog_socket.into(),
        }
    }
    
    /// Make HTTP/HTTPS request
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        // 1. Parse URL
        let uri: hyper::Uri = url.parse()?;
        let scheme = uri.scheme_str().unwrap_or("http");
        let host = uri.host().ok_or_else(|| Error::InvalidUrl)?;
        let port = uri.port_u16().unwrap_or(if scheme == "https" { 443 } else { 80 });
        
        // 2. Connect via TCP
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr).await?;
        
        // 3. If HTTPS, perform TLS handshake
        let io: Box<dyn AsyncReadWrite> = if scheme == "https" {
            let beardog = BearDogClient::connect(&self.beardog_socket).await?;
            let mut tls = TlsClient::new(beardog, tcp);
            tls.handshake(host).await?;
            Box::new(tls)
        } else {
            Box::new(tcp)
        };
        
        // 4. Build HTTP request
        let req = hyper::Request::builder()
            .method(method)
            .uri(url)
            .body(body.unwrap_or_default())?;
        
        // 5. Send request via hyper
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        
        // Spawn connection task
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                tracing::error!("Connection error: {}", e);
            }
        });
        
        // 6. Send request and get response
        let res = sender.send_request(req).await?;
        
        // 7. Parse response
        let status = res.status().as_u16();
        let headers = res.headers().iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
        let body = if let Ok(json) = serde_json::from_slice(&body_bytes) {
            json
        } else {
            serde_json::Value::String(String::from_utf8_lossy(&body_bytes).to_string())
        };
        
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}
```

---

### **Phase 5: Integration** (Days 8-10)

**Update**: `crates/songbird-orchestrator/src/ipc/unix_socket.rs`

**Replace handle_http_request** (lines 880-970):

```rust
async fn handle_http_request(params: Option<Value>) -> Result<Value, JsonRpcError> {
    use songbird_http_client::SongbirdHttpClient;
    
    #[derive(Deserialize)]
    struct HttpRequestParams {
        method: String,
        url: String,
        #[serde(default)]
        headers: std::collections::HashMap<String, String>,
        #[serde(default)]
        body: Option<Value>,
    }
    
    let params: HttpRequestParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🌐 HTTP delegation (Pure Rust Tower Atomic): {} {}", params.method, params.url);
    
    // Get BearDog socket from environment
    let beardog_socket = std::env::var("SONGBIRD_SECURITY_PROVIDER")
        .unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());
    
    // Create Pure Rust HTTP client
    let client = SongbirdHttpClient::new(&beardog_socket);
    
    // Make request
    let response = client.request(
        &params.method,
        &params.url,
        params.headers,
        params.body.map(|b| serde_json::to_vec(&b).unwrap()),
    ).await
    .map_err(|e| JsonRpcError::internal_error(&format!("HTTP request failed: {}", e)))?;
    
    info!("✅ HTTP delegation complete (Tower Atomic): {} (status: {})", params.url, response.status);
    
    Ok(serde_json::json!({
        "status": response.status,
        "headers": response.headers,
        "body": response.body
    }))
}
```

**Remove reqwest**:
```toml
# DELETE from Cargo.toml:
# reqwest = { version = "0.11", ... }

# ADD:
songbird-http-client = { path = "../songbird-http-client" }
```

---

### **Phase 6: Testing** (Days 11-14)

**Unit Tests**: `tests/unit/`
- TLS record encoding/decoding
- HTTP request building
- Response parsing
- Error handling

**E2E Tests**: `tests/e2e/`
- HTTP GET to httpbin.org
- HTTP POST with body
- HTTPS with TLS 1.3
- Large responses
- Error conditions

**Integration Tests**: `tests/integration/`
- Songbird → BearDog → External API
- Mock BearDog responses
- Timeout handling
- Connection pooling

---

## 🎯 BEARDOG REQUIREMENTS

### **RPC Methods Needed**:

1. ✅ `crypto.generate_keypair` - Generate x25519 key pair
2. ✅ `crypto.ecdh_derive` - ECDH key exchange
3. ✅ `tls.derive_secrets` - Derive TLS session secrets
4. ✅ `tls.sign_handshake` - Sign TLS handshake
5. ✅ `tls.verify_certificate` - Verify cert chain
6. ✅ `crypto.encrypt` - ChaCha20-Poly1305
7. ✅ `crypto.decrypt` - ChaCha20-Poly1305

### **Performance Requirements**:

- Crypto operation: < 1ms
- TLS handshake total: < 10ms
- Full HTTP round-trip: < 100ms (local)

---

## 📊 SUCCESS CRITERIA

### **Songbird Deliverables**:

1. ✅ `songbird-http-client` crate created
2. ✅ Pure Rust TLS 1.3 client working
3. ✅ HTTP/HTTPS via hyper working
4. ✅ Zero reqwest dependency
5. ✅ Zero C dependencies confirmed
6. ✅ Tests passing (>90% coverage)
7. ✅ Documentation complete
8. ✅ Example usage documented

### **Integration Validation**:

1. ✅ `http.request` RPC uses new client
2. ✅ HTTPS to httpbin.org works
3. ✅ HTTPS to api.anthropic.com works
4. ✅ Squirrel → Songbird → BearDog → Anthropic end-to-end
5. ✅ Performance: < 5s total AI query latency
6. ✅ ecoBin cross-compilation works

---

## 🚧 IMPLEMENTATION PHASES

### **Phase 1: Foundation** (Days 1-2)
- Create crate structure
- BearDog RPC client
- Basic types and errors

### **Phase 2: TLS Core** (Days 3-5)
- TLS 1.3 handshake
- Record layer codec
- Session management

### **Phase 3: HTTP Layer** (Days 6-7)
- HTTP client via hyper
- Request/response handling
- Connection management

### **Phase 4: Integration** (Days 8-10)
- Update Songbird RPC handler
- Remove reqwest
- End-to-end testing

### **Phase 5: Validation** (Days 11-14)
- Comprehensive testing
- Performance validation
- Documentation
- Production readiness

---

## 📚 TECHNICAL REFERENCES

### **TLS 1.3**:
- RFC 8446: https://www.rfc-editor.org/rfc/rfc8446.html
- Handshake state machine
- Record protocol
- Key schedule

### **HTTP via hyper**:
- hyper 1.0 client API
- Connection pooling
- HTTP/2 support

### **BearDog Crypto**:
- x25519 (ECDH)
- ChaCha20-Poly1305 (AEAD)
- ed25519 (signatures)
- BLAKE3 (hashing)

---

## 🎊 IMPACT

### **Immediate**:
- ✅ Unblocks Squirrel AI integration
- ✅ TRUE Pure Rust networking
- ✅ Zero C dependencies

### **Long-term**:
- ✅ Reference implementation for all primals
- ✅ ecoBin compliance at scale
- ✅ Tower Atomic pattern validated
- ✅ Crypto delegation architecture proven

---

## 📋 NEXT STEPS

### **Today** (Jan 21):
1. ✅ Review this plan
2. ⏳ Create `songbird-http-client` crate skeleton
3. ⏳ Define BearDog RPC interface
4. ⏳ Set up test infrastructure

### **Tomorrow** (Jan 22):
1. ⏳ Implement BearDog RPC client
2. ⏳ Start TLS handshake logic
3. ⏳ Coordinate with BearDog team

### **This Week**:
1. ⏳ TLS 1.3 implementation
2. ⏳ HTTP client via hyper
3. ⏳ Unit tests

### **Next Week**:
1. ⏳ Integration with Songbird
2. ⏳ End-to-end testing
3. ⏳ Performance validation
4. ⏳ Production deployment

---

**🐦🐕✨ TOWER ATOMIC HTTP - PURE RUST FUTURE! ✨🐕🐦**

---

*Plan Created: January 21, 2026*  
*Status: Ready to implement*  
*Timeline: 1-2 weeks*  
*Impact: CRITICAL for AI integration*

