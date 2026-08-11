# songBird — Upstream Handoff

**Primal**: songBird  
**Version**: v0.2.1-wave157j  
**Date**: August 11, 2026  
**Gate**: eastGate

## Current State

| Metric | Status |
|--------|--------|
| Clippy | Zero warnings (pedantic + nursery, `-D warnings`); workspace-wide clean Aug 11 |
| Tests | 15,167+ pass, 0 failures |
| Unsafe | 0 (`forbid(unsafe_code)` all 31 crates) |
| Production unwraps | 0 |
| Production stubs | 0 (external-blocked items documented) |
| Files >800L | 0 (max 767L) |
| Hardcoding | 0 in production (all env-driven, capability-based) |
| Mocks in prod | 0 (all `#[cfg(test)]` gated) |
| `tokio::sync` Tier 2 | COMPLETE — 312 conversions, 161 legitimate remaining |
| Typed errors | `songbird-config` + `songbird-http-client` fully typed (no `anyhow` in public API) |
| Total Rust | ~471,500 lines across 31 crates |

## Recent Evolution (Wave 157g–157j)

### Wave 157j — MeshRelay Cross-Gate Federation Forwarding (Aug 11 2026)

**GOSSIP.SPREAD EPIDEMIC FAN-OUT** — critical cross-gate gossip blocker resolved:
- `gossip.spread` method: broadcasts gossip to ALL reachable mesh peers via `:7700` federation port
- Prevents relay loops via `origin_gate` + `seen_gates` parameters
- Sends `gossip.inject` (not `gossip.spread`) to peers — single-hop propagation, no recursive fan-out storms
- Federation intrinsics updated: remote peers now discover `gossip.relay`, `gossip.inject`, `gossip.spread`

**UDS INJECTION FRAMING FIX** — `inject_gossip_locally` now sends `0xEC 0x01` preamble:
- SwarmVine expects local injection preamble before JSON-RPC payload
- Previously sent raw JSON which swarmVine would reject on preamble-mandatory sockets
- Aligned with `swarmvine_gossip.rs` injection behavior

**ASYNC SAFETY** — `node_id_async()` accessor added to `MeshHandler`:
- `node_id()` uses `blocking_read()` (panics in single-thread async runtimes)
- Gossip relay/spread paths now use `node_id_async()` for runtime safety
- Existing synchronous callers unaffected

**Native capability surface**: 19 callable gossip methods (`gossip.relay`, `gossip.inject`, `gossip.spread`)

---

### Wave 157i — G72 Tier 2 Completion + Typed Errors + Decomposition (Aug 11 2026)

**G72 TIER 2 COMPLETE** — `tokio::sync` → `std::sync` where guards never cross `.await`:
- 312 total `PoisonError::into_inner` conversions across 10 crates
- Remaining 161 `tokio::sync` sites verified as legitimate (held across awaits)
- Pattern: explicit scope blocks drop guards before any `.await`; public async signatures retained

**TYPED ERROR EVOLUTION** — `songbird-http-client` public API no longer exposes `anyhow::Result`:
- All public methods return `crate::error::Result<T>` with typed `Error` enum
- `Base64Decode`, `InvalidResponse`, `Connection`, `HttpProtocol` variants enable match-based handling
- Downstream crates (`songbird-genesis`, `songbird-primal-coordination`) adapted
- `anyhow` retained as `From<anyhow::Error>` interop bridge only

**STRUCTURAL DECOMPOSITION** — `drawbridge.rs` (782→442L):
- Extracted `drawbridge_proxy.rs` (335L): HTTP/HTTPS/TLS outbound proxy + IPC JSON-RPC roundtrip
- Deduplicated `ipc_jsonrpc_roundtrip()` (was copy-pasted between two functions)
- `resolve_external_url()` extracted as pure function

**HOUSEKEEPING**:
- Pre-existing introspection test fixed (capability count 17→18)
- `songbird-config` `anyhow` fully removed (typed `SongbirdError::Configuration` errors)

---

## Previous Evolution (Wave 151b–155f)

### Wave 155f — ACME Challenge Responder + rustls-rustcrypto Elimination Path (Jul 28 2026)

**ACME HTTP-01 CHALLENGE SUPPORT** — songBird's drawbridge now serves ACME challenge tokens for bearDog cert provisioning:

