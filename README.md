# Songbird - Network Orchestration & Discovery Primal

**Version**: v0.3.0  
**Status**: Production Ready - Deep Debt S+ Tier  
**License**: AGPL-3.0  
**Edition**: Rust 2021 (migrating to 2024)

Songbird is the universal network orchestrator for the ecoPrimals ecosystem. It manages service discovery, connection management, and inter-primal communication across multiple protocols. All cryptographic operations are delegated to BearDog via JSON-RPC IPC.

## Quality

| Metric | Value |
|--------|-------|
| Safe Rust | 100% (`#![forbid(unsafe_code)]` across all crates) |
| Pure Rust | Zero C dependencies |
| Crypto Delegation | 100% BearDog via JSON-RPC IPC |
| Runtime Discovery | All config: env → XDG → smart defaults |
| Production Stubs | Zero (`todo!()` only in `#[cfg(test)]`) |
| Concurrent Tests | Zero `std::env::set_var` in tests (injectable env readers) |
| Tests | 9,254 registered, 4,874+ passing |
| Clippy Pedantic | 23/27 crates clean |
| Line Coverage | ~61% (target: 90%) |
| Build | Clean (zero errors, zero warnings) |
| Formatting | Clean (`cargo fmt --all`) |
| Docs | Clean (`cargo doc --workspace --no-deps`) |
| Files >1000 lines | 0 |

## Architecture

```
Application Layer (biomeOS Neural API, Squirrel, Gorilla)
    |
    | JSON-RPC 2.0 + tarpc
    v
Songbird Orchestrator
    |-- Tor Protocol (pure Rust: directory, circuit, stream, onion service)
    |-- Sovereign Onion (P2P encrypted service + connector)
    |-- IGD Router Config (UPnP IGD + NAT-PMP, auto port forwarding)
    |-- QUIC Transport (0-RTT, connection migration, multiplexing)
    |-- NFC Genesis (Dark Forest mobile pairing, zero metadata leakage)
    |-- BLE GATT (Bluetooth Low Energy genesis)
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
2. **Runtime Discovery** - All external services discovered at runtime by capability
3. **Capability-Based** - Request by capability, not by name
4. **Zero Hardcoding** - Environment-first configuration
5. **Pure Rust** - Zero C dependencies (ecoBin compliant)
6. **Safe Rust** - `#![forbid(unsafe_code)]` everywhere
7. **Event-Driven** - Zero polling anti-patterns (`tokio::sync::Notify`)
8. **Concurrent Testing** - Injectable env readers, no global state pollution
9. **JSON-RPC + tarpc First** - Primary IPC protocols

## Quick Start

```bash
cargo build --workspace --release
cargo run --bin songbird -- server
cargo run --bin songbird -- doctor
cargo run --bin songbird -- config show
```

### Environment Variables

```bash
export BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock
export SONGBIRD_PORT=3492
export SONGBIRD_IGD_ENABLED=true
export SONGBIRD_SOCKET=/run/user/$(id -u)/biomeos/songbird.sock
export SONGBIRD_FAMILY_ID=myfamily
```

## Crate Structure (27 crates)

### Core
- `songbird-orchestrator` - Main orchestration engine (7-stage startup)
- `songbird-cli` - Command-line interface (UniBin)
- `songbird-config` - Configuration management
- `songbird-types` - Shared type definitions
- `songbird-canonical` - Canonical type system

### Networking
- `songbird-universal-ipc` - Platform-agnostic JSON-RPC IPC
- `songbird-http-client` - Pure Rust TLS 1.3 HTTP client
- `songbird-network-federation` - Peer federation + rendezvous
- `songbird-discovery` - Service discovery (mDNS, DNS-SD, Dark Forest)
- `songbird-igd` - UPnP IGD + NAT-PMP router config

### Protocols
- `songbird-tor-protocol` - Pure Rust Tor (directory, circuit, stream, onion)
- `songbird-tls` - TLS 1.3 implementation
- `songbird-stun` - STUN server RFC 5389
- `songbird-quic` - QUIC transport (0-RTT, migration)
- `songbird-sovereign-onion` - P2P onion service
- `songbird-lineage-relay` - Lineage relay + coordinated punch
- `songbird-onion-relay` - Hole punch coordinator

### Hardware
- `songbird-nfc` - NFC genesis protocol
- `songbird-bluetooth` - BLE GATT service
- `songbird-genesis` - Physical genesis bootstrap

### Shared
- `songbird-universal` - Universal capability adapters
- `songbird-primal-coordination` - Primal coordination
- `songbird-registry` - Service registry
- `songbird-observability` - Metrics and tracing
- `songbird-execution-agent` - Task execution
- `songbird-compute-bridge` - Compute bridge
- `songbird-remote-deploy` - Remote deployment
- `songbird-test-utils` - Test utilities

## Testing

```bash
cargo test --workspace
cargo test -p songbird-tor-protocol --lib
cargo test -p songbird-sovereign-onion --features standalone
cargo llvm-cov --workspace --html
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`REMAINING_WORK.md`](REMAINING_WORK.md) | Current status and pending work |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guidelines |
| [`specs/`](specs/) | Technical specifications (113 files) |
| [`docs/`](docs/) | Architecture guides |

## License

AGPL-3.0 (scyBorg provenance trio: AGPL + ORC + CC-BY-SA)
