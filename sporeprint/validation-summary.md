+++
title = "Songbird Validation Summary"
description = "Sovereign networking primal — 14,835+ tests, 31 crates, 70+ JSON-RPC methods, inner membrane port solver (drawbridge :7780), Tower Atomic transport layer (bearDog + songBird + skunkBat), tower.health + tower.mesh_status biomeOS signal graph facade, mesh.enroll BTSP-verified enrollment, 5-tier NAT traversal, cross-gate capability.call, BTSP Phase 3 encrypted framing, BTSP ClientHello consumer handshake, G1 Windows named pipe IPC, J3 deployment.hot_swap cascade automation, J4 route.* self-configuring proxy routes, J5 WG peer reg hardened (input validation + injection-safe), zero production unwraps, zero unsafe code, all deps pure Rust, crypto delegation 6/6"
date = 2026-07-28

[taxonomies]
primals = ["songbird"]
springs = ["primalspring"]
+++

## Status

- **14,835+ total tests** (677 in universal-ipc; integration suites)
- **31 workspace crates**, ~422,000 lines of Rust (Edition 2024)
- **70+ JSON-RPC methods** across 19 capability domains (network, mesh, stun, relay, http, route, tower, tor, onion, punch, discovery, birdsong, capability, ipc, primal, lifecycle, federation, graph, coordination, deployment)
- **BTSP Phase 3 FULL** — ChaCha20-Poly1305 AEAD, HKDF-SHA256 session keys, bond type enforcement
- **5-tier ConnectionFallbackChain**: direct → STUN → lineage relay → TURN → emergency tunnel
- **`capability.call` cross-gate dispatch** (Wave 211+38): local UDS + remote mesh TCP + TURN relay fallback for NAT'd peers
- **`mesh.init` bootstrap peers** (Wave 49): TCP peer entries added as Direct endpoints, enabling cross-gate discovery without onion bootstrap
- **`SONGBIRD_PEERS` auto-seeding** (Wave 51): Zero-config mesh — peers parsed from env on startup, `discovery.peers` immediately populated
- **`discovery.peers` mesh bridge** (Wave 51): Merges mesh + registry peers, de-duplicates by node_id, cross-gate health probes work
- **`--security-socket` CLI flag** (Wave 49): feeds env var overlay for plasmidBin binary deployments
- **Outbound `primal.announce`** on startup (Wave 45): biomeOS Neural API routing weight seeding with aligned capability domains
- **Zero production stubs** in networking paths (Wave 214: DirectConnection, protocol upgrade, HTTPS cert all evolved)
- **Zero unsafe blocks** (`forbid(unsafe_code)` on all 31 crates)
- **Zero `async-trait`** — fully eliminated (141→0), enum dispatch throughout
- **Zero production `unwrap()`/`panic!()`/`todo!()`**
- **Clippy pedantic** clean (31/31 crates, `-D warnings --all-targets`; Wave 121 zero errors)
- **cargo-deny** passing (advisories, bans, licenses, sources)
- **Zero `"literal".to_string()`** in production code (2,020→0, Wave 121 idiom evolution)

## Key Capabilities

| Domain | Methods | Transport |
|--------|---------|-----------|
| Network discovery | `ipc.register`, `ipc.resolve`, `ipc.discover`, `capability.resolve`, `capability.call` | UDS, TCP |
| NAT traversal | `stun.*` (7 methods), `punch.*`, `relay.*` (4), TURN | UDP, TURN relay |
| Mesh networking | `mesh.*` (8 methods incl. `probe_latency`) | BirdSong UDP, relay |
| HTTP gateway | `http.request/get/post` | HTTPS (rustls) |
| Tor/Onion | `tor.*`, `onion.*` | Tor circuits |
| Federation | `federation.*` | Cross-cluster |
| Security | BTSP P3, Ed25519 ionic tokens via BearDog delegation | AEAD frames |

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│ songbird (sovereign networking primal)                           │
├─────────────────────────────────────────────────────────────────┤
│ Universal IPC (songbird.sock / network.sock)                    │
│   ├── JSON-RPC dispatch (62 methods, typed enum routing)        │
│   ├── capability.call (local UDS + remote mesh TCP forward)     │
│   ├── IPC registry (capability-first service discovery)         │
│   └── BTSP Phase 3 encrypted framing (ChaCha20-Poly1305)       │
│                                                                  │
│ NAT Traversal (H2-13 through H2-16)                             │
│   ├── STUN client (RFC 5389, MESSAGE-INTEGRITY, FINGERPRINT)   │
│   ├── TURN client + server (RFC 5766, sovereign VPS relay)      │
│   ├── Lineage relay (family-authorized UDP forwarding)          │
│   ├── UDP hole punching (simultaneous open)                     │
│   └── Cloudflare DDNS (A/AAAA upsert via HttpExecutor)         │
│                                                                  │
│ Mesh (BeaconMesh)                                                │
│   ├── BirdSong encrypted discovery (family-gated beacons)       │
│   ├── Topology + path finding                                    │
│   └── Relay announcement + health checks                        │
│                                                                  │
│ HTTP Orchestrator (:3492 TCP)                                    │
│   ├── JSON-RPC API + compute forwarding                          │
│   ├── HTTP gateway proxy (UniversalProxy)                        │
│   └── capability.register at startup (Neural API integration)   │
└─────────────────────────────────────────────────────────────────┘
```

## Deployment

- **VPS relay**: `songbird relay` CLI subcommand, systemd unit (`deployment/systemd/songbird-relay.service`)
- **Local**: UDS at `$XDG_RUNTIME_DIR/biomeos/songbird.sock` (domain symlink: `network.sock`)
- **TCP**: `:3492` HTTP API for inter-gate communication
- **Shadow run ready**: `TurnRelayStats` periodic emission (60s), `SONGBIRD_TURN_*` env config, `probe_turn_relay()` measurement API

## primalSpring Composition Status

- **CLEAN** in 13/13 primal composition audit
- **Wire Standard**: Level 3 (full ecosystem interop)
- **JH-0 ADOPTED**, BTSP P3 FULL, Dark Forest compliant (DF-3 CallerContext wired)
- **Stadial gate**: cleared (no sentinel blockers)
- **CG-8**: Cross-gate dispatch resolved (Wave 211)

## See Also

- [Songbird README](../README.md) — quick-start and architecture overview
- [REMAINING_WORK.md](../REMAINING_WORK.md) — tracked debt and evolution waves
- [deployment/relay/README.md](../deployment/relay/README.md) — VPS TURN relay deployment guide
- [specs/](../specs/) — STUN, mesh, and protocol specifications