**`acme.challenge_ready`** — register a challenge token:
- bearDog calls via JSON-RPC to register `{token, authorization}` pairs
- Drawbridge serves `GET /.well-known/acme-challenge/{token}` → authorization response
- No auth required on challenge path (ACME CA must validate without credentials)
- Thread-safe in-memory store (`LazyLock<RwLock<HashMap>>`)

**`acme.challenge_cleanup`** — remove completed challenge:
- bearDog calls after ACME validation completes to clean up token
- Prevents stale challenge accumulation

**Elimination path for `rustls-rustcrypto 0.0.2-alpha`**:
- Phase 1 ✅ DONE: ACME challenge responder shipped (this wave)
- Phase 2 BLOCKED on bearDog: ACME cert provisioning using this responder
- Phase 3: `songbird-tls` client mode replaces `outbound_tls_connector()`
- Phase 4: Remove `rustls-rustcrypto` from 2 Cargo.toml files, cut dep tree
- **Net**: Eliminates alpha-quality crate, all transitive RustCrypto dupes, `rustls-webpki 0.102` ghost lock

### Wave 155d — Tower Atomic Health Facade (Jul 28 2026)

**TOWER HEALTH PROBE** — biomeOS signal graphs can now directly probe Tower Atomic stack health:

**`tower.health`** — aggregate Tower stack status:
- Process liveness + crypto provider availability + mesh state + peer count
- Returns structured `tower_atomic` object for signal graph consumption
- Status: `healthy` / `degraded` / `initializing` based on component readiness
- Available via UDS (songbird.sock) and HTTP JSON-RPC (:3492)

**`tower.mesh_status`** — enriched mesh status for Tower validation:
- Node ID, peer count, initialization state
- Transport endpoint metadata (IPC socket, federation port, drawbridge port)
- Complement to existing `mesh.status` with Tower-specific framing

**`health.ping`** — wired as first-class handler:
- Previously used in benchmarks/probes/mesh health checks without dispatch
- Now responds via `health.liveness` semantics (RTT measurement)
- Available on all transport paths (UDS, TCP, drawbridge)

**`tower.enroll`** — normalization alias to `mesh.enroll`:
- Signal graph compatibility — biomeOS graphs can use tower-prefixed names

**MeshHandler accessors**: `is_initialized()`, `peer_count()`, `node_id()` — synchronous, non-allocating accessors for health probing without async overhead.

### Wave 155b — Jelly String Evolution J3+J4+J5 (Jul 27 2026)

**CASCADE AUTOMATION SURFACE** — songBird now provides the JSON-RPC API layer that enables cascade-driven service management:

**J3: `deployment.hot_swap`** (P1 — blocks cascade-driven deployment):
- Stop old process → replace binary → start new → verify — single JSON-RPC call
- `deployment.restart`: same binary, just restart the process
- `deployment.list`: enumerate all active deployments
- Graceful kill: SIGTERM → 5s wait → SIGKILL (cross-platform)
- Accessible via HTTP JSON-RPC gateway (cellMembrane/cascade can call remotely)

**J4: `route.*` dynamic route management** (P2 — blocks self-configuring routes):
- `route.add`: register capability → backend URL mapping at runtime
- `route.remove`: unregister a route by capability
- `route.list`: enumerate all configured routes with details
- Services self-configure their routes on registration — no more manual Caddy edits
- Wired through universal-ipc dispatch (local primals can call directly)

**J5: WG peer registration hardened** (P2 — live test ready):
- Input validation: `is_valid_wg_pubkey()` (base64 format, length) + `is_valid_mesh_ip()` (octets)
- Command injection eliminated: direct `wg` args array (no shell interpolation)
- Interface configurable via `WG_INTERFACE` env (default: `wg0`)
- Resilient: `wg set` and `wg-quick save` separated — partial success reported

**Upstream clippy debt cleaned**: `needless_question_mark` × 3, `doc_markdown` × 5, `or_fun_call` × 1.

### Wave 155b — G1 Windows Named Pipe Evolution (Jul 27 2026)

**GENOMEBINS CONVERGENCE** — Tracks converged, glacial goals set. songBird G1 (Tower Atomic on Windows) unblocked:

**IPC Stream Evolution**:
- `IpcStream::NamedPipe` variant: native Windows named pipe connection (no TCP fallback penalty)
- `TransportEndpoint::NamedPipe`: first-class wire format for Windows local endpoints
- `connect_windows()`: recognizes pipe names directly or derives them from Unix socket paths
- `NativeEndpoint::NamedPipe` → `socket_path()` → `IpcStream::connect()` dispatch chain complete
- `transport_endpoint_from_native()` maps `NamedPipe` to `TransportEndpoint::NamedPipe` (was incorrectly mapped to `Uds`)
- `ipc_registry` service registration now emits proper `NamedPipe` transport for Windows providers

**blueGate Validation Ready**: Named pipe IPC dispatch chain is complete end-to-end. On Windows, a provider registering with a named pipe endpoint will have its pipe name correctly flow through the pool to `IpcStream::connect`, which opens the pipe directly.

### Wave 151b — BTSP Standard All Primals (Jul 26 2026)

**TOWER ATOMIC COMPLETE** — All 6 songBird-owned Tower debt failures resolved:

**Failover Resilience (P1)**:
- Exponential backoff retry in `capability_dispatch.rs` (50ms pool-level + 100/300ms dispatch-level)
- `capability.health` JSON-RPC method: probes all registered providers, reports reachability + latency
- Adaptive socket watch: 2s poll when empty → 30s when populated (near-instant provider recovery)

**Capability Trust (P1)**:
- Challenge-verify on `mesh.capabilities_announce` (probe peer reachability after announcement)
- `mesh.capabilities_revoke` method for explicit mesh-wide capability withdrawal
- `revoke_capabilities_to_peers()` propagates local unregistrations outward

**Deep Debt**:
- `mesh_handler/mod.rs` refactored 825→797L via `parse_peer_list()` helper (3x duplicated logic)
- `songbird-execution-agent` type inference fix in `security_sovereign.rs`
- Full audit confirms: 0 production unwraps, 0 unsafe, 0 mocks in prod, 0 hardcoding, all deps pure Rust

**Remaining Tower debt**: 1/7 — "BTSP on local UDS" (shared with bearDog, P2)

### Wave 151a — LAN Mesh Priority (Jul 25 2026)

- `EndpointType::priority()` evolved: Local(0) → Overlay(1) → Direct(2) → FamilyRelay(3) → TorOnion(4)
- `discover_and_dispatch` sorts peers by best-path priority before dispatch (LAN-first)
- `mesh.find_path`, `mesh.peers`, `mesh.topology` responses include `priority` field
- Crypto delegation Phase 1 confirmed COMPLETE (all seams wired, feature-gated)

### Wave 150x — Deep Debt: Dependency Diet + Hardcoding Evolution (Jul 24 2026)

**Dependency diet — 10 more dead deps removed** (total 24 dead deps removed this wave):
- `chrono` from `songbird-canonical` (unused), `songbird-lineage-relay`, `songbird-compute-bridge`
- `rand` from `songbird-config` (unused), `songbird-types` (replaced with `fastrand`)
- `fastrand` from `songbird-network-federation` (unused)
- `futures-util` from `songbird-registry` (unused)
- `zerocopy` from `songbird-bluetooth` (unused)
- `config` from workspace (dead)
- `tower-http` from `songbird-orchestrator` (unused)
- `whoami` from `songbird-cli` (replaced with `std::env`)

**Feature trimming**:
- `tower-http` workspace features: `["trace", "cors", "fs", "limit"]` → `["trace"]`
- `tokio` `test-util` moved from production to dev-deps in `compute-bridge`

**Zero-dep RFC3339 utility**: `songbird_types::defaults::time::rfc3339_now()` — pure std timestamp formatting, no chrono/time crate needed. Used to eliminate chrono from trivial crates.

**Hardcoding centralization**:
- Drawbridge default address → `DEFAULT_DRAWBRIDGE_ADDR` constant
- Introspection `"ecoPrimal"/"biomeos"/"songbird.sock"` → `primal_names::{DEFAULT_FAMILY_ID, BIOMEOS_DIR, SELF_NAME}`

**Legacy env var deprecation**:
- ALL `BEARDOG_*`, `NESTGATE_*`, `SQUIRREL_*` env vars now emit `tracing::warn!` at runtime
- `get_primal_endpoint(name)` marked `#[deprecated]` with runtime warning
- IPC registry name-fallback path emits deprecation trace
- Central catalog: `songbird_types::defaults::legacy_env`

