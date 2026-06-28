# Songbird Remaining Work

**Date**: June 28, 2026  
**Version**: v0.2.1-wave128  
**Last Deep Debt Audit**: Wave 128 (June 28, 2026) — WG auto-init validation and hardening: multi-tier discovery (1: `wg show` with CAP_NET_ADMIN, 2: WG config file parsing `/etc/wireguard/wg0.conf`, 3: user-accessible `~/.config/songbird/mesh-peers.toml`); `parse_wg_conf()` INI parser for non-root environments; `config_dir()` path resolver added to `songbird-types`; orphaned doc comment fixed; +6 new tests (WG conf parsing, mesh-peers.toml format); deployed `mesh-peers.toml` on flockGate for immediate zero-config init. Previous: Wave 124 (June 23, 2026) — Zero-config mesh auto-initialization: `discover_wireguard_peers()` reads WG peer allowed-IPs from `wg show all dump` (tier-3 after env vars and persisted state); WG-discovered peers auto-registered as overlay endpoints (priority 0); `mesh.status` returns `initialized: false` / `"awaiting_init"` instead of hard error when mesh not yet seeded (probers get structured status, not failure); +4 new tests for WG dump parsing. Previous: Wave 123 (June 22, 2026) — Topology-aware routing Phase 1: `BeaconMesh::record_overlay_connection()` for WireGuard latency tracking; overlay-aware health probing (probe WG peers alongside WAN bootstrap, record with correct EndpointType); path-optimal `find_peer_with_capability` (min-cost selection among multiple holders); targeted remote dispatch (cached-holder fast-path before fan-out); `topology_graph` module (merge local edges + peer gossip into global adjacency, partition detection); `beacon.rs` refactored (858→347L via test extraction). Previous: Wave 121 (June 21, 2026) — zero clippy --all-targets warnings, zero `"literal".to_string()` in production (2,020→0), `keys.rs` (801→555L), all external deps justified, zero production mocks. Previous: Wave 111 (June 13, 2026) — riboCipher transport signal detection shipped (Stream 7); `connection.rs` (815→472L) smart-refactored via `session_protocol.rs` extraction; `hardcoded_elimination.rs` (872→689L) via `hardcoded_replace.rs` extraction; BTSP max frame magic number evolved to `BTSP_MAX_FRAME_SIZE` constant; ad-hoc URL construction centralized to `jsonrpc_endpoint_url()` helper; stale "DEEP DEBT: polling loop" comment removed from `lineage-relay/relay.rs` (implementation was already event-driven).
**Composition Audit Status**: **CLEAR THROUGH STADIAL GATE** — UB-1 resolved (`songbird-turn-client` crate for lithoSpore). `primal.announce` adopted (biomeOS v3.57). GAP-16 (Tower atomic: mesh.* on canonical UDS) resolved. Songbird is no longer a sentinel blocker. Wave 206: deep debt cleanup — 2 files >800L refactored below threshold, 2 TURN server bugs fixed, `NoopVerifier` gated to test-only, legacy socket names deprecated, 5 dead functions removed.  
**Current Wave**: 113 — **Stream 6 Divergence: Mesh Partition Tolerance + Version Negotiation**: (1) MESH-PARTITION-TOLERANCE: `PeerMetadata` struct tracks per-peer version + cross-gate reachability reports. `mesh.capabilities_announce` now includes `reachable_peers` and `version` fields. `partition_status_for()` computes partition state per peer. `mesh.status` surfaces `partition_warnings` array. `mesh.health_check` emits `partition_detected: true` + `partitions[]` when cross-gate gossip reveals asymmetric reachability. (2) PEER-VERSION-MISMATCH: `probe_peer_full()` extracts version from `health.ping` response. `spawn_peer_health_loop` records peer versions on every successful probe. `mesh.status` reports `version_skew` array. `mesh.peers` includes `version` and `version_mismatch` per peer. (3) +8 new tests covering partition detection, version tracking, gossip storage, and version skew reporting. Previous: 112 — Deep Debt Evolution. Previous: 111 — FEDERATION-STATUS-WIRE + FEDERATION-RECONNECT. Previous: 110 — HEALTH-SB-01 + federation.status root cause. Previous: 107 — Topology-aware cross-gate routing (M1). Previous: 106 — Mesh persistence + federation port fix. Previous: 104 — env-var race conditions eliminated. Previous: 103 — Flaky test root-cause fix, mDNS wired. Previous: 102 — capability-neutral naming, P0 JWT hardening. Previous: 100 — 3-gate mesh integration gaps closed. Previous: 98 — Phase 2 M1: ipc.resolve returns TransportEndpoint JSON. Previous: 96 — Security hardening + mesh blockers. Previous: 82c — Coverage sprint (+553 tests).  
**Previous Waves** (full detail in `CHANGELOG.md`): 191 (ipc.register identity verification, whitespace-tolerant protocol detection, BufReader safety), 190 (IP literals, parse_endpoint IPv6, redundant clone, test Duration constants), 189 (ipc.resolve `socket` field for primalSpring tier-1 discovery), 188 (15 timeout constants, JSONRPC_VERSION, Box<dyn Error> elimination), 187 (smart refactor connection.rs, primal-name evolution, 4 timeout constants), 186 (BTSP Phase 3 live connection verification — 4 tests), 185 (deep debt: 11 timeout constants, JSON-RPC constructors, primal codename evolution), 184 (BTSP Phase 3 binary-framed dispatch fix), 183 (deep debt: lint evolution, timeout centralization, hardcoded elimination), 182 (BTSP Phase 3 spec alignment), 181 (port canonicalization), 180 (BTSP Phase 3 btsp.negotiate), 175 (PG-51 verified, ENVIRONMENT_VARIABLES.md), 174 (hardcoded IP/port elimination, flaky tests, dep cleanup, +18 tests), 173 (PG-51 socket discovery), 172 (root doc reconciliation), 171 (test coverage expansion 71.28%→73.41%, +271 tests), 170 (CLI flag alignment), 169 (remaining `new()` → `new_direct()` in bin_interface), 168 (BTSP routing + seed encoding), 167 (BTSP error frames, env fallbacks), 166 (root doc reconciliation), 165 (dep cleanup, hardcoded elimination, dead code removal), 162 (stream.shutdown BTSP fix), 161 (port centralization, dep cleanup, error typing), 160 (BTSP NDJSON auto-detect), 158 (BTSP Step 3→4 verification relay), 157 (hardcoded literals, dead deps, doc cleanup, debris removal), 154 (mock isolation, dead deps, lint hygiene), 153 (BTSP NDJSON wire-format alignment), 152 (dead deps, hardcoding, test hygiene), 151 (PG-37 capability-first routing), 150 (doc cleanup, debris removal), 149 (comprehensive deep debt: blanket lint removal, hardcoded paths, duplicate constants, mock features, stale CLI, expect safety), 148 (PG-21 persistent NDJSON sessions), 147 (mock isolation, hardcoded IP/path elimination, lint hygiene), 146 (stadial dyn audit + ring analysis), 139b (deep literal sweep), 139 (self-healing auto-discovery), 138b (hardcoded literal evolution), 138 (LD-08 socket auto-discovery), 137b-c (ipc.resolve dual-mode, stale features, port canonicalization, lint hygiene), 137 (capability naming), 136 (constant consolidation), 135 (SB-02/SB-03 resolved), 134 (primalSpring gaps), 133 (smart refactor), 132 (BTSP Phase 2), 131-119 (hardcoding, legacy scrub, coverage)

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 8,929 lib tests passed, 0 failures, 0 flaky (Wave 113 verified June 13, 2026) |
| **Line Coverage** | **73.41%** measured (llvm-cov `--workspace --lib`, Apr 27 2026; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 31 crates compile clean (~10s incremental) |
| **Clippy Pedantic** | 31/31 crates clean — zero warnings (`clippy::pedantic + nursery`, `-D warnings`, June 12 verified) |
| **Format** | Clean (`cargo fmt --check` passes; June 12 verified) |
| **Docs** | Clean (`cargo doc --workspace --no-deps` — 0 warnings) |
| **Files >800 lines** | **0** (production code) — Wave 111: `connection.rs` (815→472L via `session_protocol.rs`), `hardcoded_elimination.rs` (872→689L via `hardcoded_replace.rs`). Wave 78: `virtual_relay.rs` at 639L production + 272L tests = 911L total (tests don't count); `relay_security.rs` extracted (258L). Wave 76: `mesh_handler/mod.rs` (1004→783L via `capability_propagation.rs` SRP extraction). Wave 74: `multi_tier_coordinator.rs` (799→655L). Wave 209: `state.rs` (877→459L). Wave 206: `bin_interface/server.rs` (878→360L), `turn_server.rs` (898→679L). Wave 200: `method_gate.rs` (944L) → directory module |
| **Unsafe blocks** | **0** — `forbid(unsafe_code)` on all 31 crates |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 unguarded — `.unwrap()` in production only under `#[expect(clippy::unwrap_used, reason = "...")]` for provably infallible operations (e.g. `write!` to `String`); all others in `#[cfg(test)]` or doc examples |
| **Production `panic!()`** | 0 |
| **Production `unreachable!()`** | 2 (provably unreachable QUIC VarInt 2-bit prefix arms, documented) |
| **TODO/FIXME/HACK comments** | 0 in Rust source; 0 FIXME/HACK |
| **Commented-out code** | 0 in production library code (Wave 124 scrub); doc-style examples in comments kept intentionally |
| **`#[allow(` vs `#[expect(`** | **Wave 58**: 146 item-level `#[allow(clippy::...)]` evolved to `#[expect(clippy::...)]` (`too_many_lines`, `type_complexity`, `cast_*`, `float_cmp`, `useless_vec`, `unreadable_literal`, `items_after_statements`, `uninlined_format_args`); module-level `unwrap_used`/`expect_used` blanket suppressions correctly remain `#[allow]`. Wave 134: `dead_code` migration; Wave 136: reason strings; Wave 147/154: bare allows given reasons. Zero reasonless suppressions remain |
| **Mocks in production** | 0 (all inside `#[cfg(test)]` or `#[cfg(any(test, feature = "test-mocks"))]`; `NoopVerifier` gated Wave 206; `birdsong::mocks` gated Wave 147; `StubAllow`/`StubDeny`/`StubPassthrough`/`StubMockEncrypted` gated Wave 154) |
| **Capability discovery** | `find_primals_with_capability` — identity-agnostic, env-driven |
| **Hardcoded elimination** | All ports env-driven; canonical constants for all port defaults (Wave 181: 8 new constants — `DEFAULT_DISCOVERY_SERVICE_PORT`, `DEFAULT_OBSERVABILITY_PORT`, `DEFAULT_DASHBOARD_UI_PORT`, `DEFAULT_FEDERATION_COORDINATION_PORT`, `DEFAULT_TARPC_RPC_PORT`, `DEFAULT_GAMING_BASE_PORT`, `EPHEMERAL_BIND_ADDR`); `discovery_port()` bug fixed (was using metrics constant); tarpc port harmonized to 8091 across 3 conflicting locations; all `SafeEnv::get_port()` calls in orchestrator/universal replaced with canonical functions; all IPs (`"0.0.0.0"`, `"127.0.0.1"`, `"localhost"`) evolved to constants (`PRODUCTION_BIND_ADDRESS`, `LOCALHOST`, `is_loopback_host()`); duplicate mDNS `"224.0.0.251:5353"` replaced with canonical `MDNS_MULTICAST_GROUP` + `MDNS_PORT`; capability-first across 11+ crates; all legacy primal env vars deprecated with `tracing::warn!` |
| **JSON-RPC dispatch** | Typed `JsonRpcMethod` enum (53+ methods, 33 domain sub-enums including `Lifecycle` and `Inference`); `normalize_json_rpc_method_name()` absorbs `model.*`/`ai.*` → `inference.*`, `discovery.find_by_capability`/`net.discovery.find_by_capability` → `ipc.discover` |
| **License** | `AGPL-3.0-or-later` (workspace + per-crate; **Apr 7**: inconsistent `AGPL-3.0-only` strings eliminated) via workspace inheritance + ORC + CC-BY-SA 4.0 |
| **SPDX headers** | 100% `.rs` coverage — **Apr 7**: all updated to `AGPL-3.0-or-later` (aligned with `Cargo.toml`) |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok); locally enforced (`cargo deny check`); CI runs fmt + clippy + test only |
| **C dependencies** | Zero in default build (`blake3` uses `features=["pure"]`; `ring` NOT compiled — lockfile-only via `rustls-webpki` optional dep, see `deny.toml` stadial analysis; `ed25519-dalek` in quic behind `local-certs` feature); **Bluetooth** (`libudev`/USB stack paths): feature-gated; **sled** removed Wave 135 (SB-03 resolved), orphaned DB artifacts auto-cleaned on startup (Wave 51 fix); `parking_lot` removed (Wave 133) |
| **`async-trait`** | **0** annotations, dependency fully removed from workspace (Wave 145). 141→0: every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. No crate depends on `async-trait`. SB-06 resolved. Wave 155: eliminated 6 more production `dyn` sites (iterator, error, future, plugin, composable, callback). Remaining `dyn`: architectural `dyn Stream` (async watch), `Box<dyn SerialPort>` (external crate), `Arc<dyn Fn>` (test injection) |
| **Test infrastructure** | Zero `#[serial]`, zero hardcoded ports, zero startup sleep waits; all time-dependent tests use `start_paused`/`advance`; all network binds use port 0; `ConnectionPool` uses `tokio::time::Instant` for deterministic testing; only `std::thread::sleep` allowed in mockito sync callbacks and `std::time::Instant`-dependent cache tests (documented) |
| **Zero-copy** | `Arc<str>` IPC handler fields (mesh/punch/rendezvous/capability), `bytes::Bytes`, `SharedBytes`, `Cow<'_, str>` JSON-RPC wire types, move semantics, borrow-through redirects |
| **Total Rust** | ~422,000 lines across 31 crates |
| **primalSpring gaps** | All original gaps resolved Wave 134; Phase 43 downstream audit (6 items) completed Wave 140: UDS first-byte peek, mito-beacon credential tiers, STUN beacon auth, content distribution federation, ring lockfile documented; Wave 143: content distribution federation wired (`ContentAnnouncementStore`, `discovery.content_peers`, seeder/leecher coordination), `ring` deny.toml updated; Wave 145: `async-trait` fully eliminated (SB-06 resolved); `capability.resolve` + `discovery.peers` wired (Wave 137); LD-02 resolved (Wave 137b); LD-08 resolved + self-healing (Waves 138/139); **Phase 45c** BTSP wire-format fixes completed Wave 160: server.rs refactored with BTSP auto-detection (first-line peek routes `"protocol":"btsp"` → NDJSON handshake), `btsp.session.*` RPC mappings added |

---

## Remaining Work

### BTSP Phase 3 (FULL — Wave 180→186)

Phase 2 relay fixed (Waves 132–169). **Phase 3 implemented** (Wave 180): `btsp.negotiate` JSON-RPC handler, ChaCha20-Poly1305 encrypted framing, HKDF-SHA256 session key derivation via `btsp.server.export_keys` delegation to BearDog, graceful NULL cipher fallback, encrypted frame loop wired in both `pure_rust_server/connection.rs` and `bin_interface/server.rs`. **Spec-aligned** (Wave 182): `bond_type` parameter with cipher floor enforcement per `BTSP_PROTOCOL_STANDARD.md` `BondingPolicy` table, `preferred_cipher` backward compat, 12-byte server_nonce (aligned with audit spec), both `ciphers` array and `preferred_cipher` formats accepted. **Dispatch fix** (Wave 184): `btsp.negotiate` wired into `handle_btsp_frames` (binary-framed BTSP path) — was previously only on NDJSON path, causing "method not found" for binary-protocol clients. Now reachable on all 3 transport paths: NDJSON session, binary-framed BTSP, and bin_interface. **Live connection verification** (Wave 186): 4 new tests verifying post-negotiate encrypted frame loop on live async duplex streams — full negotiate→encrypt→exchange→disconnect lifecycle, notification handling, null-cipher fallback, and mock-security-provider E2E transition. 32 tests total. **Remaining**: Multi-frame session stress tests, live BearDog + primalSpring integration.

### Tor Onion Service — Security Provider Crypto (DEFERRED — blocked on external security provider)

HSDir descriptor superencryption, `ESTABLISH_INTRO` HMAC/signature, `INTRODUCE1`/`INTRODUCE2` ntor payloads, and rendezvous auth keys delegate to security provider JSON-RPC. Stub sections documented inline with `// BLOCKED:` and return `Error::CryptoUnavailable`. Not a glacial shift blocker — requires live BearDog crypto IPC surface with Ed25519/X25519 key operations available at runtime.

### TLS / Sovereign Onion (requires live security provider)

- `ed25519_public_from_secret` via security provider
- Security-provider-generated lineage-tagged certificates
- CertificateVerify signing via security provider
- Custom TLS extension building via security provider

---

## Pending: Coverage Expansion (73.41% → 90% target)

**Note (May 27, 2026)**: 73.41% measured via llvm-cov `--workspace --lib` (Apr 27 2026). Wave 53 coverage push added **+74 tests** across 5 modules in 3 crates: `primal_discovery` (+21 capability/parse/tcp tests), `graph/coordination` (+22 scheduler topology/analysis tests), `app/network` (+8 bind-address edge-case tests), `runtime_discovery/engine` (+13 cache/discovery tests), `capability_endpoints/resolver` (+8 override/cache tests), `tower_atomic/types` (+16 JSON-RPC wire-type serde tests). Prior: Wave 179 added 92 tests across 15+ files in 8 crates. Wave 171 added 271 tests (71.28% → 73.41%); Wave 174 added 18; Wave 177 added 9. Wave 180 added 19. Target 90% via ongoing coverage expansion of I/O-heavy modules with mock infrastructure.

**Wave 171 coverage push**: 271 tests added across 30+ files in 13 crates (see CHANGELOG). All previously-tracked low-coverage modules in Waves 124-127 (adapters, STUN client, HTTP handler, tower_atomic) now have comprehensive tests. Wave 171 targeted 0%-coverage pure-logic modules across songbird-orchestrator, songbird-config, songbird-discovery, songbird-universal, songbird-network-federation, songbird-http-client, songbird-lineage-relay, songbird-cli, and 5 smaller crates.

**Remaining gap** (73.41% → 90%): I/O-heavy code paths (live socket, network, filesystem), large dispatcher modules (`server/mod.rs`, `app/http_server.rs`, `ipc/connection.rs`), and integration-style orchestrator paths. Further progress requires deeper mock infrastructure or E2E integration test expansion.

---

## Pending: Platform & Infrastructure

- [ ] Platform NFC backends (Android JNI, iOS CoreNFC, Linux libnfc)
- [ ] Real hardware IGD test (Tower + Pixel 8a)
- [ ] Genesis physical channels: Bluetooth GATT/L2CAP, QR code, SoloKey (FIDO2)
- [ ] iOS XPC transport
- [ ] WASM primal registry + tokio/mio WASM support
- [ ] Android IPC: configurable fallback bind address
- [ ] USB bulk endpoint streaming
- [ ] DNS SRV integration for capability discovery

---

## Pending: Architectural Evolution

- [x] **IPC registration identity verification** (primalSpring Phase 55 audit, Wave 191): `ipc.register` now probes the registering primal's endpoint with `identity.get` and hard-rejects identity mismatch (spoofed names). Gracefully degrades to trust-on-first-use if endpoint is unreachable (handles primals still starting). Full BTSP handshake verification remains as a future hardening (requires BearDog availability at registration time)
- [ ] **Purpose key derivation for discovery signing**: two-tier crypto model describes a `discovery` purpose key; currently using BearDog's Ed25519 signing key directly — purpose key derivation is a BearDog-side evolution
- [x] **Mesh partition tolerance (Stream 6)** (Wave 113): Cross-gate reachability gossip via `mesh.capabilities_announce` — each gate reports `reachable_peers` and `version` fields. `partition_status_for()` computes per-peer partition status (Healthy / PartialPartition / LocallyUnreachable). `mesh.status` and `mesh.health_check` surface `partition_warnings` and `partition_detected` flags. `mesh.peers` enriched with `version` and `version_mismatch` per peer. 8 new tests covering all partition states
- [x] **Peer version negotiation (Stream 6)** (Wave 113): Version extracted from `health.ping` response during TCP probes (every 30s) and stored in `PeerMetadata`. `mesh.capabilities_announce` payload includes `version` field. `mesh.status` reports `version_skew` array when peers run different versions. `mesh.peers` flags `version_mismatch: true` per peer. UDP discovery already sends version — backward-compatible (receivers that don't understand the field ignore it gracefully)
- [ ] Cluster support for anonymous beacon broadcasting
- [ ] TLS handshake v2 module integration
- [ ] IPC native endpoint lifecycle management

---

## Pending: Dependency Evolution

- [x] `ring-crypto` feature removed (Wave 135, SB-02 resolved): `rustls_rustcrypto` is the sole TLS provider. `ring` is NOT compiled in any build config (`cargo tree -i ring` = empty). Cargo.lock stanza persists because `rustls-webpki` (0.102 + 0.103) declares `ring` as an optional dep — Cargo's resolver locks optional dep versions by design. Investigated git `rustls-rustcrypto` (drops webpki 0.102) but pre-release RustCrypto crates are incompatible with stable workspace. Blocked on upstream `rustls-rustcrypto` crates.io release. See `deny.toml` for full stadial gate analysis.
- [x] `hickory-resolver` 0.24 → 0.25 (Wave 70): Migrated 4 crates (`songbird-config`, `songbird-cli`, `songbird-discovery`, `songbird-universal`) from deprecated 0.24 to 0.25. Constructor `TokioAsyncResolver::tokio(config, opts)` replaced with `Resolver::builder_with_config(config, TokioConnectionProvider::default()).build()`. Type alias `TokioAsyncResolver` → `TokioResolver`. Feature `tokio-runtime` → `tokio`. All 9 DNS-SD tests + 938 songbird-config tests pass. 0.26 deferred — breaking changes to `SrvLookup`/`TxtLookup` (types removed, fields replace accessors) require full iteration rewrite when upstream stabilizes.
- [ ] Remaining transitive duplicates (hashbrown ×5, getrandom ×3, socket2 ×2, rand ×3, indexmap ×2, generic-array ×2, cpufeatures ×2) require upstream version unification; `futures 0.3` only transitive via `tarpc` — no direct deps remain
- [x] `serde_yaml` → `serde_yaml_ng` (Wave 165): `songbird-config` and `songbird-discovery` migrated from archived `serde_yaml 0.9` to `serde_yaml_ng 0.10` (maintained fork, aliased as `serde_yaml` — zero call-site changes). `kube-client` transitive dep keeps original `serde_yaml` in Cargo.lock — full lockfile removal blocked on kube upstream
- [ ] `bincode` 1.x (RUSTSEC-2025-0141): transitive via tarpc/tokio-serde — not directly actionable, blocked on tarpc upstream codec migration. `bincode` is only used as the serialization format for `tarpc` RPC transport (songbird-orchestrator, songbird-universal). Zero direct production usage outside tarpc protocol
- [x] Dead `full-discovery` feature flag removed from `songbird-universal` (Wave 208): was never referenced in code; individual backend features (`mdns`, `dns-sd`, `k8s`, `docker`) remain properly gated
- [x] `async-trait` **fully eliminated** (Wave 145): 141→0 annotations, dependency removed from all crates and workspace `Cargo.toml`. Every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. SB-06 resolved.
- [x] `dyn` dispatch evolution (Wave 155, audited Wave 159): 6 production `dyn` sites eliminated. Remaining architectural: `Pin<Box<dyn Stream>>` (discovery watch trait — `CanonicalDiscovery::watch_services`, delegation, 6+ adapter impls; all backends return empty streams; evolve to enum dispatch when real streams exist), `Box<dyn SerialPort>` (external `serialport` crate API), `Arc<dyn Fn(&str) -> Result<String, VarError>>` (env reader injection in `capability_discovery`/`runtime_engine` — intentional for testability), `&dyn Any` (`as_any()` in discovery factory/backends — standard `Any` downcast pattern, architectural)
- [x] `rand` removed from `songbird-orchestrator` production deps (Wave 140): JWT CSPRNG replaced with `getrandom::fill()`; `rand` retained as dev-dependency for tests
- [x] Dead `sled` dependency removed from `songbird-tor-protocol` (Wave 116)
- [x] `ed25519-dalek` in `songbird-quic` feature-gated behind `local-certs` (Wave 116)
- [x] Port constants consolidated (Wave 130 initial → Wave 161 completed): all 12 deprecated port constants removed from `songbird-types::constants` (zero consumers remained). `DEFAULT_PORT` retained as alias to `defaults::ports::DEFAULT_HTTP_PORT`. `DEFAULT_BIRDSONG_PORT` (42424), `DEFAULT_QUIC_PORT` (4433) added to centralized `defaults::ports`. Duplicate STUN constants in `songbird-universal-ipc` and `songbird-onion-relay` replaced with centralized imports. `songbird-config::constants::network` port/host definitions replaced with re-exports from canonical source. `DEFAULT_ORCHESTRATOR_PORT` aligned to 8080 (matching actual HTTP API bind port). Wave 161: 15+ scattered `.unwrap_or(8080)` in 8 crates replaced with canonical `songbird_types::defaults::ports` constants; `songbird-config::defaults::ports` functions now fall back to typed constants instead of magic numbers; duplicate `DEFAULT_BIND_ADDRESS` in `network.rs` replaced with re-export
- [x] `hostname` crate consolidated to `gethostname` (Wave 161/165): Wave 161 removed `hostname` from `songbird-config` (replaced `hostname::get()` with `gethostname::gethostname()`). Wave 165 expanded to `songbird-discovery`, `songbird-orchestrator`, `songbird-cli`, `songbird-types`, `songbird-compute-bridge` — zero `hostname` direct deps remain
- [x] Workspace dependency hoisting (Wave 112): `sha2`, `hmac`, `hex`, `base64`, `gethostname`, `hyper-util` — 6 crates consolidated from inline versions to `[workspace.dependencies]`. Eliminates 13 crate-local version specs, ensures single source-of-truth for version pins. `songbird-discovery`, `songbird-genesis`, `songbird-http-client`, `songbird-network-federation`, `songbird-orchestrator`, `songbird-sovereign-onion`, `songbird-stun`, `songbird-tls`, `songbird-cli`, `songbird-compute-bridge`, `songbird-config`, `songbird-types`, `songbird-universal-ipc`, `songbird-tor-protocol` updated
- [x] `futures` facade → `futures-util` (Wave 161/165/209): Wave 161 removed unused `futures` from `songbird-bluetooth`/`songbird-lineage-relay` and migrated `songbird-stun`/`songbird-orchestrator`/`songbird-universal-ipc` to `futures-util`. Wave 165 migrated `songbird-universal` (`futures 0.3` → `futures-util { workspace = true }`, 13 `block_on` tests converted to `#[tokio::test]`). Wave 209: fixed 7 remaining integration test files still using `futures::future::join_all` → `futures_util::future::join_all`. Zero direct `futures` facade deps remain, integration tests compile clean
- [x] `Box<dyn std::error::Error>` eliminated from entire workspace (Waves 161/188): production `main()` → `anyhow::Result<()>` (Wave 161); test-utils concurrent_helpers + 4 mock providers + capability_mocks + chaos test → `anyhow::Result` (Wave 188). Zero `Box<dyn Error>` anywhere in workspace

---

## Sovereignty: NAT Traversal (H2-13 — H2-16, Waves 196–197)

Step 3c on the sovereignty critical path — replacing `cloudflared` tunnels.

- [x] **H2-13**: STUN client production hardening — shared-socket dual-probe for correct NAT type detection (`discover_on_socket()`, `classify_nat_from_dual_probes()`, `discover_public_endpoint_multi()`); fixed `stun_handler/client.rs` and `onion-relay/stun.rs` callers; 10+ new tests. **Wire compliance** (Wave 197): MESSAGE-INTEGRITY (HMAC-SHA1), FINGERPRINT (CRC32 XOR 0x5354554E), IPv6 XOR-MAPPED-ADDRESS encoding/decoding per RFC 5389 §15.2
- [x] **H2-14**: RFC 5766 TURN client — `TurnClient` with `allocate()`, `refresh()`, `create_permission()`, `channel_bind()`; `MessageType` extended with TURN method variants (Allocate/Refresh/CreatePermission/ChannelBind + success/error); XOR-PEER-ADDRESS encoding; `StunCredentials.key` wired for BearDog beacon-tier auth (JH-11 resolved). TURN tier live in `ConnectionFallbackChain` via `with_turn_client()`
- [x] **H2-15**: Cloudflare DDNS provider — `CloudflareDdnsProvider` implements `DdnsProvider` trait via `HttpExecutor` callback (zero HTTP-stack dep in STUN crate); list/upsert A/AAAA records via CF API v4; `from_env()` reads `SONGBIRD_CF_API_TOKEN`/`SONGBIRD_CF_ZONE_ID`; 6 mock-driven tests. `NoopDdnsProvider` + `DdnsConfig` + env vars (`SONGBIRD_DDNS_*`) remain for disabled/test configurations
- [x] **H2-16**: Connection fallback chain fully wired — all 5 tiers live in `MultiTierCoordinator::establish_connection()`:
  1. Direct UDP hole-punch (`try_direct_punch`)
  2. STUN-assisted punch (`try_stun_punch` → `discover_public_address`)
  3. Lineage relay (`with_relay_discovery()` → `RelayDiscovery::request_relay()`)
  4. TURN relay (`with_turn_client()` → `TurnClient::allocate()`)
  5. Emergency tunnel (`try_emergency_tunnel` — probes `cloudflared` binary on `$PATH`)

- [x] **VPS Relay Server** (Wave 199, Pass 12): `TurnRelayServer` in `songbird-stun` — RFC 5766 relay for sovereign NAT traversal. Handles Allocate/Refresh/CreatePermission/ChannelBind; `CredentialStore` trait for BearDog beacon-tier auth; per-allocation relay socket with permission-gated forwarding; periodic cleanup of expired allocations; `TurnRelayStats` telemetry; STUN Binding compatibility. Full client↔server integration tests (allocate, refresh, binding, auth rejection).

**Shadow Run Readiness (Waves 210–213 — S2 Parity Proofs)**:
- [x] `TurnRelayStats` periodic structured emission (60s interval, tracing structured fields)
- [x] `TurnSessionConfig::from_env(peer_addr)` — `SONGBIRD_TURN_SERVER`/`USERNAME`/`KEY` env resolution
- [x] `LineageRelayCoordinator::probe_turn_relay(peer_addr)` — setup time + allocated address measurement
- [x] `LineageRelayCoordinator::turn_relay_configured()` — env-gated availability check
- [x] Full data-plane integration (`TurnRelayedConnection` + `ConnectionSession::TurnRelayed` variant, keepalive loop)
- [x] Shadow dual-path comparator (`shadow_comparator::compare_paths` — TURN vs cloudflared parallel metric emission)
- [x] Cross-gate integration test (`capability.call` local dispatch via UDS, routing=local error path)
- [x] TURN allocation refresh lifecycle (`TurnSession::spawn_keepalive` — 80% lifetime keepalive loop)
- [x] `cloudflared` tunnel orchestration (`CloudflaredTunnel::spawn` — process spawn, URL parse, kill-on-drop)

**Remaining hardening**:
- [x] Full `cloudflared` tunnel orchestration (spawn process, parse assigned URL, monitor lifecycle) — Wave 213
- [ ] RFC 2136 DDNS provider (`nsupdate` wire protocol)
- [x] TURN allocation refresh lifecycle (keepalive loop) — Wave 213
- [ ] BearDog cross-host key distribution for TURN credential derivation
- [ ] Integration tests with live STUN/TURN servers
- [ ] VPS deployment orchestration (systemd unit, monitoring, TLS wrapping)

---

## Composition Audit (primalSpring Full Stadial Gate — May 12, 2026)

Songbird is **CLEAN** in the 13/13 primal composition audit. Pass 12 and Pass 14 items addressed:

**Structural gate**: MethodGate, BTSP Phase 3, Edition 2024, deny.toml, plasmidBin — all cleared.

**Pass 12 (Sentinel Escalation) — RESOLVED**: VPS relay server (`TurnRelayServer`) implemented in `songbird-stun`. RFC 5766-compliant relay for sovereign NAT traversal. The chain Songbird VPS relay → NAT shadow run → NestGate extracellular → content sovereignty is now code-complete. Deployment (systemd, TLS, monitoring) is stadial ops work.

**Pass 14 (Convergence) — RESOLVED**: `capability.resolve` response enriched with `primal_name` field. Both transport paths (universal-ipc via `primal_id` and orchestrator via `primal_name`) now return the owning primal identity. Name-based discovery is fully operational for downstream springs.

**Composition readiness**: Discovery works across all transport paths. NAT traversal (STUN, TURN, DDNS, 5-tier fallback chain + VPS relay server) shipped. No transport routing gaps.

**Songbird in the ecosystem dependency chain**: Songbird is not on the critical path for any other primal. No primal is blocked on Songbird. The dependency chains that matter are:
- NestGate `content.*` transport parity → petalTongue, projectNUCLEUS Pillars 1-3, sovereign content pipeline
- skunkBat Phase 3 (JH-5) → rhizoCrypt, sweetGrass, NFT pipeline
- loamSpine API contract → RootPulse Phase 5
- bearDog crypto IPC surface → barraCuda crypto dedup
- squirrel compute delegation → toadStool IPC

**Stadial target** (ops, not code): VPS relay deployment for NestGate extracellular serving. Code is complete — deployment and shadow run validation remain as L4 absorption work.

---

## Stadial Readiness — Composition Documentation (Wave 207, May 17, 2026)

### Method Stability Tiers

All 49 registered methods are annotated by stability:

| Tier | Methods | Description |
|------|---------|-------------|
| **Stable** | `health.*`, `identity.get`, `capabilities.list`, `capabilities.methods`, `primal.info`, `primal.capabilities`, `primal.announce`, `ipc.register`, `ipc.resolve`, `ipc.discover`, `ipc.list`, `capability.resolve`, `http.request`, `http.get`, `http.post`, `auth.check`, `auth.mode`, `auth.peer_info`, `btsp.capabilities` | Wire-format frozen. Breaking changes require major version bump. |
| **Operational** | `stun.*` (7), `igd.*` (5), `relay.*` (4), `mesh.*` (8 incl. `probe_latency`), `punch.*` (3), `discovery.*` (2), `federation.*` (2), `onion.*` (5), `tor.*` (6) | Semantics stable, response shapes may gain fields. No field removal without deprecation. |
| **Introspection** | `rpc.methods`, `rpc.discover`, `discover_capabilities`, `lifecycle.*`, `btsp.negotiate` | Internal self-description; may evolve freely between minor versions. |
| **Inference (passthrough)** | `inference.*` (4) | Routed to downstream AI provider; Songbird is pass-through only. |

### Degradation Behavior (when Songbird is unavailable)

| Consumer | Impact | Mitigation |
|----------|--------|------------|
| **All primals** | No `capability.resolve` — primals cannot discover each other by capability | Primals fall back to direct UDS paths (`$XDG_RUNTIME_DIR/biomeos/<primal>.sock`) or env vars |
| **biomeOS** | Cannot register services via `ipc.register` or query mesh topology | biomeOS continues operating with last-known registry state; re-discovers on reconnect |
| **Springs** | No HTTP proxy (`http.request`) for external API calls | Springs with direct `reqwest`/`ureq` deps continue; those relying on Tower Atomic HTTP are blocked |
| **lithoSpore** | TURN relay unavailable for geo-delocalized validation | Falls back to direct connectivity or LAN-only validation |
| **cellMembrane** | No NAT traversal, no mesh discovery for multi-gate deployments | Single-gate operation continues; multi-gate requires Songbird |

### Downstream Pairing (stadial)

| Downstream | Capability Used | Pairing Mode |
|------------|-----------------|--------------|
| **cellMembrane / projectNUCLEUS** | TURN relay, mesh discovery, NAT traversal | Tower Atomic composition (bearDog + songbird + skunkBat) |
| **All gates** | `mesh.init`, `mesh.peers`, `mesh.announce` — multi-gate topology | Live mesh on canonical UDS |
| **lithoSpore** | `songbird-turn-client` crate for TURN-relayed JSON-RPC (UB-1) | Library linkage (no IPC) |
| **All springs** | `http.request` / `http.get` / `http.post` via Tower Atomic | IPC delegation |
| **esotericWebb** | mesh + federation for game networking | IPC + composition graph |

### Cross-Gate Dispatch (Phase 2 — CG-8 RESOLVED Wave 211, TURN Relay Wave 38)

- [x] **`capability.call` JSON-RPC handler** (Wave 211): Songbird now receives `capability.call`,
  resolves the capability to a local provider (UDS forward) or remote gate (mesh peer TCP
  forward), and returns the response. Prevents infinite forwarding via `routing: "local"` on
  remote hops. Wired into dispatch table, advertised in `rpc.methods`/`rpc.discover`.
- [ ] **biomeOS integration**: biomeOS needs to route `capability.call` to Songbird's UDS
  when the target capability is on a remote gate (biomeOS-side wiring, not songbird code).
- [x] **Capability advertisement in mesh** (Wave 38): `peer_has_capability()` probes remote
  peers via `capabilities.list` before dispatching. Checks both flat `capabilities` array
  and structured `provided_capabilities` (type field). Skips peers that lack the target
  capability, eliminating blind fan-out.
- [x] **TURN/relay transport for cross-gate** (Wave 38): `forward_to_remote_via_turn()` allocates
  a TURN relay session via `SONGBIRD_TURN_*` env vars when direct TCP fails. JSON-RPC bytes
  flow through the sovereign TURN relay (cellMembrane VPS). Two-phase dispatch: TCP direct
  first, TURN fallback second. Handles CGNAT, double-NAT, symmetric NAT scenarios.
- [x] **NAT field test harness** (Wave 38): `nat_field_test` module in `songbird-lineage-relay`
  with `probe_turn_path()` and `run_field_test_matrix()`. Tests TURN allocation for CGNAT,
  double-NAT, and symmetric NAT peer addresses. Live tests gated behind `--ignored` flag.
- [x] **`mesh.init` bootstrap peers** (Wave 49): `bootstrap_peers` param wired — accepts
  `[{node_id, address}]` TCP peer entries and adds them as `Direct` endpoints at init time.
  Enables cross-gate discovery without relying on onion addresses. `--security-socket` CLI
  flag added to orchestrator binary (feeds `SECURITY_PROVIDER_ENDPOINT` via `songbird_process_env`).
- [x] **`SONGBIRD_PEERS` auto-seeding** (Wave 51): Mesh peers auto-initialized from env var on
  startup. Supports both `node_id@host:port` and bare `host:port` formats. `discovery.peers`
  immediately populated on boot without requiring external `mesh.init` RPC. `mesh_seed` module
  with robust parsing + 7 tests. Resolves the glacial shift blocker (primalSpring Wave 51).
- [x] **`discovery.peers` mesh bridge** (Wave 51): Orchestrator dispatch wired to merge mesh
  peers + registry services. De-duplicates by node_id. Cross-gate health probes work:
  gate A calls `discovery.peers` on gate B and gets B's local primals.
- [x] **`ipc_registry.rs` refactoring** (Wave 51): Extracted `remote_dispatch.rs` (cross-gate
  TCP/TURN routing) — 906L → 614L. `EndpointType::socket_addr()` added for port-preserving
  peer addressing (eliminates hardcoded DEFAULT_HTTP_PORT fallback).

---

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) — Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration
- [ ] Tor consensus microdescriptor parsing (ntor_key, version fields)
- [ ] Tor HSDir descriptor upload

---

## Tracked Partial Solutions as Debt (Wave 96 Audit)

### P0 — Security-Critical Stubs

| Location | Issue | Resolution |
|----------|-------|------------|
| `orchestrator/src/access_control/auth.rs:294–366` | PostgreSQL/SQLite/Redis auth accepts any non-empty credential | Wire real DB verification or delegate to security provider |
| `orchestrator/src/access_control/auth.rs:411–435` | TOTP accepts any 6-digit code when secret set | Wire `totp-rs` or RFC 6238 via security provider |
| `orchestrator/src/trust/escalation.rs:347–354` | Hardware attestation always returns Err | Wire security provider `verify_attestation` |
| `tls/src/cert/generator.rs:272–278` | X.509 DER has 64 zero-byte signature placeholder | External signing step or provider-delegated |
| `tls/src/cert/mod.rs:61–82` | Chain validation is "non-empty DER" only | Full PKIX path validation |
| Tor onion (8 sites in `songbird-tor-protocol`) | Zero-filled crypto in ESTABLISH_INTRO, descriptors, rendezvous | BLOCKED on security provider Ed25519/X25519 |

### P1 — Core Connectivity Stubs

| Location | Issue | Resolution |
|----------|-------|------------|
| `orchestrator/src/connections/*_btsp.rs` | Bidirectional BTSP tunnel RPC returns `not_implemented` | BtspClient data-plane evolution |
| `orchestrator/src/bin_interface/server.rs:252–255` | Windows: no IPC server spawned | Named pipe or TCP parity |
| `orchestrator/src/app/core/mod.rs:466–483` | Non-Unix: IPC returns hard error | TCP or named pipe IPC |
| `orchestrator/src/network/binding.rs:345–347` | Non-Linux: interface detection bails | Platform-specific netdev |
| ~~`orchestrator/src/server/protocol_api.rs:125–126`~~ | ~~WebSocket not advertised~~ | **RESOLVED** Wave 103+106: fully wired in `AvailableProtocols`, endpoint URL corrected |
| `orchestrator/src/http_gateway/universal_proxy.rs:210–247` | Template transformation logs "not yet implemented" | Implement Tera/Handlebars template engine |

### P1 — Discovery & Registration Stubs

| Location | Issue | Resolution |
|----------|-------|------------|
| ~~`discovery/src/abstraction/adapters/kubernetes_adapter.rs:184`~~ | ~~`discover` returns `Ok(vec![])`~~ | **RESOLVED** Wave 102: real K8s API query + JSON parse wired |
| ~~`discovery/src/abstraction/adapters/consul_adapter.rs:310–387`~~ | ~~Register/unregister/watch/list_all return Err~~ | **RESOLVED** Wave 102/103: Consul HTTP API wired (register, unregister, list_all, discover) |
| ~~`config/src/defaults/service_locator.rs:209–272`~~ | ~~`discover_from_registry_with` / `register_with_*` stubs~~ | **RESOLVED** Wave 103: real UDS JSON-RPC to `ipc.resolve` / `ipc.register` |
| `config/src/discoverable_endpoint.rs:318–336` | Consul + DNS-SD discovery → `not_implemented` | Consul wired Wave 103; DNS-SD deferred (requires external responder) |
| ~~`network-federation/src/network/discovery.rs:37–40`~~ | ~~`discover_nodes` → `not_implemented`~~ | **RESOLVED** Wave 103/106: env-based + persisted peers loading, `node_id@host:port` format, dedup |

### P2 — Platform Backends

| Location | Issue | Resolution |
|----------|-------|------------|
| `songbird-nfc/src/platform.rs` | All backends return `PlatformUnsupported` | JNI/CoreNFC/libnfc |
| `universal-ipc/src/platform/ios.rs:106–197` | iOS XPC stub | XPC bridge or TCP fallback |
| `universal-ipc/src/platform/wasm.rs` | WASM listen returns error | Global in-process registry |
| `songbird-bluetooth/src/gatt/characteristics.rs:142–169` | GATT discover builds request but doesn't send | L2CAP transport wiring |
| `genesis/src/physical_channels/bluetooth_pure.rs:138–181` | Returns hardcoded demo credentials | Real BLE credential exchange |
| `genesis/src/physical_channels/qr_code.rs:37–64` | Feature `qr` but methods return Err | Wire camera/encoder dep |
| `genesis/src/physical_channels/solokey.rs:47–72` | FIDO2 shell only | CTAP2 implementation |

### P2 — Deployment & NUCLEUS

| Location | Issue | Resolution |
|----------|-------|------------|
| `deployment/genome/genome_wrapper_template.sh` | `--mode` parsed but never executes service registration | Wire systemctl/Windows Service |
| `.cargo/config.toml` | ~~No Android NDK linker~~ DONE; no macOS/RISC-V targets | Add remaining cross-compile entries |
| `remote-deploy/src/http_deploy/mod.rs:87–92` | Streaming upload not implemented | Chunked streaming path |
| `remote-deploy/src/http_deploy/chunked.rs` | Sequential chunks, no hash verify, no compression | Parallel + integrity |
| ~~`orchestrator/src/server/deployment_api/capabilities.rs:94`~~ | ~~Network type always "lan"~~ | **RESOLVED** Wave 106: env/VPN/public-IP heuristics + peer-aware subnet analysis |
| genomeBin format | `GENOMEBIN_FORMAT=1.0`, single primal only | v3.0 multi-primal NUCLEUS capsule |

---

## projectNUCLEUS Seeding — Roadmap

### Tier 1 — Unblock NUCLEUS on existing targets
- [ ] Wire genome `--mode systemd` to `systemctl enable --now`
- [x] Implement real `detect_network_type()` in deployment capabilities (Wave 106: env override, VPN interface scan, /proc/net/fib_trie public IP, + peer-aware `detect_network_type_for_peer()` with subnet analysis)
- [ ] Parallel chunked upload + chunk SHA-256 verification
- [ ] RISC-V target (`riscv64gc-unknown-linux-musl`) added to genomeBin

### Tier 2 — Multi-primal NUCLEUS capsule
- [ ] `GENOMEBIN_FORMAT=3.0` multi-primal genome (beardog + songbird + skunkBat slices)
- [ ] Signed `GENOME_METADATA` (BearDog-signed manifest)
- [ ] Per-arch thin downloads vs fat "spore" bundle
- [ ] Composition env pre-wiring (`FAMILY_ID`, security socket, peers, TURN)

### Tier 3 — Cross-hardware deployment
- [ ] macOS: `aarch64-apple-darwin` / `x86_64-apple-darwin` targets
- [ ] Windows: native Service wrapper (Phase 2)
- [x] Android NDK linker committed to `.cargo/config.toml` (aarch64-linux-android21-clang)
- [ ] Adaptive deployment spec completion (parallel chunks, compression, streaming)

---

## Priority Order

1. **Security provider crypto e2e validation** — Tor/TLS items need live security provider
2. **NUCLEUS seeding** — genomeBin v3, multi-arch, multi-primal capsules
3. **Coverage expansion** — Target pure-logic modules first (goal: 90%)
4. **Deep documentation** — Fill internal modules with full doc coverage
5. **Real hardware tests** (Tower + Pixel) — Validates cross-network
6. **Platform backends** — Mobile pairing, iOS, WASM
7. **Dependency pruning** — Reduce unique deps where possible
