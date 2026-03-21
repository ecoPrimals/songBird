# Songbird Remaining Work

**Date**: March 21, 2026  
**Version**: v0.2.1  
**Last Deep Debt Audit**: March 21, 2026

---

## Current Status

| Metric | Value |
|--------|-------|
| **Tests** | 9,734 passed, 0 failed, 273 ignored (workspace-wide `--all-features`) |
| **Line Coverage** | ~65% (llvm-cov measured; +114 new tests, orphaned dead code removed improved ratio) |
| **Edition** | Rust 2024 |
| **Build** | Zero errors, zero warnings, all 29 crates compile clean |
| **Clippy Pedantic** | 29/29 crates clean (`clippy::pedantic + nursery + cargo`, zero warnings) |
| **Format** | Clean (`cargo fmt --check` passes) |
| **Docs** | Clean (`cargo doc --all-features --no-deps` passes, zero warnings) |
| **Files >1000 lines** | 0 (largest: 962 lines) |
| **Unsafe blocks** | 2 (in `songbird-process-env` with `parking_lot::Mutex` guard + `#![deny(unsafe_code)]` + per-fn `#[allow]`) |
| **Production `todo!()`** | 0 |
| **Production `.unwrap()`** | 0 (all remaining are in `#[cfg(test)]` modules) |
| **Production `panic!()`** | 0 |
| **TODO/FIXME/HACK comments** | 0 in Rust source (wateringHole compliant) |
| **Orphaned dead code** | 0 (41 files / 11.5K lines removed this session) |
| **`#[allow()]` vs `#[expect()]`** | Bulk migration complete; `#[expect(reason)]` where lint fires, `#[allow(reason)]` where unfulfilled |
| **Capability discovery** | `find_primals_with_capability` evolved to real capability filter (env-driven, identity-agnostic) |
| **Hardcoded elimination** | All ports env-driven (tarpc, CORS, network ports); `staging.internal:8080` removed |
| **JSON-RPC handlers** | All wired to live `FederatedServiceRegistry` and `FederationState` |
| **BearDog crypto** | All placeholders evolved to explicit `CryptoUnavailable` errors with delegation paths |
| **C dependencies** | `ring` via `quinn` + `rcgen` (structural; requires upstream quinn changes) |
| **License** | AGPL-3.0-only + ORC + CC-BY-SA 4.0 (full scyBorg trio) |
| **SPDX Headers** | 100% of .rs files have `SPDX-License-Identifier: AGPL-3.0-only` |
| **cargo-deny** | Config updated for cargo-deny 0.19+ |
| **UniBin** | `songbird server`, `songbird cli` (interactive REPL), `songbird compute-bridge`, `songbird deploy`, `songbird rendezvous` |
| **Mock isolation** | `MockBearDogProvider` behind `#[cfg(any(test, feature = "test-mocks"))]` |
| **Zero-copy** | `Arc<str>` endpoints, `Arc<[u8]>` TLS keys, move semantics, clone hotspots audited |
| **Concurrent tests** | Zero `std::env::set_var` (via `songbird-process-env` Mutex-guarded facade); `#[serial_test::serial]` on env-modifying tests |
| **Event-driven** | Zero `sleep`-based polling in production |
| **Module docs** | 77 `pub mod` declarations documented across 5 crates |
| **`#[ignore]` tests** | 119 total; 100% have reason strings |
| **Binary size** | 20MB release |
| **`#[warn(missing_docs)]`** | 29/29 crates (all library crates have the lint enabled) |
| **JSON-RPC methods** | 10 semantic methods wrapping all major REST endpoints |
| **Dependencies** | ~418 unique; `kube`/`k8s-openapi`/`bollard` feature-gated |
| **Build time** | ~43s clippy (warm), ~513s test suite |
| **Total Rust lines** | 382,889 (crates + src + tests; -21.8K from dead code removal) |

---

## Deep Debt Audit (Mar 20, 2026)

