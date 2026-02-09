# 🧅 Tor Integration Roadmap - Songbird Evolution

**Date**: February 7, 2026  
**Status**: Phase 1 Active - Tor Daemon Integration  
**Vision**: Pure Rust Tor Protocol (Phase 2)

---

## 🎯 Executive Summary

Songbird is evolving to support Tor network integration in **two phases**:

1. **Phase 1 (Immediate)**: Tor daemon for symmetric NAT validation ⚡
2. **Phase 2 (Future)**: Pure Rust Tor protocol implementation 🦀

**Key Principle**: Maintain TRUE PRIMAL architecture - BearDog handles all crypto, Songbird handles networking.

---

## 📋 Phase 1: Tor Daemon Integration (Current)

### Goals

- ✅ Enable symmetric NAT traversal via Tor network
- ✅ Validate Dark Forest architecture with Tor routing
- ✅ Test family-only access through .onion addresses
- ✅ Maintain existing Songbird architecture

### Architecture: Phase 1

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tower (biomeOS Nest)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   BearDog    │────▶│   Songbird   │────▶│  Tor Daemon  │    │
│  │ (crypto)     │     │ (orchestr.)  │     │ (external)   │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
│         │                    │                    │             │
│         ▼                    ▼                    ▼             │
│  Ed25519 keys         Listen :3492        Tor Network          │
│  X25519 ECDH          Listen :9901        .onion routing       │
│  ChaCha20Poly1305     P2P Onion Service   Hidden service       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                               │
                               │ Tor Network (open infrastructure)
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    USB Spore / Pixel                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   BearDog    │────▶│   Songbird   │────▶│  Tor Client  │    │
│  │ (crypto)     │     │ (connector)  │     │ (SOCKS5)     │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
│                              │                    │             │
│                              ▼                    ▼             │
│                       P2P Onion Connector  Route via Tor       │
│                       Connect to .onion    (Orbot/torify)      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Integration Points

**Existing Songbird Components** (Reuse):
- ✅ `songbird-sovereign-onion` - P2P Service/Connector (already complete!)
- ✅ `OnionService` - TCP listener (bind to 127.0.0.1:3492)
- ✅ `OnionConnector` - P2P client (via SOCKS5 proxy)
- ✅ `OnionIdentity` - Ed25519 identity (BearDog delegation)
- ✅ Dark Forest beacons - Encrypted family announcements

**New Integration Layer**:
- 🆕 Tor daemon wrapper (systemd integration)
- 🆕 SOCKS5 proxy support for `OnionConnector`
- 🆕 .onion address mapping (Songbird identity ↔ Tor hidden service)

### Configuration Changes

**Tower (`/etc/tor/torrc`)**:
```
# Songbird Hidden Service
HiddenServiceDir /var/lib/tor/songbird_hs/
HiddenServicePort 3492 127.0.0.1:3492
HiddenServicePort 9901 127.0.0.1:9901  # IPC endpoint
```

**Songbird Config** (new env vars):
```bash
# Tor Integration (Phase 1)
SONGBIRD_TOR_ENABLED=true
SONGBIRD_TOR_SOCKS_PROXY=127.0.0.1:9050
SONGBIRD_TOR_HIDDEN_SERVICE_DIR=/var/lib/tor/songbird_hs
SONGBIRD_ONION_SERVICE_PORT=3492
```

### Implementation Tasks - Phase 1

| Task | Component | Effort | Status |
|------|-----------|--------|--------|
| Add Tor daemon systemd integration | songbird-orchestrator | 2h | 🔲 TODO |
| Add SOCKS5 proxy support | songbird-sovereign-onion | 3h | 🔲 TODO |
| Update OnionConnector for Tor | songbird-sovereign-onion | 2h | 🔲 TODO |
| Read Tor-generated .onion | songbird-orchestrator | 1h | 🔲 TODO |
| Update Dark Forest beacons | songbird-discovery | 2h | 🔲 TODO |
| Integration testing | All | 3h | 🔲 TODO |

**Total Effort**: ~13 hours (1-2 days)

---

## 🦀 Phase 2: Pure Rust Tor Protocol (Future)

### Vision

Implement minimal Tor protocol **entirely in Pure Rust** within Songbird:
- ✅ Zero external dependencies (no Tor daemon, no Arti, no C)
- ✅ TRUE PRIMAL architecture (BearDog for crypto)
- ✅ Minimal protocol subset (onion services only)
- ✅ Memory-safe, async/await, modern Rust

### Architecture: Phase 2

