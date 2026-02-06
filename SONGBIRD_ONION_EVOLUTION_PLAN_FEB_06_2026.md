# 🧅 Songbird Onion Service - Pure Rust Evolution Plan

**Date**: February 6, 2026  
**Status**: Investigation Complete - Ready for Implementation  
**Priority**: HIGH - Achieves TRUE Pure Rust sovereignty  
**Approach**: Build our own, like we did with TLS

---

## 🎯 Executive Summary

### Strategic Decision: Build Our Own

**Rationale**: Same reason we built custom TLS instead of using `rustls`:
- **Arti has C dependencies** (SQLite, possibly OpenSSL via native-tls)
- **We have Pure Rust crypto primitives** (via BearDog + RustCrypto)
- **We only need 20% of Tor** (onion services, not full anonymity network)
- **Full control** over security, privacy, and evolution

### What We Actually Need

```
NOT:
❌ Full Tor network access (guard/middle/exit relays)
❌ Onion routing (3-hop circuits)
❌ Directory authorities
❌ Consensus documents
❌ Bridge relays
❌ Pluggable transports

YES:
✅ Onion service protocol (reachable address without port forward)
✅ X25519 key exchange (already have via BearDog)
✅ Ed25519 signing (already have via ed25519-dalek)
✅ ChaCha20-Poly1305 encryption (already have)
✅ SHA3 hashing (can add Pure Rust sha3 crate)
```

**Scope**: ~10% of Tor's complexity, 100% of our use case

---

## 🦀 Our Pure Rust Crypto Stack (Already Built!)

### Current Capabilities

| Primitive | Library | Location | Status |
|-----------|---------|----------|--------|
| **Ed25519** | `ed25519-dalek = "2.1"` | songbird-tls | ✅ Complete |
| **X25519** | `x25519-dalek = "2.0"` | orchestrator | ✅ Complete |
| **ChaCha20-Poly1305** | `chacha20poly1305 = "0.10"` | orchestrator | ✅ Complete |
| **AES-GCM** | `aes-gcm = "0.10"` | orchestrator | ✅ Complete |
| **HMAC-SHA256** | `hmac + sha2 = "0.12 + 0.10"` | orchestrator | ✅ Complete |
| **SHA3** | N/A | - | ⚠️ Need to add |
| **BLAKE3** | Via BearDog | - | ✅ Available |

**Result**: We already have 90% of crypto primitives needed for onion services!

### What We Need to Add

```toml
# Pure Rust additions
sha3 = "0.10"           # For Tor's SHA3-256 usage
curve25519-dalek = "4"  # Underlying curve ops (already transitive dep)
```

**Binary impact**: ~50KB (sha3 only)

---

## 🏗️ Proposed Architecture: "Songbird Onion Service"

### Core Concept

**Minimal onion service** protocol:
1. Generate Ed25519 identity key (our device ID)
2. Derive .onion address from public key (32 chars base32)
3. Listen on local port, wrap in Tor protocol
4. Allow family devices to connect via .onion address

**NOT**: Full Tor network participation (guards, circuits, consensus)  
**INSTEAD**: Direct encrypted connections using .onion addresses as IDs

### File Structure

```
crates/songbird-sovereign-onion/  # NEW crate
├── Cargo.toml
└── src/
    ├── lib.rs              # Entry point
    ├── error.rs            # Error types
    ├── keys.rs             # Ed25519 key management
    ├── address.rs          # .onion address derivation
    ├── protocol.rs         # Minimal Tor cell protocol
    ├── service.rs          # Onion service (listening)
    ├── connector.rs        # Connect to onion addresses
    └── crypto.rs           # Crypto via BearDog
```

---

## 📐 Technical Design

### 1. Onion Address Generation

**Tor v3 Onion Address Format**:
```
{base32(pubkey || checksum || version)}.onion
```