| Principle | Status | Evidence |
|-----------|--------|----------|
| Zero `unsafe` | S+ | `#![forbid(unsafe_code)]` across 28/29 crates; 2 justified blocks in `process-env` with Mutex guard, `#![deny(unsafe_code)]` + per-fn `#[allow]` |
| Pure Rust | S | SHA3-256, SSDP, SOAP, NAT-PMP, base64, hex from scratch; `ring` remains via quinn/rcgen |
| Zero production stubs | S+ | `health_check_all` → TCP reachability; `cli` → interactive REPL; federation join → real parsing; LB → stateful |
| Zero production `panic!()` | S+ | All removed — replaced with `Result`-based error returns |
| Zero `todo!()` in production | S+ | Only in `#[cfg(test)]` functions |
| Zero `.unwrap()` in production | S+ | All remaining are in test modules |
| Zero TODO/FIXME in code | S+ | Tracked in this file instead |
| `#[expect()]` with reasons | S+ | Bulk migration complete; all bare `#[allow()]` eliminated |
| Runtime discovery | S+ | All socket paths: env → XDG → fallback; `find_primals_with_capability` capability-based |
| Event-driven architecture | S+ | Zero polling anti-patterns in production code |
| Concurrent-safe testing | S+ | Injectable env readers, `parking_lot::Mutex`-guarded env facade |
| Self-knowledge only | S+ | Introspection describes only Songbird |
| AGPL-3.0 license | S+ | 1,324/1,324 SPDX headers `AGPL-3.0-only`, cargo-deny configured |
| Capability-based discovery | S+ | No hardcoded primal names; env-driven capability filter |
| Mock isolation | S+ | All mocks behind `#[cfg(test)]` or `feature = "test-mocks"` |
| File size discipline | S+ | 0 files over 1000 lines; 5 near-limit files refactored into domain submodules |

---

## Completed (Mar 21, 2026 — Comprehensive Audit & Deep Debt Session 3)

### Wave 18: Orphaned Dead Code Removal (41 files, 11.5K lines)
- [x] Deleted `production_benchmarks/` (12 files) — severely malformed code, never compiled
- [x] Deleted `basic_iot/mod.rs` — broken syntax, never compiled
- [x] Deleted 10+ orphaned CLI command stubs (`scale.rs`, `firewall.rs`, `compose.rs`, `node.rs`, `share.rs`, `service.rs`, `basic_iot.rs`, `init.rs`, `join.rs`, `gaming/`)
- [x] Deleted orphaned test files across orchestrator, universal, discovery, CLI
- [x] Cleaned up `#[cfg(test)] #[path = "..."]` references to deleted files
- [x] Total: ~21,800 lines removed from codebase (404K → 383K)

### Wave 19: Unfulfilled Lint Expectations
- [x] Fixed `songbird-bluetooth/transport/mod.rs` — `#[expect(dead_code)]` → `#[allow(dead_code)]` (dual-feature gate)
- [x] Fixed `songbird-genesis/bluetooth_pure.rs` — same pattern
- [x] Fixed `songbird-compute-bridge/service.rs` — `#[expect]` → `#[allow]` for deserialized fields
- [x] Fixed `songbird-cli/resources.rs` — `#[expect]` → `#[allow]` for reserved API
- [x] Fixed `songbird-config/service_registry.rs` — `#[expect]` → `#[allow]` for planned fields
- [x] Zero compiler warnings across entire workspace

### Wave 20: Hardcoding Evolution
- [x] tarpc port: `SONGBIRD_TARPC_PORT` env var with `DEFAULT_TARPC_PORT` const fallback
- [x] All network ports in `CanonicalNetworkConfig` use `SafeEnv::get_port()` pattern
- [x] Issue templates rewritten — removed "Gaming Bridge" branding, reflects network orchestrator identity

### Wave 21: Version & Standards Alignment
- [x] Fixed version discrepancy: REMAINING_WORK.md aligned with Cargo.toml v0.2.1
- [x] All `#[ignore]` tests now have reason strings (14 bare ignores fixed)

### Wave 22: todo!() Test Stubs → Real Implementations
- [x] 7 workflow tests evolved from `todo!()` to real `execute_capability_workflow` assertions
- [x] 4 error handling tests evolved from `todo!()` to real error chain/context assertions
- [x] 2 tests properly marked `#[ignore = "reason"]` where API doesn't exist yet

### Wave 23: dead_code Evolution
- [x] `songbird-remote-deploy` — `#[allow(dead_code)]` → `#[expect(dead_code)]` for Deserialize fields
- [x] `songbird-compute-bridge` — same pattern for request fields

