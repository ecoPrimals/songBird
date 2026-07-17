# songBird — Upstream Handoff

**Primal**: songBird  
**Version**: v0.2.1-wave147b  
**Date**: July 17, 2026  
**Gate**: flockGate (eastGate)

## Current State

| Metric | Status |
|--------|--------|
| Clippy | Zero warnings (pedantic + nursery, `-D warnings`) |
| Tests | Full workspace pass, 0 failures |
| Unsafe | 0 (`forbid(unsafe_code)` all 31 crates) |
| Production stubs | 0 (Wave 137: last fake-data stub evolved to real probes) |
| Files >800L prod | 0 |
| Hardcoding | 0 in production (all env-driven, capability-based) |
| Mocks in prod | 0 (all `#[cfg(test)]` gated) |

## Recent Evolution (Wave 131–147)

| Wave | Summary |
|------|---------|
| 147b | `mesh.enroll` JSON-RPC method wired — BTSP gate enrollment endpoint ready for cellMembrane `gate.enroll` integration |
| 147a | Final 2 inline cfg-gated IPC connections migrated to `IpcStream` (`neural_announce`, `relay_security`) |
| 145b | Last 3 inline `#[cfg]`-gated IPC connections eliminated; zero platform-specific connect blocks remain outside platform layer |
| 143b | `IpcStream` migration batch 2: 9 additional crates migrated; `CryptoStream` + `pin-project` eliminated from `songbird-tls`; 30+ `connect_platform` pairs consolidated; `&PathBuf`→`&Path` evolution; 12 total crates on `IpcStream` |
| 142b | Phase 2 "abstraction over gating": `IpcStream` shipped in `songbird-types` (platform-abstracted async IPC); 3 crates migrated (crypto-provider, lineage-relay, federation); `drawbridge.rs` refactored (813→498L); platform cfg audit: 35 trait-backend candidates |
| 141a | Cross-platform `#[cfg]` gate evolution: **0 warnings** on Windows cross-compile (was 30+); reference implementation polish |
| 140a | tideGlass drawbridge bonds (LINCS L1000, GEO, ChEMBL, NF Data Portal); HTTPS proxy `OnceLock` optimization; 20 total bonds |
| 139b | NCBI/PubChem science bonds (6 APIs); `compute.gpu` + `access.remote` capabilities for northGate |
| 139a | Windows cross-compile (`x86_64-pc-windows-gnu`): 15 files, 7 crates — IPC abstraction, platform gating |
| 137e | Deep debt: `to_lowercase()` elimination (19 files total), zero-alloc domain matching |
| 137d | DRAWBRIDGE-CAP: Runtime capabilities in `capabilities.list`; `capability.call` drawbridge fallback |
| 137c | FP-API COMPLETE: HTTPS outbound proxy (tokio-rustls + CA certs), E2E verified |
| 137b | UDS-HTTP-PROTOCOL fix: peer.connect → BeaconMesh registration (CRITICAL) |
| 137a | FLOCKGATE-MESH fix (WG auto-discovery port 8080→7700); FP-API `?url=` compat |
| 137 | CLI fake metrics → real TCP RTT probes; dead code removal (-672L); dep diet |
| 136b | Drawbridge external proxy allowlist (footPrint composition) |
| 136a | Drawbridge auth-gate: bearer tokens, CIDR trusted peers, public paths |
| 131b | LAN peering bypass, health honesty, security fail-closed, dep diet |
| 131 | Allocation elimination, typed dispatch, hardcoding removal |

## Upstream Dependencies (consumed capabilities)

| Capability | Provider | Status |
|------------|----------|--------|
| `security.*` (Ed25519, X25519, ChaCha20) | bearDog | LIVE via IPC |
| `primal.announce` | biomeOS Neural API | LIVE (optional) |

## Provided Capabilities (17 native + runtime drawbridge)