**Implementation**:
```rust
use ed25519_dalek::{SigningKey, VerifyingKey};
use sha3::{Sha3_256, Digest};

pub struct OnionIdentity {
    signing_key: SigningKey,
    public_key: VerifyingKey,
    onion_address: String,
}

impl OnionIdentity {
    /// Generate new onion identity
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let public_key = signing_key.verifying_key();
        
        // Tor v3 address: base32(pubkey || checksum || version)
        let mut data = Vec::new();
        data.extend_from_slice(public_key.as_bytes());
        
        // Checksum: SHA3-256(".onion checksum" || pubkey || version)[0..2]
        let mut hasher = Sha3_256::new();
        hasher.update(b".onion checksum");
        hasher.update(public_key.as_bytes());
        hasher.update(&[0x03]); // Version 3
        let checksum = &hasher.finalize()[..2];
        
        data.extend_from_slice(checksum);
        data.push(0x03); // Version
        
        let onion_address = format!("{}.onion", base32_encode(&data));
        
        Self {
            signing_key,
            public_key,
            onion_address,
        }
    }
    
    /// Load from persisted key (using Sled)
    pub fn from_stored(key_bytes: &[u8]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(key_bytes.try_into()?);
        // ... derive address same way
    }
}
```

### 2. Minimal Tor Protocol (Cells)

**We don't need full Tor protocol, just onion service handshake**:

```rust
/// Tor cell types (we only need these)
pub enum TorCell {
    /// VERSIONS - negotiate protocol version
    Versions(Vec<u16>),
    /// NETINFO - network info exchange
    NetInfo { /* ... */ },
    /// CREATE2 - begin circuit extension (for connecting TO onions)
    Create2 { handshake_type: u16, data: Vec<u8> },
    /// CREATED2 - circuit creation response
    Created2 { data: Vec<u8> },
    /// RELAY - encrypted relay cell
    Relay { stream_id: u16, digest: u32, data: Vec<u8> },
    /// RELAY_DATA - actual data
    RelayData { stream_id: u16, data: Vec<u8> },
}

impl TorCell {
    /// Encode cell to bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            TorCell::Versions(versions) => {
                let mut buf = vec![0, 7]; // VERSIONS cell
                for v in versions {
                    buf.extend_from_slice(&v.to_be_bytes());
                }
                buf
            }
            // ... other cells
        }
    }
    
    /// Decode cell from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        match bytes[1] {  // Command byte
            7 => { /* VERSIONS */ }
            // ... other cells
        }
    }
}
```

### 3. Onion Service (Listen Mode)

```rust
/// Minimal onion service - creates reachable .onion address
pub struct OnionService {
    identity: OnionIdentity,
    listen_port: u16,
    listener: TcpListener,
    crypto: Arc<dyn CryptoCapability>,  // BearDog delegation
}

impl OnionService {
    /// Create new onion service
    pub async fn new(port: u16, crypto: Arc<dyn CryptoCapability>) -> Result<Self> {
        let identity = OnionIdentity::generate();
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        
        info!("🧅 Onion service created: {}", identity.onion_address);
        
        Ok(Self {
            identity,
            listen_port: port,
            listener,
            crypto,
        })
    }
    
    /// Get our .onion address
    pub fn onion_address(&self) -> &str {
        &self.identity.onion_address
    }
    
    /// Accept incoming connections
    pub async fn accept(&self) -> Result<OnionConnection> {
        let (stream, peer_addr) = self.listener.accept().await?;
        
        // Perform onion service handshake
        let conn = self.handshake(stream).await?;
        
        Ok(conn)
    }
    
    /// Minimal handshake (just enough to establish encrypted channel)
    async fn handshake(&self, mut stream: TcpStream) -> Result<OnionConnection> {
        // 1. Read VERSIONS cell
        let versions = read_cell(&mut stream).await?;
        
        // 2. Send VERSIONS response (support v5 only)
        write_cell(&mut stream, TorCell::Versions(vec![5])).await?;
        
        // 3. Perform key exchange (X25519 via BearDog)
        let shared_secret = self.key_exchange(&mut stream).await?;
        
        // 4. Derive session keys (HKDF via BearDog)
        let keys = self.derive_keys(shared_secret).await?;
        
        Ok(OnionConnection {
            stream,
            keys,
            peer_addr,
        })
    }
}
```