### Wave 24: Module Documentation (77 pub mod docs)
- [x] `songbird-orchestrator/src/lib.rs` — 36 module doc comments
- [x] `songbird-universal-ipc/src/lib.rs` — 10 module doc comments
- [x] `songbird-discovery/src/lib.rs` — 8 module doc comments
- [x] `songbird-http-client/src/lib.rs` — 9 module doc comments
- [x] `songbird-network-federation/src/lib.rs` — 14 module doc comments

### Wave 25: Coverage Expansion (+114 new tests)
- [x] `songbird-orchestrator`: 55 tests (consent, retry, scheduler, quota, admission, graph, registry, identity, self-knowledge, tokens, JWT)
- [x] `songbird-config`: 46 tests (constants, network/core, environment, runtime engine, infant config, capability/primal discovery)
- [x] `songbird-universal`: 5 tests (connection manager, discovery, circuit breaker manager)
- [x] `songbird-http-client`: 4 tests (connection pool, request, response, adaptive TLS)
- [x] `songbird-universal-ipc`: 4 tests (peer handler, discovery handler)
- [x] Test race conditions fixed with `#[serial_test::serial]` on env-modifying tests

### Wave 26: Ring Dependency Analysis
- [x] Documented exact dependency chain: songbird-quic → quinn → quinn-proto → ring; songbird-quic → rcgen → ring
- [x] Confirmed `default-features = false` on quinn eliminates ring (but requires alternative crypto backend)
- [x] Confirmed rcgen supports `default-features = false` with aws-lc-rs alternative
- [x] Pure-Rust QUIC path still blocked upstream (rustls-rustcrypto coverage gaps)
- [x] Documented migration steps and effort estimates in this file

---

## Completed (Mar 20, 2026 — Deep Debt Audit Session 2)

### Wave 7: Broken Tests & Doctests
- [x] Fixed failing doctest in `songbird-sovereign-onion/src/address.rs:147` — `SigningKey::generate()` → `SigningKey::from_bytes()` (ed25519-dalek 2.2.0 API)

### Wave 8: Smart File Refactoring (6 files → domain submodules)
- [x] Refactored `canonical.rs` (1,058 lines → `canonical/mod.rs` + `types.rs` + `adapter.rs` + `routing.rs`, largest 376 lines)
- [x] Refactored `mesh_handler.rs` (977 lines → `mesh_handler/mod.rs` + `json.rs` + `udp_discovery.rs` + `tests.rs`, largest ~406 lines)
- [x] Refactored `availability.rs` (944 lines → `availability/types.rs` + `checker.rs` + `tests.rs`)
- [x] Refactored `core/mod.rs` (933 lines → `consolidated_config.rs` + `orchestrator_health.rs` + `consolidated_engine.rs` + `consolidated_tests.rs`)
- [x] Refactored `capability_registration.rs` (928 lines → `config.rs` + `transport.rs` + `payload.rs` + `lifecycle.rs` + `tests.rs`)
- [x] Zero files now exceed 1000 lines

### Wave 9: `#[allow()]` → `#[expect(reason)]` Bulk Migration
- [x] Migrated all remaining bare `#[allow()]` to `#[expect(reason)]` where lint fires
- [x] Reverted to `#[allow(reason)]` where `expect` would be unfulfilled
- [x] Fixed syntax errors in `basic_iot/mod.rs` (`#[allow()dead_code)]` → valid attribute)
- [x] Normalized crate-level lint attributes across all crates

### Wave 10: Unsafe Code Evolution
- [x] Added `parking_lot::Mutex` serialization guard to `songbird-process-env` for caller-level safety
- [x] Changed crate to `#![deny(unsafe_code)]` with per-fn `#[allow(unsafe_code)]` (tightest scope)
- [x] Added unit test `set_remove_roundtrip` for env facade

### Wave 11: Hardcoded Constants → Capability-Based Discovery
- [x] Evolved `find_primals_with_capability` from stub to real capability filter (`SONGBIRD_CAPABILITY_<CAP>_PROVIDERS` + per-primal `*_CAPABILITIES`)
- [x] Removed hardcoded `staging.internal:8080` — all URLs use env → bind → documented fallback const chain
- [x] All ports configurable via `SONGBIRD_*_PORT` env vars with `FALLBACK_*` constants
- [x] Added `canonical_constants_evolved_tests.rs` integration tests

