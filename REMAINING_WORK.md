# Songbird Remaining Work

**Date**: April 6, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: Wave 119 — deep debt elimination pass: all remaining `8.8.8.8`/`1.1.1.1` hardcoded IPs in orchestrator network/binding replaced with `netdev`-based discovery + `SONGBIRD_ROUTE_DETECT_ADDR` env fallback (RFC 5737 `192.0.2.1`); `/tmp/` fallback socket paths evolved to XDG-compliant (`XDG_RUNTIME_DIR` → `TMPDIR` → `/tmp`); hardcoded ports 2300/3478/3479 made env-configurable (`SONGBIRD_DISCOVERY_PORT`, `SONGBIRD_STUN_PORT`, `SONGBIRD_RELAY_PORT`); 4 production `unwrap()`/`expect()` eliminated or documented with `#[expect]` + reason; `tower_atomic.rs` (990 lines) refactored into `tower_atomic/{mod,types,server,client,tests}.rs` (max 519L); remaining `#[allow(` in production CLI/config/federation/http-client/bluetooth converted to `#[expect(` with reasons; unfulfilled lint expectations removed; zero-copy `Arc<str>` evolution in mesh/punch/rendezvous/capability IPC handlers (replaces per-message `String::clone`); coverage expanded to **72.29%** (+0.74pp): 50+ new tests across `bind_and_ports`, `register`, `announcement`, `remote_probes`, `anonymous/protocol`, `security_crypto_client`, `auth`, `enhanced_router`, `unix_transport`, `discovery_bridge`, `tarpc_server`.

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 12,811 passed, 0 failed, 252 ignored (env-dependent e2e/chaos/hardware/crypto-provider) |
| **Line Coverage** | **72.29%** measured (llvm-cov `--workspace --lib`, Apr 6 2026; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 30 crates compile clean (~43s dev) |
| **Clippy Pedantic** | 30/30 crates clean — zero warnings (`clippy::pedantic + nursery`, `-D warnings`) |
| **Format** | Clean (`cargo fmt --check` passes; Apr 6 audit: no drift) |
| **Docs** | Clean (`cargo doc --workspace --no-deps` — 0 warnings) |
| **Files >800 lines** | 0 (production max 519L `tower_atomic/tests/mod.rs`; `tower_atomic.rs` refactored Wave 119 from 990→4 files) |
| **Unsafe blocks** | **0** — `forbid(unsafe_code)` on all 30 crates |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 (all remaining in `#[cfg(test)]` or doc examples; `expect()` on const parses documented with `#[expect(reason)]`) |
| **Production `panic!()`** | 0 |
| **Production `unreachable!()`** | 0 |
| **TODO/FIXME/HACK comments** | 0 in Rust source |
| **Commented-out code** | 0 (eliminated Wave 112) |
| **`#[allow(` vs `#[expect(`** | ~352 `#[expect(reason)]` in production code; `#[allow(reason)]` in `#[cfg(test)]` modules and where cfg/test interaction causes unfulfilled-expectation errors; **Apr 6**: production `dead_code` allows eliminated (prefer fixes over suppression) |
| **Mocks in production** | 0 (all inside `#[cfg(test)]`) |
| **Capability discovery** | `find_primals_with_capability` — identity-agnostic, env-driven |
| **Hardcoded elimination** | All ports env-driven (`SONGBIRD_DISCOVERY_PORT`, `SONGBIRD_STUN_PORT`, `SONGBIRD_RELAY_PORT`); all socket paths XDG-compliant; all IP probes use netdev + RFC 5737 fallback; capability-first across 11+ crates; all legacy primal env vars deprecated with `tracing::warn!`; all deprecated function/type/module names removed |
| **JSON-RPC dispatch** | Typed `JsonRpcMethod` enum (53+ methods, 14 domain sub-enums) |
| **License** | `AGPL-3.0-or-later` (workspace + per-crate; **Apr 6**: inconsistent `AGPL-3.0-only` strings eliminated) via workspace inheritance + ORC + CC-BY-SA 4.0 |
| **SPDX headers** | 100% `.rs` coverage — **Apr 6**: all updated to `AGPL-3.0-or-later` (aligned with `Cargo.toml`) |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok) |
| **C dependencies** | Zero in default build (`blake3` uses `features=["pure"]`; `ring` only via optional `k8s` feature; `ed25519-dalek` in quic behind `local-certs` feature); **Bluetooth** (`libudev`/USB stack paths): feature-gated; **sled** (`sled-storage` feature): deprecated, non-default — NestGate `storage.*` capability is the production path |
| **`async-trait`** | 99 `#[async_trait]` across 50 files — 100% require `dyn Trait` dispatch (`Arc<dyn>`, `Box<dyn>`, `&dyn`); no further mechanical migration possible without architectural changes; `Transport`, `MetricsCapabilityAdapter`, `HealthMonitor` already migrated to native `async fn in trait` |
| **Test infrastructure** | Zero `#[serial]`, zero hardcoded ports, zero startup sleep waits; all time-dependent tests use `start_paused`/`advance`; all network binds use port 0; `ConnectionPool` uses `tokio::time::Instant` for deterministic testing; only `std::thread::sleep` allowed in mockito sync callbacks and `std::time::Instant`-dependent cache tests (documented) |
| **Zero-copy** | `Arc<str>` IPC handler fields (mesh/punch/rendezvous/capability), `bytes::Bytes`, `SharedBytes`, `Cow<'_, str>` JSON-RPC wire types, move semantics, borrow-through redirects |
| **Total Rust** | ~423,800 lines across 30 crates (1,573 files) |