### 4. Onion Connector (Connect Mode)

```rust
/// Connect to .onion addresses
pub struct OnionConnector {
    crypto: Arc<dyn CryptoCapability>,
}

impl OnionConnector {
    /// Connect to onion address (direct, not via Tor network)
    pub async fn connect(&self, onion_addr: &str, port: u16) -> Result<OnionConnection> {
        // For MVP: Direct P2P using .onion as identity
        // Parse .onion to get Ed25519 public key
        let pubkey = parse_onion_address(onion_addr)?;
        
        // In family beacon, .onion is encrypted with:
        // - Actual IP:port for direct connection
        // - Or: rendezvous point for NAT traversal
        
        // For now: Use beacon mesh to find actual address
        let actual_addr = self.resolve_via_beacon(onion_addr).await?;
        
        let stream = TcpStream::connect(actual_addr).await?;
        
        // Perform handshake
        let conn = self.handshake(stream, &pubkey).await?;
        
        Ok(conn)
    }
}
```

---

## 🔄 Evolution Strategy

### Phase 1: Onion Identity & Addressing (1 day)

**Goal**: Generate .onion addresses without Arti

**Tasks**:
- [x] Create `songbird-sovereign-onion` crate
- [ ] Implement `OnionIdentity::generate()`
- [ ] Implement `.onion` address derivation (SHA3-256)
- [ ] Add Sled storage for identity persistence
- [ ] Unit tests (10 tests)

**Dependencies**:
```toml
ed25519-dalek = "2.1"  # Already have
sha3 = "0.10"          # Add this
sled = "0.34"          # Pure Rust embedded database
base32 = "0.5"         # For .onion encoding
```

**Success Criteria**:
- Generate valid v3 .onion addresses
- Persist/load identity from Sled
- 100% Pure Rust

---

### Phase 2: Minimal Protocol (2-3 days)

**Goal**: Implement just enough Tor protocol for onion connections

**Scope** (minimal):
- VERSIONS cell (protocol negotiation)
- CREATE2/CREATED2 cells (circuit creation)
- RELAY cells (encrypted data)
- Key exchange (ntor handshake using X25519)

**NOT Needed**:
- Full circuit extension
- Directory protocol
- Consensus parsing
- Guard/middle/exit logic

**Tasks**:
- [ ] Implement `TorCell` encode/decode
- [ ] Implement ntor handshake (X25519 via BearDog)
- [ ] Implement relay cell encryption (ChaCha20-Poly1305)
- [ ] Unit tests (15 tests)

**Success Criteria**:
- Can establish encrypted channel
- Can send/receive relay cells
- 100% Pure Rust

---

### Phase 3: Onion Service (2 days)

**Goal**: Create reachable .onion addresses

**Tasks**:
- [ ] Implement `OnionService::new()`
- [ ] Implement `accept()` with handshake
- [ ] Implement connection encryption
- [ ] Integration with beacon mesh
- [ ] Unit + integration tests (12 tests)

**Success Criteria**:
- Can create .onion address
- Can accept connections
- Encrypted bidirectional communication

---

### Phase 4: Onion Connector (1 day)

**Goal**: Connect to .onion addresses

**Tasks**:
- [ ] Implement `OnionConnector::connect()`
- [ ] Resolve .onion via beacon mesh
- [ ] Client-side handshake
- [ ] Integration tests (5 tests)

**Success Criteria**:
- Can connect to onion services
- Works across NAT
- Integrated with mesh

---

### Phase 5: Sled Integration (1 day)

**Goal**: Persist identity and peer info

**Schema**:
```
identity/my_key         → SigningKey bytes
identity/onion_address  → .onion string
peers/{node_id}/onion   → .onion address
peers/{node_id}/last_seen → Timestamp
mesh/endpoints/{node_id} → Serialized RelayEndpoint
```

