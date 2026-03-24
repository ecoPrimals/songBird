# Songbird — AI-Ingestible Context

**Primal**: Songbird
**Role**: Network Orchestration & Discovery Primal
**Phase**: 1 (Foundation)
**Version**: 0.2.1
**License**: AGPL-3.0-only (scyBorg triple: AGPL + ORC + CC-BY-SA)

## What It Does

Songbird is the network spine of the ecoPrimals ecosystem. It provides:

- **HTTP/HTTPS**: Pure Rust TLS 1.3 client for sovereign HTTPS (Tower Atomic with BearDog)
- **IPC**: JSON-RPC 2.0 + tarpc dual-protocol inter-primal communication
- **Discovery**: mDNS, SSDP, UDP broadcast, DNS-SD, BirdSong encrypted beacons
- **NAT Traversal**: STUN, IGD/UPnP, NAT-PMP, UDP hole punching, relay mesh
- **Federation**: Multi-node mesh networking with capability-based routing
- **Onion Routing**: Pure Rust Tor protocol + sovereign .onion services
- **Relay**: Lineage-gated relay for symmetric NAT traversal

## Ecosystem Role

Songbird is half of **Tower Atomic** (BearDog + Songbird), the minimum viable
ecosystem composition. Every other primal and spring uses Songbird for:

- Sovereign HTTPS without C dependencies (via BearDog crypto delegation)
- Service discovery and capability-based primal location
- Cross-NAT connectivity for multi-machine deployments
- Dark Forest encrypted discovery (zero metadata leakage)

## Technical Facts

| Metric | Value |
|--------|-------|
| Crates | 30 workspace members |
| Tests | 10,235 (0 failed) |
| Coverage | ~66.59% (target 90%) |
| Edition | Rust 2024 |
| Clippy | pedantic + nursery, zero warnings |
| Files >1000 LOC | 0 (max 959) |
| Unsafe blocks | 2 (justified, in `songbird-process-env`) |
| C dependencies | `ring` opt-in only (`ring-crypto` feature) |

## IPC Surface

JSON-RPC 2.0 methods (ecosystem standard `domain.verb` naming):

- `health.liveness`, `health.readiness`, `health.check`
- `capabilities.list`
- `ipc.register`, `ipc.resolve`, `ipc.discover`, `ipc.list`
- `http.request`, `http.get`, `http.post`
- `stun.*`, `igd.*`, `relay.*`, `mesh.*`, `punch.*`
- `birdsong.*`, `onion.*`, `tor.*`
- `compute.route`, `registry.*`, `consent.*`, `task.*`

## Capabilities (14 tokens)

`network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`, `bluetooth.pair`

## Dependencies on Other Primals

- **BearDog**: Crypto delegation via capability discovery (no compile-time import)
- **biomeOS**: Registers via Neural API `lifecycle.register` when available
- No other primal code imports — all coordination via JSON-RPC IPC

## Part of ecoPrimals

- Repository: `ecoPrimals/phase1/songbird`
- Standards: [wateringHole](../../wateringHole/)
- Registry: [plasmidBin](../../plasmidBin/manifest.toml)
