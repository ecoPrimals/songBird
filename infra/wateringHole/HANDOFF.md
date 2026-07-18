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

## AAR — Wave 149b (July 18, 2026)

### Accomplished (Wave 149b)

| Deliverable | Impact |
|-------------|--------|
| **Production unwrap elimination** | 71 `writeln!` unwraps → `fmt::Result` refactor; 0 production unwraps remain |
| **File splits** | `drawbridge.rs` 1,019→578L, `mesh_seed.rs` 834→523L (test extraction) |
| **Clippy pedantic evolution** | `uninlined_format_args` ×58, `doc_markdown` ×40, 24 `const fn` promotions |
| **Lint enforcement** | `doc_markdown` + `uninlined_format_args` promoted from "allowed" to "enforced" — future regressions auto-caught |
| **Format** | 10-file drift cleared, clean across all 31 crates |

### songBird Dimensional Scorecard (post-149b)

| Metric | Before 149b | After 149b |
|--------|-------------|------------|
| Clippy (pedantic+nursery) | 556 warnings | **0** (with `-D warnings`) |
| Production `unwrap()` | 81 | **0** |
| Files >800L | 2 | **0** |
| `cargo fmt` drift | 10 files | **0** |
| Tests | 14,322+ | 14,322+ (654 pass in universal-ipc) |

### What We Need from Other Teams

| Need | Owner | Priority | Detail |
|------|-------|----------|--------|
| **bearDog `enrollment.verify` endpoint** | bearDog | **P1** | songBird calls `enrollment.verify({node_id, public_key, timestamp, proof})` — bearDog must implement this method (verifies HMAC against family seed) |
| **cellMembrane integration** | cellMembrane | P2 | cellMembrane's `gate.enroll` should call songBird's `mesh.enroll` to complete mesh registration |
| **footPrint deploy on sporeGate** | sporeGate ops | P2 | Systemd unit shipped. Deploy + set drawbridge env vars |

### songBird `mesh.enroll` — ACTIVE (Wave 150b)

```
JSON-RPC method: "mesh.enroll"
Params: {
  "node_id": "<gate-name>",
  "public_key": "<wg-pubkey>",
  "timestamp": <unix-epoch-seconds>,
  "proof": "<base64 HMAC-SHA256(family_seed, node_id|public_key|timestamp)>",
  "address": "<ip:port>"  (optional — for direct mesh registration)
}

Success response: { "enrolled": true, "node_id": "...", "mesh_active": true/false }
Rejection:        { "enrolled": false, "reason": "proof_invalid"|"security_provider_unavailable" }
```

**Proof protocol**: `HMAC-SHA256(family_seed, node_id || "|" || public_key || "|" || timestamp)`
- bearDog verifies via `enrollment.verify` JSON-RPC
- On success: node persisted to `peers.toml` + added to live mesh (if active)
- Graceful degradation: if bearDog unavailable, returns structured error (no crash)

**cellMembrane integration**: call `mesh.enroll` via songBird's IPC socket or HTTP `/jsonrpc` endpoint.

### No Blockers (songBird side)

songBird has **zero P0/P1 code quality items remaining** and `mesh.enroll` is now
**ACTIVE** — ready to process enrollment requests. The only runtime dependency is
bearDog's `enrollment.verify` endpoint being live (graceful degradation if unavailable).

## Fossil Record

Docs preserved as fossil record in `ecoPrimals/`. Archived specs in `specs/archived/`.
Wave history in `REMAINING_WORK.md` and `CHANGELOG.md`.