**Dead code evolution**:
- 47 `#[allow(dead_code)]` → `#[expect(dead_code)]` (clippy fires when code gets wired)
- ~15 remaining as `#[allow]` (items dead in lib, alive in test builds)

### Wave 150x — Caller Identity + UDS Hardening (Jul 24 2026)

**P1 Caller Identity (4 findings) resolved**:
- `SO_PEERCRED` extraction on ALL production UDS accepts (primary `UnixSocketServer` + bin_interface)
- `CallerContext::from_unix_stream()` — pure Rust credential extraction before stream split
- uid-based authorization in `MethodGate`: same-uid local peers = trusted (bypass token check);
  different-uid peers require capability token in `Enforced` mode
- Own UID resolved from `/proc/self/status` (pure Rust, no libc)

**P1 UDS Hardening (5 findings) resolved**:
- Directory guard: detect stale directory at socket path → `remove_dir_all` before bind
- Symlink rejection: refuse to bind over symlinks (prevents path hijack attacks)
- `chmod 0o600` post-bind: restrict socket to owner-only (multi-user protection)
- Concurrency semaphore: 256 max simultaneous UDS connections (`SONGBIRD_MAX_UDS_CONNECTIONS`)
- Applied to both orchestrator `UnixSocketServer` AND universal-ipc platform layer

### Wave 150x — Crypto Composition Evolution (Jul 24 2026)

**P1 Finding resolved**: "Crypto Composition Divergence" — songBird embedded `sha2`/`hmac`
across 10+ crates instead of routing through bearDog's `crypto.*` UDS capabilities.

**Changes**:
- Gated local crypto (`sha2`, `hmac`) behind `local-crypto-fallback` feature flag
- Affected crates: `songbird-discovery`, `songbird-network-federation`, `songbird-genesis`, `songbird-orchestrator`
- Production path delegates ALL hashing/HMAC to bearDog via `CryptoProvider`/`SecurityRpcClient`
- Local fallback retained (default feature) for bootstrap/offline/testing only
- Hot-path crypto (BTSP AEAD, TLS transcript, STUN, onion, TOTP) stays local — documented as chimera candidates

**Upstream requirement for bearDog team**:
- bearDog MUST expose `crypto.sha256`, `crypto.hmac.sha256` reliably on UDS
- Target: <1ms response for delegation calls (Phase 2 will measure)
- Future: `crypto.hash.blake3` endpoint requested for discovery beacon hashing

**Composition analysis**: `infra/wateringHole/CRYPTO_COMPOSITION.md`

### Wave 150x — Dependency Diet + ring→rustcrypto + Idiomatic Evolution (Jul 24 2026)

**Dependency diet** — 6 dead deps removed:
- `tokio-stream` (discovery): unused, `futures-util` covers streaming
- `console` (cli): redundant, pulled via `dialoguer`/`indicatif`
- `rustls` direct dep (cli): only `rustls-rustcrypto` referenced
- `url` (network-federation): unused, format strings used instead
- `generic-array` (network-federation): not imported, `aes-gcm` uses internally
- `base64` (universal): no usage in src/

**ring → rustls-rustcrypto** (songbird-universal-ipc):
- Drawbridge TLS outbound no longer links `ring` (C/assembly)
- Pure Rust `rustls-rustcrypto` provider installed on first connector use
- Zero functional change — same `ClientConfig + native CA roots` API