**Native** (always): `network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`network.btsp`, `ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`, `bluetooth.pair`,
`compute.gpu`, `access.remote`

**Runtime** (from `SONGBIRD_PROXY_ROUTES` + `SONGBIRD_DRAWBRIDGE_ROUTES`): dynamically merged
into `capabilities.list` response. Example: `jupyter`, `inference`, etc.

## Blocking Items for Other Teams

| Item | Blocked Team | Status |
|------|-------------|--------|
| TOPO-VIS mesh heartbeat data | petalTongue + nestGate | Ready (mesh.status, mesh.peers expose data) |
| EXP-06-CADDY auth wiring | sporeGate | Auth-gate live, Caddy config pending sporeGate |
| Drawbridge external proxy | footPrint composition | **COMPLETE** (Wave 137c) |
| tideGlass drawbridge bonds | tideGlass team | **COMPLETE** (Wave 140a) — LINCS, GEO, ChEMBL, NF Data Portal |
| NF Data Portal ingestion | Gonzales explorer | **READY** — `nf` bond registered, HTTPS proxy supports Synapse API |
| Windows cross-compile ref impl | 11 Windows-blocked primals | **REFERENCE** (Wave 139a) — pattern for Phase 2 transport |

## Known Gaps (not blocking)

- Coverage 73.41% → 90% target (I/O-heavy modules need mock infra)
- BTSP Phase 3: multi-frame stress tests pending
- Tor onion crypto: blocked on live security provider Ed25519/X25519 surface
- CLI interactive prompts: `songbird init` still prints placeholder message

## AAR — Wave 147b (July 17, 2026)

### Accomplished (Waves 142b–147b)

| Deliverable | Impact |
|-------------|--------|
| `IpcStream` abstraction shipped | 15 crates migrated, `pin-project` dep eliminated, -284 lines platform boilerplate |
| Zero cfg-gated IPC connections | All local IPC uses `songbird_types::IpcStream::connect()` — full Phase 2 transport |
| `drawbridge.rs` refactor | 813→498L production via `drawbridge_auth.rs` extraction |
| `mesh.enroll` method | JSON-RPC endpoint ready for BTSP gate enrollment |
| Deep debt exhausted | Zero unsafe, zero `todo!()`, zero production mocks, zero files >800L, zero hardcoding |

### What songBird Owns (functionally complete)

| Item | Status | Delivered |
|------|--------|-----------|
| footPrint `PROXY_PATH` drawbridge wiring | **COMPLETE** | Wave 137c — E2E verified (live geocoding) |
| tideGlass drawbridge bonds | **COMPLETE** | Wave 140a — LINCS, GEO, ChEMBL, NF Data Portal |
| `mesh.enroll` server-side endpoint | **READY** | Wave 147b — awaiting BTSP proof protocol |

### What We Need from Other Teams

| Need | Owner | Detail |
|------|-------|--------|
| **BTSP enrollment proof spec** | cellMembrane + bearDog | `mesh.enroll` accepts `{node_id, public_key}` — what format is the `proof` field? Ed25519 signature over what payload? cellMembrane's `gate.enroll` client would call this. |
| **footPrint deploy topology** | sporeGate ops | Drawbridge is live locally. When footPrint is deployed on golgi, does it bind to drawbridge directly or go through Caddy? Need `PROXY_PATH` env wiring in the systemd unit. |
| **tideGlass query patterns** | tideGlass / Gonzales | NF Data Portal bond is registered (`nf` service → `https://nf.tower.nf/`). Does Gonzales need patterns beyond `GET /portal/api/v1/`? Any POST/pagination? |
| **E2E validation trigger** | primalSpring | `footprint-drawbridge-live` scenario is P2 TODO — what's the gate? Manual trigger, cron, or on-push? |

### No Blockers

songBird has **zero P0/P1 items**. All ecosystem milestones clear (Phase 2 14/14,
CAC 6/6, Glacial 8/8). Deep debt targets exhausted. Standing by for composition
wiring when infrastructure unblocks.

## Fossil Record

Docs preserved as fossil record in `ecoPrimals/`. Archived specs in `specs/archived/`.
Wave history in `REMAINING_WORK.md` and `CHANGELOG.md`.
