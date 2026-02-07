# 🚀 Phase 2B Preparation - Circuit Protocol Design

**Status**: 🟡 Design Phase (Implementation blocked by BearDog)  
**Date**: February 7, 2026  
**Prerequisites**: AES-128-CTR + SHA3-256 from BearDog

---

## Overview

While Phase 2B implementation is blocked awaiting BearDog crypto extensions, we can prepare the detailed design and architecture to enable rapid implementation once dependencies are available.

---

## Architecture

### Circuit Building Flow

```
Client                Guard Relay          Middle Relay         HSDir/Exit
  │                      │                      │                    │
  │─────CREATE2─────────>│                      │                    │
  │                      │                      │                    │
  │<────CREATED2─────────│                      │                    │
  │ (Establish crypto)   │                      │                    │
  │                      │                      │                    │
  │────EXTEND2──────────>│─────CREATE2─────────>│                    │
  │                      │                      │                    │
  │<───EXTENDED2─────────│<────CREATED2─────────│                    │
  │   (Multi-layer enc)  │                      │                    │
  │                      │                      │                    │
  │────EXTEND2──────────>│─────RELAY───────────>│─────CREATE2───────>│
  │  (3-layer onion)     │   (2-layer onion)    │   (1-layer)        │
  │                      │                      │                    │
  │<───EXTENDED2─────────│<────RELAY───────────<│<────CREATED2───────│
  │                      │                      │                    │
  └─────────────────── 3-hop circuit established ────────────────────┘
```

---

## Component Design

### 1. Circuit Manager (`circuit/manager.rs`)

**Purpose**: Manage circuit lifecycle and state

```rust
pub struct CircuitManager {
    circuits: Arc<RwLock<HashMap<CircuitId, Circuit>>>,
    beardog: Arc<BeardogCryptoClient>,
    consensus: Arc<RwLock<Consensus>>,
}

impl CircuitManager {
    /// Build a new 3-hop circuit
    pub async fn build_circuit(&self, purpose: CircuitPurpose) -> Result<CircuitId>;
    
    /// Extend an existing circuit by one hop
    pub async fn extend_circuit(&self, circuit_id: CircuitId, relay: RelayInfo) -> Result<()>;
    
    /// Close a circuit
    pub async fn close_circuit(&self, circuit_id: CircuitId) -> Result<()>;
    
    /// Get circuit by ID
    pub fn get_circuit(&self, circuit_id: CircuitId) -> Option<Circuit>;
}
```

### 2. ntor Handshake (`circuit/ntor.rs`)

**Purpose**: Implement Tor's ntor key agreement protocol

```rust
pub struct NtorHandshake {
    beardog: Arc<BeardogCryptoClient>,
}

impl NtorHandshake {
    /// Client side: Create ntor CREATE2 payload
    pub async fn create_handshake(
        &self,
        relay_identity: &[u8; 32],
        relay_ntor_key: &[u8; 32],
    ) -> Result<(Vec<u8>, HandshakeState)>;
    
    /// Client side: Process CREATED2 response
    pub async fn complete_handshake(
        &self,
        state: HandshakeState,
        response: &[u8],
    ) -> Result<KeyMaterial>;
}

pub struct KeyMaterial {
    pub forward_digest: [u8; 32],
    pub backward_digest: [u8; 32],
    pub forward_key: [u8; 16],
    pub backward_key: [u8; 16],
}
```

**Key Derivation (via BearDog SHA3-256)**:
```
shared_secret = X25519(client_ephemeral, relay_ntor_key)
key_material = KDF(shared_secret)  // SHA3-256 based
```

### 3. Circuit State (`circuit/state.rs`)

**Purpose**: Track circuit state and crypto keys

```rust
pub struct Circuit {
    pub id: CircuitId,
    pub hops: Vec<CircuitHop>,
    pub purpose: CircuitPurpose,
    pub created_at: Instant,
}

pub struct CircuitHop {
    pub relay: RelayInfo,
    pub forward_digest: [u8; 32],
    pub backward_digest: [u8; 32],
    pub forward_key: [u8; 16],
    pub backward_key: [u8; 16],
}

pub enum CircuitPurpose {
    General,
    HSDir,      // For hidden service directory queries
    Rendezvous, // For hidden service connections
}
```

