# Songbird - Network Orchestration & Discovery Primal

**Version**: v0.2.1  
**Status**: Production Ready - Deep Debt S+ Tier  
**License**: AGPL-3.0-only (scyBorg provenance trio)  
**Edition**: Rust 2024

Songbird is the universal network orchestrator for the ecoPrimals ecosystem. It manages service discovery, connection management, and inter-primal communication across multiple protocols. All cryptographic operations are delegated to BearDog via JSON-RPC IPC at runtime through capability discovery.

## Quality

| Metric | Value |
|--------|-------|
| Safe Rust | 100% (`#![forbid(unsafe_code)]` across all 30 crates; zero `unsafe` blocks) |
| Pure Rust | 100% — `quinn`/`rustls`/`ring` fully eliminated from `songbird-quic` (native QUIC engine with BearDog crypto delegation); `rcgen` removed; `sysinfo` eliminated; `ring-crypto` opt-in feature gate remains on CLI only |
| Crypto Delegation | BearDog via JSON-RPC IPC — TLS record layer, JWT, checkpoints, discovery, rendezvous all delegate via `CryptoProvider::call()`; graceful local fallback + `tracing::warn!` |
| Runtime Discovery | All config: env → XDG → smart defaults. `primal_names` constants module; capability-first discovery |
| Production panics | Zero (`panic!()`, `unreachable!()`, `todo!()` only in `#[cfg(test)]`) |
| Production `.unwrap()` | Zero (all in test modules — verified via line-by-line audit) |
| Production `FIXME`/`HACK` | Zero |
| Lint suppressions | `#[expect(reason)]` where lint fires; `#[allow(reason)]` where unfulfilled — zero stale expectations |
| Concurrent Tests | Injectable `_with` env readers; all tests fully concurrent (`#[serial_test::serial]` eliminated) |
| Tests | 11,831+ total, 0 failed, ~269 ignored |
| Line Coverage | ~69.11% (llvm-cov `--workspace --all-features`; target 90%) |
| Cast Safety | `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` denied workspace-wide |
| JSON-RPC Strict | Version validation, notification suppression, serialization-safe fallbacks across all 5 handlers |
| JSON-RPC Dispatch | Typed `JsonRpcMethod` enum routing (53+ methods, 14 domain sub-enums) — zero string matching in dispatch; `birdsong.schema` introspection |
| Clippy Pedantic | All 30 crates clean (`clippy::pedantic + nursery`, zero warnings, `--all-targets --all-features`) |
| Build | Clean (zero errors, zero warnings, ~43s dev) |
| Formatting | Clean (`cargo fmt --check`) |
| Docs | Clean (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`) |
| Files >1000 lines | 0 (`frame.rs` refactored → `frame/` 4 modules; max prod ~484 `gateway/mod.rs`) |
| License | `AGPL-3.0-only` via workspace inheritance; all crates use `license.workspace = true` |
| SPDX Headers | 100% of `.rs` files have `AGPL-3.0-only` — consistent with Cargo.toml and LICENSE body |
| JSON-RPC Gateway | 53+ semantic methods across 14 domain sub-enums (health, discovery, stun, relay, federation, tor, birdsong, ipc, etc.) |
| Nest Atomic | `health.liveness` + `health.readiness` + `health.check` + `capabilities.list` (14 capability tokens) |
| Method Normalization | `normalize_json_rpc_method_name()` in `songbird-types`; handles ecosystem naming drift |
| Lint Inheritance | 30/30 crates inherit workspace lints; 2 with justified custom tables |
| cargo-deny | Fully passing (advisories ok, bans ok, licenses ok, sources ok) |
| Dependencies | ~412 unique (`sysinfo`/`rayon`/`crossbeam` eliminated); `kube`/`k8s-openapi`/`bollard` feature-gated |
| UniBin | Single binary: `server`, `cli` (REPL), `compute-bridge`, `deploy`, `rendezvous` |
| Total Rust | ~381,498 lines across 30 crates |

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
    |-- QUIC Transport (native Rust — RFC 9000/9001/9002, BearDog crypto, 0-RTT, migration)
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
BearDog Crypto Primal (runtime capability discovery)
```

### Core Principles

1. **Self-Knowledge Only** - Each primal knows only itself
2. **Runtime Discovery** - All external services discovered at runtime by capability
3. **Capability-Based** - Request by capability, not by name
4. **Zero Hardcoding** - Environment-first configuration
5. **Pure Rust** - Zero C dependencies in Songbird code (ecoBin compliant)
6. **Safe Rust** - `#![forbid(unsafe_code)]` everywhere
7. **Event-Driven** - Zero polling anti-patterns (`tokio::sync::Notify`)
8. **Concurrent Testing** - Injectable `_with` env readers for fully concurrent tests
9. **JSON-RPC + tarpc First** - Primary IPC protocols

## Quick Start

```bash
cargo build --workspace --release
cargo run --bin songbird -- server
cargo run --bin songbird -- doctor
cargo run --bin songbird -- config show
cargo run --bin songbird -- compute-bridge
cargo run --bin songbird -- deploy
```

### Environment Variables

```bash
export CRYPTO_PROVIDER_SOCKET=/run/user/$(id -u)/biomeos/crypto.sock
export SECURITY_PROVIDER_SOCKET=/run/user/$(id -u)/biomeos/security.sock
export SONGBIRD_HTTP_PORT=3492
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_IGD_ENABLED=true
export SONGBIRD_FAMILY_ID=myfamily
```

## Crate Structure (30 crates)

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
- `songbird-quic` - Pure Rust QUIC transport (RFC 9000, BearDog crypto, 0-RTT, migration)
- `songbird-sovereign-onion` - P2P onion service
- `songbird-lineage-relay` - Lineage relay + coordinated punch
- `songbird-onion-relay` - Hole punch coordinator

### Hardware
- `songbird-nfc` - NFC genesis protocol
- `songbird-bluetooth` - BLE GATT service
- `songbird-genesis` - Physical genesis bootstrap

### Crypto
- `songbird-crypto-provider` - Shared crypto provider (Neural API + Direct BearDog routing)

### Shared
- `songbird-universal` - Universal capability adapters
- `songbird-primal-coordination` - Primal coordination
- `songbird-registry` - Service registry
- `songbird-observability` - Metrics and tracing
- `songbird-execution-agent` - Task execution
- `songbird-compute-bridge` - Compute bridge (UniBin subcommand)
- `songbird-remote-deploy` - Remote deployment (UniBin subcommand)
- `songbird-test-utils` - Test utilities
- `songbird-process-env` - Safe env var facade for Rust 2024 (`unsafe` isolation)

## Sovereign Beacon Mesh

Songbird provides a sovereign beacon mesh for encrypted peer discovery and relay coordination. The validated call sequence:

```
mesh.init            → Initialize mesh with node_id, family_id, listen endpoints
mesh.announce        → Announce presence as relay (advertise capabilities)
mesh.peers           → List reachable peers (direct, relay, onion, LAN)
mesh.status          → Mesh topology, reachable count, path types
```

Additional methods: `mesh.find_path` (best route to peer), `mesh.health_check` (probe peer connections), `mesh.auto_discover` (scan for new peers).

See [`specs/SOVEREIGN_BEACON_MESH_SPECIFICATION.md`](specs/SOVEREIGN_BEACON_MESH_SPECIFICATION.md) for the full protocol specification.

## Testing

```bash
cargo test --workspace --all-features          # Full suite (11,831 tests)
cargo test -p songbird-tor-protocol --lib      # Single crate
./scripts/test-with-beardog.sh                 # With live BearDog from plasmidBin
./scripts/coverage.sh                          # llvm-cov HTML report
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`REMAINING_WORK.md`](REMAINING_WORK.md) | Current status and pending work |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guidelines |
| [`CONTEXT.md`](CONTEXT.md) | AI-ingestible project context |
| [`specs/`](specs/) | Technical specifications |
| [`docs/architecture/`](docs/architecture/) | Architecture guides |

## License

AGPL-3.0-only (scyBorg provenance trio: AGPL-3.0-only + ORC + CC-BY-SA 4.0)

See `LICENSE`, `LICENSE-ORC`, and `LICENSE-CC-BY-SA` at repository root.
