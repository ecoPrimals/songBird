# Songbird - Network Orchestration & Discovery Primal

**Version**: v0.2.2  
**Status**: Production Ready - Deep Debt S+ Tier  
**License**: AGPL-3.0

Songbird is the universal network orchestrator for the ecoPrimals ecosystem. It manages service discovery, connection management, and inter-primal communication across multiple protocols. All cryptographic operations are delegated to BearDog via JSON-RPC IPC.

## Quality

| Metric | Value |
|--------|-------|
| Safe Rust | 100% (`#![forbid(unsafe_code)]` across all crates) |
| Pure Rust | Zero C dependencies |
| Crypto Delegation | 100% BearDog via JSON-RPC IPC (rustls-rustcrypto default, ring optional via `ring-crypto` feature) |
| Runtime Discovery | All config: env → XDG → smart defaults |
| Production Stubs | Zero (`todo!()` only in `#[cfg(test)]`) |
| Concurrent Tests | Zero `std::env::set_var` in tests (injectable env readers) |
| Lib Tests | 8,515+ passing |
| Line Coverage | 60.84% |
| Build | Clean (zero errors, zero clippy errors) |

## Architecture

```
Application Layer (biomeOS Neural API, Squirrel, Gorilla)
    |
    | JSON-RPC 2.0
    v
Songbird Orchestrator
    |-- Tor Protocol (pure Rust: directory, circuit, stream, onion service)
    |-- Sovereign Onion (P2P encrypted service + connector)
    |-- IGD Router Config (UPnP IGD + NAT-PMP, auto port forwarding)
    |-- QUIC Transport (0-RTT, connection migration, multiplexing)
    |-- NFC Genesis (Dark Forest mobile pairing, zero metadata leakage)
    |-- TLS 1.3 (RFC 8446, protocol detection)
    |-- STUN Server (RFC 5389, NAT discovery, port pattern probing)
    |-- Relay Server (lineage-based auth, packet forwarding, coordinated punch)
    |-- Dark Forest Discovery (encrypted beacons, mDNS, DNS-SD)
    |-- Universal IPC (Unix sockets, TCP, platform-agnostic)
    |
    | BearDog delegation (Ed25519, X25519, ChaCha20, SHA3-256, AES-128-CTR)
    v
BearDog Crypto Primal
```

### Core Principles

1. **Self-Knowledge Only** - Each primal knows only itself
2. **Runtime Discovery** - All external services discovered at runtime
3. **Capability-Based** - Request by capability, not by name
4. **Zero Hardcoding** - Environment-first configuration
5. **Pure Rust** - Zero C dependencies
6. **Safe Rust** - `#![forbid(unsafe_code)]` everywhere
7. **Event-Driven** - Zero polling anti-patterns (`tokio::sync::Notify` everywhere)
8. **Concurrent Testing** - Injectable env readers, no global state pollution

## Quick Start

```bash
# Build
cargo build --workspace --release

# Run server
cargo run --bin songbird -- server

# Health check
cargo run --bin songbird -- doctor

# Configuration
cargo run --bin songbird -- config show
```

### Environment Variables

```bash
# BearDog crypto provider (rustls-rustcrypto default; use --features ring-crypto for ring)
export BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock

# Network
export SONGBIRD_PORT=3492
export SONGBIRD_IGD_ENABLED=true  # Auto router port forwarding

# IPC
export SONGBIRD_SOCKET=/run/user/$(id -u)/biomeos/songbird.sock

# Multi-family support (default: "default")
export SONGBIRD_FAMILY_ID=myfamily
```

## Crate Structure

### Core
- `songbird-orchestrator` - Main orchestration engine (7-stage startup)
- `songbird-cli` - Command-line interface (UniBin)
- `songbird-config` - Configuration management
- `songbird-types` - Shared type definitions

### Networking
- `songbird-universal-ipc` - Platform-agnostic JSON-RPC IPC
- `songbird-http-client` - Pure Rust TLS 1.3 HTTP client
- `songbird-network-federation` - Peer federation + rendezvous
- `songbird-discovery` - Service discovery
- `songbird-igd` - UPnP IGD + NAT-PMP router config

### Protocols
- `songbird-tor-protocol` - Pure Rust Tor (directory, circuit, stream, onion)
- `songbird-tls` - TLS 1.3 implementation
- `songbird-stun` - STUN server RFC 5389
- `songbird-quic` - QUIC transport (0-RTT, migration)
- `songbird-sovereign-onion` - P2P onion service

### Security & P2P
- `songbird-lineage-relay` - Lineage relay + BearDog auth + coordinated punch
- `songbird-onion-relay` - Hole punch coordinator
- `songbird-nfc` - NFC genesis protocol
- `songbird-bluetooth` - BLE GATT service

### Shared
- `songbird-universal` - Universal capability adapters
- `songbird-primal-coordination` - Primal coordination
- `songbird-registry` - Service registry
- `songbird-observability` - Metrics and tracing

## Testing

```bash
# All library tests
cargo test --workspace --lib

# Specific crate
cargo test -p songbird-tor-protocol --lib

# Sovereign onion with crypto verification
cargo test -p songbird-sovereign-onion --features standalone

# Coverage
cargo llvm-cov --workspace --html
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`REMAINING_WORK.md`](REMAINING_WORK.md) | Current status and pending work |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guidelines |
| [`specs/`](specs/) | Technical specifications |
| [`docs/`](docs/) | Architecture guides |

## License

AGPL-3.0