**Tasks**:
- [ ] Create Sled database wrapper
- [ ] Implement identity persistence
- [ ] Implement peer cache
- [ ] Migration from in-memory
- [ ] Unit tests (8 tests)

---

## 🔬 Technical Analysis

### Arti Dependencies We Can Eliminate

| Arti Dep | Purpose | Our Alternative |
|----------|---------|-----------------|
| `rusqlite` | Store consensus | ❌ Don't need consensus (not using Tor network) |
| `native-tls` | TLS connections | ✅ Have custom TLS (songbird-tls) |
| `ring` | Crypto primitives | ✅ Have RustCrypto + BearDog |
| Directory auth | Bootstrap Tor | ❌ Don't need (direct P2P) |
| Consensus docs | Find relays | ❌ Don't need (family mesh) |

### Crypto Primitives Comparison

| Operation | Arti | Songbird |
|-----------|------|----------|
| **Identity** | Ed25519 (via ring) | ✅ `ed25519-dalek` |
| **Key Exchange** | X25519 (via ring) | ✅ `x25519-dalek` + BearDog |
| **AEAD** | ChaCha20-Poly1305 | ✅ `chacha20poly1305` |
| **Hash** | SHA3-256 (for .onion) | ⚠️ Add `sha3` crate |
| **KDF** | HKDF-SHA256 | ✅ Have (TLS key schedule) |

**Gap**: Only SHA3 (tiny Pure Rust crate)

---

## 📊 Effort Comparison

### Option A: Use Arti (Current Approach)

**Effort**:
- Configure Arti features: 4 hours
- Fix SQLite issue: 2-4 hours
- Integration: 1 day
- Testing: 1 day
**Total**: 3-4 days

**Result**:
- ❌ Still has C dependencies (SQLite)
- ❌ Large binary (~5MB)
- ❌ Complex dependency tree
- ⚠️ API instability (experimental onion service)

### Option B: Build Our Own (Proposed)

**Effort**:
- Phase 1 (Identity): 1 day
- Phase 2 (Protocol): 2-3 days
- Phase 3 (Service): 2 days
- Phase 4 (Connector): 1 day
- Phase 5 (Sled): 1 day
**Total**: 7-8 days

**Result**:
- ✅ 100% Pure Rust
- ✅ Smaller binary (~500KB vs ~5MB)
- ✅ Full control over evolution
- ✅ Aligned with ecoPrimal philosophy
- ✅ Can optimize for our use case

---

## 🎯 Recommendation: Option B (Build Our Own)

### Why This Makes Sense

**1. Philosophical Alignment**:
- We built custom TLS for same reason (avoid ring/C deps)
- "Evolution over dependency"
- TRUE ecoBin compliance

**2. Technical Advantages**:
- Simpler (10% of Tor complexity)
- Faster (no consensus download)
- Smaller (500KB vs 5MB)
- More secure (smaller attack surface)

**3. Long-Term Benefits**:
- Can evolve to family-specific optimizations
- Can integrate genetic lineage directly
- No upstream API breakage risk
- Educational value (understand the protocol)

**4. Reasonable Effort**:
- 7-8 days vs 3-4 days (only 4 days more)
- Investment pays off in sovereignty
- Reusable for other ecoPrimals

---

## 📋 Implementation Phases

### Phase 1: Foundation (Day 1)

**Create `songbird-sovereign-onion` crate**:

```toml
[package]
name = "songbird-sovereign-onion"
version = "0.1.0"
edition = "2021"
description = "Pure Rust minimal onion service protocol"
license = "AGPL-3.0"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
ed25519-dalek = "2.1"        # Identity keys (Pure Rust)
x25519-dalek = "2.0"         # Key exchange (Pure Rust)
chacha20poly1305 = "0.10"    # AEAD (Pure Rust)
sha3 = "0.10"                # For .onion address (Pure Rust)
sha2 = "0.10"                # For HKDF (Pure Rust)
hmac = "0.12"                # For HKDF (Pure Rust)
sled = "0.34"                # Database (Pure Rust)
base32 = "0.5"               # For .onion encoding (Pure Rust)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
thiserror = "1.0"

# Internal
songbird-types = { path = "../songbird-types" }
```