### 4. Onion Encryption (`circuit/onion.rs`)

**Purpose**: Layer/unlayer onion encryption

```rust
pub struct OnionCrypto {
    beardog: Arc<BeardogCryptoClient>,
}

impl OnionCrypto {
    /// Encrypt cell with onion layers (client → exit)
    pub async fn encrypt_forward(
        &self,
        cell: &[u8],
        hops: &[CircuitHop],
    ) -> Result<Vec<u8>>;
    
    /// Decrypt cell removing onion layers (exit → client)
    pub async fn decrypt_backward(
        &self,
        cell: &[u8],
        hops: &[CircuitHop],
    ) -> Result<Vec<u8>>;
}
```

**Encryption Flow**:
```
Plaintext → AES(hop3) → AES(hop2) → AES(hop1) → Ciphertext
```

**Decryption Flow** (at each relay):
```
Ciphertext → AES_decrypt(hop_key) → Forward to next hop
```

---

## BearDog Crypto Operations Required

### 1. AES-128-CTR (Cell Encryption)

```rust
// Encrypt a 512-byte cell
let encrypted = beardog.aes_128_ctr_encrypt(
    &forward_key,  // 16 bytes
    &iv,           // 16 bytes (derived from sequence)
    &cell_payload, // 507 bytes
)?;

// Decrypt a 512-byte cell
let decrypted = beardog.aes_128_ctr_decrypt(
    &backward_key, // 16 bytes
    &iv,           // 16 bytes
    &cell_payload, // 507 bytes
)?;
```

### 2. SHA3-256 (KDF and Digests)

```rust
// Key derivation function
let key_material = beardog.sha3_256(&shared_secret)?;

// Running digest for integrity
let new_digest = beardog.sha3_256(&[&old_digest[..], &cell_data[..]].concat())?;
```

### 3. X25519 (ECDH) - Already Available

```rust
// Generate ephemeral keypair (already exists)
let ephemeral = beardog.x25519_generate_ephemeral()?;

// Derive shared secret (already exists)
let shared = beardog.x25519_derive_secret(&ephemeral.secret, &relay_ntor_key)?;
```

---

## Implementation Phases

### Phase 2B-1: ntor Handshake (Days 1-2)
**Status**: 🔴 Blocked by BearDog

- [ ] Implement `NtorHandshake::create_handshake()`
- [ ] Implement `NtorHandshake::complete_handshake()`
- [ ] Implement KDF with SHA3-256 (via BearDog)
- [ ] Test with known ntor test vectors
- [ ] Unit tests for handshake

**Dependencies**: `sha3_256()`, `x25519_derive_secret()`

### Phase 2B-2: Circuit Building (Day 3)
**Status**: 🔴 Blocked by Phase 2B-1

- [ ] Implement `CircuitManager::build_circuit()`
- [ ] CREATE2/CREATED2 cell handling
- [ ] Circuit state management
- [ ] Error handling and retry logic
- [ ] Integration tests

**Dependencies**: Phase 2B-1 complete

### Phase 2B-3: Circuit Extension (Day 4)
**Status**: 🔴 Blocked by Phase 2B-2

- [ ] Implement `CircuitManager::extend_circuit()`
- [ ] EXTEND2/EXTENDED2 cell handling
- [ ] Relay cell encoding/decoding
- [ ] Multi-hop communication
- [ ] Integration tests

**Dependencies**: Phase 2B-2 complete

### Phase 2B-4: Onion Encryption (Day 5)
**Status**: 🔴 Blocked by Phase 2B-3

- [ ] Implement `OnionCrypto::encrypt_forward()`
- [ ] Implement `OnionCrypto::decrypt_backward()`
- [ ] Layer-by-layer encryption with AES-128-CTR
- [ ] Digest calculation with SHA3-256
- [ ] Full circuit tests with live Tor network

**Dependencies**: `aes_128_ctr_encrypt/decrypt()`, Phase 2B-3 complete

---

## File Structure (Prepared)

```
crates/songbird-tor-protocol/src/circuit/
├── mod.rs           # Public API
├── manager.rs       # Circuit lifecycle management
├── ntor.rs          # ntor handshake protocol
├── state.rs         # Circuit and hop state
├── onion.rs         # Onion encryption/decryption
├── create.rs        # CREATE2/CREATED2 handling
├── extend.rs        # EXTEND2/EXTENDED2 handling
└── tests.rs         # Integration tests
```

