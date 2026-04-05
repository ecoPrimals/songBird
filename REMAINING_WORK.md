# Songbird Remaining Work

**Date**: April 5, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: Wave 118 — legacy primal name elimination: beardog/toadstool/squirrel/nestgate identifiers removed across sovereign-onion, crypto-provider, tls, execution-agent, nfc, config, types, quic, http-client, federation, orchestrator (19+ functions, 12+ types, 5+ modules); `#[allow(`→`#[expect(` conversion (~1092 production sites); 42 new adapter coverage tests; deprecated `PrimalConfig` fields and standalone endpoint functions removed; env var fallbacks kept with deprecation warnings

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 12,613 passed, 0 failed, 252 ignored (env-dependent e2e/chaos/hardware/crypto-provider) |
| **Line Coverage** | **69.76%** measured (llvm-cov `--workspace --lib`, Apr 5 2026; target 90%) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 30 crates compile clean (~43s dev) |
| **Clippy Pedantic** | 30/30 crates clean — zero warnings (`clippy::pedantic + nursery`, `-D warnings`) |
| **Format** | Clean (`cargo fmt --check` passes) |
| **Docs** | Clean (`cargo doc --workspace --no-deps` — 0 warnings) |
| **Files >800 lines** | 0 (production max 709L `ipc/types.rs`; test files refactored Wave 113) |
| **Unsafe blocks** | **0** — `forbid(unsafe_code)` on all 30 crates |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 (all remaining in `#[cfg(test)]` or doc examples) |
| **Production `panic!()`** | 0 |
| **Production `unreachable!()`** | 0 |
| **TODO/FIXME/HACK comments** | 0 in Rust source |
| **Commented-out code** | 0 (eliminated Wave 112) |
| **`#[allow(` vs `#[expect(`** | ~352 `#[expect(reason)]` in production code; `#[allow(reason)]` in `#[cfg(test)]` modules and where cfg/test interaction causes unfulfilled-expectation errors |
| **Mocks in production** | 0 (all inside `#[cfg(test)]`) |
| **Capability discovery** | `find_primals_with_capability` — identity-agnostic, env-driven |
| **Hardcoded elimination** | All ports env-driven; capability-first across 11+ crates; all legacy primal env vars deprecated with `tracing::warn!`; all deprecated function/type/module names removed (zero-caller cleanup); remaining legacy env var strings kept as backward-compat fallbacks only |
| **JSON-RPC dispatch** | Typed `JsonRpcMethod` enum (53+ methods, 14 domain sub-enums) |
| **License** | `AGPL-3.0-or-later` via workspace inheritance + ORC + CC-BY-SA 4.0 |
| **SPDX headers** | 100% `.rs` coverage |
| **cargo-deny** | Fully passing (advisories ok, bans ok, licenses ok, sources ok) |
| **C dependencies** | Zero in default build (`blake3` uses `features=["pure"]`; `ring` only via optional `k8s` feature; `ed25519-dalek` in quic behind `local-certs` feature) |
| **`async-trait`** | 99 `#[async_trait]` across 50 files — 100% require `dyn Trait` dispatch (`Arc<dyn>`, `Box<dyn>`, `&dyn`); no further mechanical migration possible without architectural changes; `Transport`, `MetricsCapabilityAdapter`, `HealthMonitor` already migrated to native `async fn in trait` |
| **Test infrastructure** | Zero `#[serial]`, zero hardcoded ports, zero startup sleep waits; all time-dependent tests use `start_paused`/`advance`; all network binds use port 0; `ConnectionPool` uses `tokio::time::Instant` for deterministic testing; only `std::thread::sleep` allowed in mockito sync callbacks and `std::time::Instant`-dependent cache tests (documented) |
| **Zero-copy** | `Arc<str>`, `bytes::Bytes`, `SharedBytes`, move semantics, borrow-through redirects |
| **Total Rust** | ~423,800 lines across 30 crates (1,573 files) |

---

## Active Blockers

### SB-03: Sled → Storage Provider Migration (Blocked on NG-01)

**Status**: Abstraction layer complete; migration blocked until storage provider exposes `storage.*` IPC.

**What exists:**
- `storage.*` JSON-RPC methods in `songbird-types::JsonRpcMethod`
- `ConsentStorageBackend` trait (orchestrator)
- `TaskStorageBackend` trait (orchestrator)
- `OnionStorageBackend` trait (sovereign-onion)
- Sled implements all three; storage provider backends ready to wire when NG-01 lands

**Remaining sled locations:**

| Crate | Module | Backend |
|-------|--------|---------|
| `songbird-orchestrator` | `consent_management/storage_sled.rs` | `ConsentStorage` → `ConsentStorageBackend` |
| `songbird-orchestrator` | `task_lifecycle/storage_sled.rs` | `TaskStorage` → `TaskStorageBackend` |
| `songbird-sovereign-onion` | `storage.rs` | `OnionStorage` → `OnionStorageBackend` |
| `songbird-tor-protocol` | `Cargo.toml` only | Optional dep, no Rust usage yet |

### Tor Onion Service — Security Provider Crypto (BLOCKED)

HSDir descriptor superencryption, `ESTABLISH_INTRO` HMAC/signature, `INTRODUCE1`/`INTRODUCE2` ntor payloads, and rendezvous auth keys delegate to security provider JSON-RPC. Stub sections documented inline with `// BLOCKED:` and return `Error::CryptoUnavailable`.

### TLS / Sovereign Onion (requires live security provider)

- `ed25519_public_from_secret` via security provider
- Security-provider-generated lineage-tagged certificates
- CertificateVerify signing via security provider
- Custom TLS extension building via security provider

---

## Pending: Coverage Expansion (69.76% → 90% target)

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
