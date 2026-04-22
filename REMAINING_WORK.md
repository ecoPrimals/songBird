# Songbird Remaining Work

**Date**: April 15, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: Wave 159 (Apr 15, 2026)  
**Current Wave**: 159 — Comprehensive deep debt cleanup: removed 12 deprecated port constants from `songbird-types::constants` (zero consumers); centralized scattered port defaults (`DEFAULT_BIRDSONG_PORT` 42424, `DEFAULT_QUIC_PORT` 4433 added to `defaults::ports`, duplicate STUN 3478 in universal-ipc/onion-relay replaced with centralized import); aligned `DEFAULT_ORCHESTRATOR_PORT` 8000→8080 in `defaults::ports` (matching actual HTTP API port); removed stale `gaming`/`federation` feature flags from network-federation; removed stale `test-mocks` feature from bluetooth; removed dead `NESTGATE_AUTHENTICATION_PURPOSE` constant (zero callers); replaced `config::constants::network` local port/host definitions with re-exports from canonical source; cleaned stale sovereignty-violation comment block. All tests pass (7,387+), 0 clippy warnings, 0 build warnings  
**Previous Waves** (full detail in `CHANGELOG.md`): 158 (BTSP Step 3→4 verification relay), 157 (hardcoded literals, dead deps, doc cleanup, debris removal), 154 (mock isolation, dead deps, lint hygiene), 153 (BTSP NDJSON wire-format alignment), 152 (dead deps, hardcoding, test hygiene), 151 (PG-37 capability-first routing), 150 (doc cleanup, debris removal), 149 (comprehensive deep debt: blanket lint removal, hardcoded paths, duplicate constants, mock features, stale CLI, expect safety), 148 (PG-21 persistent NDJSON sessions), 147 (mock isolation, hardcoded IP/path elimination, lint hygiene), 146 (stadial dyn audit + ring analysis), 139b (deep literal sweep), 139 (self-healing auto-discovery), 138b (hardcoded literal evolution), 138 (LD-08 socket auto-discovery), 137b-c (ipc.resolve dual-mode, stale features, port canonicalization, lint hygiene), 137 (capability naming), 136 (constant consolidation), 135 (SB-02/SB-03 resolved), 134 (primalSpring gaps), 133 (smart refactor), 132 (BTSP Phase 2), 131-119 (hardcoding, legacy scrub, coverage)

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 7,387 lib passed, 0 failures, 22 ignored |
| **Line Coverage** | **72.29%** measured (llvm-cov `--workspace --lib`, Apr 8 2026; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 30 crates compile clean (~43s dev) |
| **Clippy Pedantic** | 30/30 crates clean — zero warnings (`clippy::pedantic + nursery`, `-D warnings`, Apr 22 verified) |
| **Format** | Clean (`cargo fmt --check` passes; Apr 22 verified) |
| **Docs** | Clean (`cargo doc --workspace --no-deps` — 0 warnings) |
| **Files >800 lines** | 0 (largest production 763L `primal_discovery.rs`; former 1030L monolith smart-refactored Wave 144; 4 former >700L files refactored Wave 133) |
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
| **Hardcoded elimination** | All ports env-driven (`SONGBIRD_DISCOVERY_PORT`, `SONGBIRD_STUN_PORT`, `SONGBIRD_RELAY_PORT`, `SONGBIRD_BIND_ADDRESS`, `SONGBIRD_MULTICAST_ADDRESS`); canonical `DEFAULT_SONGBIRD_PORT` (3492) constant replaces all magic-number port fallbacks (Wave 136); `BIOMEOS_RUNTIME_SUBDIR` constant replaces all `"biomeos"` path literals in production (Wave 136); all socket paths XDG-compliant; all IP probes use netdev + RFC 5737 fallback; capability-first across 11+ crates; all legacy primal env vars deprecated with `tracing::warn!`; all deprecated function/type/module names removed; Wave 137c: zero remaining hardcoded `"0.0.0.0"` / `"127.0.0.1"` / `"localhost"` in production code — all evolved to `PRODUCTION_BIND_ADDRESS` / `DEVELOPMENT_BIND_ADDRESS` / `LOCALHOST` constants; all legacy port constants deprecated to canonical `defaults::ports` |
| **JSON-RPC dispatch** | Typed `JsonRpcMethod` enum (53+ methods, 33 domain sub-enums including `Lifecycle` and `Inference`); `normalize_json_rpc_method_name()` absorbs `model.*`/`ai.*` → `inference.*`, `discovery.find_by_capability`/`net.discovery.find_by_capability` → `ipc.discover` |
| **License** | `AGPL-3.0-or-later` (workspace + per-crate; **Apr 7**: inconsistent `AGPL-3.0-only` strings eliminated) via workspace inheritance + ORC + CC-BY-SA 4.0 |
| **SPDX headers** | 100% `.rs` coverage — **Apr 7**: all updated to `AGPL-3.0-or-later` (aligned with `Cargo.toml`) |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok); enforced in CI via `ci.yml` (Wave 134) |
| **C dependencies** | Zero in default build (`blake3` uses `features=["pure"]`; `ring` NOT compiled — lockfile-only via `rustls-webpki` optional dep, see `deny.toml` stadial analysis; `ed25519-dalek` in quic behind `local-certs` feature); **Bluetooth** (`libudev`/USB stack paths): feature-gated; **sled** removed Wave 135 (SB-03 resolved); `parking_lot` removed (Wave 133) |
| **`async-trait`** | **0** annotations, dependency fully removed from workspace (Wave 145). 141→0: every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. No crate depends on `async-trait`. SB-06 resolved. Wave 155: eliminated 6 more production `dyn` sites (iterator, error, future, plugin, composable, callback). Remaining `dyn`: architectural `dyn Stream` (async watch), `Box<dyn SerialPort>` (external crate), `Arc<dyn Fn>` (test injection) |
| **Test infrastructure** | Zero `#[serial]`, zero hardcoded ports, zero startup sleep waits; all time-dependent tests use `start_paused`/`advance`; all network binds use port 0; `ConnectionPool` uses `tokio::time::Instant` for deterministic testing; only `std::thread::sleep` allowed in mockito sync callbacks and `std::time::Instant`-dependent cache tests (documented) |
| **Zero-copy** | `Arc<str>` IPC handler fields (mesh/punch/rendezvous/capability), `bytes::Bytes`, `SharedBytes`, `Cow<'_, str>` JSON-RPC wire types, move semantics, borrow-through redirects |
| **Total Rust** | ~421,000 lines across 30 crates (1,609 files) |
| **primalSpring gaps** | All original gaps resolved Wave 134; Phase 43 downstream audit (6 items) completed Wave 140: UDS first-byte peek, mito-beacon credential tiers, STUN beacon auth, content distribution federation, ring lockfile documented; Wave 143: content distribution federation wired (`ContentAnnouncementStore`, `discovery.content_peers`, seeder/leecher coordination), `ring` deny.toml updated; Wave 145: `async-trait` fully eliminated (SB-06 resolved); `capability.resolve` + `discovery.peers` wired (Wave 137); LD-02 resolved (Wave 137b); LD-08 resolved + self-healing (Waves 138/139) |