```
┌─────────────────────────────────────────────────────────────────┐
│                Songbird Pure Rust Tor (Phase 2)                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  songbird-tor-protocol (NEW CRATE - ~2,600 lines)         │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │  Directory Protocol (~500 lines)                          │  │
│  │  ├─ Fetch consensus from authorities                      │  │
│  │  ├─ Parse relay descriptors                               │  │
│  │  └─ Select guard/middle/exit nodes                        │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │  Circuit Protocol (~800 lines)                            │  │
│  │  ├─ CREATE/CREATED cells (handshake)                      │  │
│  │  ├─ EXTEND/EXTENDED cells (extension)                     │  │
│  │  └─ RELAY cells (encrypted comms)                         │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │  Onion Service Protocol (~1,000 lines)                    │  │
│  │  ├─ Generate blinded keys                                 │  │
│  │  ├─ Publish descriptors to HSDir                          │  │
│  │  └─ Handle INTRODUCE/RENDEZVOUS                           │  │
│  ├──────────────────────────────────────────────────────────┤  │
│  │  Stream Protocol (~300 lines)                             │  │
│  │  ├─ RELAY_BEGIN/CONNECTED/DATA/END                        │  │
│  │  └─ Flow control                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                   │
│                              ▼ 100% BearDog Delegation           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  BearDog Crypto Delegation (TRUE PRIMAL)                  │  │
│  │  ├─ Ed25519 signing (onion identity, circuit auth)        │  │
│  │  ├─ X25519 DH (circuit key exchange, ntor)                │  │
│  │  ├─ AES-128-CTR (cell encryption)                         │  │
│  │  ├─ SHA3-256 (cell digests, KDFs)                         │  │
│  │  └─ ChaCha20Poly1305 (optional relay encryption)          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### New Crate: `songbird-tor-protocol`

**Purpose**: Minimal Tor protocol implementation for onion services

**Cargo.toml**:
```toml
[package]
name = "songbird-tor-protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["net", "io-util", "time"] }

# Parsing
nom = "7"  # Tor cell parsing

# Storage (optional)
sled = { version = "0.34", optional = true }

# Crypto (via BearDog delegation)
# NO direct crypto dependencies!

[features]
default = []
persistent-cache = ["sled"]  # Optional consensus caching
```

**Directory Structure**:
```
crates/songbird-tor-protocol/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                    # Public API
│   ├── directory/                # Directory Protocol (~500 lines)
│   │   ├── mod.rs
│   │   ├── authorities.rs        # Directory authority list
│   │   ├── consensus.rs          # Consensus download/parse
│   │   └── descriptors.rs        # Relay descriptor parsing
│   ├── circuit/                  # Circuit Protocol (~800 lines)
│   │   ├── mod.rs
│   │   ├── create.rs             # CREATE/CREATED handshake
│   │   ├── extend.rs             # EXTEND/EXTENDED
│   │   ├── relay.rs              # RELAY cells
│   │   └── manager.rs            # Circuit lifecycle
│   ├── onion_service/            # Onion Service Protocol (~1,000 lines)
│   │   ├── mod.rs
│   │   ├── descriptor.rs         # Descriptor generation/publish
│   │   ├── introduce.rs          # INTRODUCE1/2
│   │   ├── rendezvous.rs         # RENDEZVOUS1/2
│   │   └── hsdir.rs              # HSDir upload
│   ├── stream/                   # Stream Protocol (~300 lines)
│   │   ├── mod.rs
│   │   ├── begin.rs              # RELAY_BEGIN/CONNECTED
│   │   ├── data.rs               # RELAY_DATA
│   │   └── control.rs            # Flow control
│   ├── crypto/                   # BearDog delegation wrappers
│   │   ├── mod.rs
│   │   └── beardog_client.rs     # Crypto delegation layer
│   ├── protocol/                 # Tor protocol primitives
│   │   ├── mod.rs
│   │   ├── cells.rs              # Cell encoding/decoding
│   │   └── constants.rs          # Tor constants
│   └── error.rs                  # Error types
└── tests/
    ├── directory_test.rs
    ├── circuit_test.rs
    └── integration_test.rs
