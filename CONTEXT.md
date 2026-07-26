# Songbird — AI-Ingestible Context

**Primal**: Songbird  
**Role**: Network Orchestration & Discovery Primal — Inner Membrane Port Solver  
**Phase**: Stadial (S1-S4 graduated, 6-gate mesh collective LIVE, zero-config mesh init)  
**Version**: 0.2.1-wave151b  
**License**: AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA)  
**Last Updated**: July 26, 2026

## What It Does

Songbird is the network spine of the ecoPrimals ecosystem. It provides:

- **HTTP/HTTPS**: Pure Rust TLS 1.3 client for sovereign HTTPS (Tower Atomic with security provider)
- **IPC**: JSON-RPC 2.0 + tarpc dual-protocol inter-primal communication; BTSP Phase 2 handshake on UDS accept when `FAMILY_ID` set (length-prefix + NDJSON wire formats; first-line auto-detect); BTSP Phase 3 `btsp.negotiate` encrypted framing (ChaCha20-Poly1305, HKDF session keys) on all 3 transport paths; domain symlink `network.sock` → `songbird.sock` for capability discovery
- **Discovery**: Capability-based runtime discovery — mDNS, SSDP, UDP broadcast, DNS-SD, BirdSong encrypted beacons
- **NAT Traversal**: STUN, IGD/UPnP, NAT-PMP, UDP hole punching, relay mesh, TURN client (RFC 5766), cloudflared emergency tunnels, shadow dual-path comparator
- **Cross-Gate Dispatch**: `capability.call` routes RPCs to local or remote capabilities via mesh TCP, TURN relay fallback, and UDS
- **Federation**: Multi-node mesh networking with capability-based routing
- **Onion Routing**: Pure Rust Tor protocol + sovereign .onion services
- **Relay**: Lineage-gated relay for symmetric NAT traversal

## Ecosystem Role

Songbird is one third of **Tower Atomic** (bearDog + songBird + skunkBat), the sovereign
transport stack targeting WireGuard replacement. Every other primal and spring uses Songbird for:

- Sovereign HTTPS without C dependencies (via security provider crypto delegation)
- Service discovery and capability-based primal location (zero identity hardcoding)
- Cross-NAT connectivity for multi-machine deployments
- Dark Forest encrypted discovery (zero metadata leakage)

## Technical Facts

| Metric | Value |
|--------|-------|
| Crates | 31 workspace members |
| Tests | 14,332+ total (667 in universal-ipc; integration suites) |
| Coverage | Line coverage **73.41%** (llvm-cov `--workspace --lib`, Apr 27 2026; target 90%) |
| Edition | Rust 2024 |
| Clippy | pedantic + nursery, zero warnings (`-D warnings`; Jul 21 verified); Windows cross-compile zero warnings |
| Files >800 LOC | **0** — all production code below threshold (largest: mesh_handler/mod.rs at 795L, drawbridge.rs at 749L) |
| Unsafe blocks | 0 (`forbid(unsafe_code)` all 31 crates) |
| C dependencies | Zero in default/production build; `ring` eliminated from production path (Wave 150x: `rustls-rustcrypto`); 24 dead deps removed; `chrono` eliminated from 3 crates; Bluetooth native deps only with `bluetooth` feature |
| Hardcoded primal names | 0 in production discovery (capability-first); all paths env-driven or XDG-compliant; zero `/tmp/` writes |
| Resolver / DNS probes | No fixed `8.8.8.8` in production paths — netdev-based discovery |
| Production panics/unwrap/todo | 0 unguarded; `.unwrap()` only under `#[expect(reason)]` for infallible ops |

## IPC Surface

JSON-RPC 2.0 methods via typed `JsonRpcMethod` enum dispatch (ecosystem standard `domain.verb` naming):

- `health.liveness`, `health.readiness`, `health.check`
- `capabilities.list` (Wire Standard L3 envelope: `{primal, version, methods, provided_capabilities, consumed_capabilities, protocol, transport}`), `capabilities.methods`
- `identity.get` (Wire Standard L3: `{primal, version, domain, license}`), `identity`
- `capability.resolve` (single-step capability→endpoint, wired Wave 137), `discovery.peers` (wired Wave 137), `ipc.resolve` (capability-first with primal-name fallback; `capability`/`primal_id`/`name` params; `ipc.resolve_by_name` alias; evolved Wave 137b LD-02, Wave 151 PG-37), `lifecycle.composition`, `lifecycle.validate_consumed`
- `inference.infer`, `inference.status`, `inference.list`, `inference.load` (canonical namespace; absorbs `model.*`/`ai.*`)
- `ipc.register` (Ed25519-signed payloads via `BearDog` `crypto.sign.ed25519` when `FAMILY_ID` set; graceful degradation), `ipc.resolve`, `ipc.discover`, `ipc.list`, `ipc.find_capability`, `ipc.heartbeat`
- `http.request`, `http.get`, `http.post`, `http.put`, `http.delete`
- `stun.*`, `igd.*`, `relay.*`, `mesh.*`, `punch.*`
- `birdsong.*`, `beacon.encrypt`, `beacon.decrypt`, `beacon.get_id` (mito-beacon tier)
- `onion.*`, `tor.*`
- `discovery.announce` (presence + topic-based content federation), `discovery.content_peers` (seeder/leecher query with topic/manifest/family filters)
- `compute.route`, `registry.*`, `consent.*`, `task.*`
- `graph.validate`, `graph.check_availability`, `graph.suggest_alternatives`
- `coordination.validate_pattern`
- `songbird.federation.*`, `songbird.compute.*`, `songbird.services.*`, `songbird.health`, `songbird.version`
- `network.beacon_exchange`, `network.broadcast`, `network.listen`
- `deployment.create`, `deployment.status`, `protocol.negotiate`

## Capabilities (17 tokens)

`network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`network.btsp`, `ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`,
`bluetooth.pair`, `compute.gpu`, `access.remote`

## Dependencies on Other Primals

- **Security Provider**: Crypto delegation via capability discovery (no compile-time import, no identity hardcoding)
- **biomeOS**: Outbound `primal.announce` on startup (Neural API routing weight seeding); tiered socket discovery (`$NEURAL_API_SOCKET` → XDG → `/tmp`)
- No other primal code imports — all coordination via JSON-RPC IPC

## Part of ecoPrimals

- Repository: `ecoPrimals/primals/songbird`
- Standards: [wateringHole](../../infra/wateringHole/)
- Registry: [plasmidBin](../../infra/plasmidBin/manifest.toml)
