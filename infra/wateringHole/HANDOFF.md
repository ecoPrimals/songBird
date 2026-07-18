# songBird — Upstream Handoff

**Primal**: songBird  
**Version**: v0.2.1-wave149b  
**Date**: July 18, 2026  
**Gate**: flockGate (eastGate)

## Current State

| Metric | Status |
|--------|--------|
| Clippy | Zero warnings (pedantic + nursery, `-D warnings`); `doc_markdown` + `uninlined_format_args` now enforced |
| Tests | 14,322+ pass, 1 known flaky (mesh persistence test-parallelism) |
| Unsafe | 0 (`forbid(unsafe_code)` all 31 crates) |
| Production unwraps | 0 (config.rs refactored to `fmt::Result`, RwLock sites `#[expect]`-annotated) |
| Production stubs | 0 (Wave 137: last fake-data stub evolved to real probes) |
| Files >800L | 0 (including tests — drawbridge 578L, mesh_seed 523L after test extraction) |
| Hardcoding | 0 in production (all env-driven, capability-based) |
| Mocks in prod | 0 (all `#[cfg(test)]` gated) |

## Recent Evolution (Wave 147f–149b)

| Wave | Summary |
|------|---------|
| 149b | **Code quality sweep**: `cargo fmt` (10 files), 71 `writeln!` unwraps → `fmt::Result` refactor, file splits (drawbridge 1,019→578L, mesh_seed 834→523L), clippy fixes (`uninlined_format_args` ×58, `doc_markdown` ×40, 24 `const fn` promotions), lint enforcement promoted |
| 147f | **P0 PROXY_PATH**: drawbridge routing verified for footPrint (`/footprint/*` → `:8090`). **GAP-037**: `/jsonrpc` endpoint on drawbridge (HTTP→IPC bridge for esotericWebb). **Discovery schemas**: `discovery.topology/health/query/bonds` implemented |
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
| TOPO-VIS mesh heartbeat data | petalTongue + nestGate | Ready (mesh.status, mesh.peers, discovery.topology) |
| EXP-06-CADDY auth wiring | sporeGate | Auth-gate live, Caddy config pending sporeGate |
| Drawbridge external proxy | footPrint composition | **COMPLETE** (Wave 137c) |
| **PROXY_PATH drawbridge route** | footPrint | **COMPLETE** (Wave 147f) — `/footprint=footprint` route config verified |
| **JSON-RPC endpoint for esotericWebb** | esotericWebb | **COMPLETE** (Wave 147f) — `POST /jsonrpc` on drawbridge port |
| **Discovery schemas** | esotericWebb | **COMPLETE** (Wave 147f) — topology/health/query/bonds |
| tideGlass drawbridge bonds | tideGlass team | **COMPLETE** (Wave 140a) — LINCS, GEO, ChEMBL, NF Data Portal |
| NF Data Portal ingestion | Gonzales explorer | **READY** — `nf` bond registered, HTTPS proxy supports Synapse API |
| Windows cross-compile ref impl | 11 Windows-blocked primals | **REFERENCE** (Wave 139a) — pattern for Phase 2 transport |

## Known Gaps (not blocking)

- Coverage 73.41% → 90% target (I/O-heavy modules need mock infra)
- BTSP Phase 3: multi-frame stress tests pending
- Tor onion crypto: blocked on live security provider Ed25519/X25519 surface
- CLI interactive prompts: `songbird init` still prints placeholder message

## AAR — Wave 147f (July 17, 2026)

### Accomplished (Waves 142b–147f)

| Deliverable | Impact |
|-------------|--------|
| **P0 PROXY_PATH route** (147f) | footPrint composition unblocked: `/footprint/*` → `:8090` via drawbridge env config |
| **GAP-037 /jsonrpc endpoint** (147f) | esotericWebb unblocked: `POST /jsonrpc` on drawbridge port bridges HTTP→IPC JSON-RPC |
| **Discovery schemas** (147f) | `discovery.topology/health/query/bonds` — 4 new methods, both universal-ipc and orchestrator dispatch |
| `IpcStream` abstraction | 15 crates migrated, `pin-project` dep eliminated, -284 lines platform boilerplate |
| `mesh.enroll` method | JSON-RPC endpoint ready for BTSP gate enrollment |
| Deep debt exhausted | Zero unsafe, zero `todo!()`, zero production mocks, zero files >800L, zero hardcoding |

### What songBird Delivers (functional)

| Item | Status | Delivered |
|------|--------|-----------|
| footPrint `PROXY_PATH` drawbridge route | **COMPLETE** | Wave 147f — config: `SONGBIRD_DRAWBRIDGE_ROUTES=/footprint=footprint`, `SONGBIRD_PROXY_ROUTES=footprint=http://127.0.0.1:8090` |
| esotericWebb JSON-RPC access | **COMPLETE** | Wave 147f — `POST :7780/jsonrpc` (no auth, auto-forwards to IPC) |
| Discovery schemas for esotericWebb | **COMPLETE** | Wave 147f — topology, health, query, bonds |
| tideGlass drawbridge bonds | **COMPLETE** | Wave 140a — LINCS, GEO, ChEMBL, NF Data Portal |
| `mesh.enroll` server-side endpoint | **READY** | Wave 147b — awaiting BTSP proof protocol |

### Deployment Configuration for footPrint

footPrint systemd unit on sporeGate needs these env vars for songBird drawbridge:

```bash
SONGBIRD_DRAWBRIDGE_ROUTES=/footprint=footprint
SONGBIRD_PROXY_ROUTES=footprint=http://127.0.0.1:8090
```

This routes `:7780/footprint/ext/geocode` → footPrint `:8090/ext/geocode` (path prefix stripped).

### esotericWebb Integration

esotericWebb can now call songBird via standard HTTP:

```bash
curl -X POST http://127.0.0.1:7780/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"discovery.health","params":{},"id":1}'
```

Response: `{"alive":true,"mesh_active":true,"registered_services":N}`

Available discovery methods: `discovery.topology`, `discovery.health`, `discovery.query`, `discovery.bonds`, `discovery.peers`.

### What We Need from Other Teams

| Need | Owner | Detail |
|------|-------|--------|
| **BTSP enrollment proof spec** | cellMembrane + bearDog | `mesh.enroll` accepts `{node_id, public_key}` — what format is the `proof` field? Ed25519 signature over what payload? |
| **footPrint deploy on sporeGate** | sporeGate ops | Systemd unit shipped by cellMembrane. Deploy it + set env vars above. |
| **E2E validation trigger** | primalSpring | `footprint-drawbridge-live` scenario is P2 TODO — what's the gate? |

### No Blockers

songBird has **zero P0/P1 items remaining**. All 3 upstream demands resolved this wave.
All ecosystem milestones clear (Phase 2 14/14, CAC 6/6, Glacial 8/8). Deep debt
targets exhausted. Standing by for BTSP proof spec and live deployment.

## Fossil Record

Docs preserved as fossil record in `ecoPrimals/`. Archived specs in `specs/archived/`.
Wave history in `REMAINING_WORK.md` and `CHANGELOG.md`.
