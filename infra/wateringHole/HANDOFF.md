# songBird — Upstream Handoff

**Primal**: songBird  
**Version**: v0.2.1-wave145b  
**Date**: July 16, 2026  
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

## Recent Evolution (Wave 131–145)

| Wave | Summary |
|------|---------|
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

## AAR — Wave 145b (July 16, 2026)

### Accomplished

Phase 2 Transport abstraction is **COMPLETE** for songBird. Zero inline `#[cfg]`-gated
IPC connection blocks remain outside the platform abstraction layer. 15 crates migrated
to `IpcStream` total. `pin-project` dependency eliminated. Net -248 lines of platform
boilerplate across Waves 142b–145b.

### Remaining P2: Composition Wiring — Ownership Clarification

songBird **owns** the drawbridge behaviors that footPrint and tideGlass interact with.
We are not blocked on those teams — the wiring is ours to ship:

| Item | What songBird Owns | What External Team Provides |
|------|-------------------|----------------------------|
| **footPrint `PROXY_PATH`** | Drawbridge route registration, auth-gate config, external proxy allowlist entries for GIS services | footPrint client consuming `/ext` endpoint (already migrated Wave 137c) |
| **tideGlass bonds** | Drawbridge bond definitions (LINCS L1000, GEO, ChEMBL, NF Data Portal base URLs + allowlist entries) | tideGlass consuming the bonds via drawbridge HTTP proxy |

**Status**: Both items are **already functionally complete** (Wave 137c + 140a):
- `PROXY_PATH` → footPrint Express→drawbridge migration shipped, E2E verified with live geocoding
- tideGlass bonds → 4 pharmacogenomics APIs registered in `SCIENCE_BONDS` + external allowlist

### Guidance Needed from Upstream

| Need | Owner | Question |
|------|-------|----------|
| footPrint server composition topology | sporeGate ops | When `PROXY_PATH` env is wired in deployed footPrint service on golgi, do we need Caddy blocks? Or direct drawbridge bind? |
| tideGlass Gonzales Explorer ingestion spec | tideGlass team | NF Data Portal bond is registered — does Gonzales need additional query patterns beyond `GET /portal/api/v1/`? |
| Live E2E validation scenario | primalSpring | `footprint-drawbridge-live` scenario listed as P2 TODO — what's the trigger? |

### No Blockers

songBird has zero P0/P1 items. All milestones clear. Ready for upstream to wire
composition when infrastructure (golgi sporePrint rebuild, sporeGate deploy) unblocks.

## Fossil Record

Docs preserved as fossil record in `ecoPrimals/`. Archived specs in `specs/archived/`.
Wave history in `REMAINING_WORK.md` and `CHANGELOG.md`.