```

### Implementation Roadmap - Phase 2

| Component | Lines | Days | Priority | Dependencies |
|-----------|-------|------|----------|--------------|
| **Directory Protocol** | ~500 | 2 | P0 | None |
| - Authorities list | 50 | 0.2 | P0 | None |
| - Consensus fetch | 200 | 1 | P0 | None |
| - Descriptor parse | 250 | 0.8 | P0 | Consensus |
| **Circuit Protocol** | ~800 | 3 | P0 | Directory |
| - CREATE/CREATED | 200 | 0.8 | P0 | BearDog X25519 |
| - EXTEND/EXTENDED | 250 | 1 | P0 | CREATE |
| - RELAY cells | 350 | 1.2 | P0 | BearDog AES-CTR |
| **Onion Client** | ~400 | 2 | P1 | Circuit |
| - Connect to .onion | 200 | 1 | P1 | Circuit |
| - Stream handling | 200 | 1 | P1 | Circuit |
| **Onion Service** | ~1,000 | 4 | P2 | Circuit |
| - Descriptor gen | 300 | 1.2 | P2 | BearDog Ed25519 |
| - INTRODUCE | 350 | 1.4 | P2 | Circuit |
| - RENDEZVOUS | 350 | 1.4 | P2 | Circuit |
| **Stream Protocol** | ~300 | 1 | P1 | Circuit |

**Total**: ~2,600 lines, ~11 days effort

### Crypto Requirements (BearDog Delegation)

| Operation | Tor Usage | BearDog Method |
|-----------|-----------|----------------|
| **Ed25519 signing** | Onion identity, descriptor | `ed25519_sign()` |
| **Ed25519 verify** | Relay verification | `ed25519_verify()` |
| **X25519 ECDH** | ntor handshake | `x25519_derive_secret()` |
| **AES-128-CTR** | Cell encryption | `aes_128_ctr_encrypt()` ⚠️ NEW |
| **SHA3-256** | KDF, digests | `sha3_256()` ⚠️ NEW |
| **ChaCha20Poly1305** | Optional relay | `chacha20_poly1305_encrypt()` ✅ |

**BearDog Extensions Needed**:
- `aes_128_ctr_encrypt()` / `aes_128_ctr_decrypt()` - Tor uses AES-CTR
- `sha3_256()` - Tor uses SHA3 for onion addresses and KDFs

### Storage Strategy (Phase 2)

| Deployment | Consensus Cache | Circuit State | Descriptor Cache |
|------------|-----------------|---------------|------------------|
| **Minimal** (Cold Spore) | In-memory | In-memory | None |
| **Standard** (Live Spore) | Sled (1-4h TTL) | In-memory | Sled (24h TTL) |
| **Robust** (Nest Atomic) | NestGate | In-memory | NestGate |

**Philosophy**: Tor consensus is ~2MB, changes hourly. Cache when feasible, fetch when needed.

---

## 🔄 Migration Path

### Current State (Feb 7, 2026)

| Component | Status | Tor Ready? |
|-----------|--------|------------|
| `songbird-sovereign-onion` | ✅ Complete | 🔄 Needs SOCKS5 |
| `OnionService` | ✅ TCP listener | ✅ Tor daemon compat |
| `OnionConnector` | ✅ Direct TCP | 🔄 Needs SOCKS5 |
| Dark Forest beacons | ✅ Encrypted | ✅ Can embed .onion |
| BearDog crypto | ✅ Ed25519, X25519, ChaCha | 🔄 Needs AES-CTR, SHA3 |

### Phase 1 Migration (1-2 days)

1. **Add SOCKS5 support** to `OnionConnector`
   - Tor client connects via SOCKS5 proxy (127.0.0.1:9050)
   - Standard Tor protocol, no code changes needed

2. **Read Tor .onion** from hidden service dir
   - Songbird reads `/var/lib/tor/songbird_hs/hostname`
   - Optionally: Use Songbird's own Ed25519 identity (advanced)

3. **Update beacons** with Tor .onion address
   - Dark Forest beacon includes .onion endpoint
   - Family-only decryption (existing pattern)

4. **Test validation**
   - Tower → Tor daemon → Tor network → .onion
   - Pixel → Tor client (Orbot) → Tor network → Tower's .onion
   - Symmetric NAT traversal verified ✅

### Phase 2 Migration (11 days)

1. **Week 1**: Directory + Circuit protocols
   - Fetch Tor consensus
   - Build 3-hop circuits
   - Test outbound connections

2. **Week 2**: Onion services
   - Publish descriptors
   - Handle introductions
   - Rendezvous point coordination

3. **Integration**: Replace Tor daemon
   - `songbird-tor-protocol` provides same API as Phase 1
   - Zero external dependencies
   - Pure Rust, TRUE PRIMAL compliant

---

## 📊 Benefits Analysis

### Phase 1 Benefits (Immediate)

| Benefit | Impact |
|---------|--------|
| **Symmetric NAT traversal** | Solves hardest networking problem ✅ |
| **No port forwarding** | Works everywhere (cellular, corporate) ✅ |
| **Proven infrastructure** | Tor network is battle-tested ✅ |
| **Quick validation** | Install & test in hours, not weeks ✅ |
| **Dark Forest compatible** | Family-only access via encrypted beacons ✅ |

### Phase 2 Benefits (Future)

| Benefit | Impact |
|---------|--------|
| **Zero external deps** | No Tor daemon, no Arti, no C code ✅ |
| **TRUE PRIMAL compliance** | 100% BearDog crypto delegation ✅ |
| **Full control** | Customize protocol, optimize for biomeOS ✅ |
| **Memory safe** | Pure Rust, no unsafe blocks ✅ |
| **Minimal footprint** | ~2,600 lines vs. Tor's 220,000+ lines ✅ |

---

## 🎯 Success Criteria

### Phase 1 (Tor Daemon)

- [ ] Tor daemon installed and configured on Tower
- [ ] Songbird OnionService listening on 127.0.0.1:3492
- [ ] Tor hidden service maps to OnionService
- [ ] .onion address generated and readable
- [ ] Dark Forest beacon updated with .onion
- [ ] Pixel connects via Tor to Tower's .onion
- [ ] Symmetric NAT traversal validated
- [ ] Family-only access enforced (beacon encryption)

### Phase 2 (Pure Rust)

- [ ] `songbird-tor-protocol` crate created
- [ ] Directory protocol: Fetch consensus
- [ ] Circuit protocol: Build 3-hop circuits
- [ ] Onion client: Connect to .onion addresses
- [ ] Onion service: Accept connections at .onion
- [ ] BearDog extensions: AES-CTR, SHA3
- [ ] Zero external dependencies (no Tor daemon)
- [ ] Integration tests pass
- [ ] Performance acceptable (circuit build <5s)
- [ ] Documentation complete

---

## 📚 References

### Tor Specifications

- **Tor Protocol**: https://spec.torproject.org/tor-spec
- **Onion Service (v3)**: https://spec.torproject.org/rend-spec-v3
- **Directory Protocol**: https://spec.torproject.org/dir-spec
- **Cell Formats**: https://spec.torproject.org/tor-spec/cell-formats.html

### Implementation Guides

- **Tor Protocol Overview**: https://2019.www.torproject.org/about/overview.html
- **Onion Services**: https://community.torproject.org/onion-services/
- **ntor Handshake**: https://spec.torproject.org/tor-spec/create-created-cells.html#ntor

### Existing Implementations

- **Tor (C)**: https://github.com/torproject/tor (~220k lines)
- **Arti (Rust)**: https://gitlab.torproject.org/tpo/core/arti (~50k lines, but has C deps)
- **Our approach**: Minimal subset (~2.6k lines, pure Rust)

---

## 🚀 Next Actions

### Immediate (Phase 1)

1. **User Action**: Install Tor daemon
   ```bash
   sudo apt install tor
   ```

2. **Configure hidden service**
   ```bash
   sudo tee -a /etc/tor/torrc << EOF
   HiddenServiceDir /var/lib/tor/songbird_hs/
   HiddenServicePort 3492 127.0.0.1:3492
   HiddenServicePort 9901 127.0.0.1:9901
   EOF
   ```

3. **Restart Tor**
   ```bash
   sudo systemctl restart tor
   sudo cat /var/lib/tor/songbird_hs/hostname  # Get .onion address
   ```

4. **Songbird updates** (engineering)
   - Add SOCKS5 support to OnionConnector
   - Read Tor .onion from hidden service dir
   - Update Dark Forest beacons

### Future (Phase 2)

1. **BearDog extensions**
   - Add `aes_128_ctr_*()` methods
   - Add `sha3_256()` method

2. **Create `songbird-tor-protocol` crate**
   - Start with directory protocol
   - Implement circuit building
   - Add onion service support

3. **Integration**
   - Replace Tor daemon with pure Rust
   - Validate feature parity
   - Performance tuning

---

**Status**: 🎯 **PHASE 1 READY FOR IMPLEMENTATION**

Phase 1 enables immediate symmetric NAT traversal with Tor daemon. Phase 2 provides pure Rust implementation with zero external dependencies.

🧅 **Tor Network** | 🔐 **TRUE PRIMAL** | 🦀 **Pure Rust** | 🌲 **Dark Forest**