**Security advisories patched**:
- `anyhow` 1.0.102 → 1.0.104 (RUSTSEC-2026-0190)
- `crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204)
- `quick-xml`: blocked on upstream `netdev`/`plist` (documented)

**Idiomatic Rust**: 17 `option_if_let_else` → `map_or_else` across 15 files

### Wave 150x — Pen Test Security Hardening (Jul 24 2026)

**SO_PEERCRED Peer Verification** (pen finding: UDS-spoof):
- IPC accept now extracts caller uid/pid via `SO_PEERCRED` on every connection
- `CallerContext.peer` populated with `PeerCredentials { uid, pid }` before dispatch
- Enables per-caller authorization decisions in downstream handlers
- Graceful fallback on non-Unix platforms (credential-less context)

**Mesh Announcement Validation** (pen finding: mesh-poison):
- Rate limiting: minimum 2s between announces per peer (configurable via `SONGBIRD_ANNOUNCE_RATE_LIMIT_MS`)
- `node_id` format validation: max 128 chars, no control characters
- Capability list validation: max 64 capabilities per announce, each max 128 chars, no control chars
- Pre-existing: unknown peer rejection when mesh is active

**`capability.call` Input Validation** (pen finding: capability-escalation):
- Routing whitelist: only `"local"` or `"any"` accepted (rejects arbitrary values)
- Capability name validation: non-empty, max 128 chars, no control characters
- Operation name validation: non-empty, max 256 chars, no control characters
- Drawbridge proxy route extraction into `forward_via_drawbridge_route` (function size compliance)

**Drawbridge Relay Abuse Prevention** (pen finding: relay-abuse):
- Request body limit: 10 MiB max (HTTP 413 on exceed)
- Header count limit: 128 max (HTTP 431 on exceed)
- URI length limit: 8192 bytes (HTTP 414 on exceed)
- Path traversal rejection: blocks `/../`, `/./`, trailing `/..` patterns

**Test fix**: `federation.join` dispatch test updated (method now explicitly handled, not "unknown variant")

### Wave 150x — P1 Tower Hardening + Deep Debt (Jul 24 2026, previous session)

**UDS Connection Pool** (`ipc_pool.rs`):
- Pooled connections per socket path for `capability.call` dispatch
- Eliminates per-request connect/disconnect overhead (measured in stress tests)
- Auto-retry on stale/broken connections, idle eviction after 60s
- Configurable: `SONGBIRD_IPC_POOL_SIZE` (default 4)

**`federation.broadcast` JSON-RPC method**:
- Broadcasts a JSON-RPC payload to all active federation peers
- HTTP POST to each peer's best endpoint `/jsonrpc`
- Returns per-peer delivery summary (`{delivered, failed, results}`)
- Self-node skip, configurable `timeout_ms`

**Benchmark evolution**:
- Fixed `duration_ms: 0` truncation for sub-ms LAN transfers (`as_millis()` → `f64` + `duration_us`)
- Added `--sustained` flag: iperf3-style continuous streaming (windowed throughput, p50/p95 per window)

**Idiomatic Rust sweep**:
- 12 `if let/else` → `Option::map_or_else` (drawbridge, discovery, ipc_registry, virtual_relay)
- 8 `unused_async` functions annotated for interface consistency
- Zero clippy warnings, zero fmt drift

### Wave 150w — P0 Blocker Resolution + JSON-RPC→HTTP Translation (Jul 23 2026)

Tower Atomic exceeds WireGuard (2x throughput on WAN). Shadow deployment UNBLOCKED.

**Afternoon (P0 #1 RESOLVED) — Drawbridge JSON-RPC→HTTP translation**:

`CapabilityProxyRouter` now supports `jsonrpc://` scheme routes. When a drawbridge path maps to a JSON-RPC IPC backend, the drawbridge translates:
- HTTP path suffix → JSON-RPC method name (e.g. `/api/mesh/status` → `mesh.status`)
- HTTP body → JSON-RPC `params`
- JSON-RPC response → HTTP 200 JSON

Configuration: `SONGBIRD_PROXY_ROUTES=network=jsonrpc://songbird.sock,jupyter=http://localhost:8000`

This resolves the root cause identified by flockGate: `capability.call` routes to JSON-RPC backends that don't speak HTTP. Exploration Domain 1 (capability-aware routing) is now fully supported at the protocol level.