---

## Remaining Work

### BTSP Phase 3 (pending — Phase 2 relay verified)

Phase 2 relay fixed (Waves 132/153/156/158): `perform_server_handshake` + `perform_server_handshake_ndjson` wired into UDS accept, startup resilience (graceful crypto fallback), NDJSON handshake timeouts, Neural API timeout fix. Wave 158: all 6 wire-protocol mismatches between Songbird's relay and BearDog's `btsp.server.*` methods resolved (create params, verify params, response field names, challenge source, ServerHello session_id, negotiate method). Full 4-step handshake (ClientHello → ServerHello → ChallengeResponse → HandshakeComplete) should now complete with live BearDog. **Remaining**: Phase 3 encrypted framing (ChaCha20-Poly1305 / HMAC-plain actual stream encryption via `btsp.server.export_keys`), multi-frame sessions, and E2E integration test with live BearDog + primalSpring.

### Tor Onion Service — Security Provider Crypto (BLOCKED)

HSDir descriptor superencryption, `ESTABLISH_INTRO` HMAC/signature, `INTRODUCE1`/`INTRODUCE2` ntor payloads, and rendezvous auth keys delegate to security provider JSON-RPC. Stub sections documented inline with `// BLOCKED:` and return `Error::CryptoUnavailable`.

### TLS / Sovereign Onion (requires live security provider)

- `ed25519_public_from_secret` via security provider
- Security-provider-generated lineage-tagged certificates
- CertificateVerify signing via security provider
- Custom TLS extension building via security provider

---

## Pending: Coverage Expansion (72.29% → 90% target)

**Note (Apr 8, 2026)**: 72.29% measured via llvm-cov `--workspace --lib` (Apr 8 2026). Target 90% via ongoing pure-logic module expansion.

| Module | Measured (Apr 8) | Tests Added (Waves 124-127) | Priority |
|--------|-------------------|----------------------------|----------|
| songbird-universal/adapters/compute/adapter.rs | 11.83% | +12 tests (Wave 124: discovery, transport, legacy env, metrics) | DONE |
| songbird-universal-ipc/handlers/stun_handler/client.rs | 14.22% | +15 tests (Wave 124: error paths, NAT detection, port pattern) | DONE |
| songbird-universal/adapters/security.rs | 18.42% | +9 tests (Wave 126: discovery, BEARDOG deprecation, metrics/health) | DONE |
| songbird-universal-ipc/handlers/http_handler/handler.rs | 20.00% | +12 tests (Wave 124: dispatch, error formatting, factory failures) | DONE |
| songbird-universal/adapters/ai.rs | 20.28% | +6 tests (Wave 127: MockTransport metrics/health, timeout, SQUIRREL deprecation) | DONE |
| songbird-universal/tarpc_client/ops.rs | 23.93% | +5 tests (Wave 127: empty cap, sequential ops, serde round-trips) | DONE |
| songbird-universal-ipc/tower_atomic/ (4 modules) | 26.35% | +6 tests (Wave 126: malformed JSON, concurrent clients, oversized) | DONE |
| songbird-universal/adapters/storage.rs | 30.23% | +10 tests (Wave 127: discovery chain, MockTransport, DelayTransport) | DONE |
| songbird-orchestrator (aggregate) | ~56% | — | MEDIUM |
| songbird-universal-ipc (aggregate) | ~67% | — | MEDIUM |
| songbird-config (aggregate) | ~68% | — | LOW |

