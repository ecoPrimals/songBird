# Songbird — AI-Ingestible Context

**Primal**: Songbird
**Role**: Network Orchestration & Discovery Primal
**Phase**: 1 (Foundation)
**Version**: 0.2.1
**License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA)
**Last Updated**: April 6, 2026

## What It Does

Songbird is the network spine of the ecoPrimals ecosystem. It provides:

- **HTTP/HTTPS**: Pure Rust TLS 1.3 client for sovereign HTTPS (Tower Atomic with security provider)
- **IPC**: JSON-RPC 2.0 + tarpc dual-protocol inter-primal communication
- **Discovery**: Capability-based runtime discovery — mDNS, SSDP, UDP broadcast, DNS-SD, BirdSong encrypted beacons
- **NAT Traversal**: STUN, IGD/UPnP, NAT-PMP, UDP hole punching, relay mesh
- **Federation**: Multi-node mesh networking with capability-based routing
- **Onion Routing**: Pure Rust Tor protocol + sovereign .onion services
- **Relay**: Lineage-gated relay for symmetric NAT traversal

## Ecosystem Role

Songbird is half of **Tower Atomic** (security provider + Songbird), the minimum viable
ecosystem composition. Every other primal and spring uses Songbird for:

- Sovereign HTTPS without C dependencies (via security provider crypto delegation)
- Service discovery and capability-based primal location (zero identity hardcoding)
- Cross-NAT connectivity for multi-machine deployments
- Dark Forest encrypted discovery (zero metadata leakage)

## Technical Facts

| Metric | Value |
|--------|-------|
| Crates | 30 workspace members |
| Tests | 12,764 passed (0 failed, 252 ignored env-dependent, full suite ~70s) |
| Coverage | Line coverage **72.16%** (llvm-cov `--workspace --lib`, Apr 6 2026; target 90%) |
| Edition | Rust 2024 |
| Clippy | pedantic + nursery, zero warnings (`-D warnings`) |
| Files >800 LOC | 0 (largest file 519 lines; `tower_atomic.rs` refactored 990→4 files) |
| Unsafe blocks | 0 (`songbird-process-env` uses in-memory overlay; `forbid(unsafe_code)` all 30 crates) |
| C dependencies | Zero in `songbird-quic`; `rcgen` eliminated (pure Rust test cert gen); `sled` feature-gated with in-memory fallback; `ring-crypto` opt-in feature gate on CLI only; Bluetooth stack (`libudev`, etc.) only when `bluetooth` feature is enabled |
| Hardcoded primal names | 0 in production discovery (capability-first across 11+ crates; `SecurityProvider*` APIs with deprecated `BearDog*` aliases; `--security-socket` CLI flag; `security_provider_port()` replaces `beardog_port()`) |
| Resolver / DNS probes | No fixed `8.8.8.8` in production paths — netdev-based discovery (Apr 6) |
| Production panics/unwrap/todo | 0 |

## IPC Surface

JSON-RPC 2.0 methods via typed `JsonRpcMethod` enum dispatch (ecosystem standard `domain.verb` naming):

- `health.liveness`, `health.readiness`, `health.check`
- `capabilities.list`
- `ipc.register`, `ipc.resolve`, `ipc.discover`, `ipc.list`, `ipc.find_capability`, `ipc.heartbeat`
- `http.request`, `http.get`, `http.post`
- `stun.*`, `igd.*`, `relay.*`, `mesh.*`, `punch.*`
- `birdsong.*`, `onion.*`, `tor.*`
- `compute.route`, `registry.*`, `consent.*`, `task.*`
- `songbird.federation.*`, `songbird.compute.*`, `songbird.services.*`, `songbird.health`, `songbird.version`
- `network.beacon_exchange`, `network.broadcast`, `network.listen`
- `deployment.create`, `deployment.status`, `protocol.negotiate`

## Capabilities (14 tokens)

`network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`, `bluetooth.pair`

## Dependencies on Other Primals

- **Security Provider**: Crypto delegation via capability discovery (no compile-time import, no identity hardcoding)
- **biomeOS**: Registers via Neural API `lifecycle.register` when available
- No other primal code imports — all coordination via JSON-RPC IPC

## Part of ecoPrimals

- Repository: `ecoPrimals/primals/songbird`
- Standards: [wateringHole](../../infra/wateringHole/)
- Registry: [plasmidBin](../../infra/plasmidBin/manifest.toml)