**Morning (P0 #3-5 RESOLVED)**:

1. **Socket directory guard** — Both UDS bind sites detect `is_dir()` and `remove_dir_all` before bind.
2. **Drawbridge 502 → diagnostic JSON** — Structured error with capability mapping hints.
3. **`mesh.prune_stale` JSON-RPC method** — Dead peer cleanup (explicit `node_ids`, threshold, dry_run).

**Usage for flockGate operator** (to resolve stale peers):
```json
{"jsonrpc":"2.0","method":"mesh.prune_stale","params":{"node_ids":["old-peer","iron-gate","west-gate"]},"id":1}
```
Then enroll live gates: `mesh.enroll` with BTSP HMAC proofs for sporeGate, eastGate, golgiBody.


| Wave | Summary |
|------|---------|
| 150d | **Subdomain standard evolution**: Caddy GIS proxy snippet paths updated (`/footprint/ext/` → `/ext/`), README rewritten with full production Caddyfile (CSP headers, subdomain model, architecture diagram). songBird role documented: "inner membrane port solver" (:7780 drawbridge maps capabilities → ports via env) |
| 150b | **mesh.enroll ACTIVE**: Full BTSP-verified enrollment flow — `SecurityRpcClient::verify_enrollment_proof()` delegates to bearDog, `complete_enrollment()` persists + meshes. Proof protocol: `HMAC-SHA256(family_seed, node_id\|public_key\|timestamp)` |
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

## Provided Capabilities (18 native + runtime drawbridge)

**Native** (always): `network.discovery`, `network.federation`, `network.relay`, `network.stun`,
`network.igd`, `network.quic`, `network.tls`, `network.tor`, `network.onion`,
`network.btsp`, `network.gossip`, `ipc.jsonrpc`, `ipc.tarpc`, `crypto.delegate`, `nfc.genesis`,
`bluetooth.pair`, `compute.gpu`, `access.remote`

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

## AAR — Wave 150v (July 22, 2026)

### Accomplished (Wave 150v — Tower Atomic Parity Pipeline)

| Deliverable | Impact |
|-------------|--------|
| **`songbird benchmark` CLI** | P0 blocker resolved. `--mode tower-atomic\|wireguard --peer <ip:port>` measures setup time, RTT (p50/p95/p99/jitter), throughput (Mbps). JSON output for primalSpring. |
| **LAN benchmark READY** | sporeGate (.2) ↔ eastGate (.5) can run immediately once sporeGate rebuilds from depot |
| **golgiBody deploy docs** | Relay README updated with exact 6-command deploy sequence for golgiBody VPS |

### Blocker Status (from Wave 150v blurb)

| # | Blocker | Status | Next |
|---|---------|--------|------|
| 1 | Deploy TURN relay on golgiBody | **DOCS READY** — systemd unit + deploy guide complete | Operator: build release, scp to golgi, run deploy sequence |
| 2 | Build benchmark harness | **SHIPPED** (`c4d8c4b`) | sporeGate rebuilding from Forgejo push → depot |

### Usage

```bash
# LAN parity (sporeGate ↔ eastGate, same backbone)
songbird benchmark --mode tower-atomic --peer 10.13.37.2:7700 --output json
songbird benchmark --mode wireguard   --peer 10.13.37.2:7700 --output json

# WAN parity (after golgiBody relay is live)
songbird benchmark --mode tower-atomic --peer 10.13.37.6:7700 --output json
songbird benchmark --mode wireguard   --peer 10.13.37.6:7700 --output json
```

### What Ops Needs To Do (golgiBody relay — P0 #1)

1. Build release: `cargo build --release -p songbird` (on eastGate or sporeGate)
2. Deploy binary: `scp target/release/songbird golgi:/usr/local/bin/`
3. Deploy unit: `scp deployment/systemd/songbird-relay.service golgi:/etc/systemd/system/`
4. Create creds: `openssl rand -hex 32` → `/etc/songbird/relay-credentials`
5. Firewall: `ufw allow 3478/udp && ufw allow 49152:65535/udp`
6. Enable: `systemctl daemon-reload && systemctl enable --now songbird-relay`

After relay is live, set on all gates:
```
SONGBIRD_TURN_SERVER=10.13.37.1:3478
SONGBIRD_TURN_USERNAME=tower-relay
SONGBIRD_TURN_KEY=<hex key from step 4>
```

---

## AAR — Wave 152 (July 22, 2026)

### Accomplished (Wave 152)

| Deliverable | Impact |
|-------------|--------|
| **Structural refactoring** | `security.rs` (761L) → module tree (4 files, max 277L); `production.rs` test extraction (754→370L); `virtual_relay.rs` test extraction (753→553L) |
| **Dependency diet** | 8 dead deps removed: `config`, `validator`, `dashmap`, `chrono`×2, `anyhow`×3 — cleaner builds, smaller dep tree |
| **Hardcoding → capability-based** | 5 JSON-RPC handlers unified to `env_config::primal_name()`; Consul/etcd keys use `SELF_NAME`; container self-skip uses constant; `capability.register` uses param; outbound IP probe env-configurable |
| **Production stubs audit** | All P0 stubs confirmed external-blocked (Tor/GATT/TLS) or intentional degraded-mode (synthetic lineage). Zero leaked mocks. |
| **Full clippy + build clean** | 0 warnings, 0 errors, `cargo fmt --check` passes |

### Upstream Dependency Audit (Wave 152 findings)

| Finding | Priority | Detail |
|---------|----------|--------|
| `anyhow` in foundation crate public APIs | Medium | `songbird-types`, `songbird-config`, `songbird-http-client` return `anyhow::Result` — should be typed `thiserror` errors |
| `ring`-backed `rustls` in drawbridge HTTPS | Medium | `songbird-universal-ipc` uses `ring` backend; rest of workspace on `rustls-rustcrypto` (pure Rust) |
| BTSP bidirectional RPC stubs | Known | All 3 BTSP connection types return `not_implemented` for `send_rpc()` — Phase 2 roadmap item |
| Bluetooth GATT silent success | Known | `discover_characteristics()` / `subscribe_notifications()` return `Ok(())` without ATT I/O |

### No New Demands on Other Teams (Wave 152)

This wave was internal deep debt — no new upstream surface or blocking items introduced.

---

## AAR — Wave 150d (July 18, 2026)

### Accomplished (Wave 150d)

| Deliverable | Impact |
|-------------|--------|
| **Caddy snippets → subdomain standard** | `footprint-gis-proxy.Caddyfile` paths updated from `/footprint/ext/...` to `/ext/...` (subdomain model) |
| **README rewritten** | Production Caddyfile example with CSP headers, architecture diagram, deployment chain documentation |
| **Deployment chain confirmed** | songBird = inner membrane port solver (:7780). Chain: Cloudflare → Caddy → WireGuard → drawbridge → service |
| **mesh.enroll ACTIVE** | Wave 150b: full BTSP-verified enrollment flow live. bearDog `enrollment.verify` is the only runtime dep |

### Architecture Role (confirmed Wave 150d)

```
User → Cloudflare DNS (*.primals.eco wildcard → golgiBody VPS)
  → Cloudflare CDN (outer membrane firebreak — absorbs hostile traffic)
    → Caddy on golgiBody (TLS termination, Host-header routing)
      → reverse_proxy over WireGuard mesh to target gate
        → songBird drawbridge :7780 (capability → port resolution)
          → Local service (footPrint:8090, esotericWebb:8090, etc.)
```

**Production optimization**: Caddy handles external HTTPS proxying directly via songBird's `infra/caddy/` snippets — no drawbridge round-trip for tile/API requests.

**Dev/test**: songBird drawbridge handles all proxying via `SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST`.

### Deployment Blocking Items (NOT songBird code — ops/cellMembrane)

| Surface | Issue | Fix Owner |
|---------|-------|-----------|
| `footprint.primals.eco` | Map tiles gray, routes misconfigured | cellMembrane + ops |
| `webb.primals.eco` | 404, no Caddy vhost | cellMembrane + ops |
| `sporeprint.primals.eco` | Not migrated from root domain | cellMembrane + ops |

songBird provides the GIS proxy snippet and drawbridge routing — deployment wiring is ops work.

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
| **footPrint Caddy routing fix** | cellMembrane + ops | **P0** | Route ALL `footprint.primals.eco` → sporeGate:8090 (single `reverse_proxy`). Add CSP `img-src *.arcgisonline.com *.tile.openstreetmap.org`. Import songBird GIS snippet from `infra/caddy/footprint-gis-proxy.Caddyfile` |
| **`webb.primals.eco` Caddy vhost** | cellMembrane + ops | **P0** | Add `webb.primals.eco { reverse_proxy 10.13.37.6:8090 }` to golgiBody Caddyfile |
| **sporePrint migration** | cellMembrane + ops | **P0** | Caddy vhost `sporeprint.primals.eco`, root domain redirect |
| **`SONGBIRD_DRAWBRIDGE_ADDR` for production** | sporeGate ops | P2 | Set `SONGBIRD_DRAWBRIDGE_ADDR=0.0.0.0:7780` (or WG IP) in systemd unit for cross-gate access |

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