---

## Active Blockers

### SB-03: Sled → NestGate Storage Migration (Resolved — NestGate backend wired)

**Status**: `NestGateStorage` and `NestGateOnionStorage` implemented. Runtime capability discovery delegates to `storage.*` JSON-RPC provider (NestGate) when available; in-memory fallback otherwise. `sled-storage` feature deprecated in both crates (optional, non-default, legacy only).

**Architecture:**
- `NestGateStorage` → `ConsentStorageBackend` + `TaskStorageBackend` via `storage.*` JSON-RPC
- `NestGateOnionStorage` → `OnionStorageBackend` via `storage.*` JSON-RPC
- `InMemoryStorage` / `InMemoryOnionStorage` → fallback when no storage provider available
- `ConsentStorage` / `TaskStorage` / `OnionStorage` → sled, deprecated, behind `sled-storage` feature

**Remaining**: NestGate must expose live `storage.*` IPC endpoints for end-to-end validation.

### Tor Onion Service — Security Provider Crypto (BLOCKED)

HSDir descriptor superencryption, `ESTABLISH_INTRO` HMAC/signature, `INTRODUCE1`/`INTRODUCE2` ntor payloads, and rendezvous auth keys delegate to security provider JSON-RPC. Stub sections documented inline with `// BLOCKED:` and return `Error::CryptoUnavailable`.

### TLS / Sovereign Onion (requires live security provider)

- `ed25519_public_from_secret` via security provider
- Security-provider-generated lineage-tagged certificates
- CertificateVerify signing via security provider
- Custom TLS extension building via security provider

---

## Pending: Coverage Expansion (72.29% → 90% target)

**Note (Apr 6, 2026)**: The percentage above is from the prior llvm-cov run; a new coverage total will be recorded **after** the planned test expansion and dedicated coverage pass (not part of today’s audit).

| Module | Measured Coverage | Priority |
|--------|-------------------|----------|
| songbird-universal/adapters/compute/adapter.rs | 11.83% | HIGH |
| songbird-universal/adapters/security.rs | 18.42% | HIGH |
| songbird-universal/adapters/ai.rs | 20.28% | HIGH |
| songbird-universal-ipc/handlers/stun_handler/client.rs | 14.22% | HIGH |
| songbird-universal-ipc/handlers/http_handler/handler.rs | 20.00% | HIGH |
| songbird-universal/tarpc_client/ops.rs | 23.93% | MEDIUM |
| songbird-universal-ipc/tower_atomic.rs | 26.35% | MEDIUM |
| songbird-universal/adapters/storage.rs | 30.23% | MEDIUM |
| songbird-orchestrator (aggregate) | ~56% | MEDIUM |
| songbird-config (aggregate) | ~68% | LOW |
| songbird-universal-ipc (aggregate) | ~67% | MEDIUM |

**Strategy**: Focus on pure logic modules for unit test ROI. Prioritize adapter initialization, error paths, capability routing, and protocol parsing. Target 5-10pp coverage gain per session.

**Wave 116 breakthrough**: `CapabilityTransport` trait extracted from all three adapters (Security, Compute, AI). `MockTransport` enables full unit testing of adapter methods without network dependencies. 22 new mock-transport tests added. Protocol-specific match blocks eliminated.

**Remaining highest-ROI targets**: (1) Add in-process tarpc test server for `ops.rs` coverage, (2) tower_atomic pure logic paths, (3) STUN client error paths via `MockTransport` pattern.

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

- [ ] `ring` elimination: `rcgen` replaced with pure-Rust cert gen; `ring` remains only via optional `k8s` feature (`kube` → `rustls` → `ring`)
- [ ] Remaining transitive duplicates (syn, hashbrown, getrandom, parking_lot, socket2) require upstream changes
- [x] `async-trait` partial migration (Wave 116): `Transport`, `MetricsCapabilityAdapter`, `HealthMonitor` migrated to native `async fn in trait`; `async-trait` dep removed from `songbird-bluetooth`; ~90% of remaining usages require `dyn Trait` (must keep)
- [x] Dead `sled` dependency removed from `songbird-tor-protocol` (Wave 116)
- [x] `ed25519-dalek` in `songbird-quic` feature-gated behind `local-certs` (Wave 116)

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
