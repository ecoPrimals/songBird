# Songbird — AI-Ingestible Context

**Primal**: Songbird
**Role**: Network Orchestration & Discovery Primal
**Phase**: 1 (Foundation)
**Version**: 0.2.1
**License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA)
**Last Updated**: March 29, 2026

## What It Does

Songbird is the network spine of the ecoPrimals ecosystem. It provides:

- **HTTP/HTTPS**: Pure Rust TLS 1.3 client for sovereign HTTPS (Tower Atomic with BearDog)
- **IPC**: JSON-RPC 2.0 + tarpc dual-protocol inter-primal communication
- **Discovery**: Capability-based runtime discovery — mDNS, SSDP, UDP broadcast, DNS-SD, BirdSong encrypted beacons
- **NAT Traversal**: STUN, IGD/UPnP, NAT-PMP, UDP hole punching, relay mesh
- **Federation**: Multi-node mesh networking with capability-based routing
- **Onion Routing**: Pure Rust Tor protocol + sovereign .onion services
- **Relay**: Lineage-gated relay for symmetric NAT traversal

## Ecosystem Role

Songbird is half of **Tower Atomic** (BearDog + Songbird), the minimum viable
ecosystem composition. Every other primal and spring uses Songbird for:

- Sovereign HTTPS without C dependencies (via BearDog crypto delegation)
- Service discovery and capability-based primal location (zero identity hardcoding)
- Cross-NAT connectivity for multi-machine deployments
- Dark Forest encrypted discovery (zero metadata leakage)

## Technical Facts

| Metric | Value |
|--------|-------|
| Crates | 30 workspace members |
| Tests | 11,471 (0 failed, 269 ignored) |
| Coverage | ~69.33% line coverage via llvm-cov (target 90%) |
| Edition | Rust 2024 |
| Clippy | pedantic + nursery, zero warnings (`-D warnings`) |
| Files >1000 LOC | 0 (max prod ~484 `gateway/mod.rs`; `gaming.rs` → `gaming/` 8 modules, `canonical_types.rs` → `canonical_types/` 11 modules) |
| Unsafe blocks | 0 (`songbird-process-env` evolved to BearDog in-memory overlay; `forbid(unsafe_code)` all 30 crates) |
| C dependencies | `ring` transitive via `quinn-proto`/`rcgen`; `ring-crypto` feature opt-in for `rustls/ring` |
| Hardcoded primal names | 0 in production discovery paths (capability-first: `crypto.sock`, not `beardog.sock`) |
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

- **BearDog**: Crypto delegation via capability discovery (no compile-time import, no identity hardcoding)
- **biomeOS**: Registers via Neural API `lifecycle.register` when available
- No other primal code imports — all coordination via JSON-RPC IPC

## Part of ecoPrimals

- Repository: `ecoPrimals/primals/songbird`
- Standards: [wateringHole](../../infra/wateringHole/)
- Registry: [plasmidBin](../../infra/plasmidBin/manifest.toml)
