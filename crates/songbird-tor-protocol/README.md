# 🧅 songbird-tor-protocol

**Pure Rust Tor Protocol** for Songbird - Minimal implementation for .onion services

**Version**: 0.1.0  
**Status**: 🔄 Phase 2A Active - Directory Protocol Implementation  
**TRUE PRIMAL**: ✅ 100% BearDog Crypto Delegation

---

## Overview

Minimal Tor protocol implementation in Pure Rust for hosting and connecting to .onion services. This crate implements only the subset of Tor needed for onion services, avoiding the complexity of full Tor client functionality.

**Key Features**:
- ✅ **Pure Rust**: Zero external dependencies (no Tor daemon, no C code)
- ✅ **TRUE PRIMAL**: 100% BearDog crypto delegation (no direct crypto)
- ✅ **Memory Safe**: Zero unsafe blocks, async/await
- ✅ **Minimal**: ~2,600 lines vs. Tor's 220k+ lines
- ✅ **Modern**: Tokio async, Result<T>, idiomatic Rust

---

## Components

### Directory Protocol (~500 lines)
- Fetch consensus from directory authorities
- Parse relay descriptors  
- Select circuit paths (guard, middle, hsdir)

### Circuit Protocol (~800 lines)
- ntor handshake (CREATE2/CREATED2)
- Circuit extension (EXTEND2/EXTENDED2)
- Onion encryption (layered multi-hop)

### Onion Service Protocol (~1,000 lines)
- Descriptor generation and upload
- Introduction points (INTRODUCE1/2)
- Rendezvous protocol (RENDEZVOUS1/2)

### Stream Protocol (~300 lines)
- Stream multiplexing (RELAY_BEGIN/DATA/END)
- Flow control (SENDME cells)

---

## Usage

### Fetch Tor Consensus

```rust
use songbird_tor_protocol::directory::Consensus;
use songbird_tor_protocol::crypto::BeardogCryptoClient;

let beardog = BeardogCryptoClient::from_env()?;
let consensus = Consensus::fetch(&beardog).await?;

// Select circuit path
let path = consensus.select_path()?;
println!("Guard: {}", path.guard.address);
println!("Middle: {}", path.middle.address);
println!("HSDir: {}", path.hsdir.address);
```

### Build Circuit (Phase 2B)

```rust
use songbird_tor_protocol::circuit::Circuit;

let circuit = Circuit::build(&path, &beardog).await?;
println!("Circuit ID: {}", circuit.id());
```

### Connect to .onion (Phase 2C)

```rust
use songbird_tor_protocol::TorClient;

let client = TorClient::new(beardog);
let stream = client.connect("abc123...xyz.onion", 80).await?;
stream.write_all(b"GET / HTTP/1.1\r\n\r\n").await?;
```

### Host .onion Service (Phase 2D)

```rust
use songbird_tor_protocol::TorService;

let service = TorService::new(beardog, 8080).await?;
println!("Onion address: {}", service.onion_address());

service.listen().await?; // Accept connections
```

---

## TRUE PRIMAL Architecture

**All crypto delegated to BearDog** (Zero direct crypto in this crate):

| Operation | Tor Usage | BearDog Method |
|-----------|-----------|----------------|
| **Ed25519 signing** | Identity, descriptors | `ed25519_sign()` |
| **Ed25519 verify** | Consensus validation | `ed25519_verify()` |
| **X25519 ECDH** | ntor handshake | `x25519_derive_secret()` |
| **AES-128-CTR** | Cell encryption | `aes_128_ctr_encrypt()` ⚠️ NEW |
| **SHA3-256** | KDFs, onion addresses | `sha3_256()` ⚠️ NEW |

**BearDog Extensions Needed**:
- `aes_128_ctr_encrypt/decrypt` - Tor uses AES-CTR for cell encryption
- `sha3_256` - Tor uses SHA3 for KDFs and onion address derivation

---

## Implementation Status

**Phase 2A: Directory Protocol** (Days 1-2) - 🔄 40% Complete
- [x] Crate structure
- [x] Directory authorities (9 hardcoded)
- [ ] Consensus fetching
- [ ] Consensus parsing
- [ ] Relay selection

**Phase 2B: Circuit Building** (Days 3-5) - 🔲 TODO
**Phase 2C: Onion Client** (Days 6-7) - 🔲 TODO  
**Phase 2D: Onion Service** (Days 8-11) - 🔲 TODO

---

## References

- [Tor Protocol Spec](https://spec.torproject.org/tor-spec)
- [Onion Service Spec v3](https://spec.torproject.org/rend-spec-v3)
- [Directory Protocol](https://spec.torproject.org/dir-spec)

---

## License

AGPL-3.0

---

**TRUE PRIMAL** | **Pure Rust** | **Zero Unsafe** | **BearDog Delegation**