---

## Test Strategy

### Unit Tests
- ntor handshake with test vectors
- KDF correctness
- Cell encoding/decoding
- Onion layer correctness

### Integration Tests
- Build single-hop circuit (guard only)
- Build two-hop circuit (guard + middle)
- Build three-hop circuit (full path)
- Circuit extension
- Error recovery

### Live Network Tests
- Connect to real Tor network
- Build circuits through production relays
- Measure latency and reliability
- Test circuit failure handling

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Circuit Build Time** | < 2s | For 3-hop circuit |
| **ntor Handshake** | < 100ms | Single hop via BearDog |
| **Cell Encryption** | < 1ms | 512-byte cell (all layers) |
| **Throughput** | > 1 MB/s | Per circuit |
| **Concurrent Circuits** | > 100 | Without degradation |

---

## Error Handling

### Circuit Build Failures
- Relay unreachable → Retry with different relay
- Handshake timeout → Retry with backoff
- ntor failure → Select different guard
- Max retries exceeded → Return error to caller

### Circuit Maintenance
- Periodic health checks (PADDING cells)
- Auto-rebuild on degradation
- Graceful shutdown on close
- Resource cleanup on failure

---

## Security Considerations

### Key Management
- **Never log keys** - All key material in secure memory
- **Key rotation** - Circuits rebuilt periodically
- **Forward secrecy** - Ephemeral keys for each handshake
- **BearDog delegation** - Zero keys in Songbird memory

### Timing Attacks
- **Constant-time crypto** - All crypto via BearDog
- **Padding** - Fixed-size cells (512 bytes)
- **Rate limiting** - Prevent timing correlation

### Anonymity
- **Path diversity** - No relay appears twice
- **Guard rotation** - Periodic guard changes
- **Circuit isolation** - Different circuits for different streams

---

## Next Steps

### Immediate (Awaiting BearDog)
1. Coordinate timeline with BearDog team
2. Review ntor specification details
3. Prepare test vectors for validation
4. Design error handling strategy

### Once BearDog Ready
1. Implement Phase 2B-1 (ntor handshake)
2. Test handshake with known vectors
3. Implement Phase 2B-2 (circuit building)
4. Test with single-hop circuits
5. Complete remaining phases

### Parallel Work (Can Start Now)
1. Write comprehensive ntor spec documentation
2. Prepare circuit state machine diagrams
3. Design integration test scenarios
4. Create benchmarking framework

---

## Documentation

**Specifications**:
- `specs/TOR_PROTOCOL_PURE_RUST.md` - Full technical spec
- `specs/NTOR_HANDSHAKE.md` - ntor details (TODO)
- `specs/CIRCUIT_BUILDING.md` - Circuit protocol (TODO)

**Progress Tracking**:
- `TOR_PHASE2_EVOLUTION_TRACKER.md` - Daily progress updates

**Implementation Guides**:
- Phase 2B implementation guide (TODO)
- BearDog crypto integration guide (TODO)

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **BearDog delays** | High | Parallel design work, clear requirements |
| **ntor complexity** | Medium | Test vectors, reference implementation study |
| **Performance issues** | Medium | Early benchmarking, optimization iteration |
| **Tor network changes** | Low | Follow Tor spec updates, version compatibility |

---

## Success Criteria

✅ **Phase 2B Complete** when:
- [ ] Can build 3-hop circuits through live Tor network
- [ ] ntor handshake passes reference test vectors
- [ ] Onion encryption verified correct (test vectors)
- [ ] Performance meets targets (< 2s circuit build)
- [ ] All integration tests passing
- [ ] Zero unsafe code maintained
- [ ] 100% BearDog delegation maintained
- [ ] Documentation complete

---

**Status**: 🟡 Design Complete, Implementation Blocked  
**Blocker**: BearDog AES-128-CTR + SHA3-256  
**Ready to Implement**: As soon as BearDog extensions available

---

**TRUE PRIMAL** | **Pure Rust** | **Zero Unsafe** | **100% BearDog Delegation**