**Note**: All high-priority pure-logic modules now have comprehensive MockTransport-based tests. Remaining coverage gains come from orchestrator integration paths (requires live IPC), config edge cases, and IPC handler dispatch branches. Re-measure with `cargo llvm-cov --workspace --lib` to update percentages.

**Strategy**: MockTransport pattern established across all four adapters (Security, Compute, AI, Storage). Next ROI: orchestrator consent/task lifecycle paths, config validation edge cases.

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

- [ ] Cluster support for anonymous beacon broadcasting
- [ ] TLS handshake v2 module integration
- [ ] IPC native endpoint lifecycle management

---

## Pending: Dependency Evolution

- [x] `ring-crypto` feature removed (Wave 135, SB-02 resolved): `rustls_rustcrypto` is the sole TLS provider. `ring` is NOT compiled in any build config (`cargo tree -i ring` = empty). Cargo.lock stanza persists because `rustls-webpki` (0.102 + 0.103) declares `ring` as an optional dep — Cargo's resolver locks optional dep versions by design. Investigated git `rustls-rustcrypto` (drops webpki 0.102) but pre-release RustCrypto crates are incompatible with stable workspace. Blocked on upstream `rustls-rustcrypto` crates.io release. See `deny.toml` for full stadial gate analysis.
- [ ] Remaining transitive duplicates (hashbrown ×3, getrandom ×3, socket2 ×2, rand ×2, indexmap ×2, generic-array ×2, cpufeatures ×2) require upstream version unification
- [ ] `serde_yaml` → TOML-only: 5 call sites in `songbird-config` (providers.rs, discover_impl.rs) + `songbird-discovery` (modernized_factory.rs); archived upstream crate; yaml feature already stripped from `config` workspace dep (Wave 152)
- [ ] `bincode` 1.x (RUSTSEC-2025-0141): transitive via tarpc/tokio-serde; migrate tarpc codec or swap to postcard
- [x] `async-trait` **fully eliminated** (Wave 145): 141→0 annotations, dependency removed from all crates and workspace `Cargo.toml`. Every `dyn`-dispatched async trait converted to enum dispatch, concrete types, or native AFIT. SB-06 resolved.
- [x] `dyn` dispatch evolution (Wave 155, audited Wave 159): 6 production `dyn` sites eliminated. Remaining architectural: `Pin<Box<dyn Stream>>` (discovery watch trait — `CanonicalDiscovery::watch_services`, delegation, 6+ adapter impls; all backends return empty streams; evolve to enum dispatch when real streams exist), `Box<dyn SerialPort>` (external `serialport` crate API), `Arc<dyn Fn(&str) -> Result<String, VarError>>` (env reader injection in `capability_discovery`/`runtime_engine` — intentional for testability), `&dyn Any` (`as_any()` in discovery factory/backends — standard `Any` downcast pattern, architectural)
- [x] `rand` removed from `songbird-orchestrator` production deps (Wave 140): JWT CSPRNG replaced with `getrandom::fill()`; `rand` retained as dev-dependency for tests
- [x] Dead `sled` dependency removed from `songbird-tor-protocol` (Wave 116)
- [x] `ed25519-dalek` in `songbird-quic` feature-gated behind `local-certs` (Wave 116)
- [x] Port constants consolidated (Wave 130 initial → Wave 159 completed): all 12 deprecated port constants removed from `songbird-types::constants` (zero consumers remained). `DEFAULT_PORT` retained as alias to `defaults::ports::DEFAULT_HTTP_PORT`. `DEFAULT_BIRDSONG_PORT` (42424), `DEFAULT_QUIC_PORT` (4433) added to centralized `defaults::ports`. Duplicate STUN constants in `songbird-universal-ipc` and `songbird-onion-relay` replaced with centralized imports. `songbird-config::constants::network` port/host definitions replaced with re-exports from canonical source. `DEFAULT_ORCHESTRATOR_PORT` aligned to 8080 (matching actual HTTP API bind port)

---

## Future: Protocol Enhancements

- [ ] PCP (RFC 6887) — Port Control Protocol
- [ ] QUIC multi-path into sovereign socket
- [ ] Full Tor relay mode
- [ ] LoRaWAN integration
- [ ] Full NAT type detection (requires multiple STUN requests)
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