### Wave 12: Production Stub Evolution
- [x] `health_check_all()` → real TCP reachability probes via protocol router (pre-registered `TcpReachabilityHandler`)
- [x] `songbird cli` → minimal interactive REPL with `help`/`exit`/`quit` and subcommand guidance
- [x] Federation join → parses `FederationStatus`/`nodes`/`peers` from response
- [x] Load balancer `RoundRobin`/`WeightedRoundRobin` → stateful `AtomicU64` counter
- [x] Trust `verify_hardware` → proper error with BearDog capability discovery path
- [x] Protocol negotiation ID → monotonic sequence suffix for collision prevention

### Wave 13: Coverage Expansion (+200 new tests)
- [x] `songbird-orchestrator`: consent rules, preferences, request builder, graph validation, health, config, zero-touch, biome, trust ordering + batch integration (63 tests)
- [x] `songbird-config`: network cors, limits, timeouts, ports, gaming + batch integration (16 tests)
- [x] `songbird-universal`: discovery types, capability errors, traits, communication, adapter errors + batch integration (15 tests)
- [x] Plus ~100+ inline module tests across orchestrator/config/universal

### Wave 14: Documentation (29/29 crates `#[warn(missing_docs)]`)
- [x] Enabled `#![warn(missing_docs)]` on all 17 remaining crates
- [x] Documented public API entry points across all crates
- [x] Added module-level docs for all `pub mod` declarations
- [x] Used scoped `#[allow(missing_docs, reason)]` for deep internal modules in progress

### Wave 15: JSON-RPC Gateway Completion (10 semantic methods)
- [x] `compute.route`, `deployment.create`, `deployment.status`, `task.create`, `task.list`
- [x] `consent.check`, `consent.grant`, `registry.register`, `registry.discover`, `protocol.negotiate`
- [x] All methods share handler logic with REST endpoints (no duplication)
- [x] `JsonRpcState` wired to `ComputeApiState`, `DeploymentState`, `ProtocolApiState`, `TaskLifecycleManager`, `ConsentManager`

### Wave 16: Dependency Pruning
- [x] Removed unused `thiserror` from `songbird-tls`
- [x] Removed unused `tower` from `songbird-http-client`
- [x] Verified `kube`/`k8s-openapi`/`bollard` are feature-gated (not in default builds)
- [x] Documented duplicate version alignment opportunities for future pruning

### Wave 17: Coverage Expansion Wave 2 (+56 tests)
- [x] 21 tests in `songbird-http-client` (beardog RPC, TLS finished, HTTPS connection, connection pool)
- [x] 14 tests in `songbird-universal-ipc` (error display, introspection, capability discovery)
- [x] 8 tests in `songbird-discovery` (factory config, serde, config metadata)
- [x] 13 tests in `songbird-lineage-relay` (error display, types serde, connection stats)

---

## Completed (Mar 20, 2026 — Deep Audit Session 1)

