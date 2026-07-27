# Songbird - Network Orchestration & Discovery Primal

**Version**: v0.2.1-wave155b  
**Status**: Production Ready - Deep Debt S+ Tier  
**License**: AGPL-3.0-or-later (scyBorg provenance trio)  
**Edition**: Rust 2024  
**Last Updated**: July 27, 2026

Songbird is the universal network orchestrator and **inner membrane port solver** for the ecoPrimals ecosystem. It manages service discovery, connection management, inter-primal communication across multiple protocols, and drawbridge capability→port resolution for production routing. All cryptographic operations are delegated to the security provider capability (`security.sock` / `SECURITY_PROVIDER_SOCKET`) via JSON-RPC IPC at runtime through capability-based discovery.

Songbird is one third of **Tower Atomic** (bearDog + songBird + skunkBat) — the sovereign transport stack targeting WireGuard replacement.

## Quality

| Metric | Value |
|--------|-------|
| Safe Rust | 100% (`#![forbid(unsafe_code)]` across all 31 crates; zero `unsafe` blocks) |
| Pure Rust | 100% — native QUIC engine with security provider crypto delegation; `rcgen` eliminated (pure Rust test cert gen via `ed25519-dalek` + DER); `ring-crypto` feature removed Wave 135 (SB-02 resolved) |
| Crypto Delegation | Security provider via JSON-RPC IPC — TLS record layer, JWT, checkpoints, discovery, rendezvous all delegate via `CryptoProvider::call()`; graceful local fallback + `tracing::warn!` |
| Runtime Discovery | All config: env → XDG → smart defaults; capability-based biomeos socket probing via `BIOMEOS_RUNTIME_SUBDIR` constant; **LD-08 socket auto-discovery** seeds `ipc.resolve` registry at startup and **self-heals every 30s** (Wave 139) by scanning `$XDG_RUNTIME_DIR/biomeos/*.sock` and probing with `identity.get` + `capabilities.list` (Wire Standard L3); netdev-based IP detection; all ports env-configurable; XDG-compliant socket paths; mDNS via `MDNS_MULTICAST_GROUP` constant; **DH-1 compliant** (Wave 60+67): zero `/tmp` writes — security socket honors `--security-socket`/`BEARDOG_SOCKET`/`SECURITY_PROVIDER_SOCKET` → XDG → `/var/run/biomeos/`; data→`$XDG_DATA_HOME/songbird`, cache→`$XDG_CACHE_HOME/songbird`, VPS→`/var/lib/songbird` |
| Production panics | Zero `panic!()` / `todo!()` in production; 2 provably-unreachable `unreachable!()` in QUIC VarInt (2-bit prefix exhaustive match, documented) |
| Production `.unwrap()` | Zero unguarded — `.unwrap()` in production only under `#[expect(clippy::unwrap_used, reason = "...")]` for provably infallible operations (e.g. `write!` to `String`); all others in `#[cfg(test)]` or doc examples |
| Production `FIXME`/`HACK` | Zero |
| Lint suppressions | `#[allow(reason)]` / `#[expect(reason)]` throughout — Wave 150x: 47 `#[allow(dead_code)]` evolved to `#[expect(dead_code)]` (warns if code gets wired); Wave 58: 146 item-level suppressions evolved to `#[expect(clippy::...)]`; module-level `unwrap_used`/`expect_used` blanket suppressions remain `#[allow]` (correct for module scope); Wave 149: blanket `#![allow(clippy::all, pedantic, nursery)]` removed from 11 files; zero reasonless suppressions, zero blanket suppressions remain |
| Concurrent Tests | Injectable env via `songbird-process-env` overlay (all production env sites migrated — zero `std::env` in production); all tests fully concurrent; `#[serial_test]` fully eliminated (0 suites); `tokio::time::pause()` for deterministic timing |
| Tests | 14,835+ total tests (667 in universal-ipc; integration suites) |
| Line Coverage | **73.41%** (`llvm-cov --workspace --lib`, Apr 27 2026; target 90%; Wave 53: +74 tests across pure-logic modules) |
| Cast Safety | `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, `cast_possible_wrap` denied workspace-wide |
| JSON-RPC Strict | Version validation, notification suppression, serialization-safe fallbacks across all dispatch handlers |
| JSON-RPC Dispatch | Typed `JsonRpcMethod` enum routing (58+ methods, 34 domain sub-enums including `Btsp`, `Lifecycle` and `Inference`) — zero string matching in dispatch; `birdsong.schema` introspection; `normalize_json_rpc_method_name()` absorbs `discovery.find_by_capability`, `net.discovery.find_by_capability`, `model.*`, `ai.*` aliases; Wave 60: `mesh.discover_remotes`, `mesh.mirror`, `mesh.publish`; Wave 70: `mesh.probe_latency`; Wave 74: `ipc.relay_stats`; Wave 75: `mesh.capabilities_announce` |
| Clippy Pedantic | All 31 crates clean (`clippy::pedantic + nursery`, zero warnings, `--all-targets`; Jul 21 2026 verified); **Windows cross-compile zero warnings** (`x86_64-pc-windows-gnu`) |
| Build | Clean (zero errors, zero warnings; cross-platform verified) |
| Formatting | Clean (`cargo fmt --check`; Jul 21 verified) |
| Docs | Clean (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`) |
| Files >800 lines | **0** (max 795L) — Wave 151b: deep debt audit confirmed. `mesh_handler/mod.rs` (795L), `drawbridge.rs` (749L). Previous: Wave 151a: `mesh_handler/mod.rs` (825→797L via `parse_peer_list()` dedup). Wave 152: `security.rs` (761→mod tree, max 277L), `production.rs` (754→370L via test extraction). Wave 150t: `mesh_handler/mod.rs` (841→746L); Wave 149b: `drawbridge.rs` (1,019→578L), `mesh_seed.rs` (834→523L) |
| License | `AGPL-3.0-or-later` via workspace inheritance; all crates use `license.workspace = true` (`AGPL-3.0-only` drift eliminated) |
| SPDX Headers | 100% of `.rs` files have `AGPL-3.0-or-later` — consistent with Cargo.toml and LICENSE body |
| JSON-RPC Gateway | 53+ semantic methods across 33 domain sub-enums (health, discovery, stun, relay, federation, tor, birdsong, ipc, lifecycle, inference, etc.) |
| Wire Standard L3 | `capabilities.list` returns L3 envelope: `{primal, version, methods, provided_capabilities, consumed_capabilities, protocol, transport}`; `identity.get` returns `{primal, version, domain, license}`; `capabilities.methods` token→method map |
| riboCipher (Stream 7) | Transport signal detection in all 3 accept loops (`pure_rust_server`, `bin_interface`, `http_server`). Signal bytes: `0xEC` (clear), `0xED` (mito/federation), `0xEE` (nuclear). Deterministic routing before protocol detection. Deprecation: WARN (111-112) → ERROR (112) → REJECT (113) → REMOVE (114) |
| BTSP Phase 2 | ClientHello/ChallengeResponse handshake client + server; `perform_server_handshake` (length-prefix) + `perform_server_handshake_ndjson` (JSON-line) on UDS accept when `FAMILY_ID` set; first-line auto-detect: riboCipher signal → tier routing, `"protocol":"btsp"` → NDJSON BTSP, plain `{` → JSON-RPC, other → binary BTSP; BIOMEOS_INSECURE guard; domain-based socket naming (`network.sock`); domain symlink `network.sock` → `songbird.sock` |
| BTSP Phase 3 (FULL) | `btsp.negotiate` server-side handler — ChaCha20-Poly1305 encrypted framing post-handshake; HKDF-SHA256 session key derivation (directional c2s/s2c keys); graceful NULL cipher fallback; `btsp.server.export_keys` delegation to security provider; length-prefixed encrypted frames `[4B len][12B nonce][ciphertext + Poly1305 tag]`; 16 MiB max frame; `BondingPolicy` cipher floor enforcement; negotiate dispatch wired on all 3 transport paths (NDJSON session, binary-framed BTSP, bin_interface) |
| Method Normalization | `normalize_json_rpc_method_name()` in `songbird-types`; handles ecosystem naming drift |
| Lint Inheritance | 30/30 crates inherit workspace lints; 2 with justified custom tables |
| cargo-deny | Fully passing (advisories ok, bans ok, licenses ok, sources ok); locally enforced (`cargo deny check`); CI runs fmt + clippy + test only |
| Dependencies | Pure Rust in default build; 24 dead deps removed (Wave 152: 8, Wave 150x: 16); `ring` eliminated from production path (Wave 150x: `rustls-rustcrypto` replaces `ring` in universal-ipc TLS); `rand` replaced with `fastrand` in non-crypto paths; `chrono` eliminated from 3 trivial crates; `kube`/`k8s-openapi`/`bollard` feature-gated; Bluetooth native C deps only with `bluetooth` feature; zero first-party `-sys` crates, zero `cc`, zero `build.rs`; `cargo deny check` fully passing |
| UniBin | Single binary: `server`, `cli` (REPL), `compute-bridge`, `deploy`, `rendezvous`, `relay` |
| Total Rust | ~422,000 lines across 31 crates |