### Phase 2: Crypto Implementation (Days 2-3)

**Implement ntor handshake** (Tor's key exchange):

```rust
/// ntor handshake (X25519-based, simpler than TLS)
pub struct NtorHandshake {
    crypto: Arc<dyn CryptoCapability>,
}

impl NtorHandshake {
    /// Server-side: receive client's X25519 public key, derive shared secret
    pub async fn server_handshake(&self, client_pubkey: &[u8; 32]) -> Result<HandshakeResult> {
        // 1. Generate ephemeral X25519 keypair via BearDog
        let (server_sk, server_pk) = self.crypto.x25519_generate_ephemeral().await?;
        
        // 2. Derive shared secret
        let shared = self.crypto.x25519_derive_secret(&server_sk, client_pubkey).await?;
        
        // 3. Derive session keys using HKDF
        let keys = self.derive_session_keys(&shared).await?;
        
        Ok(HandshakeResult {
            server_pubkey: server_pk,
            session_keys: keys,
        })
    }
}
```

### Phase 3: Service Implementation (Days 4-5)

**Create minimal onion service**:

```rust
pub struct SovereignOnionService {
    identity: OnionIdentity,
    listen_addr: SocketAddr,
    listener: TcpListener,
    connections: Arc<RwLock<HashMap<u32, OnionConnection>>>,
    db: sled::Db,
}

impl SovereignOnionService {
    /// Start onion service
    pub async fn start(port: u16) -> Result<Self> {
        // Load or generate identity
        let db = sled::open("./data/onion-identity")?;
        let identity = if let Some(key_bytes) = db.get("identity")? {
            OnionIdentity::from_stored(&key_bytes)?
        } else {
            let id = OnionIdentity::generate();
            db.insert("identity", id.signing_key.to_bytes())?;
            id
        };
        
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        
        info!("🧅 Onion service listening: {} -> 0.0.0.0:{}", 
              identity.onion_address, port);
        
        Ok(Self {
            identity,
            listen_addr: listener.local_addr()?,
            listener,
            connections: Arc::new(RwLock::new(HashMap::new())),
            db,
        })
    }
    
    /// Run service (accept loop)
    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, peer)) => {
                    let conn_id = rand::random();
                    let conn = self.handshake(stream, peer).await?;
                    self.connections.write().await.insert(conn_id, conn);
                }
                Err(e) => {
                    warn!("Accept error: {}", e);
                }
            }
        }
    }
}
```

---

## 🧬 Integration with Existing Systems

### Wire into BeaconMesh

```rust
// In mesh.rs
impl BeaconMesh {
    /// Set our onion service address
    pub async fn set_onion_service(&self, service: &SovereignOnionService) {
        let onion_addr = service.onion_address().to_string();
        *self.my_onion.write().await = Some(onion_addr.clone());
        
        // Store in Sled for persistence
        service.db.insert("mesh/my_onion", onion_addr.as_bytes())?;
    }
    
    /// Connect via onion address
    pub async fn connect_via_onion(&self, onion_addr: &str, port: u16) -> Result<OnionConnection> {
        let connector = OnionConnector::new(self.crypto.clone());
        connector.connect(onion_addr, port).await
    }
}
```

### JSON-RPC Methods

```rust
// In songbird-universal-ipc/src/service.rs

"onion.create_service" => {
    let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(9735);
    let service = SovereignOnionService::start(port as u16).await?;
    
    json!({
        "onion_address": service.onion_address(),
        "listen_port": port,
        "status": "running"
    })
}

"onion.connect" => {
    let onion = params.get("onion_address").and_then(|v| v.as_str())?;
    let port = params.get("port").and_then(|v| v.as_u64()).unwrap_or(9735);
    
    let conn = self.onion_connector.connect(onion, port as u16).await?;
    
    json!({
        "connected": true,
        "onion_address": onion,
        "connection_id": conn.id
    })
}
```

---

## 📈 Dependency Analysis

### Before (Arti)

```
arti-client
├── tor-dirmgr → rusqlite → libsqlite3-sys → libsqlite3 ❌ (C)
├── native-tls → openssl-sys → openssl ❌ (C)
├── (many other deps)
Total: ~150 transitive dependencies
Binary: ~5MB
```

### After (Sovereign Onion)

```
songbird-sovereign-onion
├── ed25519-dalek ✅ (Pure Rust)
├── x25519-dalek ✅ (Pure Rust)
├── chacha20poly1305 ✅ (Pure Rust)
├── sha3 ✅ (Pure Rust)
├── sled ✅ (Pure Rust)
├── base32 ✅ (Pure Rust)
└── songbird-types ✅ (Our code)
Total: ~10 direct dependencies, all Pure Rust
Binary: ~500KB
```

---

## 🧪 Testing Strategy

### Unit Tests (30 tests)

**Identity & Addressing** (10 tests):
```rust
#[test]
fn test_generate_onion_identity() { }

#[test]
fn test_onion_address_format() { }

#[test]
fn test_checksum_validation() { }

#[test]
fn test_persist_and_load_identity() { }
```

**Protocol** (15 tests):
```rust
#[test]
fn test_versions_cell_encode_decode() { }

#[test]
fn test_create2_cell() { }

#[test]
fn test_relay_cell_encryption() { }

#[test]
fn test_ntor_handshake() { }
```

**Service** (5 tests):
```rust
#[tokio::test]
async fn test_onion_service_start() { }

#[tokio::test]
async fn test_accept_connection() { }
```

### Integration Tests (12 tests)

**End-to-End** (5 tests):
```rust
#[tokio::test]
async fn test_onion_service_e2e() {
    // Start service
    let service = SovereignOnionService::start(9735).await.unwrap();
    let onion_addr = service.onion_address().to_string();
    
    // Connect to it
    let connector = OnionConnector::new(crypto);
    let conn = connector.connect(&onion_addr, 9735).await.unwrap();
    
    // Send data
    conn.write(b"Hello").await.unwrap();
    
    // Verify received
    let data = service.read().await.unwrap();
    assert_eq!(data, b"Hello");
}
```

**Mesh Integration** (7 tests):
```rust
#[tokio::test]
async fn test_mesh_with_onion() { }

#[tokio::test]
async fn test_mesh_priority_onion_fallback() { }
```

### Chaos Tests (10 tests)

```rust
#[tokio::test]
async fn test_onion_100_concurrent_connections() { }

#[tokio::test]
async fn test_onion_rapid_reconnect() { }

#[tokio::test]
async fn test_onion_network_partition() { }
```

### Fault Tests (8 tests)

```rust
#[tokio::test]
async fn test_onion_invalid_address() { }

#[tokio::test]
async fn test_onion_handshake_timeout() { }

#[tokio::test]
async fn test_onion_encryption_failure() { }
```

---

## 📊 Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Pure Rust** | 100% | 0% (Arti has C) | ⚠️ |
| **Binary Size** | <1MB | ~5MB (Arti) | ⚠️ |
| **Dependencies** | <15 direct | ~150 (Arti) | ⚠️ |
| **Test Coverage** | 85%+ | 0% | ⚠️ |
| **Connection Time** | <2s | TBD | - |
| **Memory** | <10MB | ~50MB (Arti) | ⚠️ |

After evolution:
- Pure Rust: 100% ✅
- Binary: ~500KB ✅
- Deps: 10 direct ✅
- Tests: 60+ tests ✅

---

## 🚀 Timeline

### Week 1 (Feb 6-12)

| Day | Phase | Deliverable |
|-----|-------|-------------|
| Thu | Phase 1 | Onion identity + addressing |
| Fri | Phase 2 (pt 1) | Tor cell encoding |
| Sat | Phase 2 (pt 2) | ntor handshake |
| Sun | Phase 3 (pt 1) | Onion service skeleton |
| Mon | Phase 3 (pt 2) | Service complete + tests |
| Tue | Phase 4 | Onion connector |
| Wed | Phase 5 | Sled integration |

### Week 2 (Feb 13-14)

| Day | Focus | Deliverable |
|-----|-------|-------------|
| Thu | Testing | 60+ tests (unit, e2e, chaos, fault) |
| Fri | Integration | Wire into mesh, IPC methods |

---

## 🔒 Security Considerations

### Threat Model

**In Scope**:
- ✅ Network eavesdropping (end-to-end encryption)
- ✅ MITM attacks (Ed25519 authentication)
- ✅ Replay attacks (nonce in cells)

**Out of Scope** (acceptable trade-offs):
- ⚠️ Traffic analysis (no onion routing - ACCEPTABLE for family use)
- ⚠️ Timing attacks (ACCEPTABLE for signaling)
- ⚠️ Global passive adversary (not our threat model)

**Why Acceptable**: 
- We're not trying to be anonymous
- We're trying to be **reachable** across NAT
- Family devices trust each other
- Encryption prevents eavesdropping
- Ed25519 prevents impersonation

### Privacy Properties

**What We Keep from Tor**:
- ✅ .onion addresses (crypto-derived IDs)
- ✅ No DNS (direct P2P)
- ✅ End-to-end encryption

**What We Skip**:
- ❌ Onion routing (3 hops) - too much latency
- ❌ Anonymity guarantees - not needed for family

**Result**: "Tor-inspired addressing, not Tor anonymity"

---

## 🎯 Decision Points

### Question 1: Full Protocol or Minimal?

**Options**:
- A) Full Tor cell protocol (CREATE2, EXTEND2, RELAY, etc.)
- B) Minimal custom protocol (just KEY_EXCHANGE, DATA)

