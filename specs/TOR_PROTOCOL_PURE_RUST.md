# 🦀 Pure Rust Tor Protocol Specification

**Version**: 1.0  
**Date**: February 7, 2026  
**Status**: Phase 2 Evolution - Design & Implementation  
**Crate**: `songbird-tor-protocol` (NEW)

---

## Executive Summary

This specification defines a **minimal Tor protocol implementation** in Pure Rust for Songbird, enabling .onion service hosting and client connectivity without external dependencies.

**Key Principles**:
- ✅ **TRUE PRIMAL**: 100% BearDog crypto delegation
- ✅ **Pure Rust**: Zero external dependencies (no Tor daemon, no Arti, no C)
- ✅ **Minimal Subset**: Onion services only (not full Tor functionality)
- ✅ **Memory Safe**: No unsafe blocks, async/await, modern Rust
- ✅ **~2,600 lines**: Focused implementation vs. Tor's 220k+ lines

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Directory Protocol](#directory-protocol)
3. [Circuit Protocol](#circuit-protocol)
4. [Onion Service Protocol](#onion-service-protocol)
5. [Stream Protocol](#stream-protocol)
6. [Crypto Delegation](#crypto-delegation)
7. [Storage Strategy](#storage-strategy)
8. [Implementation Plan](#implementation-plan)

---

## Architecture Overview

### Component Hierarchy

```
┌──────────────────────────────────────────────────────────────┐
│              songbird-tor-protocol (NEW CRATE)               │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ PUBLIC API                                             │ │
│  │ ├─ TorClient::new(beardog)                            │ │
│  │ ├─ TorClient::connect(onion_addr) -> TorStream       │ │
│  │ ├─ TorService::new(beardog, port)                     │ │
│  │ └─ TorService::listen() -> TorListener               │ │
│  └────────────────────────────────────────────────────────┘ │
│                             │                                │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ PROTOCOL LAYERS                                        │ │
│  │                                                        │ │
│  │  Directory Protocol  (~500 lines)                     │ │
│  │  ├─ Fetch consensus from authorities                  │ │
│  │  ├─ Parse relay descriptors                           │ │
│  │  └─ Select guard/middle/exit nodes                    │ │
│  │                                                        │ │
│  │  Circuit Protocol  (~800 lines)                       │ │
│  │  ├─ CREATE/CREATED cells (ntor handshake)            │ │
│  │  ├─ EXTEND/EXTENDED cells (circuit extension)        │ │
│  │  └─ RELAY cells (encrypted communication)            │ │
│  │                                                        │ │
│  │  Onion Service Protocol  (~1,000 lines)               │ │
│  │  ├─ Generate blinded keys                             │ │
│  │  ├─ Publish descriptors to HSDir                      │ │
│  │  └─ Handle INTRODUCE/RENDEZVOUS                       │ │
│  │                                                        │ │
│  │  Stream Protocol  (~300 lines)                        │ │
│  │  ├─ RELAY_BEGIN/CONNECTED/DATA/END                    │ │
│  │  └─ Flow control (SENDME cells)                       │ │
│  └────────────────────────────────────────────────────────┘ │
│                             │                                │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ CRYPTO DELEGATION (BearDog)                           │ │
│  │ ├─ ed25519_sign/verify (onion identity)              │ │
│  │ ├─ x25519_derive_secret (ntor handshake)             │ │
│  │ ├─ aes_128_ctr_encrypt/decrypt (cell encryption)     │ │
│  │ ├─ sha3_256 (KDF, onion addresses)                   │ │
│  │ └─ chacha20_poly1305_* (optional relay encryption)   │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Dependencies

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["net", "io-util", "time", "macros"] }

# Parsing
nom = "7"  # Tor cell parsing

# Storage (optional)
sled = { version = "0.34", optional = true }

# Error handling
thiserror = "1"

# Logging
tracing = "0.1"

# Encoding
base64 = "0.21"  # For descriptor encoding
base32 = "0.4"   # For .onion addresses

[features]
default = []
persistent-cache = ["sled"]  # Optional consensus caching
```

**Note**: Zero crypto dependencies - all delegated to BearDog!

---

## Directory Protocol

### Overview

The Directory Protocol fetches the Tor network consensus and relay descriptors, enabling circuit path selection.

**Reference**: https://spec.torproject.org/dir-spec

### Components

#### 1. Directory Authorities (Hardcoded)

Tor has 9 directory authorities. We need at least 3 for consensus validation.

```rust
// src/directory/authorities.rs

pub struct DirectoryAuthority {
    pub nickname: &'static str,
    pub address: &'static str,
    pub dir_port: u16,
    pub fingerprint: [u8; 20],  // SHA1 of identity key
}

pub const DIRECTORY_AUTHORITIES: &[DirectoryAuthority] = &[
    DirectoryAuthority {
        nickname: "moria1",
        address: "128.31.0.34",
        dir_port: 9131,
        fingerprint: hex!("9695DFC35FFEB861329B9F1AB04C46397020CE31"),
    },
    DirectoryAuthority {
        nickname: "tor26",
        address: "86.59.21.38",
        dir_port: 80,
        fingerprint: hex!("847B1F850344D7876491A54892F904934E4EB85D"),
    },
    DirectoryAuthority {
        nickname: "dizum",
        address: "45.66.33.45",
        dir_port: 80,
        fingerprint: hex!("7EA6EAD6FD83083C538F44038BBFA077587DD755"),
    },
    // ... 6 more authorities
];
```

#### 2. Consensus Fetching

```rust
// src/directory/consensus.rs

pub struct Consensus {
    pub valid_after: SystemTime,
    pub fresh_until: SystemTime,
    pub valid_until: SystemTime,
    pub relays: Vec<RelayInfo>,
}

pub struct RelayInfo {
    pub nickname: String,
    pub fingerprint: [u8; 20],
    pub address: IpAddr,
    pub or_port: u16,
    pub dir_port: Option<u16>,
    pub flags: RelayFlags,
    pub bandwidth: u64,
}

bitflags! {
    pub struct RelayFlags: u16 {
        const AUTHORITY = 1 << 0;
        const BAD_EXIT = 1 << 1;
        const EXIT = 1 << 2;
        const FAST = 1 << 3;
        const GUARD = 1 << 4;
        const HSDIR = 1 << 5;
        const RUNNING = 1 << 6;
        const STABLE = 1 << 7;
        const VALID = 1 << 8;
        const V2DIR = 1 << 9;
    }
}

impl Consensus {
    /// Fetch consensus from directory authorities
    pub async fn fetch(beardog: &BeardogCryptoClient) -> Result<Self> {
        // 1. Connect to random authority
        let authority = Self::select_random_authority();
        
        // 2. HTTP GET request to /tor/status-vote/current/consensus
        let url = format!("http://{}:{}/tor/status-vote/current/consensus",
                         authority.address, authority.dir_port);
        
        // 3. Download consensus (gzip compressed)
        let response = tokio::time::timeout(
            Duration::from_secs(30),
            Self::download_consensus(&url)
        ).await??;
        
        // 4. Parse consensus
        Self::parse(&response, beardog).await
    }
    
    /// Parse consensus document
    async fn parse(data: &[u8], beardog: &BeardogCryptoClient) -> Result<Self> {
        // Parse using nom
        // Verify signatures using BearDog ed25519_verify
        // Extract relay entries
        todo!()
    }
    
    /// Select relays for circuit path
    pub fn select_path(&self) -> Result<CircuitPath> {
        // 1. Select guard (GUARD + FAST + STABLE + VALID + RUNNING)
        let guard = self.select_guard()?;
        
        // 2. Select middle (FAST + STABLE + VALID + RUNNING)
        let middle = self.select_middle()?;
        
        // 3. Select exit for onion service (HSDIR flag)
        let hsdir = self.select_hsdir()?;
        
        Ok(CircuitPath { guard, middle, hsdir })
    }
}
```

**Consensus Format** (simplified):
```
network-status-version 3
vote-status consensus
consensus-method 30
valid-after 2026-02-07 12:00:00
fresh-until 2026-02-07 13:00:00
valid-until 2026-02-07 15:00:00

r Unnamed AAAAAAAAAAAAAAAAAAAAAA 198.51.100.1 9001 0
s Exit Fast Guard Running Stable Valid V2Dir
w Bandwidth=1000
```

---

## Circuit Protocol

### Overview

The Circuit Protocol builds 3-hop circuits through the Tor network using the ntor handshake.

**Reference**: https://spec.torproject.org/tor-spec/create-created-cells.html

### Cell Format

All Tor communication happens via **cells** (512 bytes fixed size).

```rust
// src/protocol/cells.rs

pub const CELL_LEN: usize = 512;

#[repr(u8)]
pub enum CellCommand {
    Padding = 0,
    Create = 1,
    Created = 2,
    Relay = 3,
    Destroy = 4,
    CreateFast = 5,
    CreatedFast = 6,
    Versions = 7,
    NetInfo = 8,
    RelayEarly = 9,
    Create2 = 10,
    Created2 = 11,
    // ...
}

pub struct Cell {
    pub circ_id: u32,      // Circuit ID (4 bytes)
    pub command: CellCommand,  // Command (1 byte)
    pub payload: Vec<u8>,  // Payload (507 bytes max)
}

impl Cell {
    pub fn encode(&self) -> [u8; CELL_LEN] {
        let mut buf = [0u8; CELL_LEN];
        buf[0..4].copy_from_slice(&self.circ_id.to_be_bytes());
        buf[4] = self.command as u8;
        buf[5..5 + self.payload.len()].copy_from_slice(&self.payload);
        buf
    }
    
    pub fn decode(data: &[u8; CELL_LEN]) -> Result<Self> {
        let circ_id = u32::from_be_bytes(data[0..4].try_into()?);
        let command = CellCommand::try_from(data[4])?;
        let payload = data[5..].to_vec();
        Ok(Self { circ_id, command, payload })
    }
}
```

### ntor Handshake (CREATE2/CREATED2)

```rust
// src/circuit/create.rs

pub struct NtorHandshake {
    relay_identity: [u8; 32],  // Ed25519 identity
    relay_ntor_key: [u8; 32],  // X25519 onion key
}

impl NtorHandshake {
    /// Client side: Generate CREATE2 cell
    pub async fn create_cell(
        &self,
        beardog: &BeardogCryptoClient,
    ) -> Result<(Cell, [u8; 32])> {
        // 1. Generate ephemeral X25519 keypair
        let client_ephemeral = beardog.x25519_generate_ephemeral()?;
        
        // 2. Construct CREATE2 payload (ntor)
        // Format: NODEID | KEYID | CLIENT_PK
        let mut payload = Vec::with_capacity(84);
        payload.extend_from_slice(&self.relay_identity);  // 32 bytes
        payload.extend_from_slice(&self.relay_ntor_key);  // 32 bytes
        payload.extend_from_slice(&client_ephemeral.public_key);  // 32 bytes
        
        // 3. Create CREATE2 cell
        let cell = Cell {
            circ_id: 0x80000000,  // High bit set for circuit creation
            command: CellCommand::Create2,
            payload,
        };
        
        Ok((cell, client_ephemeral.secret_key))
    }
    
    /// Client side: Process CREATED2 cell
    pub async fn process_created(
        &self,
        cell: &Cell,
        client_secret: &[u8; 32],
        beardog: &BeardogCryptoClient,
    ) -> Result<CircuitKeys> {
        // 1. Extract server ephemeral public key (32 bytes)
        let server_pk = &cell.payload[0..32];
        
        // 2. Derive shared secret via BearDog X25519
        let shared_secret = beardog.x25519_derive_secret(
            client_secret,
            server_pk.try_into()?
        )?;
        
        // 3. KDF to derive circuit keys using SHA3
        self.derive_keys(&shared_secret, beardog).await
    }
    
    /// Derive circuit keys from shared secret
    async fn derive_keys(
        &self,
        shared_secret: &[u8; 32],
        beardog: &BeardogCryptoClient,
    ) -> Result<CircuitKeys> {
        // Tor spec: KDF using HMAC-SHA256, but we use SHA3 for BearDog
        // Keys needed:
        // - Kf (forward key, 16 bytes for AES-128)
        // - Kb (backward key, 16 bytes for AES-128)
        // - Df (forward digest, 20 bytes for SHA1)
        // - Db (backward digest, 20 bytes for SHA1)
        
        let key_material = beardog.sha3_256(shared_secret)?;
        
        Ok(CircuitKeys {
            forward_key: key_material[0..16].try_into()?,
            backward_key: key_material[16..32].try_into()?,
            forward_digest: [0u8; 20],  // Simplified
            backward_digest: [0u8; 20],
        })
    }
}

pub struct CircuitKeys {
    pub forward_key: [u8; 16],    // AES-128 key for sending
    pub backward_key: [u8; 16],   // AES-128 key for receiving
    pub forward_digest: [u8; 20], // SHA1 digest for sending
    pub backward_digest: [u8; 20], // SHA1 digest for receiving
}
```

### EXTEND/EXTENDED (Circuit Extension)

```rust
// src/circuit/extend.rs

pub struct CircuitExtender {
    keys: Vec<CircuitKeys>,  // One per hop
}

impl CircuitExtender {
    /// Extend circuit by one hop
    pub async fn extend(
        &mut self,
        stream: &mut TcpStream,
        next_relay: &RelayInfo,
        beardog: &BeardogCryptoClient,
    ) -> Result<()> {
        // 1. Create EXTEND2 relay cell
        let handshake = NtorHandshake {
            relay_identity: next_relay.fingerprint,
            relay_ntor_key: next_relay.ntor_onion_key,
        };
        
        let (create_cell, client_secret) = handshake.create_cell(beardog).await?;
        
        // 2. Wrap in RELAY_COMMAND_EXTEND2
        let relay_cell = self.create_relay_cell(
            RelayCommand::Extend2,
            create_cell.payload,
        )?;
        
        // 3. Encrypt with all hop keys (onion encryption)
        let encrypted = self.encrypt_onion(&relay_cell, beardog).await?;
        
        // 4. Send to circuit
        stream.write_all(&encrypted.encode()).await?;
        
        // 5. Receive EXTENDED2 cell
        let mut buf = [0u8; CELL_LEN];
        stream.read_exact(&mut buf).await?;
        let response = Cell::decode(&buf)?;
        
        // 6. Decrypt and process
        let created = self.decrypt_onion(&response, beardog).await?;
        let keys = handshake.process_created(&created, &client_secret, beardog).await?;
        
        // 7. Add keys to circuit
        self.keys.push(keys);
        
        Ok(())
    }
    
    /// Encrypt cell for all circuit hops (onion encryption)
    async fn encrypt_onion(
        &self,
        cell: &Cell,
        beardog: &BeardogCryptoClient,
    ) -> Result<Cell> {
        let mut payload = cell.payload.clone();
        
        // Encrypt in reverse order (last hop first)
        for keys in self.keys.iter().rev() {
            payload = beardog.aes_128_ctr_encrypt(
                &keys.forward_key,
                &[0u8; 16],  // IV (simplified)
                &payload
            )?;
        }
        
        Ok(Cell {
            circ_id: cell.circ_id,
            command: cell.command,
            payload,
        })
    }
}
```

---

## Onion Service Protocol

### Overview

The Onion Service Protocol enables hosting services at `.onion` addresses using introduction points and rendezvous.

**Reference**: https://spec.torproject.org/rend-spec-v3

### Onion Address Format (v3)

```
<base32-encoded-ed25519-public-key>.onion
```

Example: `ve3lahyh7ktngjkvjdirsgfkmgsi6qcqfzrjrjkq3bffiie2n6qmdwid.onion`

### Descriptor Generation

```rust
// src/onion_service/descriptor.rs

pub struct OnionDescriptor {
    pub onion_address: String,
    pub introduction_points: Vec<IntroductionPoint>,
    pub signature: [u8; 64],  // Ed25519 signature
}

pub struct IntroductionPoint {
    pub link_specifiers: Vec<LinkSpecifier>,  // How to reach this intro point
    pub auth_key: [u8; 32],  // Ed25519 auth key
    pub enc_key: [u8; 32],   // X25519 encryption key
}

impl OnionDescriptor {
    /// Generate descriptor for this onion service
    pub async fn generate(
        identity: &OnionIdentity,
        intro_points: Vec<IntroductionPoint>,
        beardog: &BeardogCryptoClient,
    ) -> Result<Self> {
        // 1. Serialize descriptor content
        let mut content = Vec::new();
        content.extend_from_slice(b"onion-service-descriptor 3\n");
        content.extend_from_slice(identity.onion_address().as_bytes());
        content.extend_from_slice(b"\n");
        
        // Add introduction points
        for ip in &intro_points {
            Self::serialize_intro_point(&mut content, ip);
        }
        
        // 2. Sign with identity key via BearDog
        let signature = beardog.ed25519_sign(
            identity.secret_key_bytes(),
            &content
        )?;
        
        Ok(Self {
            onion_address: identity.onion_address().to_string(),
            introduction_points: intro_points,
            signature,
        })
    }
    
    /// Upload descriptor to HSDir nodes
    pub async fn upload(
        &self,
        circuit: &mut Circuit,
        beardog: &BeardogCryptoClient,
    ) -> Result<()> {
        // 1. Encode descriptor
        let encoded = self.encode()?;
        
        // 2. Send RELAY_COMMAND_ESTABLISH_INTRO
        circuit.send_relay(
            RelayCommand::EstablishIntro,
            encoded
        ).await?;
        
        Ok(())
    }
}
```

### Introduction Protocol

```rust
// src/onion_service/introduce.rs

pub struct IntroductionHandler {
    intro_points: Vec<IntroductionPoint>,
    rendezvous_queue: mpsc::Receiver<RendezvousRequest>,
}

impl IntroductionHandler {
    /// Handle INTRODUCE1 cell from client
    pub async fn handle_introduce(
        &mut self,
        cell: &Cell,
        beardog: &BeardogCryptoClient,
    ) -> Result<()> {
        // 1. Decrypt INTRODUCE1 payload
        let intro1 = self.decrypt_introduce(cell, beardog).await?;
        
        // 2. Extract rendezvous point
        let rend_point = intro1.rendezvous_point;
        let rend_cookie = intro1.rendezvous_cookie;
        
        // 3. Queue rendezvous request
        self.rendezvous_queue.send(RendezvousRequest {
            rend_point,
            rend_cookie,
            client_pk: intro1.client_pubkey,
        }).await?;
        
        Ok(())
    }
}

pub struct RendezvousRequest {
    pub rend_point: RelayInfo,
    pub rend_cookie: [u8; 20],
    pub client_pk: [u8; 32],
}
```

### Rendezvous Protocol

```rust
// src/onion_service/rendezvous.rs

pub struct RendezvousHandler {
    service_identity: OnionIdentity,
}

impl RendezvousHandler {
    /// Complete rendezvous with client
    pub async fn rendezvous(
        &self,
        request: RendezvousRequest,
        beardog: &BeardogCryptoClient,
    ) -> Result<TorStream> {
        // 1. Build circuit to rendezvous point
        let mut circuit = Circuit::build_to(&request.rend_point, beardog).await?;
        
        // 2. Send RENDEZVOUS1 with cookie
        circuit.send_relay(
            RelayCommand::Rendezvous1,
            request.rend_cookie.to_vec()
        ).await?;
        
        // 3. Derive session keys with client
        let session_keys = self.derive_session_keys(
            &request.client_pk,
            beardog
        ).await?;
        
        // 4. Return encrypted stream
        Ok(TorStream::new(circuit, session_keys))
    }
}
```

---

## Stream Protocol

### Overview

The Stream Protocol multiplexes multiple streams over a single circuit using RELAY cells.

**Reference**: https://spec.torproject.org/tor-spec/relay-cells.html

### RELAY Cell Format

```rust
// src/protocol/cells.rs (extended)

#[repr(u8)]
pub enum RelayCommand {
    Begin = 1,
    Data = 2,
    End = 3,
    Connected = 4,
    SendMe = 5,
    Extend = 6,
    Extended = 7,
    // ...
}

pub struct RelayCell {
    pub command: RelayCommand,
    pub recognized: u16,  // Always 0 for valid cell
    pub stream_id: u16,
    pub digest: [u8; 4],  // SHA1 digest
    pub length: u16,
    pub data: Vec<u8>,
}

impl RelayCell {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(509);
        buf.push(self.command as u8);
        buf.extend_from_slice(&self.recognized.to_be_bytes());
        buf.extend_from_slice(&self.stream_id.to_be_bytes());
        buf.extend_from_slice(&self.digest);
        buf.extend_from_slice(&self.length.to_be_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }
}
```

### Stream Handling

```rust
// src/stream/begin.rs

pub struct TorStream {
    circuit: Arc<Mutex<Circuit>>,
    stream_id: u16,
    session_keys: SessionKeys,
}

impl TorStream {
    /// Begin a new stream to target
    pub async fn begin(
        circuit: Arc<Mutex<Circuit>>,
        target: &str,
        port: u16,
        beardog: &BeardogCryptoClient,
    ) -> Result<Self> {
        let stream_id = circuit.lock().await.allocate_stream_id();
        
        // Send RELAY_BEGIN
        let begin_cell = RelayCell {
            command: RelayCommand::Begin,
            recognized: 0,
            stream_id,
            digest: [0; 4],
            length: target.len() as u16,
            data: format!("{}:{}", target, port).into_bytes(),
        };
        
        circuit.lock().await.send_relay_cell(begin_cell, beardog).await?;
        
        // Wait for RELAY_CONNECTED
        let response = circuit.lock().await.recv_relay_cell().await?;
        if response.command != RelayCommand::Connected {
            return Err(Error::StreamBeginFailed);
        }
        
        Ok(Self {
            circuit,
            stream_id,
            session_keys: SessionKeys::default(),
        })
    }
    
    /// Send data on stream
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        let data_cell = RelayCell {
            command: RelayCommand::Data,
            recognized: 0,
            stream_id: self.stream_id,
            digest: [0; 4],
            length: data.len() as u16,
            data: data.to_vec(),
        };
        
        self.circuit.lock().await.send_relay_cell(data_cell, &self.session_keys).await?;
        Ok(data.len())
    }
    
    /// Receive data from stream
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let cell = self.circuit.lock().await.recv_relay_cell_for_stream(self.stream_id).await?;
        
        if cell.command != RelayCommand::Data {
            return Err(Error::UnexpectedCell);
        }
        
        let len = cell.data.len().min(buf.len());
        buf[..len].copy_from_slice(&cell.data[..len]);
        Ok(len)
    }
}
```

---

## Crypto Delegation

### BearDog Interface

All cryptographic operations are delegated to BearDog via IPC.

```rust
// src/crypto/beardog_client.rs

pub struct BeardogCryptoClient {
    // IPC client to BearDog
}

impl BeardogCryptoClient {
    // ===== Ed25519 Operations (Identity) =====
    
    /// Sign data with Ed25519
    pub async fn ed25519_sign(&self, secret_key: &[u8; 32], data: &[u8]) -> Result<[u8; 64]> {
        // IPC call to BearDog
    }
    
    /// Verify Ed25519 signature
    pub async fn ed25519_verify(&self, public_key: &[u8; 32], data: &[u8], signature: &[u8; 64]) -> Result<bool> {
        // IPC call to BearDog
    }
    
    // ===== X25519 Operations (Key Exchange) =====
    
    /// Generate ephemeral X25519 keypair
    pub fn x25519_generate_ephemeral(&self) -> Result<X25519Keypair> {
        // IPC call to BearDog
    }
    
    /// Derive shared secret (ECDH)
    pub fn x25519_derive_secret(&self, secret: &[u8; 32], public: &[u8; 32]) -> Result<[u8; 32]> {
        // IPC call to BearDog
    }
    
    // ===== AES-128-CTR Operations (Cell Encryption) =====
    
    /// Encrypt with AES-128-CTR
    pub fn aes_128_ctr_encrypt(&self, key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Result<Vec<u8>> {
        // IPC call to BearDog - NEW METHOD NEEDED
    }
    
    /// Decrypt with AES-128-CTR
    pub fn aes_128_ctr_decrypt(&self, key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Result<Vec<u8>> {
        // IPC call to BearDog - NEW METHOD NEEDED
    }
    
    // ===== SHA3-256 Operations (KDF, Onion Addresses) =====
    
    /// Hash with SHA3-256
    pub fn sha3_256(&self, data: &[u8]) -> Result<[u8; 32]> {
        // IPC call to BearDog - NEW METHOD NEEDED
    }
    
    // ===== ChaCha20Poly1305 Operations (Optional Relay Encryption) =====
    
    /// Encrypt with ChaCha20Poly1305
    pub fn chacha20_poly1305_encrypt(&self, key: &[u8; 32], nonce: &[u8; 12], data: &[u8]) -> Result<Vec<u8>> {
        // IPC call to BearDog - ALREADY EXISTS
    }
}
```

### BearDog Extensions Required

**NEW Methods Needed**:

1. **AES-128-CTR** (Tor cell encryption)
   ```rust
   fn aes_128_ctr_encrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8>
   fn aes_128_ctr_decrypt(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Vec<u8>
   ```

2. **SHA3-256** (Tor KDFs, onion addresses)
   ```rust
   fn sha3_256(data: &[u8]) -> [u8; 32]
   ```

**Existing Methods** (reused):
- ✅ `ed25519_sign()` / `ed25519_verify()`
- ✅ `x25519_generate_ephemeral()` / `x25519_derive_secret()`
- ✅ `chacha20_poly1305_encrypt()` / `chacha20_poly1305_decrypt()`

---

## Storage Strategy

### Deployment-Specific Storage

| Deployment | Consensus | Circuit State | Descriptors | Strategy |
|------------|-----------|---------------|-------------|----------|
| **Minimal** (Cold Spore) | In-memory | In-memory | None | Fetch on demand |
| **Standard** (Live Spore) | Sled (1h TTL) | In-memory | Sled (24h TTL) | Cache hot data |
| **Robust** (Nest Atomic) | NestGate | In-memory | NestGate | Persistent cache |

### Storage Interface

```rust
// src/storage/mod.rs

pub trait Storage: Send + Sync {
    async fn store_consensus(&self, consensus: &Consensus) -> Result<()>;
    async fn load_consensus(&self) -> Result<Option<Consensus>>;
    
    async fn store_descriptor(&self, addr: &str, desc: &OnionDescriptor) -> Result<()>;
    async fn load_descriptor(&self, addr: &str) -> Result<Option<OnionDescriptor>>;
}

// In-memory implementation (default)
pub struct MemoryStorage {
    consensus: RwLock<Option<Consensus>>,
    descriptors: RwLock<HashMap<String, OnionDescriptor>>,
}

// Sled implementation (optional)
#[cfg(feature = "persistent-cache")]
pub struct SledStorage {
    db: sled::Db,
}
```

---

## Implementation Plan

### Phase 2A: Foundation (Days 1-2)

**Goal**: Directory protocol + relay selection

**Tasks**:
1. Set up `songbird-tor-protocol` crate structure
2. Implement directory authorities (hardcoded list)
3. Implement consensus fetching (HTTP GET)
4. Implement consensus parsing (nom)
5. Implement relay selection (guard/middle/hsdir)

**Deliverable**: Can fetch Tor consensus and select circuit paths

### Phase 2B: Circuit Building (Days 3-5)

**Goal**: Build 3-hop circuits

**Tasks**:
1. Implement cell encoding/decoding
2. Implement ntor handshake (CREATE2/CREATED2)
3. Implement circuit extension (EXTEND2/EXTENDED2)
4. Implement onion encryption (multi-hop)
5. Add BearDog AES-128-CTR delegation

**Deliverable**: Can build circuits through Tor network

### Phase 2C: Onion Client (Days 6-7)

**Goal**: Connect to .onion addresses

**Tasks**:
1. Implement stream protocol (RELAY_BEGIN/DATA/END)
2. Implement flow control (SENDME cells)
3. Implement onion address parsing
4. Integration with OnionConnector

**Deliverable**: Can connect to existing .onion services

### Phase 2D: Onion Service (Days 8-11)

**Goal**: Host .onion services

**Tasks**:
1. Implement descriptor generation
2. Implement descriptor upload to HSDir
3. Implement introduction protocol (INTRODUCE1/2)
4. Implement rendezvous protocol (RENDEZVOUS1/2)
5. Integration with OnionService

**Deliverable**: Can host .onion services

### Testing Strategy

**Unit Tests**:
- Cell encoding/decoding
- ntor handshake calculations
- Relay selection logic
- Descriptor generation

**Integration Tests**:
- Connect to real Tor directory authorities
- Build circuits through live Tor network
- Connect to existing .onion services (e.g., DuckDuckGo)

**E2E Tests**:
- Host .onion service
- Connect from Tor Browser
- Validate end-to-end encryption

---

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| **Consensus fetch** | <10s | Including parse |
| **Circuit build** | <5s | 3-hop circuit |
| **Stream begin** | <1s | After circuit exists |
| **Throughput** | >1 MB/s | Per circuit |
| **Descriptor upload** | <2s | To HSDir |

---

## Security Considerations

1. **TRUE PRIMAL Compliance**
   - ✅ All crypto via BearDog (no direct crypto)
   - ✅ Zero external dependencies
   - ✅ Memory safe (no unsafe blocks)

2. **Tor Protocol Security**
   - ✅ ntor handshake (forward secrecy)
   - ✅ Onion encryption (layered)
   - ✅ Stream isolation (per-stream keys)

3. **Network Security**
   - ⚠️ Guard fingerprinting (use consistent guards)
   - ⚠️ Traffic analysis (inherent Tor limitation)
   - ✅ Dark Forest compatible (family-only beacons)

---

## References

1. **Tor Specifications**
   - https://spec.torproject.org/tor-spec (Main protocol)
   - https://spec.torproject.org/rend-spec-v3 (Onion services v3)
   - https://spec.torproject.org/dir-spec (Directory protocol)

2. **Implementations**
   - https://github.com/torproject/tor (Reference C implementation)
   - https://gitlab.torproject.org/tpo/core/arti (Rust, but has C deps)

3. **Research Papers**
   - "Tor: The Second-Generation Onion Router" (Dingledine et al., 2004)
   - "Vanguards: Improving Security for Onion Services" (2019)

---

**Status**: Phase 2 Design Complete - Ready for Implementation

**Next**: Begin Phase 2A implementation (directory protocol)
