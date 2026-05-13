# Songbird Remaining Work

**Date**: May 13, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: Wave 204 (May 13, 2026)  
**Composition Audit Status**: **CLEAR THROUGH STADIAL GATE** — GAP-16 (Tower atomic: mesh.* on canonical UDS) resolved. primalSpring Ecosystem Wave Sync (May 12, 2026) confirms Pass 12 and Pass 14 fully resolved. Songbird is no longer a sentinel blocker. Tower atomic validation by ludoSpring is now unblocked.  
**Current Wave**: 204 — GAP-16 Tower Atomic: `mesh.*` methods wired into orchestrator UDS (`songbird.sock` / `network.sock`); `IpcHandlers::mesh_dispatch()` adapter; 15 new tests covering the full Tower atomic validation path. Wave 203 — Deep debt: `turn_server.rs` (1027L → 898L via `#[path]` test extraction); audit confirms zero production hardcoding, mocks properly isolated, pure Rust deps. Wave 202 — VPS relay ops deployment: credential loading (file/env), `songbird-relay.service` systemd unit, deployment guide (`deployment/relay/README.md`). Wave 201 — Pass 12/14 sentinel resolution: bidirectional TURN data plane (Send Indication, ChannelData, Data Indication); `songbird relay` CLI subcommand (VPS relay startup path); `capability.resolve` wire-format parity (orchestrator now emits `socket`, `native_endpoint`, `virtual_endpoint` matching universal-ipc). Wave 200 — Deep debt cleanup & evolution: `method_gate.rs` (944L monolith) → directory module (6 files, 45 tests); `BearDogVerifier` stub → live IPC (`SecurityRpcClient::verify_ionic`); 7× hardcoded `"0.0.0.0:0"` → `EPHEMERAL_BIND_ADDR`; dependency audit (hickory-proto RUSTSEC-2026-0119 tracked, ring ban verified, cargo-deny clean); `examples/future/` stale examples removed; docs reconciled (README CI/deny claim, largest-file metric, STUN spec status, specs index date, wateringHole handoff inventory). Wave 199 — Pass 12: `TurnRelayServer` (RFC 5766 sovereign VPS relay) with `CredentialStore` trait, allocation lifecycle, permission-gated data forwarding, and full client↔server integration tests. Pass 14: `capability.resolve` response enriched with `primal_name` field on orchestrator path (parity with universal-ipc `primal_id`). Wave 198: Record clean composition audit. Wave 197: Sovereignty NAT traversal completion (H2-13 through H2-16 Step 3c). STUN wire compliance: MESSAGE-INTEGRITY (HMAC-SHA1), FINGERPRINT (CRC32 XOR 0x5354554E), IPv6 XOR-MAPPED-ADDRESS encoding/decoding (RFC 5389 §15.2 — XOR with magic_cookie||transaction_id); `encode_authenticated()` for credential-bearing requests. RFC 5766 TURN client: `TurnClient` with `allocate()`, `refresh()`, `create_permission()`, `channel_bind()`; `MessageType` extended with TURN method variants; XOR-PEER-ADDRESS encoding. Cloudflare DDNS provider: `CloudflareDdnsProvider` with `HttpExecutor` callback pattern (dep-free); list/upsert via CF API v4; 6 mock-driven tests. Lineage relay injection: `MultiTierCoordinator` now accepts `with_relay_discovery()` + `with_turn_client()`; Tier 3 (lineage relay) calls `RelayDiscovery::request_relay()`; Tier 4 (TURN) calls `TurnClient::allocate()`. `cloudflared` emergency tunnel: Tier 5 probes for `cloudflared` binary availability. Wave 196: H2-13 shared-socket dual-probe NAT type fix, H2-14/15/16 scaffolding. 0 clippy warnings, 3804 tests across 4 affected crates.  
**Previous Waves** (full detail in `CHANGELOG.md`): 191 (ipc.register identity verification, whitespace-tolerant protocol detection, BufReader safety), 190 (IP literals, parse_endpoint IPv6, redundant clone, test Duration constants), 189 (ipc.resolve `socket` field for primalSpring tier-1 discovery), 188 (15 timeout constants, JSONRPC_VERSION, Box<dyn Error> elimination), 187 (smart refactor connection.rs, primal-name evolution, 4 timeout constants), 186 (BTSP Phase 3 live connection verification — 4 tests), 185 (deep debt: 11 timeout constants, JSON-RPC constructors, primal codename evolution), 184 (BTSP Phase 3 binary-framed dispatch fix), 183 (deep debt: lint evolution, timeout centralization, hardcoded elimination), 182 (BTSP Phase 3 spec alignment), 181 (port canonicalization), 180 (BTSP Phase 3 btsp.negotiate), 175 (PG-51 verified, ENVIRONMENT_VARIABLES.md), 174 (hardcoded IP/port elimination, flaky tests, dep cleanup, +18 tests), 173 (PG-51 socket discovery), 172 (root doc reconciliation), 171 (test coverage expansion 71.28%→73.41%, +271 tests), 170 (CLI flag alignment), 169 (remaining `new()` → `new_direct()` in bin_interface), 168 (BTSP routing + seed encoding), 167 (BTSP error frames, env fallbacks), 166 (root doc reconciliation), 165 (dep cleanup, hardcoded elimination, dead code removal), 162 (stream.shutdown BTSP fix), 161 (port centralization, dep cleanup, error typing), 160 (BTSP NDJSON auto-detect), 158 (BTSP Step 3→4 verification relay), 157 (hardcoded literals, dead deps, doc cleanup, debris removal), 154 (mock isolation, dead deps, lint hygiene), 153 (BTSP NDJSON wire-format alignment), 152 (dead deps, hardcoding, test hygiene), 151 (PG-37 capability-first routing), 150 (doc cleanup, debris removal), 149 (comprehensive deep debt: blanket lint removal, hardcoded paths, duplicate constants, mock features, stale CLI, expect safety), 148 (PG-21 persistent NDJSON sessions), 147 (mock isolation, hardcoded IP/path elimination, lint hygiene), 146 (stadial dyn audit + ring analysis), 139b (deep literal sweep), 139 (self-healing auto-discovery), 138b (hardcoded literal evolution), 138 (LD-08 socket auto-discovery), 137b-c (ipc.resolve dual-mode, stale features, port canonicalization, lint hygiene), 137 (capability naming), 136 (constant consolidation), 135 (SB-02/SB-03 resolved), 134 (primalSpring gaps), 133 (smart refactor), 132 (BTSP Phase 2), 131-119 (hardcoding, legacy scrub, coverage)

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 7,803 lib passed, 0 failures, 22 ignored |
| **Line Coverage** | **73.41%** measured (llvm-cov `--workspace --lib`, Apr 27 2026; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 30 crates compile clean (~43s dev) |
| **Clippy Pedantic** | 30/30 crates clean — zero warnings (`clippy::pedantic + nursery`, `-D warnings`, May 6 verified) |
| **Format** | Clean (`cargo fmt --check` passes; May 6 verified) |
| **Docs** | Clean (`cargo doc --workspace --no-deps` — 0 warnings) |
| **Files >800 lines** | 1 deferred (`bin_interface/server.rs` 878L — tightly coupled startup); `turn_server.rs` resolved Wave 203 (1027L → 898L via `#[path]` test extraction); largest non-deferred: `primal_discovery.rs` 763L; Wave 200: `method_gate.rs` (944L) → directory module; Wave 176: `information_layers.rs` (1121L) → directory module; Wave 133: 4 former >700L files refactored |
| **Unsafe blocks** | **0** — `forbid(unsafe_code)` on all 30 crates |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 unguarded — `.unwrap()` in production only under `#[expect(clippy::unwrap_used, reason = "...")]` for provably infallible operations (e.g. `write!` to `String`); all others in `#[cfg(test)]` or doc examples |
| **Production `panic!()`** | 0 |
| **Production `unreachable!()`** | 2 (provably unreachable QUIC VarInt 2-bit prefix arms, documented) |
| **TODO/FIXME/HACK comments** | 0 in Rust source; 0 FIXME/HACK |
| **Commented-out code** | 0 in production library code (Wave 124 scrub); doc-style examples in comments kept intentionally |
| **`#[allow(` vs `#[expect(`** | Wave 134 completed full `#[expect(dead_code)]` → `#[allow(dead_code)]` migration across all 30 crates (45+ attributes in 30 files); Wave 136: all generic `"reserved or API surface"` reason strings replaced with specific contextual reasons; Wave 137c: all bare `#[allow()]` in test files and TLS crate given reason strings; Wave 147: all remaining bare `#[allow(clippy::type_complexity)]`, `#[allow(clippy::too_many_lines)]`, `#[allow(unused_mut)]`, `#[allow(deprecated)]` given reason strings (6 e2e test files + 4 production); Wave 154: blanket `#![cfg_attr(test, allow(...))]` in universal-ipc and lineage-relay given reason strings; `#[expect(reason)]` retained where non-dead-code lints provably fire; zero reasonless suppressions remain |
| **Mocks in production** | 0 (all inside `#[cfg(test)]` or `#[cfg(any(test, feature = "test-mocks"))]`; `birdsong::mocks` gated Wave 147; `StubAllow`/`StubDeny`/`StubPassthrough`/`StubMockEncrypted` gated Wave 154) |
| **Capability discovery** | `find_primals_with_capability` — identity-agnostic, env-driven |
| **Hardcoded elimination** | All ports env-driven; canonical constants for all port defaults (Wave 181: 8 new constants — `DEFAULT_DISCOVERY_SERVICE_PORT`, `DEFAULT_OBSERVABILITY_PORT`, `DEFAULT_DASHBOARD_UI_PORT`, `DEFAULT_FEDERATION_COORDINATION_PORT`, `DEFAULT_TARPC_RPC_PORT`, `DEFAULT_GAMING_BASE_PORT`, `EPHEMERAL_BIND_ADDR`); `discovery_port()` bug fixed (was using metrics constant); tarpc port harmonized to 8091 across 3 conflicting locations; all `SafeEnv::get_port()` calls in orchestrator/universal replaced with canonical functions; all IPs (`"0.0.0.0"`, `"127.0.0.1"`, `"localhost"`) evolved to constants (`PRODUCTION_BIND_ADDRESS`, `LOCALHOST`, `is_loopback_host()`); duplicate mDNS `"224.0.0.251:5353"` replaced with canonical `MDNS_MULTICAST_GROUP` + `MDNS_PORT`; capability-first across 11+ crates; all legacy primal env vars deprecated with `tracing::warn!` |
| **JSON-RPC dispatch** | Typed `JsonRpcMethod` enum (53+ methods, 33 domain sub-enums including `Lifecycle` and `Inference`); `normalize_json_rpc_method_name()` absorbs `model.*`/`ai.*` → `inference.*`, `discovery.find_by_capability`/`net.discovery.find_by_capability` → `ipc.discover` |
| **License** | `AGPL-3.0-or-later` (workspace + per-crate; **Apr 7**: inconsistent `AGPL-3.0-only` strings eliminated) via workspace inheritance + ORC + CC-BY-SA 4.0 |
| **SPDX headers** | 100% `.rs` coverage — **Apr 7**: all updated to `AGPL-3.0-or-later` (aligned with `Cargo.toml`) |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok); locally enforced (`cargo deny check`); CI runs fmt + clippy + test only |
| **C dependencies** | Zero in default build (`blake3` uses `features=["pure"]`; `ring` NOT compiled — lockfile-only via `rustls-webpki` optional dep, see `deny.toml` stadial analysis; `ed25519-dalek` in quic behind `local-certs` feature); **Bluetooth** (`libudev`/USB stack paths): feature-gated; **sled** removed Wave 135 (SB-03 resolved); `parking_lot` removed (Wave 133) |
| **`async-trait`** | **0** annotations, dependency fully removed from workspace (Wave 145). 141→0: every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. No crate depends on `async-trait`. SB-06 resolved. Wave 155: eliminated 6 more production `dyn` sites (iterator, error, future, plugin, composable, callback). Remaining `dyn`: architectural `dyn Stream` (async watch), `Box<dyn SerialPort>` (external crate), `Arc<dyn Fn>` (test injection) |
| **Test infrastructure** | Zero `#[serial]`, zero hardcoded ports, zero startup sleep waits; all time-dependent tests use `start_paused`/`advance`; all network binds use port 0; `ConnectionPool` uses `tokio::time::Instant` for deterministic testing; only `std::thread::sleep` allowed in mockito sync callbacks and `std::time::Instant`-dependent cache tests (documented) |
| **Zero-copy** | `Arc<str>` IPC handler fields (mesh/punch/rendezvous/capability), `bytes::Bytes`, `SharedBytes`, `Cow<'_, str>` JSON-RPC wire types, move semantics, borrow-through redirects |
| **Total Rust** | ~421,000 lines across 30 crates (1,609 files) |
| **primalSpring gaps** | All original gaps resolved Wave 134; Phase 43 downstream audit (6 items) completed Wave 140: UDS first-byte peek, mito-beacon credential tiers, STUN beacon auth, content distribution federation, ring lockfile documented; Wave 143: content distribution federation wired (`ContentAnnouncementStore`, `discovery.content_peers`, seeder/leecher coordination), `ring` deny.toml updated; Wave 145: `async-trait` fully eliminated (SB-06 resolved); `capability.resolve` + `discovery.peers` wired (Wave 137); LD-02 resolved (Wave 137b); LD-08 resolved + self-healing (Waves 138/139); **Phase 45c** BTSP wire-format fixes completed Wave 160: server.rs refactored with BTSP auto-detection (first-line peek routes `"protocol":"btsp"` → NDJSON handshake), `btsp.session.*` RPC mappings added |