**Recommendation**: **B (Minimal)**
- Simpler (2 days vs 4 days)
- Easier to audit
- Fits our use case perfectly

### Question 2: Persist Identity or Generate Per-Session?

**Options**:
- A) Persistent .onion address (Sled storage)
- B) New .onion each session

**Recommendation**: **A (Persistent)**
- Can share .onion in beacon
- Easier for family to remember
- Reduces beacon updates

### Question 3: Full Compatibility or Custom?

**Options**:
- A) Wire-compatible with Tor (could use Tor Browser to connect)
- B) Custom protocol (only Songbird devices)

**Recommendation**: **B (Custom)**
- Simpler implementation
- Can optimize for our use case
- No need for Tor Browser compatibility

---

## 📚 Specifications to Create

1. `specs/SOVEREIGN_ONION_PROTOCOL.md` - Protocol design
2. `specs/ONION_ADDRESS_DERIVATION.md` - Crypto details
3. `specs/NTOR_HANDSHAKE_PURE_RUST.md` - Key exchange
4. `specs/SOVEREIGN_ONION_TESTING.md` - Test strategy

---

## ✅ Next Steps

### Immediate (Today)

1. ✅ Create evolution plan (this document)
2. ⚠️ Create specifications
3. ⚠️ Create `songbird-sovereign-onion` crate skeleton
4. ⚠️ Begin Phase 1 (onion identity)

### This Week

5. Complete Phases 1-5
6. Integration tests
7. Wire into BeaconMesh
8. Update IPC methods

---

**Evolution Plan Complete**: February 6, 2026  
**Approach**: Build our own (like we did with TLS)  
**Timeline**: 7-8 days (vs 3-4 for Arti with C deps)  
**Result**: TRUE 100% Pure Rust sovereignty

🦀 **Pure Rust** | 🧬 **Evolution Over Dependency** | ✨ **Full Control**