## Architecture

```
Application Layer (biomeOS Neural API, AI Coordination, Compute)
    |
    | JSON-RPC 2.0 + tarpc
    v
Songbird Orchestrator
    |-- Tor Protocol (pure Rust: directory, circuit, stream, onion service)
    |-- Sovereign Onion (P2P encrypted service + connector)
    |-- IGD Router Config (UPnP IGD + NAT-PMP, auto port forwarding)
    |-- QUIC Transport (native Rust — RFC 9000/9001/9002, security provider crypto, 0-RTT, migration)
    |-- NFC Genesis (Dark Forest mobile pairing, zero metadata leakage)
    |-- BLE GATT (Bluetooth Low Energy genesis)
    |-- TLS 1.3 (RFC 8446, protocol detection)
    |-- STUN Server (RFC 5389, NAT discovery, port pattern probing)
    |-- Relay Server (lineage-based auth, packet forwarding, coordinated punch)
    |-- Dark Forest Discovery (encrypted beacons, mDNS, DNS-SD)
    |-- Universal IPC (Unix sockets, TCP, platform-agnostic)
    |
    | Security capability delegation (Ed25519, X25519, ChaCha20, SHA3-256, AES-128-CTR)
    v
Security Provider (capability discovery: security.sock / SECURITY_PROVIDER_SOCKET)
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
10. **Canonical Naming** - `normalize_json_rpc_method_name()` absorbs all ecosystem aliases to canonical `domain.verb`

## Quick Start

```bash
cargo build --workspace --release
cargo run --bin songbird -- server
cargo run --bin songbird -- server --bind 0.0.0.0 --port 9200  # LAN-exposed
cargo run --bin songbird -- doctor
cargo run --bin songbird -- config show
cargo run --bin songbird -- compute-bridge
cargo run --bin songbird -- deploy
```

### Network Flags

| Flag | Controls | Default |
|------|----------|---------|
| `--bind` | HTTP/HTTPS server bind address | `127.0.0.1` (localhost only) |
| `--port` | HTTP/HTTPS server port | `8080` (env: `SONGBIRD_HTTP_PORT`) |
| `--listen` | **IPC only** — TCP JSON-RPC inter-primal listener | derived from `--port` |
| `--socket` | **IPC only** — Unix socket path for inter-primal IPC | XDG auto |

`--bind` defaults to localhost for security. Use `--bind 0.0.0.0` to expose on all interfaces (LAN/WAN). Accepts `host` or `host:port` (port overrides `--port`).

### Environment Variables

```bash
export SECURITY_PROVIDER_SOCKET=/run/user/$(id -u)/biomeos/security.sock
export NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos/neural-api.sock
export SONGBIRD_HTTP_PORT=3492
export SONGBIRD_BIND_ADDRESS=0.0.0.0
export SONGBIRD_IGD_ENABLED=true
export SONGBIRD_FAMILY_ID=myfamily
export SONGBIRD_DISCOVERY_PORT=2300
export SONGBIRD_STUN_PORT=3478
export SONGBIRD_RELAY_PORT=3479
export SONGBIRD_ROUTE_DETECT_ADDR=192.0.2.1:80
```

See [`docs/ENVIRONMENT_VARIABLES.md`](docs/ENVIRONMENT_VARIABLES.md) for the complete reference including the security provider socket fallback chain.

## Crate Structure (31 crates)

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
- `songbird-quic` - Pure Rust QUIC transport (RFC 9000, security provider crypto delegation, 0-RTT, migration)
- `songbird-sovereign-onion` - P2P onion service
- `songbird-lineage-relay` - Lineage relay + coordinated punch
- `songbird-onion-relay` - Hole punch coordinator
- `songbird-turn-client` - Reusable TURN relay client (data-plane sessions for downstream consumers)

### Hardware
- `songbird-nfc` - NFC genesis protocol
- `songbird-bluetooth` - BLE GATT service
- `songbird-genesis` - Physical genesis bootstrap

### Crypto
- `songbird-crypto-provider` - Shared crypto provider (Neural API + direct security provider routing)

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
cargo test --workspace --all-features          # Full suite (~70s)
cargo test -p songbird-tor-protocol --lib      # Single crate
./scripts/test-with-security-provider.sh        # With live security provider from plasmidBin
./scripts/coverage.sh                          # llvm-cov HTML report
```

## Documentation

| Document | Purpose |
|----------|---------|
| [`REMAINING_WORK.md`](REMAINING_WORK.md) | Current status, metrics, and pending work |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Contribution guidelines |
| [`CONTEXT.md`](CONTEXT.md) | AI-ingestible project context |
| [`docs/ENVIRONMENT_VARIABLES.md`](docs/ENVIRONMENT_VARIABLES.md) | Complete environment variable reference and socket fallback chains |
| [`specs/`](specs/) | Technical specifications |
| [`docs/architecture/`](docs/architecture/) | Security provider crypto API spec |

## License

AGPL-3.0-or-later (scyBorg provenance trio: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

See `LICENSE`, `LICENSE-ORC`, and `LICENSE-CC-BY-SA` at repository root.