---

## Remaining Work

### BTSP Phase 3 (FULL — Wave 180→186)

Phase 2 relay fixed (Waves 132–169). **Phase 3 implemented** (Wave 180): `btsp.negotiate` JSON-RPC handler, ChaCha20-Poly1305 encrypted framing, HKDF-SHA256 session key derivation via `btsp.server.export_keys` delegation to BearDog, graceful NULL cipher fallback, encrypted frame loop wired in both `pure_rust_server/connection.rs` and `bin_interface/server.rs`. **Spec-aligned** (Wave 182): `bond_type` parameter with cipher floor enforcement per `BTSP_PROTOCOL_STANDARD.md` `BondingPolicy` table, `preferred_cipher` backward compat, 12-byte server_nonce (aligned with audit spec), both `ciphers` array and `preferred_cipher` formats accepted. **Dispatch fix** (Wave 184): `btsp.negotiate` wired into `handle_btsp_frames` (binary-framed BTSP path) — was previously only on NDJSON path, causing "method not found" for binary-protocol clients. Now reachable on all 3 transport paths: NDJSON session, binary-framed BTSP, and bin_interface. **Live connection verification** (Wave 186): 4 new tests verifying post-negotiate encrypted frame loop on live async duplex streams — full negotiate→encrypt→exchange→disconnect lifecycle, notification handling, null-cipher fallback, and mock-security-provider E2E transition. 32 tests total. **Remaining**: Multi-frame session stress tests, live BearDog + primalSpring integration.