### Wave 1: Standards Compliance
- [x] Migrated 122 `#[allow()]` → `#[expect(reason)]` across all 29 crates (wateringHole standard)
- [x] Reverted 23 to `#[allow(reason)]` where lint doesn't fire (correct behavior for unfulfillable expectations)
- [x] Removed 13 stale lint suppressions discovered by `#[expect()]` (e.g. dead_code that wasn't dead)
- [x] Fixed example crate license: `AGPL-3.0` → `AGPL-3.0-only` (SPDX alignment)
- [x] Isolated `MockBearDogProvider` behind `#[cfg(any(test, feature = "test-mocks"))]`
- [x] Added `test-mocks` feature to `songbird-network-federation`

### Wave 2: Safety & Production Hardening
- [x] Removed 3 production `panic!()`/`unreachable!()` — replaced with `Result`-based error returns
  - `sovereign-onion/service.rs`: standalone mode no longer panics
  - `orchestrator/http_server.rs`: `unreachable!()` → `Err(anyhow!(...))`
  - `lineage-relay/relay_server.rs`: `unreachable!()` removed via match restructure
- [x] Added SAFETY documentation to `songbird-process-env` unsafe blocks
- [x] Enhanced `tower` CLI: `tower info`/`tower config` now honor `SONGBIRD_BIND_ADDRESS`

### Wave 3: Architecture & Refactoring
- [x] Refactored `unified_adapter.rs` (956 lines → 5-module tree, largest 243 lines)
- [x] Refactored `http_handler.rs` (949 lines → 8-module tree, largest 166 lines)
- [x] Extracted `src/lib.rs` from binary-only `songbird` crate (testable CLI types)
- [x] Audited top .clone() hotspots — eliminated 6 unnecessary clones:
  - `discovery_bridge.rs`: `String` clones → moves on `AutoAccept`/`Reject` paths
  - `canonical.rs`: `service.clone()` → move semantics, `protocol` clone → borrowed lookup
  - `real_service_discovery.rs`: `.to_string()` in filter → `.any()` comparison
- [x] Feature-gated `infer_capabilities_from_name` behind `#[cfg(any(feature = "k8s", feature = "docker"))]`

### Wave 4: Coverage Expansion (+150 tests total)
- [x] 16 CLI parsing tests (`tests/cli_parsing_tests.rs`) — `main.rs` from 0% to testable
- [x] 27 tests in `songbird-config` (discovery, endpoints, constants, cache TTL)
- [x] 10 tests in `songbird-orchestrator` (availability, components, compute API, registration)
- [x] 30 tests in `songbird-universal` (tarpc, jsonrpc, connection_manager, query, sovereignty)
- [x] 31 tests in `songbird-http-client` (redirect, IPC client, TLS record, beardog RPC)
- [x] Fixed test race conditions in env-var-dependent tests (primal_discovery, constants, hardcoded_elimination)
- [x] 5 tests in `songbird-tls` (crypto, record layer, cert)
- [x] 8 tests in `songbird-discovery` (real_service_discovery, broadcaster, listener, self_knowledge)
- [x] 8 tests in `songbird-types` (canonical adapters, errors, environment, lineage, memory_optimized)
- [x] 7 tests in `songbird-registry` (production_storage, federation, registry core)
- [x] 3 tests in `songbird-stun` (message decoding, attribute roundtrip)
- [x] 25+ tests in `songbird-orchestrator` (capability_registration, core, graph, compute_api, trust, capability_router, process_manager)

### Wave 5: Analysis & Planning
- [x] Complete `ring` dependency elimination analysis:
  - `rcgen` → removable via BearDog cert delegation or pure-Rust PKIX encoder
  - `quinn` → blocked by upstream (quinn-proto defaults to `rustls-ring`, no C-free alternative)
  - Documented concrete elimination steps and effort estimates

### Wave 6: Root Documentation & Debris Cleanup
- [x] Updated README.md with current metrics (v0.3.3, 63.50% coverage, 6,300+ tests)
- [x] Added CHANGELOG v0.3.3 entry with all session work
- [x] Rewrote CONTRIBUTING.md with `#[expect(reason)]` standard, removed stale patterns
- [x] Deleted broken `config/docker/Dockerfile` (gaming bridge CMD, rust:1.75, wrong layout)
- [x] Deleted broken `config/docker/Dockerfile.federation` (no `songbird-federation` binary)
- [x] Deleted broken `docker/Dockerfile.production` (nonexistent `songbird-lib`)
- [x] Deleted broken `docker/docker-compose.production.yml` (referenced deleted Dockerfile)
- [x] Deleted broken `config/docker/docker-compose.production.yml` (gaming bridge, deleted Dockerfile)
- [x] Deleted broken `config/docker/docker-compose.core.yml` (gaming bridge, deleted Dockerfile)
- [x] Deleted orphaned `docker/entrypoint.sh` (was for deleted Dockerfile.production)
- [x] Deleted `config/scripts/production-deployment-demo.sh` (echo-only marketing script)
- [x] Deleted `config/scripts/deploy.sh` (wrong PROJECT_ROOT, builds `songbird-orchestrator`)
- [x] Corrected `#[warn(missing_docs)]` count: 13/29 crates (was reported as 2)
- [x] Created wateringHole handoff for v0.3.3

---

## Pending: BearDog Crypto Integration

BearDog provides pure Rust crypto via runtime capability discovery.
All stubs currently return `CryptoUnavailable`; wiring requires BearDog running.

### Tor Protocol
- [ ] AES-128-CTR encryption roundtrip via BearDog
- [ ] Running digest (SHA-1/SHA3-256) via BearDog for relay cell integrity
- [ ] HMAC-SHA256 for ESTABLISH_INTRO handshake auth
- [ ] ntor handshake (CREATE2/EXTEND2) via BearDog
- [ ] Heuristic relay selection (Phase 2A: intelligent selection based on consensus weights)

### TLS / Sovereign Onion
- [ ] `ed25519_public_from_secret` via BearDog
- [ ] BearDog-generated lineage-tagged certificates (full X.509 chain validation)
- [ ] CertificateVerify BearDog signing
- [ ] Custom TLS extension building via BearDog

### Ring-Free Workspace
- [ ] `rcgen` removal: replace with BearDog-issued certs or pure-Rust PKIX builder (`x509-cert`)
- [ ] Quinn `default-features = false` + selective feature enablement (avoids `rustls-ring`)
- [ ] Track upstream quinn/rustls for ring-free QUIC (quinn-rs/quinn#2253)

---

## Pending: Coverage Expansion (~67% → 90% target)

### High-Impact Targets (by missed lines)
| Module | Missed | Coverage |
|--------|--------|----------|
| songbird-orchestrator | ~7,000 | ~56% |
| songbird-config | ~2,600 | ~68% |
| songbird-universal | ~2,200 | ~72% |
| songbird-http-client | ~1,600 | ~65% |
| songbird-universal-ipc | ~1,200 | ~67% |

### Strategy
- Focus on pure logic modules for unit test ROI
- Add `#[cfg(test)] mod tests` to ~400 files that lack inline tests
- Prioritize: cache logic, graph algorithms, capability routing, TLS record parsing
- Target: 5-10 pp coverage gain per deep session

---

## Completed: Standards Compliance

### `#[warn(missing_docs)]` on library crates
- **29/29 crates** have `#[warn(missing_docs)]` enabled and compile clean
- Large internal modules use scoped `#[allow(missing_docs, reason = "...")]` where bulk documentation is in progress
- All public entry points and re-exports are documented

---

## Pending: Platform & Infrastructure

- [ ] Platform NFC backends (Android JNI, iOS CoreNFC, Linux libnfc)
- [ ] Real hardware IGD test (Tower + Pixel 8a)
- [ ] Genesis physical channels: Bluetooth GATT/L2CAP, QR code, SoloKey (FIDO2)
- [ ] iOS XPC transport (requires platform-specific bindings)
- [ ] WASM primal registry + tokio/mio WASM support
- [ ] Android IPC: configurable fallback bind address
- [ ] USB bulk endpoint streaming (currently uses control transfers)
- [ ] DNS SRV integration for capability discovery

---

## Pending: Architectural Evolution

- [x] REST endpoints → JSON-RPC wrapping (10 semantic methods in gateway)
- [x] Federation join logic (parses FederationStatus/nodes/peers from response)
- [x] Capability router selection strategy (stateful round-robin + weighted + least-response-time)
- [ ] Cluster support for anonymous beacon broadcasting
- [ ] TLS handshake v2 module integration into main handshake flow
- [ ] IPC native endpoint lifecycle management

---

## Pending: Dependency Evolution

- [ ] `ring` elimination: see Ring-Free Workspace section above
- [x] Evaluate `kube` + `k8s-openapi` weight — **feature-gated** behind `k8s` feature, not in default builds
- [x] Evaluate `bollard` weight — **feature-gated** behind `docker` feature, not in default builds
- [x] Removed 2 unused deps (`thiserror` from songbird-tls, `tower` from songbird-http-client)
- [ ] ~418 unique dependencies: align duplicate versions (base64, base32, socket2, thiserror, tower, rand)

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

1. **BearDog crypto wiring** — Unblocks circuit build + onion encryption (pure Rust via capability discovery)
2. **Coverage expansion** — Target pure-logic modules first (goal: 90%)
3. **Ring-free workspace** — `rcgen` replacement + quinn feature reconfiguration
4. **Deep documentation** — Fill `#[allow(missing_docs)]` internal modules with full doc coverage
5. **Real hardware tests** (Tower + Pixel) — Validates cross-network
6. **Platform backends** — Mobile pairing, iOS, WASM
7. **Dependency pruning** — Reduce ~412 unique deps where possible