### Tor Onion Service — Security Provider Crypto (BLOCKED)

HSDir descriptor superencryption, `ESTABLISH_INTRO` HMAC/signature, `INTRODUCE1`/`INTRODUCE2` ntor payloads, and rendezvous auth keys delegate to security provider JSON-RPC. Stub sections documented inline with `// BLOCKED:` and return `Error::CryptoUnavailable`.

### TLS / Sovereign Onion (requires live security provider)

- `ed25519_public_from_secret` via security provider
- Security-provider-generated lineage-tagged certificates
- CertificateVerify signing via security provider
- Custom TLS extension building via security provider

---

## Pending: Coverage Expansion (73.41% → 90% target)

**Note (May 2, 2026)**: 73.41% measured via llvm-cov `--workspace --lib` (Apr 27 2026). Wave 179 added 92 tests across 15+ files in 8 crates targeting low-coverage pure-logic modules. Wave 171 added 271 tests (71.28% → 73.41%); Wave 174 added 18; Wave 177 added 9. Wave 180 added 19. 7,803 total lib tests. Target 90% via ongoing coverage expansion of I/O-heavy modules with mock infrastructure.

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
- [ ] Cluster support for anonymous beacon broadcasting
- [ ] TLS handshake v2 module integration
- [ ] IPC native endpoint lifecycle management

---

## Pending: Dependency Evolution

- [x] `ring-crypto` feature removed (Wave 135, SB-02 resolved): `rustls_rustcrypto` is the sole TLS provider. `ring` is NOT compiled in any build config (`cargo tree -i ring` = empty). Cargo.lock stanza persists because `rustls-webpki` (0.102 + 0.103) declares `ring` as an optional dep — Cargo's resolver locks optional dep versions by design. Investigated git `rustls-rustcrypto` (drops webpki 0.102) but pre-release RustCrypto crates are incompatible with stable workspace. Blocked on upstream `rustls-rustcrypto` crates.io release. See `deny.toml` for full stadial gate analysis.
- [ ] Remaining transitive duplicates (hashbrown ×3, getrandom ×3, socket2 ×2, rand ×2, indexmap ×2, generic-array ×2, cpufeatures ×2) require upstream version unification; `futures 0.3` only transitive via `tarpc` — no direct deps remain
- [x] `serde_yaml` → `serde_yaml_ng` (Wave 165): `songbird-config` and `songbird-discovery` migrated from archived `serde_yaml 0.9` to `serde_yaml_ng 0.10` (maintained fork, aliased as `serde_yaml` — zero call-site changes). `kube-client` transitive dep keeps original `serde_yaml` in Cargo.lock — full lockfile removal blocked on kube upstream
- [ ] `bincode` 1.x (RUSTSEC-2025-0141): transitive via tarpc/tokio-serde; migrate tarpc codec or swap to postcard
- [x] `async-trait` **fully eliminated** (Wave 145): 141→0 annotations, dependency removed from all crates and workspace `Cargo.toml`. Every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. SB-06 resolved.
- [x] `dyn` dispatch evolution (Wave 155, audited Wave 159): 6 production `dyn` sites eliminated. Remaining architectural: `Pin<Box<dyn Stream>>` (discovery watch trait — `CanonicalDiscovery::watch_services`, delegation, 6+ adapter impls; all backends return empty streams; evolve to enum dispatch when real streams exist), `Box<dyn SerialPort>` (external `serialport` crate API), `Arc<dyn Fn(&str) -> Result<String, VarError>>` (env reader injection in `capability_discovery`/`runtime_engine` — intentional for testability), `&dyn Any` (`as_any()` in discovery factory/backends — standard `Any` downcast pattern, architectural)
- [x] `rand` removed from `songbird-orchestrator` production deps (Wave 140): JWT CSPRNG replaced with `getrandom::fill()`; `rand` retained as dev-dependency for tests
- [x] Dead `sled` dependency removed from `songbird-tor-protocol` (Wave 116)
- [x] `ed25519-dalek` in `songbird-quic` feature-gated behind `local-certs` (Wave 116)
- [x] Port constants consolidated (Wave 130 initial → Wave 161 completed): all 12 deprecated port constants removed from `songbird-types::constants` (zero consumers remained). `DEFAULT_PORT` retained as alias to `defaults::ports::DEFAULT_HTTP_PORT`. `DEFAULT_BIRDSONG_PORT` (42424), `DEFAULT_QUIC_PORT` (4433) added to centralized `defaults::ports`. Duplicate STUN constants in `songbird-universal-ipc` and `songbird-onion-relay` replaced with centralized imports. `songbird-config::constants::network` port/host definitions replaced with re-exports from canonical source. `DEFAULT_ORCHESTRATOR_PORT` aligned to 8080 (matching actual HTTP API bind port). Wave 161: 15+ scattered `.unwrap_or(8080)` in 8 crates replaced with canonical `songbird_types::defaults::ports` constants; `songbird-config::defaults::ports` functions now fall back to typed constants instead of magic numbers; duplicate `DEFAULT_BIND_ADDRESS` in `network.rs` replaced with re-export
- [x] `hostname` crate consolidated to `gethostname` (Wave 161/165): Wave 161 removed `hostname` from `songbird-config` (replaced `hostname::get()` with `gethostname::gethostname()`). Wave 165 expanded to `songbird-discovery`, `songbird-orchestrator`, `songbird-cli`, `songbird-types`, `songbird-compute-bridge` — zero `hostname` direct deps remain
- [x] `futures` facade → `futures-util` (Wave 161/165): Wave 161 removed unused `futures` from `songbird-bluetooth`/`songbird-lineage-relay` and migrated `songbird-stun`/`songbird-orchestrator`/`songbird-universal-ipc` to `futures-util`. Wave 165 migrated `songbird-universal` (`futures 0.3` → `futures-util { workspace = true }`, 13 `block_on` tests converted to `#[tokio::test]`, `url` aligned to 2.5). Zero direct `futures` facade deps remain
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

**Remaining hardening**:
- [ ] Full `cloudflared` tunnel orchestration (spawn process, parse assigned URL, monitor lifecycle)
- [ ] RFC 2136 DDNS provider (`nsupdate` wire protocol)
- [ ] TURN allocation refresh lifecycle (keepalive loop)
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

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) — Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration
- [ ] Tor consensus microdescriptor parsing (ntor_key, version fields)
- [ ] Tor HSDir descriptor upload

---

## Priority Order

1. **Security provider crypto e2e validation** — Tor/TLS items need live security provider
2. **Coverage expansion** — Target pure-logic modules first (goal: 90%)
3. **Deep documentation** — Fill internal modules with full doc coverage
4. **Real hardware tests** (Tower + Pixel) — Validates cross-network
5. **Platform backends** — Mobile pairing, iOS, WASM
6. **Dependency pruning** — Reduce unique deps where possible
